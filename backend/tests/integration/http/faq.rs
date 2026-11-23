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
    let created_faq_id =
        Uuid::parse_str(body["id"].as_str().expect("id da entrada")).expect("uuid valido");

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
    let alias_faq_id =
        Uuid::parse_str(body["id"].as_str().expect("id da entrada")).expect("uuid valido");

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

    // 4. Cadastra perguntas frequentes em outros CAs para validação de isolamento e status
    let other_faq_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at) \
         VALUES ($1, $2, 'Pergunta do CA de Matemática?', 'Resposta do CA de Matemática.', NULL, $3, $3, $4, $4)",
    )
    .bind(other_faq_id)
    .bind(other_ca_id)
    .bind(other_creator_id)
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    let inactive_faq_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at) \
         VALUES ($1, $2, 'Pergunta do CA Inativo?', 'Resposta do CA Inativo.', NULL, $3, $3, $4, $4)",
    )
    .bind(inactive_faq_id)
    .bind(inactive_ca_id)
    .bind(manager_user_id)
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    // CENÁRIO J: Edição com sucesso via PATCH /api/v1/academic-centers/{ca_id}/faq/{entry_id} (200 OK)
    let update_payload = json!({
        "question": "Como solicitar apoio pedagógico e acadêmico?",
        "answer": "A solicitação deve ser feita pela plataforma online do CA.",
        "category": "Apoio e Atendimento"
    });

    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        Some(update_payload),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], created_faq_id.to_string());
    assert_eq!(body["academicCenterId"], active_ca_id.to_string());
    assert_eq!(
        body["question"],
        "Como solicitar apoio pedagógico e acadêmico?"
    );
    assert_eq!(
        body["answer"],
        "A solicitação deve ser feita pela plataforma online do CA."
    );
    assert_eq!(body["category"], "Apoio e Atendimento");
    assert_eq!(body["updatedBy"], manager_user_id.to_string());

    // CENÁRIO K: Edição via alias sem /api/v1 (200 OK)
    let alias_update_payload = json!({
        "question": "Qual o horário de atendimento presencial do CA?",
        "answer": "De segunda a sexta das 10h às 19h."
    });

    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/academic-centers/{active_ca_id}/faq/{alias_faq_id}"),
        Some(&manager_jwt),
        Some(alias_update_payload),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["question"],
        "Qual o horário de atendimento presencial do CA?"
    );
    assert_eq!(body["answer"], "De segunda a sexta das 10h às 19h.");

    // CENÁRIO L: Edição parcial (somente resposta) preservando pergunta e categoria (200 OK)
    let partial_update_payload = json!({
        "answer": "Horário atualizado: seg a sex das 08h às 20h."
    });

    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{alias_faq_id}"),
        Some(&manager_jwt),
        Some(partial_update_payload),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["question"],
        "Qual o horário de atendimento presencial do CA?"
    );
    assert_eq!(
        body["answer"],
        "Horário atualizado: seg a sex das 08h às 20h."
    );

    // CENÁRIO M: Tentativa de edição por estudante sem cargo de gestão (403 user_not_permitted_in_ca)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&student_jwt),
        Some(json!({"question": "Tentativa de alteração não autorizada"})),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // CENÁRIO N: Isolamento entre CAs na edição: gestor do CA 1 tenta editar entrada do CA 2 (403 ou 404)
    // N.1: Passando rota do CA 2 onde não tem gestão -> 403
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{other_ca_id}/faq/{other_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "Tentativa invasão CA 2"})),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // N.2: Passando rota do seu próprio CA ativo, mas ID de FAQ do CA 2 -> 404 faq_entry_not_found
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{other_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "Tentativa cruzada"})),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "faq_entry_not_found");

    // CENÁRIO O: Edição de FAQ inexistente (404 faq_entry_not_found)
    let non_existent_faq_id = Uuid::new_v4();
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{non_existent_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "Pergunta fantasma"})),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "faq_entry_not_found");

    // CENÁRIO P: Edição em CA inexistente (404 academic_center_not_found)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{non_existent_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "CA inexistente"})),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    // CENÁRIO Q: Edição em CA inativo (422 academic_center_inactive)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{inactive_ca_id}/faq/{inactive_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "Edição em CA inativo"})),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_inactive");

    // CENÁRIO R: Edição com campos inválidos (422 empty_question / empty_answer)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "   "})),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "empty_question");

    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"answer": "   "})),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "empty_answer");

    // CENÁRIO S: Edição com body vazio {} (400 bad_request)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "bad_request");

    // CENÁRIO T: Edição sem token (401 unauthorized)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        None,
        Some(json!({"question": "Sem token"})),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["code"], "unauthorized");

    // CENÁRIO U: Tentativa de remoção por estudante sem cargo de gestão (403 user_not_permitted_in_ca)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{alias_faq_id}"),
        Some(&student_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // CENÁRIO V: Isolamento entre CAs na remoção
    // V.1: Gestor do CA 1 tenta remover na rota do CA 2 -> 403 user_not_permitted_in_ca
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{other_ca_id}/faq/{other_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    // V.2: Gestor do CA 1 tenta remover no CA 1 um FAQ que pertence ao CA 2 -> 404 faq_entry_not_found
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{other_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "faq_entry_not_found");

    // CENÁRIO W: Remoção de FAQ inexistente (404 faq_entry_not_found)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{non_existent_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "faq_entry_not_found");

    // CENÁRIO X: Remoção em CA inexistente (404 academic_center_not_found)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{non_existent_ca_id}/faq/{alias_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    // CENÁRIO Y: Remoção em CA inativo (422 academic_center_inactive)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{inactive_ca_id}/faq/{inactive_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_inactive");

    // CENÁRIO Z: Remoção sem token (401 unauthorized)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{alias_faq_id}"),
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["code"], "unauthorized");

    // CENÁRIO AA: Remoção com sucesso via alias sem /api/v1 (204 No Content)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/academic-centers/{active_ca_id}/faq/{alias_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    // CENÁRIO AB: Remoção com sucesso via /api/v1 (204 No Content)
    let (status, body) = request(
        &app,
        "DELETE",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    // Confirma que a entrada removida não pode mais ser editada (404 faq_entry_not_found)
    let (status, body) = request(
        &app,
        "PATCH",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{created_faq_id}"),
        Some(&manager_jwt),
        Some(json!({"question": "Tentativa pós remoção"})),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "faq_entry_not_found");
}

#[tokio::test]
async fn http_faq_navigation_and_search_flows() {
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

    // 1. Provisiona gestor oficial do CA via verificação institucional
    let manager_object = Uuid::new_v4();
    let manager_uid = "firebase-manager-uid-nav";
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

    // 2. Provisiona segundo estudante institucional sem cargo de gestão no CA
    let student_object = Uuid::new_v4();
    let student_uid = "firebase-student-uid-nav";
    let student_email = "241098766@aluno.unb.br";
    let _student_user_id = provision_user_officially(
        &app,
        &graph,
        tenant,
        group,
        student_object,
        student_uid,
        student_email,
        "Engenharia de Software",
    )
    .await;

    let active_ca_id = Uuid::new_v4();
    let math_ca_id = Uuid::new_v4();
    let empty_ca_id = Uuid::new_v4();
    let inactive_ca_id = Uuid::new_v4();

    // CA ativo de Computação (gerenciado pelo gestor)
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, $3, 'Centro Acadêmico de Computação', 'CACo', 'CA oficial de Computação', 'active')",
    )
    .bind(active_ca_id)
    .bind(institution_id)
    .bind(manager_user_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // CA ativo de Matemática (gerenciado por outro usuário)
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, NULL, 'Centro Acadêmico de Matemática', 'CAMat', 'CA oficial de Matemática', 'active')",
    )
    .bind(math_ca_id)
    .bind(institution_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // CA ativo vazio (sem perguntas)
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, NULL, 'Centro Acadêmico de Física', 'CAFis', 'CA sem perguntas', 'active')",
    )
    .bind(empty_ca_id)
    .bind(institution_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // CA inativo
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status) \
         VALUES ($1, $2, NULL, 'Centro Acadêmico Inativo', 'CAInat', 'CA inativo para testes', 'archived')",
    )
    .bind(inactive_ca_id)
    .bind(institution_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let manager_jwt = token(manager_uid, manager_object, manager_email);
    let student_jwt = token(student_uid, student_object, student_email);

    // Gestor cadastra pergunta 1 no CA de Computação
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&manager_jwt),
        Some(json!({
            "question": "Como solicitar apoio acadêmico?",
            "answer": "Entre em contato com a gestão do CA",
            "category": "Apoio Acadêmico"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let faq_1_id = body["id"].as_str().unwrap().to_string();

    // Gestor cadastra pergunta 2 no CA de Computação
    let (status, body) = request(
        &app,
        "POST",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&manager_jwt),
        Some(json!({
            "question": "Quais os prazos para matrícula em disciplinas?",
            "answer": "Consulte o calendário oficial publicado no SIGAA",
            "category": "Matrícula"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let _faq_2_id = body["id"].as_str().unwrap().to_string();

    // Cadastra pergunta 3 no CA de Matemática diretamente no banco
    let math_faq_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by) \
         VALUES ($1, $2, 'Qual o horário da biblioteca?', 'A biblioteca funciona das 08h às 20h', 'Biblioteca', $3, $3)",
    )
    .bind(math_faq_id)
    .bind(math_ca_id)
    .bind(manager_user_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // CENÁRIO 1: Estudante institucional (sem cargo de gestão) consulta FAQ do CA ativo
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("array de faq");
    assert_eq!(list.len(), 2);
    let questions: Vec<&str> = list
        .iter()
        .map(|item| item["question"].as_str().unwrap())
        .collect();
    assert!(questions.contains(&"Como solicitar apoio acadêmico?"));
    assert!(questions.contains(&"Quais os prazos para matrícula em disciplinas?"));
    assert!(
        !questions.contains(&"Qual o horário da biblioteca?"),
        "não deve ver perguntas do CA de Matemática"
    );

    // CENÁRIO 1b: Consulta via rota alias sem /api/v1
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{active_ca_id}/faq"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 2);

    // CENÁRIO 2: Visualizar a resposta de uma pergunta frequente por id
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{faq_1_id}"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], faq_1_id);
    assert_eq!(body["question"], "Como solicitar apoio acadêmico?");
    assert_eq!(body["answer"], "Entre em contato com a gestão do CA");

    // CENÁRIO 3: Buscar perguntas frequentes por um termo (q = "matrícula")
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq?q=matrícula"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("array de busca");
    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0]["question"],
        "Quais os prazos para matrícula em disciplinas?"
    );

    // CENÁRIO 4: Buscar pelo alias "search"
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq?search=matrícula"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 1);

    // CENÁRIO 5: Buscar termo presente apenas na resposta ("SIGAA")
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq?q=sigaa"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("array de busca por resposta");
    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0]["question"],
        "Quais os prazos para matrícula em disciplinas?"
    );

    // CENÁRIO 6: Buscar um termo sem correspondências
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq?q=termo sem correspondência"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("array vazio");
    assert!(list.is_empty());

    // CENÁRIO 7: Consultar um FAQ sem perguntas disponíveis
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{empty_ca_id}/faq"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.as_array().unwrap().is_empty());

    // CENÁRIO 8: Não exibir perguntas de outro CA na busca
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq?q=biblioteca"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.as_array().unwrap().is_empty(),
        "não deve exibir pergunta de outro CA"
    );

    // CENÁRIO 9: Filtrar por categoria
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq?category=Apoio Acadêmico"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("array de categoria");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["question"], "Como solicitar apoio acadêmico?");

    // CENÁRIO 10: Listar categorias do FAQ
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/categories"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let cats = body.as_array().expect("array de categorias");
    assert_eq!(cats, &vec![json!("Apoio Acadêmico"), json!("Matrícula")]);

    // CENÁRIO 10b: Listar categorias via alias
    let (status, body) = request(
        &app,
        "GET",
        &format!("/academic-centers/{active_ca_id}/faq/categories"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 2);

    // CENÁRIO 11: Requisição não autenticada retorna 401
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["code"], "unauthorized");

    // CENÁRIO 12: CA inexistente retorna 404
    let unknown_ca = Uuid::new_v4();
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{unknown_ca}/faq"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    // CENÁRIO 13: CA inativo retorna 422
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{inactive_ca_id}/faq"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_inactive");

    // CENÁRIO 14: Pergunta de outro CA em rota individual retorna 404
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/v1/academic-centers/{active_ca_id}/faq/{math_faq_id}"),
        Some(&student_jwt),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "faq_entry_not_found");
}
