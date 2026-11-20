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
