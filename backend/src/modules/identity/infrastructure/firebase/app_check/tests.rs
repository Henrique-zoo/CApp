//! Validação criptográfica real com chaves de teste, JWKS local e claims hostis.

use super::*;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

const NUMBER: &str = "1234567890";

const APP: &str = "1:1234567890:android:capp";

const PEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/test-only-rsa-private.pem"
));

const JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

fn claims() -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    json!({
        "sub": APP,
        "iss": format!("https://firebaseappcheck.googleapis.com/{NUMBER}"),
        "aud": [format!("projects/{NUMBER}")],
        "iat": now - 60,
        "exp": now + 3600
    })
}

fn token(claims: &Value, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);

    header.kid = Some(kid.into());

    encode(&header, claims, &EncodingKey::from_rsa_pem(PEM).unwrap()).unwrap()
}

async fn verifier() -> (AppCheckVerifier, MockServer) {
    let server = MockServer::start().await;

    Mock::given(path("/jwks"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=3600")
                .set_body_raw(JWKS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let verifier = AppCheckVerifier::new_with_endpoint(
        NUMBER.into(),
        vec![APP.into()],
        http,
        format!("{}/jwks", server.uri()),
    )
    .unwrap();

    (verifier, server)
}

#[tokio::test]
async fn valid_tokens_and_concurrent_clones_share_keys() {
    let (v, server) = verifier().await;
    let token = token(&claims(), "test-kid");
    let clone = v.clone();

    let (a, b) = tokio::join!(v.verify(&token), clone.verify(&token));

    assert_eq!(a.unwrap().app_id, APP);

    assert_eq!(b.unwrap().app_id, APP);

    v.verify(&token).await.unwrap();

    server.verify().await;
}

/// O adaptador aplica o fallback de App Check quando não há prazo HTTP explícito.
#[tokio::test]
async fn absent_cache_headers_use_app_check_fallback() {
    let server = MockServer::start().await;

    Mock::given(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(JWKS, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let verifier = AppCheckVerifier::new_with_endpoint(
        NUMBER.into(),
        vec![APP.into()],
        Client::new(),
        format!("{}/jwks", server.uri()),
    )
    .unwrap();
    let token = token(&claims(), "test-kid");

    verifier.verify(&token).await.unwrap();
    verifier.verify(&token).await.unwrap();

    server.verify().await;
}

#[tokio::test]
async fn rejects_claims_signature_and_other_firebase_token_types() {
    let (v, _server) = verifier().await;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (field, value) in [
        ("iss", json!("https://securetoken.google.com/capp")),
        ("iss", json!("https://firebaseappcheck.googleapis.com/999")),
        ("aud", json!(["projects/999"])),
        ("sub", json!("1:1234567890:android:other")),
        ("sub", json!("")),
        ("exp", json!(now)),
        ("iat", json!(now + 100)),
        ("iat", json!(now + 3600)),
    ] {
        let mut c = claims();

        c[field] = value;

        assert_eq!(
            v.verify(&token(&c, "test-kid")).await.unwrap_err(),
            AppCheckError::Invalid,
            "{field}"
        );
    }

    for field in ["sub", "aud", "iss", "exp", "iat"] {
        let mut c = claims();

        c.as_object_mut().unwrap().remove(field);

        assert_eq!(
            v.verify(&token(&c, "test-kid")).await.unwrap_err(),
            AppCheckError::Invalid
        );
    }

    let signed = token(&claims(), "test-kid");
    let mut segments: Vec<_> = signed.split('.').map(str::to_owned).collect();
    let first = if segments[2].starts_with('a') {
        "b"
    } else {
        "a"
    };

    segments[2].replace_range(..1, first);

    assert_eq!(
        v.verify(&segments.join(".")).await.unwrap_err(),
        AppCheckError::Invalid
    );
}

#[tokio::test]
async fn rejects_bad_headers_without_network() {
    let server = MockServer::start().await;

    let v = AppCheckVerifier::new_with_endpoint(
        NUMBER.into(),
        vec![APP.into()],
        Client::new(),
        format!("{}/jwks", server.uri()),
    )
    .unwrap();

    for header in [
        Header {
            typ: None,
            kid: Some("test-kid".into()),
            ..Header::new(Algorithm::RS256)
        },
        Header {
            typ: Some("OTHER".into()),
            kid: Some("test-kid".into()),
            ..Header::new(Algorithm::RS256)
        },
        Header::new(Algorithm::RS256),
    ] {
        let t = encode(&header, &claims(), &EncodingKey::from_rsa_pem(PEM).unwrap()).unwrap();

        assert_eq!(v.verify(&t).await.unwrap_err(), AppCheckError::Invalid);
    }

    let mut header = Header::new(Algorithm::HS256);

    header.kid = Some("test-kid".into());

    let t = encode(&header, &claims(), &EncodingKey::from_secret(b"test-key")).unwrap();

    assert_eq!(v.verify(&t).await.unwrap_err(), AppCheckError::Invalid);

    assert_eq!(
        v.verify("malformed").await.unwrap_err(),
        AppCheckError::Invalid
    );

    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn unknown_kids_do_not_flood_jwks() {
    let (v, server) = verifier().await;

    v.verify(&token(&claims(), "test-kid")).await.unwrap();

    for n in 0..10 {
        assert_eq!(
            v.verify(&token(&claims(), &format!("unknown-{n}")))
                .await
                .unwrap_err(),
            AppCheckError::Invalid
        );
    }

    server.verify().await;
}

#[tokio::test]
async fn jwks_failure_is_unavailability_with_backoff() {
    let server = MockServer::start().await;

    Mock::given(path("/jwks"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;

    let v = AppCheckVerifier::new_with_endpoint(
        NUMBER.into(),
        vec![APP.into()],
        Client::new(),
        format!("{}/jwks", server.uri()),
    )
    .unwrap();

    for _ in 0..2 {
        assert_eq!(
            v.verify(&token(&claims(), "test-kid")).await.unwrap_err(),
            AppCheckError::Unavailable
        );
    }

    server.verify().await;
}

#[tokio::test]
async fn unusable_jwks_is_not_cached_as_valid() {
    for body in [
        json!({"keys":[]}),
        json!({"keys":[{"kty":"RSA","kid":"bad","alg":"HS256","n":"AQAB","e":"AQAB"}]}),
        json!({"bad":"json"}),
    ] {
        let server = MockServer::start().await;

        Mock::given(path("/jwks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let v = AppCheckVerifier::new_with_endpoint(
            NUMBER.into(),
            vec![APP.into()],
            Client::new(),
            format!("{}/jwks", server.uri()),
        )
        .unwrap();

        assert_eq!(
            v.verify(&token(&claims(), "test-kid")).await.unwrap_err(),
            AppCheckError::Unavailable
        );
    }
}

#[test]
fn configuration_is_strict() {
    for number in ["", "0", "01", "abc", "123 "] {
        assert!(AppCheckVerifier::new(number.into(), vec![APP.into()], Client::new()).is_err());
    }

    for apps in [vec![], vec!["".into()], vec!["1:999:android:capp".into()]] {
        assert!(AppCheckVerifier::new(NUMBER.into(), apps, Client::new()).is_err());
    }
}
