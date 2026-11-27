//! Provedores locais para o router real das suítes de materiais.

use axum::{
    Router,
    http::{Request, header, request::Builder},
};
use backend::{
    api::{router::create_router, state::AppState},
    infrastructure::database::PgDatabase,
    modules::{
        identity::infrastructure::firebase::AppCheckVerifier,
        materials::application::ports::{MaterialAccount, MaterialStorage},
    },
    test_support::local_identity_clients,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

const PROJECT_NUMBER: &str = "1234567890";
const APP_ID: &str = "1:1234567890:android:capp";

pub struct LocalMaterialAuth {
    pub app: Router,
    pub attestation: String,
    project: String,
    _jwks: MockServer,
}

impl LocalMaterialAuth {
    pub async fn new(
        pool: &PgPool,
        project: String,
        storage: Option<Arc<dyn MaterialStorage>>,
    ) -> Self {
        let jwks = MockServer::start().await;
        let endpoint = format!("{}/materials-jwks", jwks.uri());
        Mock::given(path("/materials-jwks"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("cache-control", "max-age=300")
                    .set_body_raw(include_str!("../jwt/jwks.json"), "application/json"),
            )
            .mount(&jwks)
            .await;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let clients =
            local_identity_clients(project.clone(), http.clone(), endpoint.clone(), jwks.uri())
                .unwrap();
        let app_check = AppCheckVerifier::new_with_endpoint(
            PROJECT_NUMBER.into(),
            vec![APP_ID.into()],
            http,
            endpoint,
        )
        .unwrap();
        let state = AppState::new(PgDatabase::new(pool.clone()), clients);
        let app = create_router(
            match storage {
                Some(storage) => state.with_material_storage(storage),
                None => state,
            },
            app_check,
        );
        let now = Utc::now().timestamp();
        let attestation = jwt(json!({
            "sub": APP_ID,
            "iss": format!("https://firebaseappcheck.googleapis.com/{PROJECT_NUMBER}"),
            "aud": [format!("projects/{PROJECT_NUMBER}")],
            "iat": now - 60,
            "exp": now + 3600
        }));
        Self {
            app,
            attestation,
            project,
            _jwks: jwks,
        }
    }

    pub fn request(&self, method: &str, path: &str, account: MaterialAccount) -> Builder {
        Request::builder()
            .method(method)
            .uri(path)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.token(account)),
            )
            .header("x-firebase-appcheck", &self.attestation)
    }

    pub fn token(&self, account: MaterialAccount) -> String {
        let now = Utc::now().timestamp();
        jwt(json!({
            "sub": account.id,
            "aud": self.project,
            "iss": format!("https://securetoken.google.com/{}", self.project),
            "iat": now - 60,
            "auth_time": now - 120,
            "exp": now + 3600,
            "firebase": {
                "sign_in_provider": "password",
                "identities": { "microsoft.com": [account.microsoft_object_id] }
            }
        }))
    }
}

fn jwt(claims: Value) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-kid".into());
    let key =
        EncodingKey::from_rsa_pem(include_bytes!("../jwt/test-only-rsa-private.pem")).unwrap();
    encode(&header, &claims, &key).unwrap()
}
