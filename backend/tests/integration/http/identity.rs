//! Integração do router e extractor com PostgreSQL real e provedores HTTP locais.
//! Uma única instância PostgreSQL atende todos os fluxos deste teste.
//! Execute `cargo test --test integration --features test-support http::identity::`.
use crate::support::TestDatabase;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::{
    api::state::AppState,
    infrastructure::database::PgDatabase,
    test_support::{identity_router, local_identity_clients},
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROJECT: &str = "http-identity-tests";
const KEY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/test-only-rsa-private.pem"
));
const JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

/// Gera token local RS256 com identidade Microsoft vinculada, sem Firebase real.
fn token(uid: &str, object: Uuid) -> String {
    let now = Utc::now().timestamp();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-kid".into());
    encode(&header,&json!({"sub":uid,"aud":PROJECT,"iss":format!("https://securetoken.google.com/{PROJECT}"),"iat":now-60,"auth_time":now-120,"exp":now+3600,"firebase":{"identities":{"microsoft.com":[object]}}}),&EncodingKey::from_rsa_pem(KEY.as_bytes()).unwrap()).unwrap()
}

/// Envia uma requisição em memória e interpreta seu corpo, sem servidor HTTP do backend.
async fn request(
    app: &Router,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
    let req = if let Some(body) = body {
        builder
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Configura a sequência Graph para um usuário e exige token e grupo corretos.
async fn graph_mocks(graph: &MockServer, tenant: Uuid, group: Uuid, object: Uuid, student: bool) {
    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .and(header("authorization", "Bearer graph-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value":[{"id":tenant}]})))
        .expect(1)
        .mount(graph)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1.0/me/checkMemberGroups"))
        .and(header("authorization", "Bearer graph-token"))
        .and(body_json(json!({"groupIds":[group]})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"value":if student {vec![group]} else {vec![]}})),
        )
        .expect(1)
        .mount(graph)
        .await;
    Mock::given(method("GET")).and(path("/v1.0/me")).and(header("authorization","Bearer graph-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":object,"displayName":"Pessoa","jobTitle":if student {"Curso"} else {"Professor"}}))).expect(1).mount(graph).await;
}

/// Exercita verificação, persistência, reutilização e rejeições através do router real.
///
/// Estudantes e não estudantes recebem 200; identidade divergente, tenant desconhecido,
/// falha Graph, evidência ausente/expirada e falha SQL produzem os erros públicos corretos.
/// A rota protegida usa o extractor real e deve reutilizar o JWKS e dispensar Graph.
#[tokio::test]
async fn institutional_http_flows_with_real_database() {
    let database = TestDatabase::new().await;
    let pool = database.pool.clone();
    let jwks = MockServer::start().await;
    let graph = MockServer::start().await;
    Mock::given(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_raw(JWKS, "application/json"),
        )
        .expect(1)
        .mount(&jwks)
        .await;
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let state = AppState::new(
        PgDatabase::new(pool.clone()),
        local_identity_clients(
            PROJECT.into(),
            http,
            format!("{}/keys", jwks.uri()),
            format!("{}/v1.0", graph.uri()),
        )
        .unwrap(),
    );
    let app = identity_router().with_state(state);
    let tenant = Uuid::new_v4();
    let group = Uuid::new_v4();
    let institution = Uuid::new_v4();
    let object = Uuid::new_v4();
    sqlx::query("INSERT INTO institution (id,name,microsoft_tenant_id,microsoft_student_group_id) VALUES ($1,'Instituição HTTP',$2,$3)")
        .bind(institution).bind(tenant).bind(group).execute(&pool).await.unwrap();
    for student in [true, false] {
        let uid = Uuid::new_v4().to_string();
        let jwt = token(&uid, object);
        let (status, body) = request(&app, "/protected", &jwt, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"]["code"], "institutional_verification_required");
        graph_mocks(&graph, tenant, group, object, student).await;
        let (status, body) = request(
            &app,
            "/auth/institutional/verify",
            &jwt,
            Some(json!({"microsoftAccessToken":"graph-token"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            body["profile"]["affiliation"]["type"],
            if student { "student" } else { "nonStudent" }
        );
        if student {
            assert_eq!(body["profile"]["affiliation"]["courseName"], "Curso");
        } else {
            assert!(body["profile"]["affiliation"].get("courseName").is_none());
        }
        let stored:bool=sqlx::query_scalar("SELECT student_membership_confirmed FROM institutional_verification WHERE firebase_uid=$1").bind(&uid).fetch_one(&pool).await.unwrap();
        assert_eq!(stored, student);
        graph.verify().await;
        graph.reset().await;
        let (status, body) = request(&app, "/protected", &jwt, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({"uid":uid,"institution":institution}));
        assert!(graph.received_requests().await.unwrap().is_empty());
        sqlx::query("UPDATE institutional_verification SET checked_at=now()-interval '2 hours', expires_at=now()-interval '1 hour' WHERE firebase_uid=$1").bind(&uid).execute(&pool).await.unwrap();
        assert_eq!(
            request(&app, "/protected", &jwt, None).await.0,
            StatusCode::FORBIDDEN
        );
    }
    let uid = "mismatched-user";
    let jwt = token(uid, object);
    graph_mocks(&graph, tenant, group, Uuid::new_v4(), true).await;
    let (status, body) = request(
        &app,
        "/auth/institutional/verify",
        &jwt,
        Some(json!({"microsoftAccessToken":"graph-token"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "identity_mismatch");
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM institutional_verification WHERE firebase_uid=$1")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
    graph.verify().await;
    graph.reset().await;
    Mock::given(path("/v1.0/organization"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"value":[{"id":Uuid::new_v4()}]})),
        )
        .expect(1)
        .mount(&graph)
        .await;
    let (status, body) = request(
        &app,
        "/auth/institutional/verify",
        &jwt,
        Some(json!({"microsoftAccessToken":"graph-token"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "unknown_institution");
    graph.verify().await;
    graph.reset().await;
    Mock::given(path("/v1.0/organization"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&graph)
        .await;
    let (status, body) = request(
        &app,
        "/auth/institutional/verify",
        &jwt,
        Some(json!({"microsoftAccessToken":"graph-token"})),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "authentication_provider_unavailable");
    pool.close().await;
    let (status, body) = request(&app, "/protected", &jwt, None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "internal_server_error");
    assert_eq!(body["error"]["message"], "Erro interno do servidor");
    database.close().await;
}
