//! Testes de integração HTTP para Centros Acadêmicos e Mandatos de Gestão (Issue #296).
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

const PROJECT: &str = "http-academic-center-mandate-tests";

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
            "displayName": "Estudante UnB",
            "mail": email,
            "jobTitle": course_name
        })))
        .mount(graph)
        .await;
}

#[tokio::test]
async fn academic_center_mandate_http_flows_with_real_database() {
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

    let app = academic_center_router().with_state(state);

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
    .execute(&pool)
    .await
    .unwrap();

    // -----------------------------------------------------------------------
    // 1. Provisionamento oficial do Estudante A (241098765@aluno.unb.br)
    // -----------------------------------------------------------------------
    let object_a = Uuid::new_v4();
    let uid_a = Uuid::new_v4().to_string();
    let email_a = "241098765@aluno.unb.br";
    let jwt_a = test_token(&uid_a, object_a, email_a);

    graph_mocks_for_student(
        &graph,
        tenant,
        group,
        object_a,
        email_a,
        "Engenharia de Redes",
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

    // -----------------------------------------------------------------------
    // 2. Provisionamento oficial do Estudante B (241098766@aluno.unb.br)
    // -----------------------------------------------------------------------
    let object_b = Uuid::new_v4();
    let uid_b = Uuid::new_v4().to_string();
    let email_b = "241098766@aluno.unb.br";
    let jwt_b = test_token(&uid_b, object_b, email_b);

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
    // 3. Cadastra os Centros Acadêmicos associando seus respectivos gestores
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
    // 4. Cenário: Consulta quando ainda não há gestão ativa registrada (404 no_active_mandate)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/current"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "no_active_mandate");

    // -----------------------------------------------------------------------
    // 5. Cenário: Gestor autorizado (user_a) configura mandato inicial de CAER com sucesso (201 Created)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates"),
        &jwt_a,
        Some(json!({
            "startDate": "2026-01-01",
            "endDate": "2026-12-31",
            "title": "Gestão 2026-2027"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let mandate_id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();
    assert_eq!(body["academicCenterId"], caer_id.to_string());
    assert_eq!(body["createdById"], user_a_id.to_string());
    assert_eq!(body["title"], "Gestão 2026-2027");
    assert_eq!(body["startDate"], "2026-01-01");
    assert_eq!(body["endDate"], "2026-12-31");
    assert_eq!(body["status"], "active");
    assert_eq!(body["durationInDays"], 364);

    // -----------------------------------------------------------------------
    // 6. Cenário: Consulta do mandato vigente com sucesso pelo gestor
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/current"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], mandate_id.to_string());
    assert_eq!(body["status"], "active");
    assert_eq!(body["durationInDays"], 364);

    // -----------------------------------------------------------------------
    // 7. Cenário: Estudante comum (user_b) consulta mandato vigente de CAER sem permissão de gestão (200 OK)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/current"),
        &jwt_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], mandate_id.to_string());

    // -----------------------------------------------------------------------
    // 8. Cenário: Alias no singular `/academic-centers/{id}/mandate`
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandate"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], mandate_id.to_string());

    // -----------------------------------------------------------------------
    // 9. Cenário: Tentativa de configuração por usuário sem permissão no CA (403 user_not_permitted_in_ca)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates"),
        &jwt_b, // user_b é gestor de CACC, não de CAER
        Some(json!({
            "startDate": "2027-01-01",
            "endDate": "2027-12-31",
            "title": "Tentativa Invasiva"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // -----------------------------------------------------------------------
    // 10. Cenário: Tentativa de duplicar mandato ativo (409 active_mandate_already_exists)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates"),
        &jwt_a,
        Some(json!({
            "startDate": "2026-06-01",
            "endDate": "2027-05-31",
            "title": "Gestão Duplicada"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "active_mandate_already_exists");

    // -----------------------------------------------------------------------
    // 11. Cenário: Tentativa com datas inválidas (end_date <= start_date) (422 invalid_mandate_dates)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{cacc_id}/mandates"),
        &jwt_b, // user_b tem permissão em CACC
        Some(json!({
            "startDate": "2026-12-31",
            "endDate": "2026-01-01",
            "title": "Gestão com datas invertidas"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "invalid_mandate_dates");

    // -----------------------------------------------------------------------
    // 12. Cenário: Tentativa em Centro Acadêmico inexistente (404 ca_not_found)
    // -----------------------------------------------------------------------
    let fake_ca_id = Uuid::new_v4();
    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{fake_ca_id}/mandates"),
        &jwt_a,
        Some(json!({
            "startDate": "2026-01-01",
            "endDate": "2026-12-31"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "ca_not_found");

    // -----------------------------------------------------------------------
    // 13. Cenário: Tentativa em Centro Acadêmico arquivado (422 ca_inactive)
    // -----------------------------------------------------------------------
    sqlx::query("UPDATE academic_center SET status = 'archived' WHERE id = $1")
        .bind(cacc_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{cacc_id}/mandates"),
        &jwt_b,
        Some(json!({
            "startDate": "2026-01-01",
            "endDate": "2026-12-31"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "ca_inactive");

    // -----------------------------------------------------------------------
    // 14. Cenário: Listagem do histórico de mandatos (200 OK)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("deve ser array");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], mandate_id.to_string());

    // -----------------------------------------------------------------------
    // 15. Cenário: Consulta do histórico de gestões concluídas quando há apenas mandato ativo (deve retornar vazio [])
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/history"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let history = body.as_array().expect("deve ser array");
    assert!(
        history.is_empty(),
        "histórico de gestões concluídas deve ser vazio enquanto só houver mandato ativo"
    );

    // -----------------------------------------------------------------------
    // 16. Cenário: Gestor autorizado adiciona integrantes ao mandato vigente
    // -----------------------------------------------------------------------
    let (status, body_member_1) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates/{mandate_id}/members"),
        &jwt_a,
        Some(json!({
            "name": "Maria Silva",
            "role": "Presidente",
            "userId": user_a_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body_member_1}");
    let member_1_id = Uuid::parse_str(body_member_1["id"].as_str().unwrap()).unwrap();
    assert_eq!(body_member_1["name"], "Maria Silva");
    assert_eq!(body_member_1["role"], "Presidente");
    assert_eq!(body_member_1["isActive"], true);
    assert_eq!(body_member_1["userId"], user_a_id.to_string());

    let (status, body_member_2) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates/{mandate_id}/members"),
        &jwt_a,
        Some(json!({
            "name": "João Souza",
            "role": "Diretor Financeiro",
            "userId": null
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body_member_2}");
    let member_2_id = Uuid::parse_str(body_member_2["id"].as_str().unwrap()).unwrap();
    assert_eq!(body_member_2["isActive"], true);

    // -----------------------------------------------------------------------
    // 17. Cenário: Usuário sem permissão não pode adicionar integrantes (403 user_not_permitted_in_ca)
    // -----------------------------------------------------------------------
    let (status, body) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates/{mandate_id}/members"),
        &jwt_b,
        Some(json!({
            "name": "Intruso",
            "role": "Vice-Presidente"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // -----------------------------------------------------------------------
    // 18. Cenário: Desfiliação de integrante da gestão preserva o registro histórico
    // -----------------------------------------------------------------------
    let (status, body_disaffiliate) = request(
        &app,
        "POST",
        &format!(
            "/academic-centers/{caer_id}/mandates/{mandate_id}/members/{member_2_id}/disaffiliate"
        ),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body_disaffiliate}");
    assert_eq!(body_disaffiliate["id"], member_2_id.to_string());
    assert_eq!(body_disaffiliate["isActive"], false);
    assert!(body_disaffiliate["leftAt"].is_string());

    // -----------------------------------------------------------------------
    // 19. Cenário: Estudante (inclusive user_b sem permissão de gestão) consulta detalhes do mandato por ID (transparência pública)
    // -----------------------------------------------------------------------
    let (status, body_detail) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/{mandate_id}"),
        &jwt_b,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body_detail}");
    assert_eq!(body_detail["id"], mandate_id.to_string());
    let members = body_detail["members"]
        .as_array()
        .expect("deve ter lista de membros");
    assert_eq!(members.len(), 2);
    let m1 = members
        .iter()
        .find(|m| m["id"] == member_1_id.to_string())
        .unwrap();
    assert_eq!(m1["isActive"], true);
    let m2 = members
        .iter()
        .find(|m| m["id"] == member_2_id.to_string())
        .unwrap();
    assert_eq!(m2["isActive"], false);
    assert!(m2["leftAt"].is_string());

    // -----------------------------------------------------------------------
    // 20. Cenário: Transição de mandato (conclui o mandato ativo e cria novo mandato)
    // -----------------------------------------------------------------------
    let (status, body_transition) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates/transition"),
        &jwt_a,
        Some(json!({
            "startDate": "2027-01-01",
            "endDate": "2027-12-31",
            "title": "Gestão 2027-2028 Renovação"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body_transition}");
    let new_mandate_id = Uuid::parse_str(body_transition["id"].as_str().unwrap()).unwrap();
    assert_ne!(new_mandate_id, mandate_id);
    assert_eq!(body_transition["status"], "active");
    assert_eq!(body_transition["title"], "Gestão 2027-2028 Renovação");

    // -----------------------------------------------------------------------
    // 21. Cenário: Consulta do mandato vigente agora retorna a nova gestão
    // -----------------------------------------------------------------------
    let (status, body_current) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/current"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_current["id"], new_mandate_id.to_string());
    assert_eq!(body_current["status"], "active");

    // -----------------------------------------------------------------------
    // 22. Cenário: Consulta do histórico de gestões agora retorna a gestão anterior concluída
    // -----------------------------------------------------------------------
    let (status, body_history) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/history"),
        &jwt_b, // Estudante comum pode consultar
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let history = body_history.as_array().expect("deve ser array");
    assert_eq!(
        history.len(),
        1,
        "deve conter 1 gestão no histórico anterior"
    );
    assert_eq!(history[0]["id"], mandate_id.to_string());
    assert_eq!(history[0]["status"], "concluded");
    assert!(history[0]["concludedAt"].is_string());
    assert_eq!(history[0]["members"].as_array().unwrap().len(), 2);

    // -----------------------------------------------------------------------
    // 23. Cenário: Consulta de listagem geral retorna ambos os mandatos ordenados decrescente
    // -----------------------------------------------------------------------
    let (status, body_all) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let all = body_all.as_array().expect("deve ser array");
    assert_eq!(all.len(), 2);
    assert_eq!(
        all[0]["id"],
        new_mandate_id.to_string(),
        "novo mandato deve vir primeiro"
    );
    assert_eq!(
        all[1]["id"],
        mandate_id.to_string(),
        "mandato anterior deve vir em segundo"
    );

    // -----------------------------------------------------------------------
    // 24. Cenário: Concluir o mandato ativo restante via /conclude
    // -----------------------------------------------------------------------
    let (status, body_conclude) = request(
        &app,
        "POST",
        &format!("/academic-centers/{caer_id}/mandates/conclude"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_conclude["id"], new_mandate_id.to_string());
    assert_eq!(body_conclude["status"], "concluded");

    // Agora histórico tem 2 gestões concluídas ordenadas cronologicamente
    let (status, body_history_after) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/history"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let history_after = body_history_after.as_array().unwrap();
    assert_eq!(history_after.len(), 2);
    assert_eq!(history_after[0]["id"], new_mandate_id.to_string());
    assert_eq!(history_after[1]["id"], mandate_id.to_string());

    // -----------------------------------------------------------------------
    // 25. Cenário: Isolamento entre Centros Acadêmicos e validações 404
    // -----------------------------------------------------------------------
    let fake_ca = Uuid::new_v4();
    let (status, body_fake_ca) = request(
        &app,
        "GET",
        &format!("/academic-centers/{fake_ca}/mandates/history"),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body_fake_ca["error"]["code"], "ca_not_found");

    let (status, body_fake_mandate) = request(
        &app,
        "GET",
        &format!("/academic-centers/{caer_id}/mandates/{}", Uuid::new_v4()),
        &jwt_a,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body_fake_mandate["error"]["code"], "mandate_not_found");

    database.close().await;
}
