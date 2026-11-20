//! Testes de integração HTTP para o módulo de demandas.
//!
//! Exercita a criação de demandas nos dois fóruns (espaço físico e institucional),
//! validações de autenticação e regras de negócio de pertencimento ao CA/curso.

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
use sqlx::{PgPool, Row};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

const PROJECT: &str = "http-demand-tests";

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
    token_with_email(uid, object, "241098765@aluno.unb.br")
}

fn token_with_email(uid: &str, object: Uuid, email: &str) -> String {
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

/// Envia uma requisição em memória e interpreta seu corpo e status code.
async fn request(
    app: &Router,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().uri(path).method("POST");

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

/// Envia uma requisição GET em memória e interpreta seu corpo e status code.
async fn get_request(app: &Router, path: &str, token: Option<&str>) -> (StatusCode, Value) {
    let builder = Request::builder().uri(path).method("GET");

    let builder = if let Some(tok) = token {
        builder.header("authorization", format!("Bearer {tok}"))
    } else {
        builder
    };

    let req = builder.body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();

    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Helper para configurar o ambiente de banco e mock JWKS.
async fn setup_app(db: &TestDatabase) -> (Router, MockServer) {
    let jwks = MockServer::start().await;

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
            "http://127.0.0.1:9/v1.0".to_string(),
        )
        .unwrap(),
    );

    let app = backend::modules::demands::api::router().with_state(state);
    (app, jwks)
}

/// Cria instituição, Centro Acadêmico, usuário e verificação institucional.
#[allow(clippy::too_many_arguments)]
async fn create_test_context(
    pool: &PgPool,
    ca_name: &str,
    ca_acronym: &str,
    user_firebase_uid: &str,
    object_id: Uuid,
    is_student: bool,
    course_name: Option<&str>,
    is_ca_manager: bool,
    ca_status: &str,
) -> (Uuid, Uuid, Uuid) {
    let institution_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO institution (id, name, acronym, microsoft_tenant_id, microsoft_student_group_id, status)
         VALUES ($1, 'Universidade de Brasília', 'UnB', $2, $3, 'active')",
    )
    .bind(institution_id)
    .bind(tenant_id)
    .bind(group_id)
    .execute(pool)
    .await
    .expect("should insert institution");

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid, status) VALUES ($1, $2, 'incomplete')")
        .bind(user_id)
        .bind(user_firebase_uid)
        .execute(pool)
        .await
        .expect("should insert app_user");

    let ca_id = Uuid::new_v4();
    let created_by = if is_ca_manager { Some(user_id) } else { None };
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status)
         VALUES ($1, $2, $3, $4, $5, 'CA para testes HTTP de demandas', $6)",
    )
    .bind(ca_id)
    .bind(institution_id)
    .bind(created_by)
    .bind(ca_name)
    .bind(ca_acronym)
    .bind(ca_status)
    .execute(pool)
    .await
    .expect("should insert academic_center");

    let now = Utc::now();
    let expires = now + chrono::Duration::days(30);

    sqlx::query(
        "INSERT INTO institutional_verification (
            user_id, institution_id, microsoft_object_id, microsoft_tenant_id,
            microsoft_student_group_id, student_membership_confirmed, course_name,
            checked_at, expires_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(user_id)
    .bind(institution_id)
    .bind(object_id)
    .bind(tenant_id)
    .bind(group_id)
    .bind(is_student)
    .bind(course_name)
    .bind(now)
    .bind(expires)
    .execute(pool)
    .await
    .expect("should insert institutional_verification");

    (institution_id, ca_id, user_id)
}

