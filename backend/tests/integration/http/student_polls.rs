//! Testes de integração HTTP para Centros Acadêmicos e Consultas/Enquetes Estudantis (Issue #299).
//!
//! Exercita os endpoints REST através do router real, PostgreSQL descartável com migrations
//! e provedores locais de identidade (WireMock). Reutiliza os fluxos oficiais de verificação
//! institucional para provisionar estudantes com e-mails didáticos da UnB sem inserts manuais em app_user.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::{
    api::state::AppState,
    infrastructure::database::PgDatabase,
    test_support::{academic_center_router, local_identity_clients},
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::TestDatabase;

const PROJECT: &str = "http-academic-center-poll-tests";

const KEY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/test-only-rsa-private.pem"
));

const JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

/// Gera token local RS256 com identidade Microsoft vinculada para testes.
fn test_token(uid: &str, object: Uuid, email: &str) -> String {
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

/// Envia requisição em memória para o router Axum e devolve status e corpo JSON.
async fn request(
    app: &Router,
    method_str: &str,
    uri_path: &str,
    bearer_token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .method(method_str)
        .uri(uri_path)
        .header("authorization", format!("Bearer {bearer_token}"));

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

/// Registra no servidor WireMock as expectativas de verificação institucional da Microsoft.
async fn graph_mocks_for_student(
    graph: &MockServer,
    tenant: Uuid,
    group: Uuid,
    object: Uuid,
    email: &str,
    course_name: &str,
    graph_token: &str,
) {
    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .and(header("authorization", format!("Bearer {graph_token}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value":[{"id":tenant}]})))
        .mount(graph)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1.0/me/checkMemberGroups"))
        .and(header("authorization", format!("Bearer {graph_token}")))
        .and(body_json(json!({"groupIds":[group]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value":vec![group]})))
        .mount(graph)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1.0/me"))
        .and(header("authorization", format!("Bearer {graph_token}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": object,
            "displayName": "Estudante Teste",
            "mail": email,
            "jobTitle": course_name,
        })))
        .mount(graph)
        .await;
}

#[tokio::test]
async fn test_academic_center_student_polls_complete_lifecycle() {
    let database = TestDatabase::new().await;
    let pool = database.pool.clone();

    // -----------------------------------------------------------------------
    // 1. Provedores mockados de identidade (Firebase e Microsoft Graph)
    // -----------------------------------------------------------------------
    let jwks = MockServer::start().await;
    Mock::given(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_raw(JWKS, "application/json"),
        )
        .mount(&jwks)
        .await;

    let graph = MockServer::start().await;
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let clients = local_identity_clients(
        PROJECT.into(),
        http,
        format!("{}/keys", jwks.uri()),
        format!("{}/v1.0", graph.uri()),
    )
    .unwrap();

    let state = AppState::new(PgDatabase::new(pool.clone()), clients);
    let app = academic_center_router().with_state(state);

    // -----------------------------------------------------------------------
    // 2. Criação da instituição de ensino no banco de teste
    // -----------------------------------------------------------------------
    let institution_id = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let group = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO institution (id, name, acronym, microsoft_tenant_id, microsoft_student_group_id, status) \
         VALUES ($1, 'Universidade de Brasília', 'UnB', $2, $3, 'active')",
    )
    .bind(institution_id)
    .bind(tenant)
    .bind(group)
    .execute(&pool)
    .await
    .unwrap();

    // -----------------------------------------------------------------------
    // 3. Provisionamento oficial dos estudantes através de /auth/institutional/verify
    //    Estudante A: 241098765@aluno.unb.br (Gestor de CAER)
    //    Estudante B: 241098766@aluno.unb.br (Gestor de CACC / Estudante em CAER)
    // -----------------------------------------------------------------------
    let object_a = Uuid::new_v4();
    let email_a = "241098765@aluno.unb.br";
    let jwt_a = test_token("firebase-uid-a", object_a, email_a);

    graph_mocks_for_student(
        &graph,
        tenant,
        group,
        object_a,
        email_a,
        "Engenharia de Redes de Comunicação",
        "graph-token-a",
    )
    .await;

    let (status, body_verify_a) = request(
        &app,
        "POST",
        "/auth/institutional/verify",
        &jwt_a,
        Some(json!({"microsoftAccessToken": "graph-token-a"})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "verificação do estudante A deve ter sucesso: {body_verify_a}"
    );
    let user_a_id = Uuid::parse_str(body_verify_a["user"]["id"].as_str().unwrap()).unwrap();

    let object_b = Uuid::new_v4();
    let email_b = "241098766@aluno.unb.br";
    let jwt_b = test_token("firebase-uid-b", object_b, email_b);

    graph_mocks_for_student(
        &graph,
        tenant,
        group,
        object_b,
        email_b,
        "Ciência da Computação",
        "graph-token-b",
    )
    .await;

    let (status, body_verify_b) = request(
        &app,
        "POST",
        "/auth/institutional/verify",
        &jwt_b,
        Some(json!({"microsoftAccessToken": "graph-token-b"})),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "verificação do estudante B deve ter sucesso: {body_verify_b}"
    );
    let user_b_id = Uuid::parse_str(body_verify_b["user"]["id"].as_str().unwrap()).unwrap();

    // -----------------------------------------------------------------------
    // 4. Cadastra os Centros Acadêmicos associando seus respectivos criadores/gestores
    // -----------------------------------------------------------------------
    let caer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, $3, 'Centro Acadêmico de Engenharia de Redes', 'CAER', 'CAER UnB', 'active')",
    )
    .bind(caer_id)
    .bind(institution_id)
    .bind(user_a_id)
    .execute(&pool)
    .await
    .unwrap();

    let cacc_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, $3, 'Centro Acadêmico de Ciência da Computação', 'CACC', 'CACC UnB', 'active')",
    )
    .bind(cacc_id)
    .bind(institution_id)
    .bind(user_b_id)
    .execute(&pool)
    .await
    .unwrap();

    // -----------------------------------------------------------------------
    // 5. Cenário: Gestor autorizado (user_a) cria consulta estudantil em CAER (201 Created)
    // -----------------------------------------------------------------------
    let (status, body_created) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls"),
        &jwt_a,
        Some(json!({
            "title": "Ampliação do horário dos laboratórios",
            "description": "Manter os laboratórios abertos até as 22h nos dias letivos",
            "options": [
                "Apoio a proposta",
                "Não apoio a proposta"
            ],
            "targetAudience": "all_students",
            "voteModality": "identified"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body_created}");
    let poll_id = Uuid::parse_str(body_created["id"].as_str().unwrap()).unwrap();
    assert_eq!(body_created["academicCenterId"], caer_id.to_string());
    assert_eq!(body_created["createdById"], user_a_id.to_string());
    assert_eq!(
        body_created["title"],
        "Ampliação do horário dos laboratórios"
    );
    assert_eq!(body_created["status"], "open");
    assert_eq!(body_created["totalVotes"], 0);

    let options = body_created["options"].as_array().expect("options array");
    assert_eq!(options.len(), 2);
    let opt_1_id = Uuid::parse_str(options[0]["id"].as_str().unwrap()).unwrap();
    let opt_2_id = Uuid::parse_str(options[1]["id"].as_str().unwrap()).unwrap();

    // -----------------------------------------------------------------------
    // 6. Cenário: Estudante sem permissão de gestão (user_b) tenta criar consulta em CAER (403 Forbidden)
    // -----------------------------------------------------------------------
    let (status, body_forbidden) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls"),
        &jwt_b,
        Some(json!({
            "title": "Tentativa Invasiva",
            "options": ["Sim", "Não"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body_forbidden["error"]["code"], "user_not_permitted_in_ca");

    // -----------------------------------------------------------------------
    // 7. Cenário: Listagem de consultas do Centro Acadêmico (200 OK)
    // -----------------------------------------------------------------------
    let (status, body_list) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{caer_id}/polls"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list = body_list.as_array().expect("array de consultas");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], poll_id.to_string());

    // -----------------------------------------------------------------------
    // 8. Cenário: Consulta de detalhes da consulta (200 OK)
    // -----------------------------------------------------------------------
    let (status, body_detail) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}"),
        &jwt_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_detail["id"], poll_id.to_string());
    assert_eq!(body_detail["totalVotes"], 0);

    // -----------------------------------------------------------------------
    // 9. Cenário: Estudante A registra voto na Opção 1 (201 Created)
    // -----------------------------------------------------------------------
    let (status, body_vote_a) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/votes"),
        &jwt_a,
        Some(json!({
            "optionId": opt_1_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body_vote_a}");
    assert_eq!(body_vote_a["pollId"], poll_id.to_string());
    assert_eq!(body_vote_a["optionId"], opt_1_id.to_string());
    assert_eq!(body_vote_a["userId"], user_a_id.to_string());

    // -----------------------------------------------------------------------
    // 10. Cenário: Estudante B registra voto na Opção 2 (201 Created)
    // -----------------------------------------------------------------------
    let (status, body_vote_b) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/votes"),
        &jwt_b,
        Some(json!({
            "optionId": opt_2_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body_vote_b}");
    assert_eq!(body_vote_b["userId"], user_b_id.to_string());

    // -----------------------------------------------------------------------
    // 11. Cenário: Tentativa de voto duplicado pelo Estudante A (409 Conflict)
    // -----------------------------------------------------------------------
    let (status, body_dup) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/votes"),
        &jwt_a,
        Some(json!({
            "optionId": opt_2_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body_dup["error"]["code"], "duplicate_vote");

    // -----------------------------------------------------------------------
    // 12. Cenário: Tentativa de voto com opção inexistente (422 Unprocessable Entity)
    // -----------------------------------------------------------------------
    let fake_opt_id = Uuid::new_v4();
    let (status, body_invalid_opt) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/votes"),
        &jwt_a,
        Some(json!({
            "optionId": fake_opt_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_invalid_opt["error"]["code"], "option_not_found");

    // -----------------------------------------------------------------------
    // 13. Cenário: Isolamento estrito de CA (tentar votar em poll de CAER usando rota do CACC)
    // -----------------------------------------------------------------------
    let (status, body_iso) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{cacc_id}/polls/{poll_id}/votes"),
        &jwt_b,
        Some(json!({
            "optionId": opt_2_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body_iso["error"]["code"], "poll_not_found");

    // -----------------------------------------------------------------------
    // 14. Cenário: Gestor sem permissão (user_b) tenta fechar consulta de CAER (403 Forbidden)
    // -----------------------------------------------------------------------
    let (status, body_close_forbidden) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/close"),
        &jwt_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        body_close_forbidden["error"]["code"],
        "user_not_permitted_in_ca"
    );

    // -----------------------------------------------------------------------
    // 15. Cenário: Gestor autorizado (user_a) encerra consulta antecipadamente (200 OK)
    // -----------------------------------------------------------------------
    let (status, body_closed) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/close"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_closed["status"], "closed");
    assert!(body_closed["closedAt"].is_string());
    assert_eq!(body_closed["closedById"], user_a_id.to_string());
    assert_eq!(body_closed["totalVotes"], 2);

    // -----------------------------------------------------------------------
    // 16. Cenário: Tentativa de voto após encerramento (422 Unprocessable Entity)
    // -----------------------------------------------------------------------
    let student_c_object = Uuid::new_v4();
    let student_c_email = "241098767@aluno.unb.br";
    let jwt_c = test_token("firebase-uid-c", student_c_object, student_c_email);

    graph_mocks_for_student(
        &graph,
        tenant,
        group,
        student_c_object,
        student_c_email,
        "Engenharia de Redes",
        "graph-token-c",
    )
    .await;

    request(
        &app,
        "POST",
        "/auth/institutional/verify",
        &jwt_c,
        Some(json!({"microsoftAccessToken": "graph-token-c"})),
    )
    .await;

    let (status, body_after_close) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}/votes"),
        &jwt_c,
        Some(json!({
            "optionId": opt_1_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_after_close["error"]["code"], "poll_closed");

    // -----------------------------------------------------------------------
    // 17. Cenário: Detalhes da consulta após encerramento exibem resultados consolidados
    // -----------------------------------------------------------------------
    let (status, body_final) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{caer_id}/polls/{poll_id}"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_final["totalVotes"], 2);
    let final_options = body_final["options"].as_array().unwrap();
    let r1 = final_options
        .iter()
        .find(|o| o["id"] == opt_1_id.to_string())
        .unwrap();
    assert_eq!(r1["voteCount"], 1);
    assert_eq!(r1["percentage"], 50.0);
    let r2 = final_options
        .iter()
        .find(|o| o["id"] == opt_2_id.to_string())
        .unwrap();
    assert_eq!(r2["voteCount"], 1);
    assert_eq!(r2["percentage"], 50.0);

    // -----------------------------------------------------------------------
    // 18. Cenário: Tentativa com opções inválidas (menos de 2 opções) (422)
    // -----------------------------------------------------------------------
    let (status, body_invalid) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{caer_id}/polls"),
        &jwt_a,
        Some(json!({
            "title": "Apenas uma opção",
            "options": ["Opção Única"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_invalid["error"]["code"], "invalid_poll_options");

    // -----------------------------------------------------------------------
    // 19. Cenário: Tentativa em Centro Acadêmico inexistente (404)
    // -----------------------------------------------------------------------
    let fake_ca = Uuid::new_v4();
    let (status, body_not_found) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{fake_ca}/polls"),
        &jwt_a,
        Some(json!({
            "title": "Consulta em CA fantasma",
            "options": ["A", "B"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body_not_found["error"]["code"], "ca_not_found");

    database.close().await;
}
