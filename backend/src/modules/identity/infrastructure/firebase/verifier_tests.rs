//! Testes do adaptador REAL, com JWT assinado por chave RSA descartável e
//! JWKS hospedado em servidor HTTP local Wiremock (sem Google externo).
//!
//! Chaves descartáveis de teste e endpoint local; nenhuma credencial de produção.

use super::*;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_PROJECT_ID: &str = "firebase-project-tests";

const TEST_PRIVATE_PEM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/test-only-rsa-private.pem"
));

const TEST_JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

/// Cria claims temporais válidas para um projeto fictício.
fn jwt_claims() -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    json!({
        "sub":"firebase-uid-123",
        "aud": TEST_PROJECT_ID,
        "iss": format!("https://securetoken.google.com/{TEST_PROJECT_ID}"),
        "iat": now - 60,
        "auth_time": now - 120,
        "exp": now + 3600,
        "firebase": {"identities": {"microsoft.com": [
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        ]}}
    })
}

/// Assina claims controladas com a chave RSA descartável dos testes.
fn signed_token(claims: Value, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);

    header.kid = Some(kid.to_owned());

    encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_PEM.as_bytes()).unwrap(),
    )
    .unwrap()
}

/// Direciona JWKS ao Wiremock, mantendo a validação criptográfica real.
fn new_test_verifier(server: &MockServer) -> FirebaseTokenVerifier {
    FirebaseTokenVerifier::new_with_endpoint(
        TEST_PROJECT_ID.to_owned(),
        Client::new(),
        format!("{}/keys", server.uri()),
    )
    .unwrap()
}

/// Publica a chave de teste e exige a quantidade indicada de downloads.
async fn mock_jwks(server: &MockServer, max_age: u64, expected_calls: u64) {
    Mock::given(method("GET"))
        .and(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Cache-Control", format!("public, max-age={max_age}"))
                .set_body_raw(TEST_JWKS, "application/json"),
        )
        .expect(expected_calls)
        .mount(server)
        .await;
}

/// Valida RS256 e reutiliza as chaves sem novos downloads.
#[tokio::test]
async fn valid_rs256_token_verifies_offline_after_first_jwks_fetch() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);
    let token = signed_token(jwt_claims(), "test-kid");

    for _ in 0..3 {
        let result = verifier.verify(&token).await.unwrap();

        assert_eq!(result.firebase_uid, "firebase-uid-123");

        assert_eq!(
            result.microsoft_object_id.as_deref(),
            Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
        );
    }
    // Wiremock confirma que somente UMA requisição HTTPS simulada ocorreu.
}

/// O adaptador Auth exige prazo explícito e não herda o fallback do App Check.
#[tokio::test]
async fn absent_cache_headers_do_not_reuse_identity_keys() {
    let server = MockServer::start().await;

    Mock::given(path("/keys"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TEST_JWKS, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let verifier = new_test_verifier(&server);
    let token = signed_token(jwt_claims(), "test-kid");

    verifier.verify(&token).await.unwrap();

    assert!(matches!(
        verifier.verify(&token).await,
        Err(AuthenticationError::ProviderUnavailable)
    ));

    server.verify().await;
}

/// Rejeita adulteração da assinatura mesmo quando a chave está em cache.
#[tokio::test]
async fn altered_signature_is_rejected_after_key_is_cached() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);

    verifier
        .verify(&signed_token(jwt_claims(), "test-kid"))
        .await
        .unwrap();

    let mut bad_claims = jwt_claims();

    bad_claims["sub"] = json!("attacker");

    // Altera a assinatura sem modificar o cabeçalho nem o payload.

    let valid = signed_token(bad_claims, "test-kid");
    let mut parts: Vec<String> = valid.split('.').map(str::to_owned).collect();

    let mut bytes = parts[2].clone().into_bytes();

    bytes[0] = if bytes[0] == b'A' { b'B' } else { b'A' };
    parts[2] = String::from_utf8(bytes).unwrap();

    let modified_token = parts.join(".");

    assert!(matches!(
        verifier.verify(&modified_token).await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Recusa tokens assinados corretamente mas emitidos para outro projeto ou emissor.
#[tokio::test]
async fn invalid_issuer_and_audience_are_rejected() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);

    let mut wrong_issuer = jwt_claims();

    wrong_issuer["iss"] = json!("https://evil.invalid");

    assert!(matches!(
        verifier
            .verify(&signed_token(wrong_issuer, "test-kid"))
            .await,
        Err(AuthenticationError::InvalidToken)
    ));

    let mut wrong_audience = jwt_claims();

    wrong_audience["aud"] = json!("other-firebase-project");

    assert!(matches!(
        verifier
            .verify(&signed_token(wrong_audience, "test-kid"))
            .await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Distingue token expirado de outras credenciais inválidas.
#[tokio::test]
async fn expired_token_is_rejected() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);
    let mut expired = jwt_claims();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    expired["exp"] = json!(now - 1);

    assert!(matches!(
        verifier.verify(&signed_token(expired, "test-kid")).await,
        Err(AuthenticationError::ExpiredToken)
    ));
}

/// Recusa autenticação declarada no futuro.
#[tokio::test]
async fn auth_time_in_future_is_rejected() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);
    let mut claims = jwt_claims();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    claims["auth_time"] = json!(now + 500);

    assert!(matches!(
        verifier.verify(&signed_token(claims, "test-kid")).await,
        Err(AuthenticationError::InvalidToken)
    ));
}