#[tokio::test]
async fn should_create_physical_space_demand_successfully() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-phys-1";
    let object_id = Uuid::new_v4();
    let (_inst_id, ca_id, user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "physical_space",
        "title": "Reparo do projetor",
        "description": "O projetor da sala de reuniões está piscando constantemente.",
        "category": "Infraestrutura"
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "deve retornar 201 Created: {body}"
    );
    assert_eq!(body["academicCenterId"], ca_id.to_string());
    assert_eq!(body["authorId"], user_id.to_string());
    assert_eq!(body["forum"], "physical_space");
    assert_eq!(body["title"], "Reparo do projetor");
    assert_eq!(
        body["description"],
        "O projetor da sala de reuniões está piscando constantemente."
    );
    assert_eq!(body["category"], "Infraestrutura");
    assert_eq!(body["status"], "open");
    assert!(body["courseName"].is_null());

    let demand_id = Uuid::parse_str(body["id"].as_str().expect("id de demanda"))
        .expect("UUID válido de demanda");

    // Confere no banco de dados
    let demand_row = sqlx::query("SELECT status, forum FROM demand WHERE id = $1")
        .bind(demand_id)
        .fetch_one(&db.pool)
        .await
        .expect("demanda deve existir no banco");

    let db_status: String = demand_row.get("status");
    let db_forum: String = demand_row.get("forum");
    assert_eq!(db_status, "open");
    assert_eq!(db_forum, "physical_space");

    // Confere histórico inicial
    let hist_row = sqlx::query(
        "SELECT to_status, from_status, changed_by_id FROM demand_status_history WHERE demand_id = $1",
    )
    .bind(demand_id)
    .fetch_one(&db.pool)
    .await
    .expect("histórico deve existir");

    let hist_to: String = hist_row.get("to_status");
    let hist_from: Option<String> = hist_row.get("from_status");
    let hist_author: Uuid = hist_row.get("changed_by_id");
    assert_eq!(hist_to, "open");
    assert_eq!(hist_from, None);
    assert_eq!(hist_author, user_id);

    db.close().await;
}

#[tokio::test]
async fn should_create_institutional_demand_successfully_for_enrolled_student() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-inst-1";
    let object_id = Uuid::new_v4();
    let (_inst_id, ca_id, user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "institutional",
        "title": "Ajuste na grade horária",
        "description": "Conflito entre duas disciplinas obrigatórias do quarto semestre.",
        "category": "Acadêmico",
        "course_name": "Ciência da Computação"
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "deve retornar 201 Created: {body}"
    );
    assert_eq!(body["academicCenterId"], ca_id.to_string());
    assert_eq!(body["authorId"], user_id.to_string());
    assert_eq!(body["forum"], "institutional");
    assert_eq!(body["title"], "Ajuste na grade horária");
    assert_eq!(body["courseName"], "Ciência da Computação");
    assert_eq!(body["status"], "open");

    db.close().await;
}

#[tokio::test]
async fn should_create_institutional_demand_successfully_for_ca_manager_even_if_different_course() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-manager-1";
    let object_id = Uuid::new_v4();
    // Gestor é estudante de outro curso (ex: Engenharia de Redes), mas administra o CACC
    let (_inst_id, ca_id, user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Engenharia de Redes"),
        true, // is_ca_manager = true
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "institutional",
        "title": "Abertura de turma extra de Cálculo",
        "description": "Demanda acolhida pela gestão para representação junto ao departamento.",
        "category": "Coordenação",
        "course_name": "Ciência da Computação"
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "gestor do CA pode registrar demanda institucional: {body}"
    );
    assert_eq!(body["authorId"], user_id.to_string());
    assert_eq!(body["courseName"], "Ciência da Computação");

    db.close().await;
}

#[tokio::test]
async fn should_reject_unauthenticated_request_with_401() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let ca_id = Uuid::new_v4();
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "physical_space",
        "description": "Sem autenticação"
    });

    let (status, body) = request(&app, &path, None, Some(payload)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");

    db.close().await;
}

