//! Conversão dos erros de autenticação para o contrato HTTP da API.
//!
//! Credenciais recusadas resultam em 401; elegibilidade recusada ou pendente, em 403;
//! indisponibilidade externa, em 503; falha interna, em 500. O código estável em JSON
//! distingue causas com o mesmo status. Somente a recusa de vínculo inclui
//! `details.institution_name`; diagnósticos de persistência não são expostos.

use axum::http::StatusCode;
use serde_json::json;

use crate::{api::error::ApiError, modules::identity::application::error::AuthenticationError};

impl From<AuthenticationError> for ApiError {
    /// Converte uma falha preservando sua classificação pública e contexto permitido.
    ///
    /// A mensagem vem de `Display`, exceto em `Internal`, que recebe a mensagem genérica
    /// de [`ApiError::internal_server_error`]. A cadeia técnica original não integra o
    /// JSON. `InstitutionalVerificationRequired` orienta renovação institucional, sem
    /// classificar a sessão Firebase como expirada.
    fn from(error: AuthenticationError) -> Self {
        let (status, code, details) = match &error {
            AuthenticationError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token", None),
            AuthenticationError::ExpiredToken => (StatusCode::UNAUTHORIZED, "expired_token", None),
            AuthenticationError::IdentityMismatch => {
                (StatusCode::UNAUTHORIZED, "identity_mismatch", None)
            }
            AuthenticationError::InvalidMicrosoftId => {
                (StatusCode::UNAUTHORIZED, "invalid_microsoft_id", None)
            }
            AuthenticationError::UnknownInstitution => {
                (StatusCode::FORBIDDEN, "unknown_institution", None)
            }
            AuthenticationError::StudentMembershipRequired { institution_name } => (
                StatusCode::FORBIDDEN,
                "student_membership_required",
                Some(json!({
                    "institution_name": institution_name,
                })),
            ),
            AuthenticationError::ProviderUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_provider_unavailable",
                None,
            ),
            AuthenticationError::Internal => {
                return Self::internal_server_error();
            }
            AuthenticationError::InstitutionalVerificationRequired => (
                StatusCode::FORBIDDEN,
                "institutional_verification_required",
                None,
            ),
        };

        Self::new(status, code, error.to_string(), details)
    }
}
