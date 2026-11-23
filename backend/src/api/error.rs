//! Contrato público de erros HTTP.
//!
//! [`ApiError`] serializa `error.code`, `error.message` e `error.details` e acrescenta
//! `WWW-Authenticate: Bearer` às respostas 401. Códigos são destinados ao tratamento
//! programático; mensagens são textos públicos. O tipo não serializa cadeias de
//! erro nem registra logs automaticamente; os adaptadores registram suas causas.

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::WWW_AUTHENTICATE},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

/// Erro público da API, com classificação HTTP e contexto para o cliente.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ApiError {
    /// Status HTTP usado na resposta.
    status: StatusCode,
    /// Identificador estável para tratamento pelo cliente, independente da mensagem.
    code: &'static str,
    /// Texto público exibível ao usuário; não deve conter diagnósticos internos.
    message: String,
    /// Contexto JSON permitido; `None` é serializado como `null`.
    details: Option<Value>,
}

/// Resultado de uma operação da camada HTTP.
pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    /// Cria um erro 400 para uma requisição inválida, sem detalhes adicionais.
    pub fn bad_request() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "Requisição inválida",
            None,
        )
    }

    /// Cria um erro 401 para ausência de credenciais válidas.
    ///
    /// A resposta HTTP inclui o desafio de autenticação `Bearer`.
    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Autenticação necessária",
            None,
        )
    }

    /// Cria um erro 403 para acesso não permitido, sem detalhes adicionais.
    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Acesso não permitido",
            None,
        )
    }

    /// Cria um erro 503 para indisponibilidade temporária do serviço.
    pub fn service_unavailable() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "Serviço temporariamente indisponível",
            None,
        )
    }

    /// Cria um erro 500 com mensagem genérica, sem expor detalhes internos.
    pub fn internal_server_error() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            "Erro interno do servidor",
            None,
        )
    }

    /// Constrói um erro com informações apropriadas para exposição ao cliente.
    ///
    /// # Parâmetros
    ///
    /// - `status`: classificação HTTP, escolhida pela camada API.
    /// - `code`: identificador estável, usado pelo cliente para tratamento programático.
    /// - `message`: texto público, materializado como `String`.
    /// - `details`: contexto público opcional; nunca tokens, senhas ou diagnóstico SQL.
    ///
    /// # Retorno
    ///
    /// Erro pronto para conversão em resposta. Não valida a coerência entre status e
    /// código, não remove informações sensíveis e não registra logs. Cabe ao chamador
    /// fornecer somente conteúdo autorizado para exposição.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use axum::{http::StatusCode, response::IntoResponse};
    /// use backend::api::error::ApiError;
    ///
    /// let response = ApiError::bad_request().into_response();
    /// assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    ///
    /// let response = ApiError::unauthorized().into_response();
    /// assert_eq!(response.headers()["www-authenticate"], "Bearer");
    /// ```
    pub fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details,
        }
    }
}

impl IntoResponse for ApiError {
    /// Consome o erro e produz o status e o envelope JSON público.
    ///
    /// `details` ausente é serializado como `null`. Respostas 401 recebem
    /// `WWW-Authenticate: Bearer`; as demais não recebem esse cabeçalho.
    /// A conversão não inspeciona causas internas nem decide política de autorização.
    fn into_response(self) -> Response {
        let Self {
            status,
            code,
            message,
            details,
        } = self;

        let mut response = (
            status,
            Json(json!({
                "error": {
                    "code": code,
                    "message": message,
                    "details": details,
                }
            })),
        )
            .into_response();

        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }

        response
    }
}