#[tokio::test]
async fn should_reject_demand_for_nonexistent_ca_with_404() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-valid-1";
    let object_id = Uuid::new_v4();
    let (_inst_id, _real_ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let non_existent_ca = Uuid::new_v4();
    let path = format!("/api/v1/academic-centers/{non_existent_ca}/demands");

    let payload = json!({
        "forum": "physical_space",
        "description": "Demanda para CA inexistente"
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    db.close().await;
}

#[tokio::test]
async fn should_reject_institutional_demand_when_student_not_in_course_or_management_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-other-course";
    let object_id = Uuid::new_v4();
    // Estudante de Engenharia Mecânica tentando criar demanda institucional no CACC (sem ser gestor)
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Engenharia Mecânica"),
        false, // não é gestor
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "institutional",
        "title": "Reclamação de prova",
        "description": "A prova de compiladores foi excessivamente longa.",
        "course_name": "Ciência da Computação"
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    db.close().await;
}

#[tokio::test]
async fn should_reject_physical_space_demand_with_course_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-phys-err";
    let object_id = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "physical_space",
        "title": "Infiltração no teto",
        "description": "Goteira na sala de estudos do Centro Acadêmico.",
        "course_name": "Ciência da Computação" // proibido para physical_space
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(
        body["error"]["code"],
        "course_not_allowed_for_physical_space"
    );

    db.close().await;
}

#[tokio::test]
async fn should_reject_institutional_demand_without_course_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-inst-err";
    let object_id = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "institutional",
        "title": "Falta de professor",
        "description": "Turma sem professor designado há três semanas.",
        // falta course_name
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "course_name_required");

    db.close().await;
}

#[tokio::test]
async fn should_reject_demand_with_empty_description_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-desc-err";
    let object_id = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        object_id,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, object_id);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");

    let payload = json!({
        "forum": "physical_space",
        "title": "Título válido",
        "description": "   " // em branco
    });

    let (status, body) = request(&app, &path, Some(&jwt), Some(payload)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "empty_description");

    db.close().await;
}

#[tokio::test]
async fn should_list_physical_space_demands_for_student_of_another_course() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid_a = "student-course-cc";
    let obj_a = Uuid::new_v4();
    let (_inst_id, ca_id, _user_a_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid_a,
        obj_a,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    // Usuário B é estudante de outro curso (Engenharia Mecânica)
    let uid_b = "student-other-course";
    let obj_b = Uuid::new_v4();
    let user_b_id = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(user_b_id)
        .bind(uid_b)
        .execute(&db.pool)
        .await
        .unwrap();

    let expires_at = Utc::now() + chrono::Duration::days(120);
    sqlx::query(
        "INSERT INTO institutional_verification (user_id, student_membership_confirmed, course_name, expires_at) \
         VALUES ($1, true, 'Engenharia Mecânica', $2)",
    )
    .bind(user_b_id)
    .bind(expires_at)
    .execute(&db.pool)
    .await
    .unwrap();

    // Usuário A cria demanda de espaço físico no CA
    let jwt_a = token(uid_a, obj_a);
    let path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let create_payload = json!({
        "forum": "physical_space",
        "title": "Reparo da porta",
        "description": "Porta da sala do CA está emperrada.",
        "category": "Infraestrutura"
    });
    let (status, _created) = request(&app, &path, Some(&jwt_a), Some(create_payload)).await;
    assert_eq!(status, StatusCode::CREATED);

    // Usuário B (de outro curso) consulta demandas de espaço físico
    let jwt_b = token(uid_b, obj_b);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands?forum=physical_space");
    let (status, body) = get_request(&app, &list_path, Some(&jwt_b)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("deve retornar array");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["title"], "Reparo da porta");
    assert_eq!(list[0]["forum"], "physical_space");

    db.close().await;
}

