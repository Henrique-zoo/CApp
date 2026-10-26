//! Interceptação, geração e propagação do cabeçalho de correlação (`x-request-id`).
//!
//! Este módulo gerencia o identificador único atribuído a cada requisição HTTP que
//! transita pelo backend. O identificador permite correlacionar registros de log,
//! métricas de latência e diagnósticos ponta a ponta.
//!
//! # Regras de Validação e Geração
//!
//! - Se a requisição contiver o cabeçalho `x-request-id` com um valor válido (ASCII
//!   imprimível, sem espaços ou caracteres de controle, e com até 128 caracteres),
//!   esse identificador é preservado integralmente.
//! - Se o cabeçalho estiver ausente ou contiver caracteres inválidos, um novo
//!   identificador único no padrão UUID v4 é gerado.
//! - Todas as respostas HTTP emitidas pelo servidor recebem obrigatoriamente o
//!   cabeçalho `x-request-id` correspondente.

use std::{fmt, ops::Deref};

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Nome padronizado do cabeçalho HTTP de correlação de requisições.
pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Nome padronizado do cabeçalho em formato literal de texto.
pub const REQUEST_ID_HEADER_NAME: &str = "x-request-id";

/// Comprimento máximo permitido para um cabeçalho `x-request-id` recebido do cliente.
///
/// Evita sobrecarga de memória e ataques de injeção em cabeçalhos HTTP.
pub const MAX_REQUEST_ID_LENGTH: usize = 128;

/// Identificador de correlação vinculado ao contexto da requisição HTTP.
///
/// Armazenado nas extensões da requisição (`Request::extensions`) para acesso
/// direto por handlers ou serviços internos.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequestId(pub String);

impl RequestId {
    /// Cria uma nova instância a partir de um valor de texto.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Retorna a representação em fatia de string (`&str`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for RequestId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RequestId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for RequestId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Valida se uma sequência de caracteres é aceitável como identificador de correlação.
///
/// Um identificador é considerado válido quando:
/// - Não é vazio;
/// - Não ultrapassa [`MAX_REQUEST_ID_LENGTH`] caracteres;
/// - Contém exclusivamente caracteres ASCII gráficos visíveis (`33..=126`),
///   rejeitando espaços em branco, quebras de linha (`\r`, `\n`) e caracteres de controle.
///
/// # Exemplos
///
/// ```
/// use backend::api::request_id::is_valid_request_id;
///
/// assert!(is_valid_request_id("capp-req-12345"));
/// assert!(is_valid_request_id("550e8400-e29b-41d4-a716-446655440000"));
/// assert!(!is_valid_request_id(""));
/// assert!(!is_valid_request_id("id com espacos"));
/// assert!(!is_valid_request_id("id\ncom\nnewline"));
/// ```
#[must_use]
pub fn is_valid_request_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_REQUEST_ID_LENGTH && id.chars().all(|c| c.is_ascii_graphic())
}

/// Extrai e valida o identificador do cabeçalho informado, ou gera um novo UUID v4.
///
/// # Parâmetros
///
/// - `header`: valor opcional do cabeçalho `x-request-id` extraído da requisição.
///
/// # Retorno
///
/// Uma tupla contendo o [`RequestId`] validado ou gerado e o respectivo [`HeaderValue`].
#[must_use]
pub fn sanitize_or_generate_request_id(header: Option<&HeaderValue>) -> (RequestId, HeaderValue) {
    if let Some(val) = header
        && let Ok(val_str) = val.to_str()
        && is_valid_request_id(val_str)
    {
        return (RequestId::new(val_str), val.clone());
    }

    let new_uuid = Uuid::new_v4().to_string();
    let header_value =
        HeaderValue::from_str(&new_uuid).expect("UUID v4 deve ser um HeaderValue válido");
    (RequestId::new(new_uuid), header_value)
}

/// Middleware Axum que assegura a rastreabilidade e propagação do `x-request-id`.
///
/// 1. Intercepta o cabeçalho `x-request-id` da requisição;
/// 2. Preserva-o se for válido, ou gera um novo UUID v4;
/// 3. Injeta o valor normalizado nos cabeçalhos da requisição e nas suas extensões;
/// 4. Executa a cadeia de middlewares e o handler interno;
/// 5. Injeta obrigatoriamente o mesmo `x-request-id` nos cabeçalhos da resposta HTTP.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let (request_id, header_value) =
        sanitize_or_generate_request_id(request.headers().get(&REQUEST_ID_HEADER));

    request
        .headers_mut()
        .insert(REQUEST_ID_HEADER.clone(), header_value.clone());
    request.extensions_mut().insert(request_id);

    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER.clone(), header_value);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_request_ids() {
        assert!(is_valid_request_id("018f0000-0000-7000-8000-000000000001"));
        assert!(is_valid_request_id("capp-req-42"));
        assert!(is_valid_request_id("client.id.123_456"));
    }

    #[test]
    fn test_invalid_request_ids() {
        assert!(!is_valid_request_id(""));
        assert!(!is_valid_request_id(" "));
        assert!(!is_valid_request_id("id com espaco"));
        assert!(!is_valid_request_id("id\ncom\nnewline"));
        assert!(!is_valid_request_id("id\rcom\rretorno"));
        assert!(!is_valid_request_id("id\tcom\ttab"));

        let oversized = "a".repeat(MAX_REQUEST_ID_LENGTH + 1);
        assert!(!is_valid_request_id(&oversized));
    }

    #[test]
    fn test_sanitize_preserves_valid_id() {
        let incoming = HeaderValue::from_static("minha-requisicao-123");
        let (id, header_val) = sanitize_or_generate_request_id(Some(&incoming));

        assert_eq!(id.as_str(), "minha-requisicao-123");
        assert_eq!(header_val, incoming);
    }

    #[test]
    fn test_sanitize_generates_uuid_when_missing() {
        let (id, header_val) = sanitize_or_generate_request_id(None);

        assert!(Uuid::parse_str(id.as_str()).is_ok());
        assert_eq!(header_val.to_str().unwrap(), id.as_str());
    }

    #[test]
    fn test_sanitize_generates_uuid_when_invalid() {
        let invalid_header = HeaderValue::from_static("invalido com espacos");
        let (id, header_val) = sanitize_or_generate_request_id(Some(&invalid_header));

        assert_ne!(id.as_str(), "invalido com espacos");
        assert!(Uuid::parse_str(id.as_str()).is_ok());
        assert_eq!(header_val.to_str().unwrap(), id.as_str());
    }
}
