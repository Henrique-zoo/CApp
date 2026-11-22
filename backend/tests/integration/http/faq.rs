//! Testes de integração HTTP para o módulo de FAQ.
//!
//! Exercita o cadastro de perguntas e respostas de FAQ pelo endpoint HTTP,
//! reutilizando a infraestrutura oficial de verificação institucional e
//! validando autenticação, autorização de gestão e isolamento estrito entre CAs.

use crate::support::TestDatabase;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::{
    api::state::AppState, infrastructure::database::PgDatabase,
    test_support::local_identity_clients,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path},
};

const PROJECT: &str = "http-faq-tests";

const KEY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/test-only-rsa-private.pem"
));

const JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

/// Gera token local RS256 com identidade vinculada, sem Firebase real.
fn token(uid: &str, object: Uuid, email: &str) -> String {
    let now = Utc::now().timestamp();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-kid".into());

    encode(
        &header,
        &json!({
            "sub": uid,
            "aud": PROJECT,
            "iss": format!("https://securetoken.google.com/{PROJECT}"),
            "iat": now - 60,
            "auth_time": now - 120,
            "exp": now + 3600,
            "email": email,
            "firebase": {
                "sign_in_provider": "microsoft.com",
                "identities": {
                    "microsoft.com": [object]
                }
            }
        }),
        &EncodingKey::from_rsa_pem(KEY.as_bytes()).unwrap(),
    )
    .unwrap()
}

