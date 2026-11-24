//! Testes de integração HTTP para o módulo de demandas.
//!
//! Exercita a criação de demandas nos dois fóruns (espaço físico e institucional),
//! validações de autenticação e regras de negócio de pertencimento ao CA/curso.

use crate::support::TestDatabase;
use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::Request;
use axum::http::StatusCode;
use backend::api::state::AppState;
use backend::infrastructure::database::PgDatabase;
use backend::test_support::local_identity_clients;
use chrono::Utc;
use jsonwebtoken::Algorithm;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::encode;
use serde_json::Value;
use serde_json::json;
use sqlx::PgPool;
use sqlx::Row;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::path;
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
/// Envia uma requisição PATCH em memória e interpreta seu corpo e status code.
async fn patch_request(
    app: &Router,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().uri(path).method("PATCH");

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
/// Cria e vincula um estudante adicional com verificação institucional válida à mesma instituição.
async fn create_test_student(
    pool: &PgPool,
    institution_id: Uuid,
    firebase_uid: &str,
    object_id: Uuid,
    is_student: bool,
    course_name: Option<&str>,
) -> Uuid {
    let institution = sqlx::query(
        "SELECT microsoft_tenant_id, microsoft_student_group_id FROM institution WHERE id = $1",
    )
    .bind(institution_id)
    .fetch_one(pool)
    .await
    .expect("should find institution");

    let tenant_id: Uuid = institution.get("microsoft_tenant_id");
    let group_id: Uuid = institution.get("microsoft_student_group_id");

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid, status) VALUES ($1, $2, 'incomplete')")
        .bind(user_id)
        .bind(firebase_uid)
        .execute(pool)
        .await
        .expect("should insert app_user");

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

    user_id
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
    let (inst_id, ca_id, _user_a_id) = create_test_context(
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
    let _user_b_id = create_test_student(
        &db.pool,
        inst_id,
        uid_b,
        obj_b,
        true,
        Some("Engenharia Mecânica"),
    )
    .await;

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
    let jwt_b = token_with_email(uid_b, obj_b, "241098766@aluno.unb.br");
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
    let (inst_id, ca_id, _user_a_id) = create_test_context(
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
    let _user_b_id = create_test_student(
        &db.pool,
        inst_id,
        uid_b,
        obj_b,
        true,
        Some("Ciência da Computação"),
    )
    .await;

    let jwt_a = token(uid_a, obj_a);
    let jwt_b = token_with_email(uid_b, obj_b, "241098766@aluno.unb.br");
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
        "archived", // CA arquivado / inativo
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
#[tokio::test]
async fn should_allow_ca_manager_to_update_physical_space_demand_status_with_opinion() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-user-caer";
    let manager_obj = Uuid::new_v4();
    let (_inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Engenharia de Redes",
        "CAER",
        manager_uid,
        manager_obj,
        true,
        Some("Engenharia de Redes"),
        true, // is_ca_manager = true
        "active",
    )
    .await;

    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    // Cria demanda de espaço físico
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Reparo no ar-condicionado",
            "description": "Ar-condicionado da sala do CA parou de refrigerar"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // 1. Gestor altera status para em análise com parecer
    let patch_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p1_status, p1_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Demanda recebida pela diretoria de patrimônio para triagem"
        })),
    )
    .await;

    assert_eq!(p1_status, StatusCode::OK, "{p1_body}");
    assert_eq!(p1_body["status"], "in_analysis");
    assert_eq!(p1_body["id"], demand_id);
    let history_1 = p1_body["statusHistory"].as_array().expect("statusHistory");
    assert_eq!(history_1.len(), 2);
    assert_eq!(history_1[1]["toStatus"], "in_analysis");
    assert_eq!(
        history_1[1]["opinion"],
        "Demanda recebida pela diretoria de patrimônio para triagem"
    );

    // 2. Gestor conclui demanda via rota de conveniência sem prefixo /api/v1
    let alias_patch_path = format!("/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p2_status, p2_body) = patch_request(
        &app,
        &alias_patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "completed",
            "opinion": "Manutenção realizada e equipamento em pleno funcionamento"
        })),
    )
    .await;

    assert_eq!(p2_status, StatusCode::OK, "{p2_body}");
    assert_eq!(p2_body["status"], "completed");
    let history_2 = p2_body["statusHistory"].as_array().expect("statusHistory");
    assert_eq!(history_2.len(), 3);
    assert_eq!(history_2[2]["toStatus"], "completed");
    assert_eq!(
        history_2[2]["opinion"],
        "Manutenção realizada e equipamento em pleno funcionamento"
    );

    db.close().await;
}
#[tokio::test]
async fn should_allow_ca_manager_to_forward_institutional_demand_to_coordination() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-inst-caer";
    let manager_obj = Uuid::new_v4();
    let (_inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Engenharia de Redes",
        "CAER",
        manager_uid,
        manager_obj,
        true,
        Some("Engenharia de Redes"),
        true, // is_ca_manager = true
        "active",
    )
    .await;

    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    // Cria demanda institucional
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_jwt),
        Some(json!({
            "forum": "institutional",
            "courseName": "Engenharia de Redes",
            "title": "Revisão da ementa de Redes 1",
            "description": "Necessidade de atualização bibliográfica e de laboratório"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // 1. Move para in_analysis
    let patch_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p1_status, p1_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Triagem acadêmica inicial concluída"
        })),
    )
    .await;
    assert_eq!(p1_status, StatusCode::OK, "{p1_body}");

    // 2. Encaminha à coordenação
    let (p2_status, p2_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "forwarded_to_coordination",
            "opinion": "Pauta formalizada no processo SEI e remetida à coordenação do curso"
        })),
    )
    .await;

    assert_eq!(p2_status, StatusCode::OK, "{p2_body}");
    assert_eq!(p2_body["status"], "forwarded_to_coordination");

    // 3. Conclui com parecer de resolução
    let (p3_status, p3_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "completed",
            "opinion": "Coordenação aprovou a atualização da ementa para o próximo semestre"
        })),
    )
    .await;

    assert_eq!(p3_status, StatusCode::OK, "{p3_body}");
    assert_eq!(p3_body["status"], "completed");

    db.close().await;
}
#[tokio::test]
async fn should_reject_forwarding_physical_space_demand_to_coordination_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-ps-reject";
    let manager_obj = Uuid::new_v4();
    let (_inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        manager_uid,
        manager_obj,
        true,
        Some("Ciência da Computação"),
        true, // is_ca_manager = true
        "active",
    )
    .await;

    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Porta danificada"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    let patch_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p_status, p_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "forwarded_to_coordination",
            "opinion": "Solicitando auxílio da coordenação para manutenção"
        })),
    )
    .await;

    assert_eq!(p_status, StatusCode::UNPROCESSABLE_ENTITY, "{p_body}");
    assert_eq!(
        p_body["error"]["code"],
        "coordination_forwarding_not_allowed"
    );

    db.close().await;
}
#[tokio::test]
async fn should_reject_status_update_without_opinion_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-no-opinion";
    let manager_obj = Uuid::new_v4();
    let (_inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        manager_uid,
        manager_obj,
        true,
        Some("Ciência da Computação"),
        true,
        "active",
    )
    .await;

    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Lâmpada queimada"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    let patch_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p_status, p_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "   "
        })),
    )
    .await;

    assert_eq!(p_status, StatusCode::UNPROCESSABLE_ENTITY, "{p_body}");
    assert_eq!(p_body["error"]["code"], "opinion_required");

    db.close().await;
}
#[tokio::test]
async fn should_reject_status_update_by_regular_student_without_management_role_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let inst_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO institution (id, name, acronym, microsoft_tenant_id, microsoft_student_group_id, status)
         VALUES ($1, 'Universidade de Brasília', 'UnB', $2, $3, 'active')",
    )
    .bind(inst_id)
    .bind(tenant_id)
    .bind(group_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let manager_uid = "ca-creator-user";
    let manager_id = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid, status) VALUES ($1, $2, 'incomplete')")
        .bind(manager_id)
        .bind(manager_uid)
        .execute(&db.pool)
        .await
        .unwrap();

    let ca_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status)
         VALUES ($1, $2, $3, 'Centro Acadêmico de Engenharia de Redes', 'CAER', 'CA', 'active')",
    )
    .bind(ca_id)
    .bind(inst_id)
    .bind(manager_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // Cria estudante regular sem cargo de gestão
    let student_uid = "student-regular-no-manager";
    let student_obj = Uuid::new_v4();
    let student_id = create_test_student(
        &db.pool,
        inst_id,
        student_uid,
        student_obj,
        true,
        Some("Engenharia de Redes"),
    )
    .await;

    let student_jwt = token_with_email(student_uid, student_obj, "241098766@aluno.unb.br");

    // Estudante cria demanda
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&student_jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Reparo na fechadura"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // Estudante tenta atualizar status da demanda -> recusado com 403
    let patch_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p_status, p_body) = patch_request(
        &app,
        &patch_path,
        Some(&student_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Tentativa não autorizada por discente regular"
        })),
    )
    .await;

    assert_eq!(p_status, StatusCode::FORBIDDEN, "{p_body}");
    assert_eq!(p_body["error"]["code"], "user_not_permitted_in_ca");

    let _ = student_id;
    db.close().await;
}
#[tokio::test]
async fn should_reject_status_update_by_manager_of_another_ca_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    // CA 1: CAER
    let manager_1_uid = "manager-caer";
    let manager_1_obj = Uuid::new_v4();
    let (inst_id, ca_1_id, _manager_1_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Engenharia de Redes",
        "CAER",
        manager_1_uid,
        manager_1_obj,
        true,
        Some("Engenharia de Redes"),
        true,
        "active",
    )
    .await;

    let manager_1_jwt = token_with_email(manager_1_uid, manager_1_obj, "241098765@aluno.unb.br");

    // Cria demanda no CA 1
    let create_path = format!("/api/v1/academic-centers/{ca_1_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_1_jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Limpeza necessária na sala de reuniões"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // CA 2: CACC, com gestor 2
    let manager_2_uid = "manager-cacc";
    let manager_2_obj = Uuid::new_v4();
    let manager_2_id = create_test_student(
        &db.pool,
        inst_id,
        manager_2_uid,
        manager_2_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;

    let ca_2_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status)
         VALUES ($1, $2, $3, 'Centro Acadêmico de Ciência da Computação', 'CACC', 'CA 2', 'active')",
    )
    .bind(ca_2_id)
    .bind(inst_id)
    .bind(manager_2_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let manager_2_jwt = token_with_email(manager_2_uid, manager_2_obj, "241098766@aluno.unb.br");

    // Gestor 2 tenta atualizar a demanda do CA 1 -> 403
    let patch_path = format!("/api/v1/academic-centers/{ca_1_id}/demands/{demand_id}/status");
    let (p_status, p_body) = patch_request(
        &app,
        &patch_path,
        Some(&manager_2_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Intervenção cruzada indevida entre CAs"
        })),
    )
    .await;

    assert_eq!(p_status, StatusCode::FORBIDDEN, "{p_body}");
    assert_eq!(p_body["error"]["code"], "user_not_permitted_in_ca");

    db.close().await;
}
#[tokio::test]
async fn should_reject_status_update_for_demand_belonging_to_another_ca_with_404() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    // CA 1: CAER com gestor 1
    let manager_1_uid = "manager-caer-iso";
    let manager_1_obj = Uuid::new_v4();
    let (inst_id, ca_1_id, _manager_1_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Engenharia de Redes",
        "CAER",
        manager_1_uid,
        manager_1_obj,
        true,
        Some("Engenharia de Redes"),
        true,
        "active",
    )
    .await;

    let manager_1_jwt = token_with_email(manager_1_uid, manager_1_obj, "241098765@aluno.unb.br");

    // CA 2: CACC com gestor 2
    let manager_2_uid = "manager-cacc-iso";
    let manager_2_obj = Uuid::new_v4();
    let manager_2_id = create_test_student(
        &db.pool,
        inst_id,
        manager_2_uid,
        manager_2_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;

    let ca_2_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, created_by_id, name, acronym, description, status)
         VALUES ($1, $2, $3, 'Centro Acadêmico de Ciência da Computação', 'CACC', 'CA 2', 'active')",
    )
    .bind(ca_2_id)
    .bind(inst_id)
    .bind(manager_2_id)
    .execute(&db.pool)
    .await
    .unwrap();

    let manager_2_jwt = token_with_email(manager_2_uid, manager_2_obj, "241098766@aluno.unb.br");

    // Cria demanda no CA 2
    let create_path_ca2 = format!("/api/v1/academic-centers/{ca_2_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path_ca2,
        Some(&manager_2_jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Demanda criada e restrita ao CA 2"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_ca2_id = c_body["id"].as_str().unwrap();

    // Gestor 1 tenta atualizar a demanda do CA 2 passando a URL do CA 1 -> 404 (isolamento estrito)
    let patch_path_ca1 =
        format!("/api/v1/academic-centers/{ca_1_id}/demands/{demand_ca2_id}/status");
    let (p_status, p_body) = patch_request(
        &app,
        &patch_path_ca1,
        Some(&manager_1_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Tentando atualizar demanda de outro CA"
        })),
    )
    .await;

    assert_eq!(p_status, StatusCode::NOT_FOUND, "{p_body}");
    assert_eq!(p_body["error"]["code"], "demand_not_found");

    db.close().await;
}
#[tokio::test]
async fn should_allow_getting_demand_details_and_history_with_200() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-get-details";
    let manager_obj = Uuid::new_v4();
    let (inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Engenharia de Redes",
        "CAER",
        manager_uid,
        manager_obj,
        true,
        Some("Engenharia de Redes"),
        true,
        "active",
    )
    .await;

    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    // Cria demanda
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Reparo de tomada",
            "description": "Tomada com defeito no laboratório"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // Atualiza status para in_analysis
    let patch_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (p_status, _) = patch_request(
        &app,
        &patch_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Técnico acionado para avaliação"
        })),
    )
    .await;
    assert_eq!(p_status, StatusCode::OK);

    // Consulta detalhes da demanda por outro estudante qualquer
    let student_uid = "student-observer";
    let student_obj = Uuid::new_v4();
    create_test_student(
        &db.pool,
        inst_id,
        student_uid,
        student_obj,
        true,
        Some("Engenharia Elétrica"),
    )
    .await;

    let student_jwt = token_with_email(student_uid, student_obj, "241098766@aluno.unb.br");

    let get_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}");
    let (g_status, g_body) = get_request(&app, &get_path, Some(&student_jwt)).await;

    assert_eq!(g_status, StatusCode::OK, "{g_body}");
    assert_eq!(g_body["id"], demand_id);
    assert_eq!(g_body["status"], "in_analysis");
    let history = g_body["statusHistory"].as_array().expect("statusHistory");
    assert_eq!(history.len(), 2);
    assert_eq!(history[1]["toStatus"], "in_analysis");
    assert_eq!(history[1]["opinion"], "Técnico acionado para avaliação");

    // Consulta pela rota de conveniência sem /api/v1
    let alias_get_path = format!("/academic-centers/{ca_id}/demands/{demand_id}");
    let (ga_status, ga_body) = get_request(&app, &alias_get_path, Some(&student_jwt)).await;
    assert_eq!(ga_status, StatusCode::OK, "{ga_body}");
    assert_eq!(ga_body["id"], demand_id);

    db.close().await;
}
#[tokio::test]
async fn should_reject_student_of_another_course_from_getting_institutional_demand_details() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-inst-details";
    let manager_obj = Uuid::new_v4();
    let (inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Engenharia de Redes",
        "CAER",
        manager_uid,
        manager_obj,
        true,
        Some("Engenharia de Redes"),
        true,
        "active",
    )
    .await;

    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    // Cria demanda institucional do curso de Engenharia de Redes
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&manager_jwt),
        Some(json!({
            "forum": "institutional",
            "courseName": "Engenharia de Redes",
            "title": "Ajuste na grade curricular",
            "description": "Discussão de pré-requisitos"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // Estudante de Letras (não pertence ao curso atendido pelo CA)
    let outsider_uid = "student-outsider-letras";
    let outsider_obj = Uuid::new_v4();
    create_test_student(
        &db.pool,
        inst_id,
        outsider_uid,
        outsider_obj,
        true,
        Some("Letras"),
    )
    .await;

    let outsider_jwt = token_with_email(outsider_uid, outsider_obj, "241098766@aluno.unb.br");

    let get_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}");
    let (g_status, g_body) = get_request(&app, &get_path, Some(&outsider_jwt)).await;

    assert_eq!(g_status, StatusCode::FORBIDDEN, "{g_body}");
    assert_eq!(g_body["error"]["code"], "user_not_permitted_in_ca");

    db.close().await;
}
#[tokio::test]
async fn should_post_comment_successfully_by_demand_author() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-author-cmt";
    let obj = Uuid::new_v4();
    let (_inst_id, ca_id, user_id) = create_test_context(
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
    let demands_path = format!("/api/v1/academic-centers/{ca_id}/demands");

    // Cria a demanda inicial
    let (status, demand_body) = request(
        &app,
        &demands_path,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Reparo da janela",
            "description": "A janela da sala do CA está com o fecho quebrado."
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{demand_body}");

    let demand_id = demand_body["id"].as_str().expect("demand id");
    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");

    let comment_payload = json!({
        "content": "Anexei também a foto detalhada do fecho emperrado."
    });

    let (c_status, c_body) = request(&app, &comment_path, Some(&jwt), Some(comment_payload)).await;

    assert_eq!(
        c_status,
        StatusCode::CREATED,
        "autor deve conseguir comentar: {c_body}"
    );
    assert_eq!(c_body["demandId"], demand_id);
    assert_eq!(c_body["authorId"], user_id.to_string());
    assert_eq!(c_body["authorRole"], "student");
    assert_eq!(
        c_body["content"],
        "Anexei também a foto detalhada do fecho emperrado."
    );

    // Confere no banco de dados
    let comment_id = Uuid::parse_str(c_body["id"].as_str().unwrap()).unwrap();
    let row = sqlx::query(
        "SELECT demand_id, author_id, author_role, content FROM demand_comment WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_one(&db.pool)
    .await
    .expect("comentário deve existir no banco");

    let db_demand_id: Uuid = row.get("demand_id");
    let db_author_id: Uuid = row.get("author_id");
    let db_role: String = row.get("author_role");
    let db_content: String = row.get("content");

    assert_eq!(db_demand_id.to_string(), demand_id);
    assert_eq!(db_author_id, user_id);
    assert_eq!(db_role, "student");
    assert_eq!(
        db_content,
        "Anexei também a foto detalhada do fecho emperrado."
    );

    db.close().await;
}
#[tokio::test]
async fn should_post_official_response_successfully_by_ca_management() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-cmt-1";
    let manager_obj = Uuid::new_v4();
    let (inst_id, ca_id, manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        manager_uid,
        manager_obj,
        true,
        Some("Ciência da Computação"),
        true, // is_ca_manager = true
        "active",
    )
    .await;

    // Estudante secundário autor da demanda
    let student_uid = "student-author-secondary";
    let student_obj = Uuid::new_v4();
    let _student_id = create_test_student(
        &db.pool,
        inst_id,
        student_uid,
        student_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;

    let student_jwt = token_with_email(student_uid, student_obj, "241098765@aluno.unb.br");
    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098766@aluno.unb.br");

    // Estudante cria a demanda
    let demands_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (status, demand_body) = request(
        &app,
        &demands_path,
        Some(&student_jwt),
        Some(json!({
            "forum": "institutional",
            "title": "Revisão curricular",
            "description": "Necessidade de atualização de bibliografia.",
            "course_name": "Ciência da Computação"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{demand_body}");
    let demand_id = demand_body["id"].as_str().expect("demand id");

    // Gestor responde oficialmente à demanda
    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");
    let response_payload = json!({
        "content": "Entramos em contato com a coordenação para agendar reunião sobre a ementa."
    });

    let (c_status, c_body) = request(
        &app,
        &comment_path,
        Some(&manager_jwt),
        Some(response_payload),
    )
    .await;

    assert_eq!(
        c_status,
        StatusCode::CREATED,
        "gestor deve conseguir responder oficialmente: {c_body}"
    );
    assert_eq!(c_body["demandId"], demand_id);
    assert_eq!(c_body["authorId"], manager_id.to_string());
    assert_eq!(c_body["authorRole"], "ca_management");
    assert_eq!(
        c_body["content"],
        "Entramos em contato com a coordenação para agendar reunião sobre a ementa."
    );

    // Estudante autor visualiza a resposta oficial da gestão
    let (list_status, list_body) = get_request(&app, &comment_path, Some(&student_jwt)).await;
    assert_eq!(list_status, StatusCode::OK);
    let comments = list_body.as_array().expect("array de comentários");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0]["authorRole"], "ca_management");
    assert_eq!(comments[0]["authorId"], manager_id.to_string());

    db.close().await;
}
#[tokio::test]
async fn should_list_comments_in_chronological_order() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-timeline";
    let manager_obj = Uuid::new_v4();
    let (inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        manager_uid,
        manager_obj,
        true,
        Some("Ciência da Computação"),
        true, // gestor
        "active",
    )
    .await;

    let student_uid = "student-timeline";
    let student_obj = Uuid::new_v4();
    let _student_id = create_test_student(
        &db.pool,
        inst_id,
        student_uid,
        student_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;

    let student_jwt = token_with_email(student_uid, student_obj, "241098765@aluno.unb.br");
    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098766@aluno.unb.br");

    // Estudante cria demanda
    let demands_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (_, demand_body) = request(
        &app,
        &demands_path,
        Some(&student_jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Ar-condicionado",
            "description": "Equipamento parado"
        })),
    )
    .await;
    let demand_id = demand_body["id"].as_str().unwrap();
    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");

    // 1. Estudante posta comentário 1
    let (s1, _) = request(
        &app,
        &comment_path,
        Some(&student_jwt),
        Some(json!({ "content": "Primeira mensagem do estudante" })),
    )
    .await;
    assert_eq!(s1, StatusCode::CREATED);

    // 2. Gestor posta resposta 2
    let (s2, _) = request(
        &app,
        &comment_path,
        Some(&manager_jwt),
        Some(json!({ "content": "Resposta da gestão do CA" })),
    )
    .await;
    assert_eq!(s2, StatusCode::CREATED);

    // 3. Estudante posta réplica 3
    let (s3, _) = request(
        &app,
        &comment_path,
        Some(&student_jwt),
        Some(json!({ "content": "Obrigado pelo retorno da gestão" })),
    )
    .await;
    assert_eq!(s3, StatusCode::CREATED);

    // Lista linha do tempo
    let (list_status, list_body) = get_request(&app, &comment_path, Some(&student_jwt)).await;
    assert_eq!(list_status, StatusCode::OK);
    let comments = list_body.as_array().expect("array de comentários");
    assert_eq!(comments.len(), 3);

    assert_eq!(comments[0]["authorRole"], "student");
    assert_eq!(comments[0]["content"], "Primeira mensagem do estudante");

    assert_eq!(comments[1]["authorRole"], "ca_management");
    assert_eq!(comments[1]["content"], "Resposta da gestão do CA");

    assert_eq!(comments[2]["authorRole"], "student");
    assert_eq!(comments[2]["content"], "Obrigado pelo retorno da gestão");

    db.close().await;
}
#[tokio::test]
async fn should_reject_comment_by_unauthorized_third_party_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let author_uid = "student-author-legit";
    let author_obj = Uuid::new_v4();
    let (inst_id, ca_id, _author_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        author_uid,
        author_obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;

    // Terceiro: estudante com vínculo ativo, mas não é autor nem gestor
    let third_party_uid = "student-third-party";
    let third_party_obj = Uuid::new_v4();
    let _third_party_id = create_test_student(
        &db.pool,
        inst_id,
        third_party_uid,
        third_party_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;

    let author_jwt = token_with_email(author_uid, author_obj, "241098765@aluno.unb.br");
    let third_party_jwt =
        token_with_email(third_party_uid, third_party_obj, "241098766@aluno.unb.br");

    // Autor cria demanda
    let demands_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (_, demand_body) = request(
        &app,
        &demands_path,
        Some(&author_jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Demanda privada do autor"
        })),
    )
    .await;
    let demand_id = demand_body["id"].as_str().unwrap();

    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");

    // Terceiro tenta comentar -> 403
    let (c_status, c_body) = request(
        &app,
        &comment_path,
        Some(&third_party_jwt),
        Some(json!({ "content": "Comentário de terceiro não autorizado" })),
    )
    .await;
    assert_eq!(c_status, StatusCode::FORBIDDEN, "{c_body}");
    assert_eq!(c_body["error"]["code"], "user_not_permitted_in_ca");

    // Terceiro tenta visualizar comentários -> 403
    let (l_status, l_body) = get_request(&app, &comment_path, Some(&third_party_jwt)).await;
    assert_eq!(l_status, StatusCode::FORBIDDEN, "{l_body}");
    assert_eq!(l_body["error"]["code"], "user_not_permitted_in_ca");

    db.close().await;
}
#[tokio::test]
async fn should_reject_comment_with_empty_or_whitespace_content_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-empty-cmt";
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
    let demands_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (_, demand_body) = request(
        &app,
        &demands_path,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Demanda válida"
        })),
    )
    .await;
    let demand_id = demand_body["id"].as_str().unwrap();

    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");

    let (status, body) = request(
        &app,
        &comment_path,
        Some(&jwt),
        Some(json!({ "content": "   \t \n  " })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "empty_comment");

    db.close().await;
}
#[tokio::test]
async fn should_reject_comment_exceeding_max_length_with_422() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-long-cmt";
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
    let demands_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (_, demand_body) = request(
        &app,
        &demands_path,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Demanda válida"
        })),
    )
    .await;
    let demand_id = demand_body["id"].as_str().unwrap();

    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");

    let long_text = "c".repeat(2001);
    let (status, body) = request(
        &app,
        &comment_path,
        Some(&jwt),
        Some(json!({ "content": long_text })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "comment_too_long");

    db.close().await;
}
#[tokio::test]
async fn should_reject_comment_for_nonexistent_ca_with_404() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-valid-ca-404";
    let obj = Uuid::new_v4();
    let (_inst_id, _real_ca_id, _user_id) = create_test_context(
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
    let demand_id = Uuid::new_v4();
    let comment_path =
        format!("/api/v1/academic-centers/{non_existent_ca}/demands/{demand_id}/comments");

    let (status, body) = request(
        &app,
        &comment_path,
        Some(&jwt),
        Some(json!({ "content": "Comentário para CA inexistente" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "academic_center_not_found");

    db.close().await;
}
#[tokio::test]
async fn should_reject_comment_for_nonexistent_demand_with_404() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-valid-demand-404";
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
    let non_existent_demand = Uuid::new_v4();
    let comment_path =
        format!("/api/v1/academic-centers/{ca_id}/demands/{non_existent_demand}/comments");

    let (status, body) = request(
        &app,
        &comment_path,
        Some(&jwt),
        Some(json!({ "content": "Comentário para demanda inexistente" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "demand_not_found");

    db.close().await;
}
#[tokio::test]
async fn should_reject_comment_when_demand_belongs_to_another_ca_with_400() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let uid = "student-ca-isolation";
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

    // Cria CA 2 na mesma instituição
    let ca_2 = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO academic_center (id, institution_id, name, acronym, description, status)
         VALUES ($1, $2, 'Centro Acadêmico de Redes', 'CAER', 'CA 2', 'active')",
    )
    .bind(ca_2)
    .bind(inst_id)
    .execute(&db.pool)
    .await
    .expect("insert ca 2");

    let jwt = token(uid, obj);

    // Cria demanda no CA 2
    let demands_path_ca2 = format!("/api/v1/academic-centers/{ca_2}/demands");
    let (_, demand_body) = request(
        &app,
        &demands_path_ca2,
        Some(&jwt),
        Some(json!({
            "forum": "physical_space",
            "description": "Demanda no CA 2"
        })),
    )
    .await;
    let demand_id = demand_body["id"].as_str().unwrap();

    // Tenta comentar na demanda apontando para CA 1 na rota
    let comment_path_ca1 = format!("/api/v1/academic-centers/{ca_1}/demands/{demand_id}/comments");
    let (status, body) = request(
        &app,
        &comment_path_ca1,
        Some(&jwt),
        Some(json!({ "content": "Tentativa de comentar através do CA 1" })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "demand_belongs_to_another_ca");

    db.close().await;
}
#[tokio::test]
async fn should_reject_unauthenticated_comment_with_401() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let ca_id = Uuid::new_v4();
    let demand_id = Uuid::new_v4();
    let comment_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/comments");

    let (status, body) = request(
        &app,
        &comment_path,
        None,
        Some(json!({ "content": "Sem token" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["error"]["code"], "unauthorized");

    db.close().await;
}
#[tokio::test]
async fn should_dispatch_notification_to_student_when_ca_manager_updates_status_and_allow_reading()
{
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-notif-http-1";
    let manager_obj = Uuid::new_v4();
    let (inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        manager_uid,
        manager_obj,
        true,
        Some("Ciência da Computação"),
        true, // is_ca_manager = true
        "active",
    )
    .await;
    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    let student_uid = "student-notif-http-author";
    let student_obj = Uuid::new_v4();
    let student_id = create_test_student(
        &db.pool,
        inst_id,
        student_uid,
        student_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;
    let student_jwt = token_with_email(student_uid, student_obj, "241098766@aluno.unb.br");

    // Estudante cria demanda
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&student_jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Projetor com defeito",
            "description": "O projetor da sala de reuniões do CA não liga"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // Gestor atualiza status da demanda para em análise com parecer
    let status_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (u_status, u_body) = patch_request(
        &app,
        &status_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Técnico acionado para avaliar a lâmpada do projetor"
        })),
    )
    .await;
    assert_eq!(u_status, StatusCode::OK, "{u_body}");
    assert_eq!(u_body["status"], "in_analysis");

    // Estudante autor consulta notificações
    let notifs_path = "/api/v1/demands/notifications";
    let (n_status, n_body) = get_request(&app, notifs_path, Some(&student_jwt)).await;
    assert_eq!(n_status, StatusCode::OK, "{n_body}");

    let notifs = n_body.as_array().expect("deve retornar array");
    assert_eq!(
        notifs.len(),
        1,
        "deve existir 1 notificação para o estudante"
    );
    let notif = &notifs[0];
    assert_eq!(notif["demandId"], demand_id);
    assert_eq!(notif["recipientUserId"], student_id.to_string());
    assert_eq!(notif["academicCenterId"], ca_id.to_string());
    assert_eq!(notif["eventType"], "status_updated");
    assert_eq!(notif["title"], "Demanda em análise");
    assert!(
        notif["message"]
            .as_str()
            .unwrap()
            .contains("Técnico acionado para avaliar a lâmpada do projetor")
    );
    assert!(notif["readAt"].is_null());

    let notification_id = notif["id"].as_str().unwrap();

    // Estudante marca notificação como lida
    let read_path = format!("/api/v1/demands/notifications/{notification_id}/read");
    let (r_status, r_body) = patch_request(&app, &read_path, Some(&student_jwt), None).await;
    assert_eq!(r_status, StatusCode::OK, "{r_body}");
    assert_eq!(r_body["id"], notification_id);
    assert!(!r_body["readAt"].is_null());

    // Consulta filtrando por não lidas (deve retornar vazio)
    let unread_path = "/api/v1/demands/notifications?unreadOnly=true";
    let (un_status, un_body) = get_request(&app, unread_path, Some(&student_jwt)).await;
    assert_eq!(un_status, StatusCode::OK, "{un_body}");
    let unread_list = un_body.as_array().unwrap();
    assert!(
        unread_list.is_empty(),
        "não deve haver notificações não lidas"
    );

    db.close().await;
}
#[tokio::test]
async fn should_reject_marking_notification_as_read_by_different_student_with_403() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let manager_uid = "manager-notif-403";
    let manager_obj = Uuid::new_v4();
    let (inst_id, ca_id, _manager_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        manager_uid,
        manager_obj,
        true,
        Some("Ciência da Computação"),
        true,
        "active",
    )
    .await;
    let manager_jwt = token_with_email(manager_uid, manager_obj, "241098765@aluno.unb.br");

    let student_owner_uid = "student-owner";
    let student_owner_obj = Uuid::new_v4();
    create_test_student(
        &db.pool,
        inst_id,
        student_owner_uid,
        student_owner_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;
    let student_owner_jwt = token_with_email(
        student_owner_uid,
        student_owner_obj,
        "241098766@aluno.unb.br",
    );

    let student_intruder_uid = "student-intruder";
    let student_intruder_obj = Uuid::new_v4();
    create_test_student(
        &db.pool,
        inst_id,
        student_intruder_uid,
        student_intruder_obj,
        true,
        Some("Ciência da Computação"),
    )
    .await;
    let student_intruder_jwt = token_with_email(
        student_intruder_uid,
        student_intruder_obj,
        "241098767@aluno.unb.br",
    );

    // Estudante A cria demanda
    let create_path = format!("/api/v1/academic-centers/{ca_id}/demands");
    let (c_status, c_body) = request(
        &app,
        &create_path,
        Some(&student_owner_jwt),
        Some(json!({
            "forum": "physical_space",
            "title": "Janela emperrada",
            "description": "A janela não abre"
        })),
    )
    .await;
    assert_eq!(c_status, StatusCode::CREATED, "{c_body}");
    let demand_id = c_body["id"].as_str().unwrap();

    // Gestor atualiza status
    let status_path = format!("/api/v1/academic-centers/{ca_id}/demands/{demand_id}/status");
    let (u_status, u_body) = patch_request(
        &app,
        &status_path,
        Some(&manager_jwt),
        Some(json!({
            "status": "in_analysis",
            "opinion": "Verificando mecanismo da janela"
        })),
    )
    .await;
    assert_eq!(u_status, StatusCode::OK, "{u_body}");

    // Estudante A obtém id da notificação
    let notifs_path = "/api/v1/demands/notifications";
    let (n_status, n_body) = get_request(&app, notifs_path, Some(&student_owner_jwt)).await;
    assert_eq!(n_status, StatusCode::OK, "{n_body}");
    let notification_id = n_body[0]["id"].as_str().unwrap();

    // Estudante B tenta marcar notificação do Estudante A como lida
    let read_path = format!("/api/v1/demands/notifications/{notification_id}/read");
    let (r_status, r_body) =
        patch_request(&app, &read_path, Some(&student_intruder_jwt), None).await;
    assert_eq!(r_status, StatusCode::FORBIDDEN, "{r_body}");
    assert_eq!(r_body["error"]["code"], "notification_forbidden");

    db.close().await;
}
#[tokio::test]
async fn should_return_404_when_notification_does_not_exist() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    let student_uid = "student-notif-404";
    let student_obj = Uuid::new_v4();
    let (_inst_id, _ca_id, _user_id) = create_test_context(
        &db.pool,
        "Centro Acadêmico de Computação",
        "CACC",
        student_uid,
        student_obj,
        true,
        Some("Ciência da Computação"),
        false,
        "active",
    )
    .await;
    let student_jwt = token_with_email(student_uid, student_obj, "241098765@aluno.unb.br");

    let fake_id = Uuid::new_v4();
    let read_path = format!("/api/v1/demands/notifications/{fake_id}/read");
    let (r_status, r_body) = patch_request(&app, &read_path, Some(&student_jwt), None).await;
    assert_eq!(r_status, StatusCode::NOT_FOUND, "{r_body}");
    assert_eq!(r_body["error"]["code"], "notification_not_found");

    db.close().await;
}
#[tokio::test]
async fn should_reject_unauthenticated_notifications_requests_with_401() {
    let db = TestDatabase::new().await;
    let (app, _jwks) = setup_app(&db).await;

    // GET sem autenticação
    let (g_status, g_body) = get_request(&app, "/api/v1/demands/notifications", None).await;
    assert_eq!(g_status, StatusCode::UNAUTHORIZED, "{g_body}");

    // PATCH sem autenticação
    let fake_id = Uuid::new_v4();
    let read_path = format!("/api/v1/demands/notifications/{fake_id}/read");
    let (p_status, p_body) = patch_request(&app, &read_path, None, None).await;
    assert_eq!(p_status, StatusCode::UNAUTHORIZED, "{p_body}");

    db.close().await;
}
