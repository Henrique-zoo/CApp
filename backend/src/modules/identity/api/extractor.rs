//! Extrator de identidade autenticada para requisições HTTP.
//!
//! Identifica o usuário associado à requisição por meio de cabeçalhos
//! de autenticação, desacoplando os handlers da implementação concreta
//! de tokens ou sessões.

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

/// Representa a identidade autenticada do usuário na requisição HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedUser {
    /// Identificador único do usuário autenticado.
    pub user_id: Uuid,
}

impl AuthenticatedUser {
    /// Cria uma nova instância de usuário autenticado a partir de seu identificador.
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

/// Erro de extração de identidade autenticada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthExtractionError {
    /// Cabeçalho de autorização ausente.
    MissingCredentials,
    /// Formato de credenciais inválido.
    InvalidCredentials,
}

impl IntoResponse for AuthExtractionError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::MissingCredentials => (StatusCode::UNAUTHORIZED, "Credenciais ausentes."),
            Self::InvalidCredentials => (
                StatusCode::BAD_REQUEST,
                "Identificador de usuário inválido.",
            ),
        };
        (status, message).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthExtractionError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Suporta cabeçalho X-User-Id para testes e comunicação interna
        if let Some(user_id_header) = parts.headers.get("X-User-Id") {
            let user_id_str = user_id_header
                .to_str()
                .map_err(|_| AuthExtractionError::InvalidCredentials)?;
            let user_id = Uuid::parse_str(user_id_str)
                .map_err(|_| AuthExtractionError::InvalidCredentials)?;
            return Ok(AuthenticatedUser { user_id });
        }

        // Suporta cabeçalho Authorization: Bearer <uuid>
        if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AuthExtractionError::InvalidCredentials)?;
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let user_id = Uuid::parse_str(token.trim())
                    .map_err(|_| AuthExtractionError::InvalidCredentials)?;
                return Ok(AuthenticatedUser { user_id });
            }
        }

        Err(AuthExtractionError::MissingCredentials)
    }
}
