//! Middleware de rastreabilidade HTTP e spans assíncronos (`TraceLayer`).
//!
//! Este módulo estende o [`tower_http::trace::TraceLayer`] para instrumentar cada
//! requisição atendida pelo servidor Axum com um span tracing assíncrono vinculado.
//!
//! # Campos Contextuais do Span
//!
//! Cada span de requisição (`http_request`) registra:
//! - `request_id`: Identificador de correlação extraído do cabeçalho `x-request-id`.
//! - `method`: Método HTTP da requisição (ex: `GET`, `POST`).
//! - `uri`: Caminho e consulta solicitados (URI).
//! - `status`: Código de status HTTP retornado na resposta (preenchido ao concluir).
//! - `latency_ms`: Tempo total de processamento em milissegundos (preenchido ao concluir).
//!
//! Quaisquer eventos de log emitidos durante o ciclo de vida da requisição herdam
//! automaticamente o contexto do span ativo, permitindo agrupamento e filtragem
//! precisos em sistemas de observabilidade.

use std::time::Duration;

use axum::http::{Request, Response};
use tower_http::{
    classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier},
    trace::{
        DefaultOnBodyChunk, DefaultOnEos, MakeSpan, OnFailure, OnRequest, OnResponse, TraceLayer,
    },
};
use tracing::Span;

use super::request_id::REQUEST_ID_HEADER;

/// Construtor de spans contextuais para requisições HTTP.
///
/// Extrai o cabeçalho `x-request-id` previamente injetado pelo middleware de
/// correlação e inicializa o span `http_request` com os metadados da requisição.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpMakeSpan;

impl<B> MakeSpan<B> for HttpMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get(&REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %request.method(),
            uri = %request.uri(),
            status = tracing::field::Empty,
            latency_ms = tracing::field::Empty,
        )
    }
}

/// Callback acionado no início do processamento de uma requisição HTTP.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpOnRequest;

impl<B> OnRequest<B> for HttpOnRequest {
    fn on_request(&mut self, request: &Request<B>, _span: &Span) {
        tracing::info!(
            method = %request.method(),
            uri = %request.uri(),
            "Iniciando processamento da requisição"
        );
    }
}

/// Callback acionado após o término do processamento e produção da resposta HTTP.
///
/// Registra no span os campos `status` e `latency_ms`, e emite log estruturado.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpOnResponse;

impl<B> OnResponse<B> for HttpOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status_code = response.status().as_u16();
        let latency_ms = latency.as_millis() as u64;

        span.record("status", status_code);
        span.record("latency_ms", latency_ms);

        tracing::info!(
            status = status_code,
            latency_ms = latency_ms,
            "Requisição concluída"
        );
    }
}

/// Callback acionado em caso de falha de conexão ou erro grave no processamento HTTP.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpOnFailure;

impl OnFailure<ServerErrorsFailureClass> for HttpOnFailure {
    fn on_failure(
        &mut self,
        classification: ServerErrorsFailureClass,
        latency: Duration,
        span: &Span,
    ) {
        let latency_ms = latency.as_millis() as u64;
        span.record("latency_ms", latency_ms);

        tracing::error!(
            %classification,
            latency_ms = latency_ms,
            "Falha no processamento da requisição"
        );
    }
}

/// Tipo concreto da camada `TraceLayer` customizada para observabilidade HTTP.
pub type CustomTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    HttpMakeSpan,
    HttpOnRequest,
    HttpOnResponse,
    DefaultOnBodyChunk,
    DefaultOnEos,
    HttpOnFailure,
>;

/// Cria e configura uma camada [`TraceLayer`] padronizada para o backend.
///
/// Vincula os construtores de spans e callbacks contextuais com medição de latência
/// e registro de status HTTP.
#[must_use]
pub fn create_trace_layer() -> CustomTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(HttpMakeSpan)
        .on_request(HttpOnRequest)
        .on_response(HttpOnResponse)
        .on_failure(HttpOnFailure)
}
