//! O router global exige atestado antes de consultar autenticação ou banco.

use super::*;
use crate::{
    api::{router::create_router, state::Clients},
    infrastructure::database::PgDatabase,
    modules::identity::infrastructure::{
        firebase::{AppCheckVerifier, FirebaseTokenVerifier},
        microsoft::MicrosoftGraphClient,
    },
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

const APP: &str = "1:1234567890:android:capp";

async fn app() -> (axum::Router, MockServer) {
    let server = MockServer::start().await;

    Mock::given(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/jwt/jwks.json"
            )),
            "application/json",
        ))
        .mount(&server)
        .await;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:1/test")
        .unwrap();

    pool.close().await;

    let http = reqwest::Client::new();
    let app_check = AppCheckVerifier::new_with_endpoint(
        "1234567890".into(),
        vec![APP.into()],
        http.clone(),
        format!("{}/jwks", server.uri()),
    )
    .unwrap();
    let state = AppState::new(
        PgDatabase::new(pool),
        Clients::new(
            FirebaseTokenVerifier::new("test-project".into(), http.clone()).unwrap(),
            MicrosoftGraphClient::new(http),
            app_check,
        ),
    );

    (create_router(state), server)
}

fn token() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut header = Header::new(Algorithm::RS256);

    header.kid = Some("test-kid".into());

    encode(&header,&json!({"sub":APP,"iss":"https://firebaseappcheck.googleapis.com/1234567890","aud":["projects/1234567890"],"iat":now-60,"exp":now+3600}),&EncodingKey::from_rsa_pem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/jwt/test-only-rsa-private.pem"))).unwrap()).unwrap()
}

#[tokio::test]
async fn attestation_is_required_before_user_authentication_on_all_business_routes() {
    let (app, server) = app().await;

    for (method, path) in [
        ("POST", "/auth/institutional/verify"),
        ("GET", "/auth/session"),
        ("PUT", "/users/me/profile"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(!response.headers().contains_key("www-authenticate"));

        let json: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();

        assert_eq!(json["error"]["code"], "app_check_required");
    }

    assert!(server.received_requests().await.unwrap().is_empty());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/session")
                .header("x-firebase-appcheck", token())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // App Check não autentica uma pessoa.
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn malformed_headers_are_rejected_without_jwks_and_monitoring_stays_public() {
    let (app, server) = app().await;

    for tokens in [
        vec![""],
        vec![" token "],
        vec!["invalid"],
        vec!["one", "two"],
    ] {
        let mut request = Request::builder().uri("/auth/session");

        for token in tokens {
            request = request.header("x-firebase-appcheck", token);
        }

        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    for (method, path, expected) in [
        ("GET", "/health", StatusCode::OK),
        ("HEAD", "/health", StatusCode::OK),
        ("GET", "/ready", StatusCode::SERVICE_UNAVAILABLE),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), expected);
    }

    assert!(server.received_requests().await.unwrap().is_empty());
}