/// Envia uma requisição em memória e interpreta status code e corpo JSON.
async fn request(
    app: &Router,
    http_method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().uri(uri).method(http_method);

    let builder = if let Some(tok) = token {
        builder.header("authorization", format!("Bearer {tok}"))
    } else {
        builder
    };

    let req = if let Some(body) = body {
        builder
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

/// Helper para configurar o router do app com mock JWKS e Graph local.
async fn setup_app(db: &TestDatabase) -> (Router, MockServer, MockServer) {
    let jwks = MockServer::start().await;
    let graph = MockServer::start().await;

    Mock::given(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_raw(JWKS, "application/json"),
        )
        .mount(&jwks)
        .await;

    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let state = AppState::new(
        PgDatabase::new(db.pool.clone()),
        local_identity_clients(
            PROJECT.into(),
            http,
            format!("{}/keys", jwks.uri()),
            format!("{}/v1.0", graph.uri()),
        )
        .unwrap(),
    );

    let app = Router::new()
        .merge(backend::modules::identity::api::router())
        .merge(backend::modules::faq::api::router())
        .with_state(state);

    (app, jwks, graph)
}

/// Provisiona um usuário pelo fluxo oficial de verificação institucional.
#[allow(clippy::too_many_arguments)]
async fn provision_user_officially(
    app: &Router,
    graph: &MockServer,
    tenant: Uuid,
    group: Uuid,
    object: Uuid,
    uid: &str,
    email: &str,
    course_name: &str,
) -> Uuid {
    let graph_token = format!("graph-token-{uid}");

    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .and(header("authorization", format!("Bearer {graph_token}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value":[{"id":tenant}]})))
        .expect(1)
        .mount(graph)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1.0/me/checkMemberGroups"))
        .and(header("authorization", format!("Bearer {graph_token}")))
        .and(body_json(json!({"groupIds":[group]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value":[group]})))
        .expect(1)
        .mount(graph)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1.0/me"))
        .and(header("authorization", format!("Bearer {graph_token}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": object,
            "displayName": "Estudante Teste",
            "mail": email,
            "userPrincipalName": email,
            "jobTitle": course_name
        })))
        .expect(1)
        .mount(graph)
        .await;

    let jwt = token(uid, object, email);
    let (status, body) = request(
        app,
        "POST",
        "/auth/institutional/verify",
        Some(&jwt),
        Some(json!({"microsoftAccessToken": graph_token})),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "verificação institucional oficial falhou: {body}"
    );

    graph.verify().await;
    graph.reset().await;

    Uuid::parse_str(body["user"]["id"].as_str().expect("id do usuario")).expect("uuid valido")
}

#[tokio::test]
async fn http_faq_creation_flows_and_authorization() {
    let db = TestDatabase::new().await;
    let (app, _jwks, graph) = setup_app(&db).await;

    let tenant = Uuid::new_v4();
    let group = Uuid::new_v4();
    let institution_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO institution (id, name, acronym, microsoft_tenant_id, microsoft_student_group_id, status) \
         VALUES ($1, 'Universidade de Brasília', 'UnB', $2, $3, 'active')",
    )
    .bind(institution_id)
    .bind(tenant)
    .bind(group)
    .execute(&db.pool)
    .await
    .unwrap();

    // 1. Provisiona gestor oficial via verificação institucional
    let manager_object = Uuid::new_v4();
    let manager_uid = "firebase-manager-uid";
    let manager_email = "241098765@aluno.unb.br";
    let manager_user_id = provision_user_officially(
        &app,
        &graph,
        tenant,
        group,
        manager_object,
        manager_uid,
        manager_email,
        "Ciência da Computação",
    )
    .await;

    // 2. Provisiona segundo estudante oficial sem cargo no CA via verificação institucional
    let student_object = Uuid::new_v4();
    let student_uid = "firebase-student-uid";
    let student_email = "241098766@aluno.unb.br";
    let _student_user_id = provision_user_officially(
        &app,
        &graph,
        tenant,
        group,
        student_object,
        student_uid,
        student_email,
        "Ciência da Computação",
    )
    .await;

    // 3. Cadastra Centros Acadêmicos no banco
    let active_ca_id = Uuid::new_v4();
    let inactive_ca_id = Uuid::new_v4();
    let other_ca_id = Uuid::new_v4();

    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();

    // CA ativo onde manager_user_id é gestor (created_by_id)
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at) \
         VALUES ($1, $2, $3, 'CA de Computação', 'CAC', 'CA dos estudantes de computacao', 'active', $4, $4)",
    )
    .bind(active_ca_id)
    .bind(institution_id)
    .bind(manager_user_id)
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    // CA inativo (arquivado) onde manager_user_id é gestor
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at) \
         VALUES ($1, $2, $3, 'CA Desativado', 'CAD', 'CA arquivado', 'archived', $4, $4)",
    )
    .bind(inactive_ca_id)
    .bind(institution_id)
    .bind(manager_user_id)
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    // CA ativo de outro curso/gestão (onde manager_user_id NÃO é gestor)
    let other_creator_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO app_user (id, firebase_uid, status) VALUES ($1, 'other-creator-uid', 'incomplete')",
    )
    .bind(other_creator_id)
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at) \
         VALUES ($1, $2, $3, 'CA de Matemática', 'CAMAT', 'CA de Matematica', 'active', $4, $4)",
    )
    .bind(other_ca_id)
    .bind(institution_id)
    .bind(other_creator_id)
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    let manager_jwt = token(manager_uid, manager_object, manager_email);
    let student_jwt = token(student_uid, student_object, student_email);

    // CENÁRIO A: Gestor cria pergunta e resposta no CA ativo (201 Created)
    let create_payload = json!({
        "question": "Como solicitar apoio acadêmico?",
        "answer": "Entre em contato com a gestão do CA.",
        "category": "Apoio Acadêmico"
    });

    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&manager_jwt),
        Some(create_payload.clone()),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["question"], "Como solicitar apoio acadêmico?");
    assert_eq!(body["answer"], "Entre em contato com a gestão do CA.");
    assert_eq!(body["category"], "Apoio Acadêmico");
    assert_eq!(body["academicCenterId"], active_ca_id.to_string());
    assert_eq!(body["createdBy"], manager_user_id.to_string());

    // CENÁRIO B: Alias sem /api/v1 também funciona (201 Created)
    let alias_payload = json!({
        "question": "Qual o horário de atendimento do CA?",
        "answer": "De segunda a sexta das 12h às 18h."
    });

    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{active_ca_id}/faq"),
        Some(&manager_jwt),
        Some(alias_payload),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["question"], "Qual o horário de atendimento do CA?");

    // CENÁRIO C: Usuário sem cargo de gestão no CA é recusado (403 user_not_permitted_in_ca)
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&student_jwt),
        Some(create_payload.clone()),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // CENÁRIO D: Isolamento entre CAs: gestor do CA 1 tenta cadastrar no CA 2 (403 user_not_permitted_in_ca)
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{other_ca_id}/faq"),
        Some(&manager_jwt),
        Some(create_payload.clone()),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // CENÁRIO E: CA inexistente (404 academic_center_not_found)
    let non_existent_ca_id = Uuid::new_v4();
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{non_existent_ca_id}/faq"),
        Some(&manager_jwt),
        Some(create_payload.clone()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    // CENÁRIO F: CA inativo (422 academic_center_inactive)
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{inactive_ca_id}/faq"),
        Some(&manager_jwt),
        Some(create_payload.clone()),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_inactive");

    // CENÁRIO G: Pergunta vazia (422 empty_question)
    let invalid_question_payload = json!({
        "question": "   ",
        "answer": "Resposta válida."
    });

    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&manager_jwt),
        Some(invalid_question_payload),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "empty_question");

    // CENÁRIO H: Resposta vazia (422 empty_answer)
    let invalid_answer_payload = json!({
        "question": "Pergunta válida?",
        "answer": "   "
    });

    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&manager_jwt),
        Some(invalid_answer_payload),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "empty_answer");

    // CENÁRIO I: Requisição sem token (401 unauthorized)
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        None,
        Some(create_payload),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["code"], "unauthorized");
}
