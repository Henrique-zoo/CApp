//! Cadastro prévio de chave e consulta de participação da própria conta.

use std::time::Duration;

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State, rejection::BytesRejection},
    http::{
        HeaderMap, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::IntoResponse,
};
use uuid::Uuid;

use super::account::AuthenticatedAccount;
use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::elections::{
        application::registration::{RegistrationError, RegistrationStatus},
        infrastructure::PgVoterRegistry,
    },
};

pub(super) async fn read(
    State(state): State<AppState>,
    AuthenticatedAccount(identity): AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let result = tokio::time::timeout(
        Duration::from_secs(15),
        PgVoterRegistry::status(&state.pg_database.pool, election_id, identity.account_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(registration_error)?;
    Ok(private_status(result))
}

pub(super) async fn register(
    State(state): State<AppState>,
    AuthenticatedAccount(identity): AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> ApiResult<impl IntoResponse> {
    let mut types = headers.get_all(CONTENT_TYPE).iter();
    let media_type = types
        .next()
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next());
    if types.next().is_some()
        || !media_type.is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(ApiError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Envie a prova como application/json.",
            None,
        ));
    }
    let raw = body.map_err(|error| {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "Prova de cadastro excede o limite.",
                None,
            )
        } else {
            ApiError::bad_request()
        }
    })?;
    // O codec verifica o JSON original para detectar propriedades duplicadas na prova assinada.
    let result = tokio::time::timeout(
        Duration::from_secs(15),
        PgVoterRegistry::register(&state.pg_database.pool, election_id, identity, &raw),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(registration_error)?;
    Ok(private_status(result))
}

fn private_status(status: RegistrationStatus) -> impl IntoResponse {
    ([(CACHE_CONTROL, "no-store")], Json(status))
}

fn registration_error(error: RegistrationError) -> ApiError {
    let (status, code, message) = match error {
        RegistrationError::NotFound => (
            StatusCode::NOT_FOUND,
            "election_not_found",
            "Eleição não encontrada.",
        ),
        RegistrationError::NotEligible => (
            StatusCode::FORBIDDEN,
            "voter_not_eligible",
            "Eleitor sem vínculo vigente com o curso da eleição.",
        ),
        RegistrationError::RegistrationClosed => (
            StatusCode::CONFLICT,
            "registration_closed",
            "O período de cadastro da eleição terminou.",
        ),
        RegistrationError::KeyConflict => (
            StatusCode::CONFLICT,
            "registration_key_conflict",
            "A chave conflita com um cadastro existente.",
        ),
        RegistrationError::InvalidProof => (
            StatusCode::BAD_REQUEST,
            "invalid_registration_proof",
            "Prova de cadastro inválida.",
        ),
        RegistrationError::Storage(_) => {
            tracing::warn!(%error, "Falha de armazenamento no cadastro eleitoral.");
            return ApiError::service_unavailable();
        }
    };
    ApiError::new(status, code, message, None)
}
