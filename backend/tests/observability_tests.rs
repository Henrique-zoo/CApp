//! Testes de integração para rastreabilidade, correlação de requisições e observabilidade.
//!
//! Verifica que:
//! - Respostas HTTP recebem obrigatoriamente o cabeçalho `x-request-id`;
//! - Identificadores válidos enviados pelo cliente são preservados e propagados;
//! - Identificadores ausentes ou inválidos são substituídos por UUIDs v4 gerados;
//! - Rotas inexistentes (HTTP 404) continuam recebendo `x-request-id`;
//! - Logs emitidos durante o ciclo de vida da requisição herdam o `request_id` no span tracing.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    body::Body,
    http::{HeaderValue, Request, StatusCode},
};
use backend::api::{
    logging::JsonLogLayer,
    request_id::{MAX_REQUEST_ID_LENGTH, REQUEST_ID_HEADER},
    router::create_router,
    state::AppState,
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use tracing_subscriber::{Registry, prelude::*};
use uuid::Uuid;

static SHARED_LOG_BUFFER: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

/// Inicializa um subscriber global único para captura de logs nos testes.
fn get_or_init_test_subscriber() -> Arc<Mutex<Vec<u8>>> {
    SHARED_LOG_BUFFER
        .get_or_init(|| {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let layer = JsonLogLayer::new(BufferWriter(buffer.clone()));
            let filter = tracing_subscriber::EnvFilter::new("info");
            let _ = Registry::default().with(filter).with(layer).try_init();
            buffer
        })
        .clone()
}

/// Cria uma instância em memória da aplicação para testes HTTP.
fn build_test_app() -> axum::Router {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://capp:capp@localhost:5432/capp")
        .expect("pool lazy para testes em memória");
    create_router().with_state(AppState::new(pool))
}

#[tokio::test]
async fn test_response_contains_generated_x_request_id_when_not_provided() {
    let app = build_test_app();

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .expect("requisição válida");

    let response = app.oneshot(req).await.expect("resposta HTTP");

    assert_eq!(response.status(), StatusCode::OK);

    let header_val = response
        .headers()
        .get(&REQUEST_ID_HEADER)
        .expect("deve conter x-request-id")
        .to_str()
        .expect("header deve ser string válida");

    assert!(
        Uuid::parse_str(header_val).is_ok(),
        "identificador gerado deve ser um UUID válido: {header_val}"
    );
}

#[tokio::test]
async fn test_response_propagates_valid_client_x_request_id() {
    let app = build_test_app();
    let client_id = "client-correlacao-xyz-789";

    let req = Request::builder()
        .uri("/health")
        .header(&REQUEST_ID_HEADER, client_id)
        .body(Body::empty())
        .expect("requisição válida");

    let response = app.oneshot(req).await.expect("resposta HTTP");

    assert_eq!(response.status(), StatusCode::OK);

    let header_val = response
        .headers()
        .get(&REQUEST_ID_HEADER)
        .expect("deve conter x-request-id")
        .to_str()
        .expect("header deve ser string válida");

    assert_eq!(header_val, client_id);
}

#[tokio::test]
async fn test_response_replaces_invalid_client_x_request_id() {
    let app = build_test_app();
    let invalid_id = "id com espacos";

    let req = Request::builder()
        .uri("/health")
        .header(&REQUEST_ID_HEADER, HeaderValue::from_static(invalid_id))
        .body(Body::empty())
        .expect("requisição válida");

    let response = app.oneshot(req).await.expect("resposta HTTP");

    assert_eq!(response.status(), StatusCode::OK);

    let header_val = response
        .headers()
        .get(&REQUEST_ID_HEADER)
        .expect("deve conter x-request-id")
        .to_str()
        .expect("header deve ser string válida");

    assert_ne!(header_val, invalid_id);
    assert!(
        Uuid::parse_str(header_val).is_ok(),
        "deve gerar novo UUID v4 para ID inválido"
    );
}

#[tokio::test]
async fn test_response_replaces_oversized_x_request_id() {
    let app = build_test_app();
    let oversized = "a".repeat(MAX_REQUEST_ID_LENGTH + 10);

    let req = Request::builder()
        .uri("/health")
        .header(
            &REQUEST_ID_HEADER,
            HeaderValue::from_str(&oversized).expect("ascii válido"),
        )
        .body(Body::empty())
        .expect("requisição válida");

    let response = app.oneshot(req).await.expect("resposta HTTP");

    assert_eq!(response.status(), StatusCode::OK);

    let header_val = response
        .headers()
        .get(&REQUEST_ID_HEADER)
        .expect("deve conter x-request-id")
        .to_str()
        .expect("header deve ser string válida");

    assert_ne!(header_val, oversized);
    assert!(
        Uuid::parse_str(header_val).is_ok(),
        "deve gerar novo UUID v4 para ID com tamanho excessivo"
    );
}

#[tokio::test]
async fn test_unmatched_route_404_still_contains_x_request_id() {
    let app = build_test_app();
    let client_id = "req-rota-inexistente-404";

    let req = Request::builder()
        .uri("/rota-inexistente-aleatoria")
        .header(&REQUEST_ID_HEADER, client_id)
        .body(Body::empty())
        .expect("requisição válida");

    let response = app.oneshot(req).await.expect("resposta HTTP");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let header_val = response
        .headers()
        .get(&REQUEST_ID_HEADER)
        .expect("deve conter x-request-id mesmo em 404")
        .to_str()
        .expect("header deve ser string válida");

    assert_eq!(header_val, client_id);
}

#[tokio::test]
async fn test_tracing_logs_inherit_request_id() {
    let log_buffer = get_or_init_test_subscriber();

    let client_id = "req-tracing-test-unique-555";
    let app = build_test_app();

    let req = Request::builder()
        .uri("/health")
        .header(&REQUEST_ID_HEADER, client_id)
        .body(Body::empty())
        .expect("requisição válida");

    let response = app.oneshot(req).await.expect("resposta HTTP");
    assert_eq!(response.status(), StatusCode::OK);

    let output = String::from_utf8(log_buffer.lock().unwrap().clone()).expect("logs utf8");
    assert!(!output.is_empty(), "deve haver logs emitidos");

    let mut found_request_id = false;
    for line in output.lines() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line)
            && parsed.get("request_id").and_then(|v| v.as_str()) == Some(client_id)
        {
            found_request_id = true;
            break;
        }
    }

    assert!(
        found_request_id,
        "logs emitidos durante a requisição devem herdar request_id '{client_id}':\n{output}"
    );
}

#[derive(Clone)]
struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}