/// Todas as verificações concorrentes aguardam a mesma busca inicial de chaves.
#[tokio::test]
async fn concurrent_requests_share_one_initial_jwks_fetch() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);
    let token = signed_token(jwt_claims(), "test-kid");
    let mut handles = Vec::new();

    for _ in 0..12 {
        let verifier = verifier.clone();
        let token = token.clone();

        handles.push(tokio::spawn(async move { verifier.verify(&token).await }));
    }

    for handle in handles {
        let identity = handle.await.unwrap().unwrap();

        assert_eq!(identity.firebase_uid, "firebase-uid-123");
    }

    assert!(verifier.verify(&token).await.is_ok());
}

/// A configuração rejeita projeto vazio ou com espaços antes de acessar a rede.
#[test]
fn invalid_project_configuration_is_rejected() {
    for project in ["", " ", "project id", "project\n"] {
        assert!(FirebaseTokenVerifier::new(project.into(), Client::new()).is_err());
    }

    assert!(FirebaseTokenVerifier::new(TEST_PROJECT_ID.into(), Client::new()).is_ok());
}

/// Cabeçalhos inválidos, algoritmo simétrico e ausência de kid dispensam JWKS.
#[tokio::test]
async fn malformed_headers_are_rejected_without_network() {
    let server = MockServer::start().await;

    let verifier = new_test_verifier(&server);
    let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_PEM.as_bytes()).unwrap();
    let mut empty_kid = Header::new(Algorithm::RS256);

    empty_kid.kid = Some(String::new());

    let tokens = [
        "not-a-jwt".to_owned(),
        "invalid.base64.signature".to_owned(),
        encode(
            &Header::new(Algorithm::HS256),
            &jwt_claims(),
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap(),
        encode(&Header::new(Algorithm::RS256), &jwt_claims(), &key).unwrap(),
        encode(&empty_kid, &jwt_claims(), &key).unwrap(),
    ];

    for token in tokens {
        assert!(matches!(
            verifier.verify(&token).await,
            Err(AuthenticationError::InvalidToken)
        ));
    }

    assert!(server.received_requests().await.unwrap().is_empty());
}

/// Claims obrigatórias ausentes ou de tipo incompatível nunca autenticam.
#[tokio::test]
async fn missing_or_malformed_claims_are_rejected() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);

    for field in ["sub", "aud", "iss", "iat", "exp", "auth_time", "firebase"] {
        let mut claims = jwt_claims();

        claims.as_object_mut().unwrap().remove(field);

        assert!(
            matches!(
                verifier.verify(&signed_token(claims, "test-kid")).await,
                Err(AuthenticationError::InvalidToken)
            ),
            "{field}"
        );
    }

    for (field, value) in [
        ("sub", json!(42)),
        ("exp", json!("tomorrow")),
        ("firebase", json!({})),
        ("iat", json!(-1)),
    ] {
        let mut claims = jwt_claims();

        claims[field] = value;

        assert!(
            matches!(
                verifier.verify(&signed_token(claims, "test-kid")).await,
                Err(AuthenticationError::InvalidToken)
            ),
            "{field}"
        );
    }
}

/// UID vazio/maior que 128 caracteres é rejeitado; o limite conta caracteres Unicode.
#[tokio::test]
async fn uid_length_boundaries_are_enforced() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);

    for uid in [String::new(), "a".repeat(129)] {
        let mut claims = jwt_claims();

        claims["sub"] = json!(uid);

        assert!(matches!(
            verifier.verify(&signed_token(claims, "test-kid")).await,
            Err(AuthenticationError::InvalidToken)
        ));
    }

    let uid = "é".repeat(128);
    let mut claims = jwt_claims();

    claims["sub"] = json!(uid);

    assert_eq!(
        verifier
            .verify(&signed_token(claims, "test-kid"))
            .await
            .unwrap()
            .firebase_uid,
        uid
    );
}

/// Datas futuras, autenticação posterior à emissão e emissão após expiração são recusadas.
#[tokio::test]
async fn inconsistent_temporal_claims_are_rejected() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (field, value) in [
        ("iat", now + 600),
        ("auth_time", now - 30),
        ("nbf", now + 600),
        ("iat", now + 3600),
    ] {
        let mut claims = jwt_claims();

        claims[field] = json!(value);

        assert!(
            matches!(
                verifier.verify(&signed_token(claims, "test-kid")).await,
                Err(AuthenticationError::InvalidToken)
            ),
            "{field}"
        );
    }

    let mut claims = jwt_claims();

    claims["exp"] = json!(now);

    assert!(matches!(
        verifier.verify(&signed_token(claims, "test-kid")).await,
        Err(AuthenticationError::ExpiredToken)
    ));
}

/// Ausência de provedor Microsoft é permitida; listas vazias, ambíguas ou ID vazio não são.
#[tokio::test]
async fn microsoft_identity_cardinality_is_validated() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_verifier(&server);
    let mut claims = jwt_claims();

    claims["firebase"]["identities"] = json!({});

    assert!(
        verifier
            .verify(&signed_token(claims, "test-kid"))
            .await
            .unwrap()
            .microsoft_object_id
            .is_none()
    );

    for ids in [
        json!([]),
        json!([""]),
        json!(["a", "b"]),
        json!("not-an-array"),
    ] {
        let mut claims = jwt_claims();

        claims["firebase"]["identities"]["microsoft.com"] = ids;

        assert!(matches!(
            verifier.verify(&signed_token(claims, "test-kid")).await,
            Err(AuthenticationError::InvalidToken)
        ));
    }
}
