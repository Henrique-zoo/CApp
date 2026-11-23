//! Cadastro prévio de chave e consulta de participação da própria conta.

use std::time::Duration;

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header::CACHE_CONTROL},
    response::IntoResponse,
};
use uuid::Uuid;

use super::{account::AuthenticatedAccount, signed_json::SignedJson};
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
    SignedJson(raw): SignedJson,
) -> ApiResult<impl IntoResponse> {
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
