//! Valida o cadastro e o recibo privados com autenticação e App Check locais.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::{
    api::{router::create_router, state::AppState},
    infrastructure::database::PgDatabase,
    modules::identity::infrastructure::firebase::AppCheckVerifier,
    test_support::local_identity_clients,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

fn jwt(claims: Value) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-kid".into());
    let key = EncodingKey::from_rsa_pem(include_bytes!(
        "../../../fixtures/jwt/test-only-rsa-private.pem"
    ))
    .unwrap();
    encode(&header, &claims, &key).unwrap()
}

async fn authenticated_router(pool: &PgPool) -> (Router, MockServer, String) {
    let jwks = MockServer::start().await;
    let uri = format!("{}/jwks", jwks.uri());
    Mock::given(path("/jwks"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_raw(
                    include_str!("../../../fixtures/jwt/jwks.json"),
                    "application/json",
                ),
        )
        .mount(&jwks)
        .await;
    let http = reqwest::Client::new();
    let mut clients =
        local_identity_clients("board-test".into(), http.clone(), uri.clone(), jwks.uri()).unwrap();
    let app_id = "1:1234567890:android:capp";
    clients.app_check =
        AppCheckVerifier::new_with_endpoint("1234567890".into(), vec![app_id.into()], http, uri)
            .unwrap();
    let app = create_router(AppState::new(PgDatabase::new(pool.clone()), clients));
    let now = Utc::now().timestamp();
    let attestation = jwt(
        json!({"sub":app_id, "iss":"https://firebaseappcheck.googleapis.com/1234567890",
        "aud":["projects/1234567890"], "iat":now-60,"exp":now+3600}),
    );
    (app, jwks, attestation)
}

fn account_token(account: Uuid) -> String {
    let now = Utc::now().timestamp();
    jwt(
        json!({"sub":account.to_string(), "aud":"board-test", "iss":"https://securetoken.google.com/board-test",
        "iat":now-60,"auth_time":now-120,"exp":now+3600,"email":"teste@example.test",
        "firebase":{"sign_in_provider":"microsoft.com","identities":{"microsoft.com":[account]}}}),
    )
}

pub(super) async fn assert_own_receipts(pool: &PgPool, id: Uuid, expected: [(Uuid, &Value); 2]) {
    let (app, _jwks, attestation) = authenticated_router(pool).await;
    for (account, receipt) in expected {
        let token = account_token(account);
        let request = Request::builder()
            .uri(format!("/elections/{id}/receipts/me"))
            .header("authorization", format!("Bearer {token}"))
            .header("x-firebase-appcheck", &attestation)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let json: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(status, StatusCode::OK, "{json}");
        assert_eq!(&json, receipt);
    }
}

pub(super) async fn assert_own_registration(
    pool: &PgPool,
    election_id: Uuid,
    account_id: Uuid,
    proof: &[u8],
    public_key: &Value,
) {
    let (app, _jwks, attestation) = authenticated_router(pool).await;
    let token = account_token(account_id);
    let uri = format!("/elections/{election_id}/registration/me");
    let expected = json!({
        "requiresRegistration": true,
        "voterId": account_id,
        "publicKey": public_key,
        "participated": false,
    });
    for (method, body) in [("PUT", Body::from(proof.to_vec())), ("GET", Body::empty())] {
        let request = Request::builder()
            .method(method)
            .uri(&uri)
            .header("authorization", format!("Bearer {token}"))
            .header("x-firebase-appcheck", &attestation)
            .header("content-type", "application/json")
            .body(body)
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let json: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(json, expected);
    }
}
