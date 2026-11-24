//! Roteador real com um provedor de chaves local para tokens Firebase e App Check.

use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{HeaderValue, Request, StatusCode},
};
use backend::{
    api::{router::create_router, state::AppState},
    infrastructure::database::PgDatabase,
    modules::elections::crypto_wire::ServerSigner,
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

pub(super) struct SessionApi {
    app: axum::Router,
    attestation: HeaderValue,
    _jwks: MockServer,
}

impl SessionApi {
    pub(super) async fn new(pool: &PgPool) -> Self {
        Self::with_signer(pool, None).await
    }

    pub(super) async fn with_signer(pool: &PgPool, signer: Option<Arc<ServerSigner>>) -> Self {
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
            local_identity_clients("board-test".into(), http.clone(), uri.clone(), jwks.uri())
                .unwrap();
        let app_id = "1:1234567890:android:capp";
        clients.app_check = AppCheckVerifier::new_with_endpoint(
            "1234567890".into(),
            vec![app_id.into()],
            http,
            uri,
        )
        .unwrap();
        let state = AppState::new(PgDatabase::new(pool.clone()), clients);
        let app = create_router(match signer {
            Some(signer) => state.with_election_signer(signer),
            None => state,
        });
        let now = Utc::now().timestamp();
        let attestation = jwt(
            json!({"sub":app_id, "iss":"https://firebaseappcheck.googleapis.com/1234567890",
        "aud":["projects/1234567890"], "iat":now-60,"exp":now+3600}),
        );
        Self {
            app,
            attestation: attestation.parse().unwrap(),
            _jwks: jwks,
        }
    }

    pub(super) async fn send(&self, account: Uuid, request: Request<Body>) -> (StatusCode, Value) {
        self.send_with_microsoft_identity(account, account, request)
            .await
    }

    pub(super) async fn send_with_microsoft_identity(
        &self,
        account: Uuid,
        object_id: Uuid,
        mut request: Request<Body>,
    ) -> (StatusCode, Value) {
        let now = Utc::now().timestamp();
        let token = jwt(
            json!({"sub":account.to_string(), "aud":"board-test", "iss":"https://securetoken.google.com/board-test",
            "iat":now-60,"auth_time":now-120,"exp":now+3600,"email":"teste@example.test",
            "firebase":{"sign_in_provider":"microsoft.com","identities":{"microsoft.com":[object_id]}}}),
        );
        request
            .headers_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
        self.send_attested(request).await
    }

    pub(super) async fn send_attested(&self, mut request: Request<Body>) -> (StatusCode, Value) {
        request
            .headers_mut()
            .insert("x-firebase-appcheck", self.attestation.clone());
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let json: Value = serde_json::from_slice(
            &to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, json)
    }
}
