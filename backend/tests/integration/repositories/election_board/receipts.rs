//! Cadastro e recibo na API real, com sessões locais de duas contas distintas.

use axum::{
    body::{Body, to_bytes},
    http::{HeaderValue, Method, Request, StatusCode},
};
use backend::{
    api::{router::create_router, state::AppState},
    infrastructure::database::PgDatabase,
    modules::elections::application::registration::RegistrationStatus,
    modules::elections::crypto_wire::{MAX_SIGNED_MESSAGE_BYTES, ServerSigner},
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

struct SessionApi {
    app: axum::Router,
    attestation: HeaderValue,
    _jwks: MockServer,
}

impl SessionApi {
    async fn new(pool: &PgPool) -> Self {
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
        let app = create_router(AppState::new(PgDatabase::new(pool.clone()), clients));
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

    async fn send(&self, account: Uuid, request: Request<Body>) -> (StatusCode, Value) {
        self.send_with_microsoft_identity(account, account, request)
            .await
    }

    async fn send_with_microsoft_identity(
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
        request
            .headers_mut()
            .insert("x-firebase-appcheck", self.attestation.clone());
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let json: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16384).await.unwrap()).unwrap();
        (status, json)
    }
}

pub(super) async fn assert_own_receipts(pool: &PgPool, id: Uuid, expected: [(Uuid, &Value); 2]) {
    let api = SessionApi::new(pool).await;
    for (account, receipt) in expected {
        let request = Request::get(format!("/elections/{id}/receipts/me"))
            .body(Body::empty())
            .unwrap();
        let (status, json) = api.send(account, request).await;
        assert_eq!(status, StatusCode::OK, "{json}");
        assert_eq!(&json, receipt);
        let request = Request::get(format!("/elections/{id}/registration/me"))
            .body(Body::empty())
            .unwrap();
        let (status, registration) = api.send(account, request).await;
        assert_eq!(status, StatusCode::OK, "{registration}");
        assert_eq!(registration["participated"], true);
        assert!(registration["voterId"].is_string() && registration["publicKey"].is_string());
    }
}

pub(super) async fn assert_legacy_http(
    pool: &PgPool,
    id: Uuid,
    owner: Uuid,
    other: Uuid,
    recovered: &RegistrationStatus,
    receipt: &Value,
) {
    let api = SessionApi::new(pool).await;
    let get = |path: &str| {
        Request::get(format!("/elections/{id}/{path}/me"))
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(
        api.send(owner, get("registration")).await,
        (StatusCode::OK, json!(recovered))
    );
    assert_eq!(
        api.send(owner, get("receipts")).await,
        (StatusCode::OK, receipt.clone())
    );
    assert_eq!(
        api.send(other, get("registration")).await,
        (
            StatusCode::OK,
            json!({"voterId":null,"publicKey":null,"participated":false})
        )
    );
    let (status, error) = api.send(other, get("receipts")).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{error}");
    assert_eq!(error["error"]["code"], "receipt_not_found");
}

pub(super) async fn assert_registration_http(
    pool: &PgPool,
    election_id: Uuid,
    account: Uuid,
    other_account: Uuid,
    signer: &ServerSigner,
) {
    let api = SessionApi::new(pool).await;
    let uri = format!("/elections/{election_id}/registration/me");
    let get = || Request::get(&uri).body(Body::empty()).unwrap();
    let put = |media_type: &str, body: String| {
        Request::builder()
            .method(Method::PUT)
            .uri(&uri)
            .header("content-type", media_type)
            .body(Body::from(body))
            .unwrap()
    };
    let empty = json!({"publicKey":null, "voterId":null, "participated":false});
    assert_eq!(
        api.send(account, get()).await,
        (StatusCode::OK, empty.clone())
    );
    let key = signer.public_key_base64().unwrap();
    let payload = json!({"version": 1, "kind":"voter-registration", "electionId":election_id,
                        "accountId":account, "publicKey":key});
    let signed = signer.sign_payload(&payload).unwrap().to_string();
    let (status, response) = api
        .send_with_microsoft_identity(
            account,
            other_account,
            put("application/json", signed.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{response}");
    assert_eq!(response["error"]["code"], "voter_not_eligible");
    let (status, response) = api
        .send(other_account, put("application/json", signed.clone()))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{response}");
    assert_eq!(response["error"]["code"], "invalid_registration_proof");
    let duplicate = signed.replace("\"version\":1", "\"version\":1,\"version\":1");
    assert_ne!(duplicate, signed);
    let (status, response) = api.send(account, put("application/json", duplicate)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{response}");
    assert_eq!(response["error"]["code"], "invalid_registration_proof");
    let (status, _) = api.send(account, put("text/plain", signed.clone())).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let oversized = " ".repeat(MAX_SIGNED_MESSAGE_BYTES + 1);
    let (status, response) = api.send(account, put("application/json", oversized)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{response}");
    assert_eq!(response["error"]["code"], "payload_too_large");

    let (status, saved) = api
        .send(
            account,
            put("application/json; charset=utf-8", signed.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let voter_id: Uuid = serde_json::from_value(saved["voterId"].clone()).unwrap();
    assert_ne!(voter_id, account);
    assert_eq!(
        saved,
        json!({"voterId":voter_id,"publicKey":key,"participated":false})
    );
    assert_eq!(
        api.send(account, put("application/json", signed)).await,
        (StatusCode::OK, saved.clone())
    );
    assert_eq!(api.send(account, get()).await, (StatusCode::OK, saved));
    assert_eq!(
        api.send(other_account, get()).await,
        (StatusCode::OK, empty)
    );
}