#[tokio::test]
async fn should_filter_own_demands_with_mine_param() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid_a = "student-mine-a";
    let obj_a = Uuid::new_v4();
    let (_inst_id, ca_id, _user_a_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid_a,
        obj_a,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let uid_b = "student-mine-b";
    let obj_b = Uuid::new_v4();
    let user_b_id = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(user_b_id)
        .bind(uid_b)
        .execute(&db.pool)
        .await
        .unwrap();

    let expires_at = Utc::now() + chrono::Duration::days(120);
    sqlx::query(
        "INSERT INTO institutional_verification (user_id, student_membership_confirmed, course_name, expires_at) \
         VALUES ($1, true, 'Ciência da Computação', $2)",
    )
    .bind(user_b_id)
    .bind(expires_at)
    .execute(&db.pool)
    .await
    .unwrap();

    let jwt_a = token(uid_a, obj_a);
    let jwt_b = token(uid_b, obj_b);
    let post_path = format!("/api/v1/academic-centers/{ca_id}/demands");

    // Usuário A cria "Reparo da porta"
    let (status_a, _) = request(
        &app,
        &post_path,
        Some(&jwt_a),
        Some(json!({
            "forum": "physical_space",
            "title": "Reparo da porta",
            "description": "Demanda de A"
        })),
    )
    .await;
    assert_eq!(status_a, StatusCode::CREATED);

    // Usuário B cria "Troca da lâmpada"
    let (status_b, _) = request(
        &app,
        &post_path,
        Some(&jwt_b),
        Some(json!({
            "forum": "physical_space",
            "title": "Troca da lâmpada",
            "description": "Demanda de B"
        })),
    )
    .await;
    assert_eq!(status_b, StatusCode::CREATED);

    // Usuário A consulta somente as próprias demandas (mine=true)
    let list_mine_path =
        format!("/api/v1/academic-centers/{ca_id}/demands?forum=physical_space&mine=true");
    let (status, body_mine) = get_request(&app, &list_mine_path, Some(&jwt_a)).await;
    assert_eq!(status, StatusCode::OK, "{body_mine}");
    let list_mine = body_mine.as_array().unwrap();
    assert_eq!(list_mine.len(), 1);
    assert_eq!(list_mine[0]["title"], "Reparo da porta");

    // Usuário A remove o filtro das próprias demandas (sem mine=true) -> vê ambas
    let list_all_path = format!("/api/v1/academic-centers/{ca_id}/demands?forum=physical_space");
    let (status_all, body_all) = get_request(&app, &list_all_path, Some(&jwt_a)).await;
    assert_eq!(status_all, StatusCode::OK, "{body_all}");
    let list_all = body_all.as_array().unwrap();
    assert_eq!(list_all.len(), 2);

    db.close().await;
}

#[tokio::test]
async fn should_filter_demands_by_forum_and_status() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-filter-fs";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let post_path = format!("/api/v1/academic-centers/{ca_id}/demands");

    // Cria demanda de espaço físico
    let (status_ps, body_ps) = request(
        &app,
        &post_path,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Reparo da janela",
            "description": "Janela quebrada"
        })),
    )
    .await;
    assert_eq!(status_ps, StatusCode::CREATED);
    let ps_id: Uuid = body_ps["id"].as_str().unwrap().parse().unwrap();

    // Cria demanda institucional
    let (status_inst, _) = request(
        &app,
        &post_path,
        Some(&jwt),
        Some(json!({
            "forum": "institutional",
            "title": "Ajuste de grade",
            "description": "Conflito de horários",
            "courseName": "Ciência da Computação"
        })),
    )
    .await;
    assert_eq!(status_inst, StatusCode::CREATED);

    // Atualiza status da demanda de espaço físico para in_analysis
    sqlx::query("UPDATE demand SET status = 'in_analysis' WHERE id = $1")
        .bind(ps_id)
        .execute(&db.pool)
        .await
        .unwrap();

    // Filtra por fórum institucional
    let list_inst_path = format!("/api/v1/academic-centers/{ca_id}/demands?forum=institutional");
    let (status, body_inst) = get_request(&app, &list_inst_path, Some(&jwt)).await;
    assert_eq!(status, StatusCode::OK);
    let arr_inst = body_inst.as_array().unwrap();
    assert_eq!(arr_inst.len(), 1);
    assert_eq!(arr_inst[0]["forum"], "institutional");
    assert_eq!(arr_inst[0]["title"], "Ajuste de grade");

    // Filtra por status in_analysis
    let list_status_path = format!("/api/v1/academic-centers/{ca_id}/demands?status=in_analysis");
    let (status, body_status) = get_request(&app, &list_status_path, Some(&jwt)).await;
    assert_eq!(status, StatusCode::OK);
    let arr_status = body_status.as_array().unwrap();
    assert_eq!(arr_status.len(), 1);
    assert_eq!(arr_status[0]["status"], "in_analysis");
    assert_eq!(arr_status[0]["title"], "Reparo da janela");

    db.close().await;
}

