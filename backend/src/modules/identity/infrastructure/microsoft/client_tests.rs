//! Contratos HTTP do Graph exercitados com Reqwest e servidor local.
use std::time::Duration;

use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::config::HttpConfig;

const TOKEN: &str = "graph-test-token";
const ID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

/// Constrói o cliente com redirecionamento desabilitado, origem local e delays curtos para testes.
fn client(server: &MockServer) -> MicrosoftGraphClient {
    MicrosoftGraphClient::new_with_base_url_and_config(
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        format!("{}/v1.0", server.uri()),
        HttpConfig::default_for_test(),
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
        matches!(profile.affiliation, InstitutionalAffiliation::Student { course_name: name } if name == "  Engenharia  ")
    );
}

/// Nome/e-mail podem faltar; curso ausente, nulo ou em branco limita o acesso.
#[tokio::test]
async fn missing_or_blank_courses_do_not_confirm_student_affiliation() {
    for job in [
        None,
        Some(serde_json::Value::Null),
        Some(json!("")),
        Some(json!(" \t\n")),
    ] {
        let server = MockServer::start().await;
        let mut body = json!({"id": ID});

        if let Some(job) = job {
            body["jobTitle"] = job;
        }

        Mock::given(path("/v1.0/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let profile = client(&server).get_profile(TOKEN, true).await.unwrap();
        assert!(profile.name.is_none() && profile.mail.is_none());
        assert!(matches!(
            profile.affiliation,
            InstitutionalAffiliation::NonStudent
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

/// Falhas HTTP preservam a distinção entre token recusado e provedor indisponível,
/// retentando falhas transitórias (429 e 503) e rejeitando sem retentativa erros de cliente ou outros status.
#[tokio::test]
async fn http_errors_are_not_negative_membership() {
    for status in [401, 403, 429, 500, 503, 204] {
        let server = MockServer::start().await;
        let expected_calls = if status == 429 || status == 503 { 3 } else { 1 };
        Mock::given(path("/v1.0/me/checkMemberGroups"))
            .respond_with(ResponseTemplate::new(status))
            .expect(expected_calls)
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

/// Falha de conexão é traduzida para indisponibilidade do provedor após retentativas.
#[tokio::test]
async fn connection_failure_is_a_provider_error() {
    let server = MockServer::start().await;
    let graph = MicrosoftGraphClient::new_with_base_url_and_config(
        Client::new(),
        "http://127.0.0.1:0".into(),
        HttpConfig::default_for_test(),
    );
    assert!(matches!(
        graph.get_profile(TOKEN, false).await,
        Err(AuthenticationError::ProviderUnavailable)
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

/// Falha transitória 503 seguida de 200 OK conclui com sucesso após retentativa.
#[tokio::test]
async fn transient_503_then_200_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": [{"id": ID}]})))
        .mount(&server)
        .await;

    let graph = client(&server);
    let tenant_id = graph.get_tenant_id(TOKEN).await.unwrap();
    assert_eq!(tenant_id, Uuid::parse_str(ID).unwrap());

    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        2,
        "deve ter realizado exatamente 2 chamadas"
    );
}

/// Falha transitória 429 com cabeçalho Retry-After seguida de 200 OK conclui com sucesso.
#[tokio::test]
async fn transient_429_with_retry_after_then_200_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1.0/me"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1.0/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": ID, "displayName": "Pessoa", "mail": "pessoa@example.edu",
            "jobTitle": "Engenharia"
        })))
        .mount(&server)
        .await;

    let graph = client(&server);
    let profile = graph.get_profile(TOKEN, true).await.unwrap();
    assert_eq!(profile.id, ID);

    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        2,
        "deve ter realizado exatamente 2 chamadas"
    );
}

/// Esgotamento de tentativas em 503 contínuo encerra com ProviderUnavailable após o limite máximo de retentativas.
#[tokio::test]
async fn continuous_503_exhausts_retries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .respond_with(ResponseTemplate::new(503))
        .expect(3)
        .mount(&server)
        .await;

    let graph = client(&server);
    let result = graph.get_tenant_id(TOKEN).await;
    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));

    server.verify().await;
}

/// Erros definitivos do cliente (400, 401, 403, 404) nunca realizam retentativa (exatamente 1 requisição recebida).
#[tokio::test]
async fn permanent_client_errors_do_not_retry() {
    for (status, is_auth_error) in [(400, false), (401, true), (403, false), (404, false)] {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1.0/organization"))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&server)
            .await;

        let graph = client(&server);
        let result = graph.get_tenant_id(TOKEN).await;

        if is_auth_error {
            assert!(matches!(result, Err(AuthenticationError::InvalidToken)));
        } else {
            assert!(matches!(
                result,
                Err(AuthenticationError::ProviderUnavailable)
            ));
        }

        server.verify().await;
    }
}

/// Timeout de resposta lenta é interrompido e retentado com segurança até o esgotamento.
#[tokio::test]
async fn slow_response_times_out_and_retries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1.0/organization"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(150))
                .set_body_json(json!({"value": [{"id": ID}]})),
        )
        .expect(3)
        .mount(&server)
        .await;

    let config = HttpConfig {
        connect_timeout: Duration::from_millis(30),
        request_timeout: Duration::from_millis(30),
        max_retries: 2,
        min_retry_delay: Duration::from_millis(1),
        max_retry_delay: Duration::from_millis(5),
    };

    let client_http = Client::builder()
        .connect_timeout(config.connect_timeout)
        .timeout(config.request_timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let graph = MicrosoftGraphClient::new_with_base_url_and_config(
        client_http,
        format!("{}/v1.0", server.uri()),
        config,
    );

    let result = graph.get_tenant_id(TOKEN).await;
    assert!(matches!(
        result,
        Err(AuthenticationError::ProviderUnavailable)
    ));

    server.verify().await;
}
