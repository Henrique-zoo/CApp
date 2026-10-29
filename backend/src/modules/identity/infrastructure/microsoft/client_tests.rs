//! Contratos HTTP do Graph exercitados com Reqwest e servidor local.
use super::*;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TOKEN: &str = "graph-test-token";
const ID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

/// Constrói o cliente com redirecionamento desabilitado e origem local.
fn client(server: &MockServer) -> MicrosoftGraphClient {
    MicrosoftGraphClient::new_with_base_url(
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        format!("{}/v1.0", server.uri()),
    )
}

/// Confere Bearer, seleção de campos e preservação dos dados do estudante.
#[tokio::test]
async fn student_profile_preserves_external_values() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1.0/me"))
        .and(header("authorization", format!("Bearer {TOKEN}")))
        .and(query_param("$select", "id,displayName,mail,jobTitle"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": ID, "displayName": "Pessoa", "mail": "pessoa@example.edu",
            "jobTitle": "  Engenharia  "
        })))
        .expect(1)
        .mount(&server)
        .await;
    let profile = client(&server).get_profile(TOKEN, true).await.unwrap();
    assert_eq!(profile.id, ID);
    assert_eq!(profile.name.as_deref(), Some("Pessoa"));
    assert_eq!(profile.mail.as_deref(), Some("pessoa@example.edu"));
    assert!(
        matches!(profile.affiliation, InstitutionalAffiliation::Student { course_name: Some(name) } if name == "  Engenharia  ")
    );
}

/// Campos ausentes, nulos e curso em branco são aceitos sem inventar um curso.
#[tokio::test]
async fn optional_profile_fields_and_blank_courses_are_supported() {
    for job in [serde_json::Value::Null, json!(""), json!(" \t\n")] {
        let server = MockServer::start().await;
        Mock::given(path("/v1.0/me"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"id": ID, "jobTitle": job})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let profile = client(&server).get_profile(TOKEN, true).await.unwrap();
        assert!(profile.name.is_none() && profile.mail.is_none());
        assert!(matches!(
            profile.affiliation,
            InstitutionalAffiliation::Student { course_name: None }
        ));
    }
}

/// Um cargo profissional não é interpretado como curso de um não estudante.
#[tokio::test]
async fn non_student_job_title_is_not_a_course() {
    let server = MockServer::start().await;
    Mock::given(path("/v1.0/me"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"id": ID, "jobTitle": "Professor"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    assert!(matches!(
        client(&server)
            .get_profile(TOKEN, false)
            .await
            .unwrap()
            .affiliation,
        InstitutionalAffiliation::NonStudent
    ));
}

/// Obtém o tenant da organização consultada, usando o token e `$select` corretos.
#[tokio::test]
async fn tenant_comes_from_organization() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .and(header("authorization", format!("Bearer {TOKEN}")))
        .and(query_param("$select", "id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": [{"id": ID}]})))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        client(&server).get_tenant_id(TOKEN).await.unwrap(),
        Uuid::parse_str(ID).unwrap()
    );
}

/// Respostas sem exatamente uma organização com UUID válido são recusadas.
#[tokio::test]
async fn ambiguous_or_malformed_organizations_are_provider_errors() {
    for body in [
        json!({}),
        json!({"value": []}),
        json!({"value": [{"id": ID}, {"id": ID}]}),
        json!({"value": [{"id": "bad"}]}),
        json!({"value": [{}]}),
    ] {
        let server = MockServer::start().await;
        Mock::given(path("/v1.0/organization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        assert!(matches!(
            client(&server).get_tenant_id(TOKEN).await,
            Err(AuthenticationError::ProviderUnavailable)
        ));
    }
}

/// Confere POST, corpo exato e participação apenas no UUID solicitado.
#[tokio::test]
async fn membership_checks_exact_group_with_post_and_bearer() {
    let group = Uuid::parse_str(ID).unwrap();
    for (values, expected) in [
        (json!([ID]), true),
        (json!([]), false),
        (json!([Uuid::new_v4()]), false),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1.0/me/checkMemberGroups"))
            .and(header("authorization", format!("Bearer {TOKEN}")))
            .and(header("content-type", "application/json"))
            .and(body_json(json!({"groupIds": [ID]})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": values})))
            .expect(1)
            .mount(&server)
            .await;
        assert_eq!(
            client(&server)
                .is_student_member(TOKEN, group)
                .await
                .unwrap(),
            expected
        );
    }
}

/// Falhas HTTP preservam a distinção entre token recusado e provedor indisponível.
#[tokio::test]
async fn http_errors_are_not_negative_membership() {
    for status in [401, 403, 429, 500, 503, 204] {
        let server = MockServer::start().await;
        Mock::given(path("/v1.0/me/checkMemberGroups"))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&server)
            .await;
        let error = client(&server)
            .is_student_member(TOKEN, Uuid::new_v4())
            .await
            .unwrap_err();
        if status == 401 {
            assert!(matches!(error, AuthenticationError::InvalidToken));
        } else {
            assert!(matches!(error, AuthenticationError::ProviderUnavailable));
        }
    }
}

/// JSON inválido ou incompatível não é interpretado como perfil ou grupo vazio.
#[tokio::test]
async fn malformed_json_is_a_provider_error() {
    for (endpoint, body) in [
        ("me", "not-json"),
        ("me", "{}"),
        ("me", "{\"id\":42}"),
        ("me/checkMemberGroups", "{}"),
        ("me/checkMemberGroups", "{\"value\":[\"bad\"]}"),
    ] {
        let server = MockServer::start().await;
        Mock::given(path(format!("/v1.0/{endpoint}")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;
        let graph = client(&server);
        let result = if endpoint == "me" {
            graph.get_profile(TOKEN, true).await.map(|_| ())
        } else {
            graph
                .is_student_member(TOKEN, Uuid::new_v4())
                .await
                .map(|_| ())
        };
        assert!(matches!(
            result,
            Err(AuthenticationError::ProviderUnavailable)
        ));
    }
}

/// Não segue redirects nem encaminha o Bearer para a origem indicada pelo provedor.
#[tokio::test]
async fn redirect_does_not_forward_credentials() {
    let server = MockServer::start().await;
    let destination = MockServer::start().await;
    Mock::given(path("/v1.0/me"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", destination.uri()))
        .expect(1)
        .mount(&server)
        .await;
    assert!(matches!(
        client(&server).get_profile(TOKEN, true).await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
    assert!(destination.received_requests().await.unwrap().is_empty());
}

/// Falha de conexão é traduzida para indisponibilidade do provedor.
#[tokio::test]
async fn connection_failure_is_a_provider_error() {
    let server = MockServer::start().await;
    let graph = MicrosoftGraphClient::new_with_base_url(Client::new(), "http://127.0.0.1:0".into());
    assert!(matches!(
        graph.get_profile(TOKEN, false).await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}
