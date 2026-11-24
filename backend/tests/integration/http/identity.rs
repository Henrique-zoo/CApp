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
    config::HttpConfig,
    infrastructure::database::PgDatabase,
    test_support::{identity_router, local_identity_clients_with_config},
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
    token_with_provider(uid, object, "microsoft.com", "institucional@example.test")
}

fn token_with_provider(uid: &str, object: Uuid, provider: &str, email: &str) -> String {
    let now = Utc::now().timestamp();
    let mut header = Header::new(Algorithm::RS256);

    header.kid = Some("test-kid".into());

    encode(&header,&json!({"sub":uid,"aud":PROJECT,"iss":format!("https://securetoken.google.com/{PROJECT}"),"iat":now-60,"auth_time":now-120,"exp":now+3600,"email":email,"firebase":{"sign_in_provider":provider,"identities":{"microsoft.com":[object]}}}),&EncodingKey::from_rsa_pem(KEY.as_bytes()).unwrap()).unwrap()
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
            .method(if path == "/users/me/profile" {
                "PUT"
            } else {
                "POST"
            })
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
    graph_mocks_with_course(
        graph,
        tenant,
        group,
        object,
        student,
        Some(json!("Curso")),
        "graph@example.test",
    )
    .await;
}

/// Permite distinguir curso ausente de nulo e de texto vazio no Graph real simulado.
///
/// Registra no servidor `graph` uma resposta para organização, grupo e perfil,
/// cada qual esperada uma vez. `tenant`, `group` e `object` permitem confrontar os
/// identificadores com a instituição/token preparados pelo teste. `student` altera
/// somente a confirmação do grupo; não força a classificação de vínculo no backend.
///
/// `course: None` omite `jobTitle`; `Some(Value::Null)` devolve nulo explicitamente;
/// outros valores são enviados sem normalização para exercitar o cliente real.
/// `mail` é o endereço desta consulta, permitindo verificar que uma sincronização
/// posterior não substitui o e-mail interno. O helper monta expectativas HTTP,
/// sem gravar evidência ou simular a decisão de acesso que está sob teste.
async fn graph_mocks_with_course(
    graph: &MockServer,
    tenant: Uuid,
    group: Uuid,
    object: Uuid,
    student: bool,
    course: Option<Value>,
    mail: &str,
) {
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

    let mut body = json!({"id": object, "displayName": "Pessoa", "mail": mail});

    if let Some(course) = course {
        body["jobTitle"] = course;
    }

    Mock::given(method("GET"))
        .and(path("/v1.0/me"))
        .and(header("authorization", "Bearer graph-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(graph)
        .await;
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

    let http_config = HttpConfig::default_for_test();
    let http = http_config.build_client().unwrap();
    let state = AppState::new(
        PgDatabase::new(pool.clone()),
        local_identity_clients_with_config(
            PROJECT.into(),
            http,
            format!("{}/keys", jwks.uri()),
            format!("{}/v1.0", graph.uri()),
            http_config.clone(),
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

    for (group_member, course) in [
        (true, Some(json!("Curso"))),
        (false, Some(json!("Professor"))),
        (true, None),
        (true, Some(Value::Null)),
        (true, Some(json!(""))),
        (true, Some(json!(" \t\n"))),
    ] {
        let student = group_member
            && course
                .as_ref()
                .and_then(Value::as_str)
                .is_some_and(|course| !course.trim().is_empty());
        let uid = Uuid::new_v4().to_string();
        let jwt = token(&uid, object);
        let (status, body) = request(&app, "/protected", &jwt, None).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"]["code"], "institutional_verification_required");

        let (status, body) = request(&app, "/auth/session", &jwt, None).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "account_not_provisioned");
        assert!(graph.received_requests().await.unwrap().is_empty());

        graph_mocks_with_course(
            &graph,
            tenant,
            group,
            object,
            group_member,
            course.clone(),
            "graph@example.test",
        )
        .await;

        let (status, body) = request(
            &app,
            "/auth/institutional/verify",
            &jwt,
            Some(json!({"microsoftAccessToken":"graph-token"})),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "{body}");

        let provisioned_user_id = Uuid::parse_str(body["user"]["id"].as_str().unwrap()).unwrap();

        assert_eq!(body["user"]["status"], "incomplete");
        assert_eq!(body["suggestedEmail"], "graph@example.test");
        assert!(body["verification"]["profile"].get("email").is_none());
        assert!(body["user"].get("institutionalEmail").is_none());
        assert!(body["user"]["nickname"].is_null());
        assert_eq!(
            body["verification"]["profile"]["affiliation"]["type"],
            if student { "student" } else { "nonStudent" }
        );

        if student {
            assert_eq!(
                body["verification"]["profile"]["affiliation"]["courseName"],
                "Curso"
            );
        } else {
            assert!(
                body["verification"]["profile"]["affiliation"]
                    .get("courseName")
                    .is_none()
            );
        }

        let stored:bool=sqlx::query_scalar("SELECT student_membership_confirmed FROM institutional_verification WHERE user_id=(SELECT id FROM app_user WHERE firebase_uid=$1)").bind(&uid).fetch_one(&pool).await.unwrap();

        assert_eq!(stored, student);

        graph.verify().await;
        graph.reset().await;

        let (status, body) = request(&app, "/protected", &jwt, None).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({"uid":uid,"institution":institution}));
        assert!(graph.received_requests().await.unwrap().is_empty());

        let (status, session) = request(&app, "/auth/session", &jwt, None).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(session["user"]["id"], provisioned_user_id.to_string());
        assert_eq!(session["user"]["status"], "incomplete");
        assert!(session.get("suggestedEmail").is_none());
        assert!(session["verification"]["profile"].get("email").is_none());
        assert_eq!(
            session["verification"]["profile"]["microsoftObjectId"],
            object.to_string()
        );

        assert_eq!(
            session["verification"]["profile"]["affiliation"]["type"],
            if student { "student" } else { "nonStudent" }
        );

        if student {
            // Uma sincronização sem curso revoga a confirmação anterior, mantendo a conta.
            graph_mocks_with_course(
                &graph,
                tenant,
                group,
                object,
                true,
                Some(Value::Null),
                "graph@example.test",
            )
            .await;

            let (status, limited) = request(
                &app,
                "/auth/institutional/verify",
                &jwt,
                Some(json!({"microsoftAccessToken": "graph-token"})),
            )
            .await;

            assert_eq!(status, StatusCode::OK, "{limited}");
            assert_eq!(limited["user"]["id"], session["user"]["id"]);
            assert_eq!(
                limited["verification"]["profile"]["affiliation"]["type"],
                "nonStudent"
            );

            graph.verify().await;
            graph.reset().await;

            let (_, restored) = request(&app, "/auth/session", &jwt, None).await;

            assert_eq!(
                restored["verification"]["profile"]["affiliation"]["type"],
                "nonStudent"
            );
        }

        let profile =
            json!({"nickname":" Apelido ","currentSemester":4,"institutionEntrySemester":"2023.2"});

        assert_eq!(
            request(&app, "/users/me/profile", &jwt, Some(profile.clone()))
                .await
                .0,
            StatusCode::FORBIDDEN
        );

        let password_token =
            token_with_provider(&uid, object, "password", "escolhido@example.test");
        let mut invalid = profile.clone();

        invalid["currentSemester"] = json!(0);

        assert_eq!(
            request(&app, "/users/me/profile", &password_token, Some(invalid))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );

        let mut spoofed = profile.clone();

        spoofed["email"] = json!("falso@example.test");

        assert_eq!(
            request(&app, "/users/me/profile", &password_token, Some(spoofed))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );

        let (status, completed) = request(
            &app,
            "/users/me/profile",
            &password_token,
            Some(profile.clone()),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "{completed}");
        assert_eq!(completed["status"], "active");
        assert_eq!(completed["nickname"], "Apelido");
        assert_eq!(completed["email"], "escolhido@example.test");
        assert_eq!(completed["currentSemester"], 4);
        assert_eq!(completed["institutionEntrySemester"], "2023.2");
        assert_eq!(completed["id"], session["user"]["id"]);

        graph_mocks_with_course(
            &graph,
            tenant,
            group,
            object,
            group_member,
            course,
            "novo-graph@example.test",
        )
        .await;
        let (status, refreshed) = request(
            &app,
            "/auth/institutional/verify",
            &jwt,
            Some(json!({"microsoftAccessToken": "graph-token"})),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "{refreshed}");
        assert!(refreshed.get("suggestedEmail").is_none());
        assert_eq!(refreshed["user"]["email"], "escolhido@example.test");
        assert!(refreshed["verification"]["profile"].get("email").is_none());

        graph.verify().await;
        graph.reset().await;

        // O backend também protege os dados do Graph, mesmo sem a interface.

        for (key, value) in [("name", "Outro nome"), ("courseName", "Outro curso")] {
            let mut overwrite = profile.clone();

            overwrite[key] = json!(value);

            assert_eq!(
                request(&app, "/users/me/profile", &password_token, Some(overwrite))
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }

        // Nem um endereço declarado nem null são aceitos no perfil institucional.
        for email in [json!("outro@example.test"), Value::Null] {
            let mut overwrite = profile.clone();
            overwrite["institutionalEmail"] = email;

            assert_eq!(
                request(&app, "/users/me/profile", &password_token, Some(overwrite))
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }

        // Complementos são persistidos em app_user; a evidência continua nula.

        sqlx::query("UPDATE institutional_verification SET profile_name=NULL,course_name=NULL,student_membership_confirmed=FALSE WHERE user_id=(SELECT id FROM app_user WHERE firebase_uid=$1)")
            .bind(&uid).execute(&pool).await.unwrap();

        let mut supplement = profile.clone();

        supplement["name"] = json!(" Nome declarado ");

        for course in [json!("Curso declarado"), Value::Null] {
            let mut declared = supplement.clone();
            declared["courseName"] = course;

            assert_eq!(
                request(&app, "/users/me/profile", &password_token, Some(declared))
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }

        let (status, supplemented) =
            request(&app, "/users/me/profile", &password_token, Some(supplement)).await;

        assert_eq!(status, StatusCode::OK, "{supplemented}");
        assert_eq!(supplemented["name"], "Nome declarado");

        let (_, restored) = request(&app, "/auth/session", &password_token, None).await;

        assert_eq!(restored["user"]["name"], "Nome declarado");
        assert!(restored["user"].get("institutionalEmail").is_none());
        assert!(restored["verification"]["profile"]["name"].is_null());
        assert!(restored["verification"]["profile"].get("email").is_none());
        assert!(restored.get("suggestedEmail").is_none());

        assert!(restored["user"].get("courseName").is_none());
        assert_eq!(
            restored["verification"]["profile"]["affiliation"]["type"],
            "nonStudent"
        );
        assert!(
            restored["verification"]["profile"]["affiliation"]
                .get("courseName")
                .is_none()
        );

        assert_eq!(
            request(&app, "/auth/session", &password_token, None)
                .await
                .1["user"]["status"],
            "active"
        );
        assert!(graph.received_requests().await.unwrap().is_empty());

        sqlx::query("UPDATE institutional_verification SET checked_at=now()-interval '2 hours', expires_at=now()-interval '1 hour' WHERE user_id=(SELECT id FROM app_user WHERE firebase_uid=$1)").bind(&uid).execute(&pool).await.unwrap();

        assert_eq!(
            request(&app, "/protected", &jwt, None).await.0,
            StatusCode::FORBIDDEN
        );

        let (status, session) = request(&app, "/auth/session", &password_token, None).await;

        assert_eq!(status, StatusCode::OK);
        assert!(session["verification"].is_null());
        assert_eq!(session["user"]["status"], "active");

        sqlx::query("UPDATE app_user SET status='inactive' WHERE firebase_uid=$1")
            .bind(&uid)
            .execute(&pool)
            .await
            .unwrap();

        assert_eq!(
            request(&app, "/auth/session", &password_token, None)
                .await
                .0,
            StatusCode::FORBIDDEN
        );

        graph_mocks(&graph, tenant, group, object, student).await;

        let (status, inactive) = request(
            &app,
            "/auth/institutional/verify",
            &jwt,
            Some(json!({"microsoftAccessToken":"graph-token"})),
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(inactive["error"]["code"], "account_inactive");

        let state: String =
            sqlx::query_scalar("SELECT status::text FROM app_user WHERE firebase_uid=$1")
                .bind(&uid)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(state, "inactive");

        graph.verify().await;
        graph.reset().await;
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
        sqlx::query_scalar("SELECT count(*) FROM institutional_verification WHERE user_id=(SELECT id FROM app_user WHERE firebase_uid=$1)")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(count, 0);

    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM app_user WHERE firebase_uid=$1")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(users, 0);

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
        .expect(1 + u64::from(http_config.max_retries))
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

    graph.verify().await;
    graph.reset().await;

    pool.close().await;

    let (status, body) = request(&app, "/protected", &jwt, None).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "internal_server_error");
    assert_eq!(body["error"]["message"], "Erro interno do servidor");

    database.close().await;
}
