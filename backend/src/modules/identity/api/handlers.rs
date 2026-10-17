//! Handlers HTTP do módulo de identidade institucional.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};

use crate::{
    api::state::AppState,
    modules::identity::{
        api::dto::CurrentIdentityResponse,
        application::queries::current_identity::IdentityVerificationError,
    },
};

/// Retorna a identidade do usuário após validar o token de autenticação.
pub async fn current_identity(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CurrentIdentityResponse>, (StatusCode, &'static str)> {
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let Some(authorization) = authorization else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "É necessário realizar a autenticação.",
        ));
    };

    let Some((scheme, token)) = authorization.split_once(' ') else {
        return Err((StatusCode::UNAUTHORIZED, "Token de autenticação inválido."));
    };

    if !scheme.eq_ignore_ascii_case("Bearer") || token.trim().is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Token de autenticação inválido."));
    }

    match state
        .identity_token_verifier
        .verify_id_token(token.trim())
        .await
    {
        Ok(identity) => Ok(Json(CurrentIdentityResponse {
            uid: identity.uid,
            email: identity.email,
            email_verified: identity.email_verified,
        })),
        Err(IdentityVerificationError::InvalidToken) => Err((
            StatusCode::UNAUTHORIZED,
            "Token de autenticação inválido ou expirado.",
        )),
        Err(IdentityVerificationError::MissingConfiguration) => {
            tracing::error!("A configuração de autenticação está ausente.");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "A autenticação do servidor não está configurada.",
            ))
        }
        Err(IdentityVerificationError::ProviderUnavailable) => {
            tracing::warn!("O provedor de autenticação está indisponível.");
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "O serviço de autenticação está temporariamente indisponível.",
            ))
        }
    }
}
