//! Extrator de identidade autenticada para requisições HTTP.
//!
//! Identifica o usuário associado à requisição por meio dos cabeçalhos
//! `Authorization: Bearer <uuid>` ou `X-User-Id`, desacoplando os handlers
//! dos detalhes de transporte do token.

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

/// Representa a identidade autenticada do usuário extraída da requisição HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedUser {
    /// Identificador único do usuário autenticado.
    pub user_id: Uuid,
}

impl AuthenticatedUser {
    /// Constrói uma nova identidade autenticada a partir do UUID do usuário.
    pub const fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

/// Erros possíveis durante a extração das credenciais do usuário na requisição HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthExtractionError {
    /// Nenhum cabeçalho de autenticação foi enviado na requisição.
    MissingCredentials,
    /// O cabeçalho enviado possui formato inválido ou não contém um UUID válido.
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
        if let Some(user_id_header) = parts
            .headers
            .get("X-User-Id")
            .or_else(|| parts.headers.get("x-user-id"))
            .or_else(|| parts.headers.get("X-Actor-Id"))
        {
            let user_id_str = user_id_header
                .to_str()
                .map_err(|_| AuthExtractionError::InvalidCredentials)?;
            let user_id = Uuid::parse_str(user_id_str.trim())
                .map_err(|_| AuthExtractionError::InvalidCredentials)?;
            return Ok(Self { user_id });
        }

        if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AuthExtractionError::InvalidCredentials)?;
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let user_id = Uuid::parse_str(token.trim())
                    .map_err(|_| AuthExtractionError::InvalidCredentials)?;
                return Ok(Self { user_id });
            }
            return Err(AuthExtractionError::InvalidCredentials);
        }

        Err(AuthExtractionError::MissingCredentials)
    }
}

impl<S> OptionalFromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthExtractionError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <Self as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(user) => Ok(Some(user)),
            Err(AuthExtractionError::MissingCredentials) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