#[tokio::test]
async fn should_isolate_demands_between_different_academic_centers() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-multi-ca";
    let obj = Uuid::new_v4();
    let (inst_id, ca_1, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let ca_2 = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, name, acronym, description, status) \
         VALUES ($1, $2, 'Centro Acadêmico de Matemática', 'CAMAT', 'CA MAT', 'active')",
    )
    .bind(ca_2)
    .bind(inst_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let jwt = token(uid, obj);

    // Demanda no CA 1
    let post_ca1 = format!("/api/v1/academic-centers/{ca_1}/demands");
    request(
        &app,
        &post_ca1,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Reparo da porta",
            "description": "Demanda no CA 1"
        })),
    )
    .await;

    // Demanda no CA 2
    let post_ca2 = format!("/api/v1/academic-centers/{ca_2}/demands");
    request(
        &app,
        &post_ca2,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Pintura da sala",
            "description": "Demanda no CA 2"
        })),
    )
    .await;

    // Consulta CA 1: vê "Reparo da porta" e NÃO vê "Pintura da sala"
    let list_ca1_path = format!("/api/v1/academic-centers/{ca_1}/demands?forum=physical_space");
    let (status_1, body_1) = get_request(&app, &list_ca1_path, Some(&jwt)).await;
    assert_eq!(status_1, StatusCode::OK);
    let arr_1 = body_1.as_array().unwrap();
    assert_eq!(arr_1.len(), 1);
    assert_eq!(arr_1[0]["title"], "Reparo da porta");

    // Consulta CA 2: vê "Pintura da sala" e NÃO vê "Reparo da porta"
    let list_ca2_path = format!("/api/v1/academic-centers/{ca_2}/demands?forum=physical_space");
    let (status_2, body_2) = get_request(&app, &list_ca2_path, Some(&jwt)).await;
    assert_eq!(status_2, StatusCode::OK);
    let arr_2 = body_2.as_array().unwrap();
    assert_eq!(arr_2.len(), 1);
    assert_eq!(arr_2[0]["title"], "Pintura da sala");

    db.close().await;
}

#[tokio::test]
async fn should_return_empty_list_when_no_demands_exist_in_ca() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-empty-list";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands?forum=physical_space");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().unwrap();
    assert!(list.is_empty());

    db.close().await;
}

#[tokio::test]
async fn should_reject_listing_unauthenticated_request_with_401() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let ca_id = Uuid::new_v4();
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (status, _body) = get_request(&app, &list_path, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.close().await;
}

#[tokio::test]
async fn should_reject_listing_when_ca_does_not_exist_with_404() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-ca-404";
    let obj = Uuid::new_v4();
    let (_inst_id, _ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let non_existent_ca = Uuid::new_v4();
    let list_path = format!("/api/v1/academic-centers/{non_existent_ca}/demands");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    db.close().await;
}

#[tokio::test]
async fn should_reject_listing_when_ca_is_inactive_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-ca-inactive";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "inactive", // CA inativo
    )
    .await;

    let jwt = token(uid, obj);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_inactive");

    db.close().await;
}

#[tokio::test]
async fn should_reject_listing_when_student_membership_not_active_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-no-membership";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        false, // Sem vínculo estudantil ativo
        None,
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "student_membership_required");

    db.close().await;
}

#[tokio::test]
async fn should_reject_institutional_listing_when_student_not_in_course_or_management_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-mec-eng";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Engenharia Mecânica"), // Curso divergente
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands?forum=institutional");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "user_not_permitted_in_ca");

    db.close().await;
}

#[tokio::test]
async fn should_reject_listing_with_invalid_forum_query_param_with_400() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-valid";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands?forum=forum_invalido");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "invalid_forum");

    db.close().await;
}

#[tokio::test]
async fn should_reject_listing_with_invalid_status_query_param_with_400() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-valid-status";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        uid,
        obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    let jwt = token(uid, obj);
    let list_path = format!("/api/v1/academic-centers/{ca_id}/demands?status=status_invalido");
    let (status, body) = get_request(&app, &list_path, Some(&jwt)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "invalid_status");

    db.close().await;
}
