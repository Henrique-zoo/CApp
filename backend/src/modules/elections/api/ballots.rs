//! Aceite da cédula assinada no mesmo quadro usado pela exportação e pelo recibo.

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
    modules::elections::application::board::{BoardError, ElectionBoard},
};

pub(super) async fn submit(
    State(state): State<AppState>,
    AuthenticatedAccount(identity): AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
    SignedJson(raw): SignedJson,
) -> ApiResult<impl IntoResponse> {
    let board = state
        .election_board
        .as_ref()
        .ok_or_else(ApiError::service_unavailable)?;
    let recorded = tokio::time::timeout(
        Duration::from_secs(15),
        board.record(election_id, identity, &raw),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(ballot_error)?;
    let status = if recorded.recovered {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, [(CACHE_CONTROL, "no-store")], Json(recorded)))
}

fn ballot_error(error: BoardError) -> ApiError {
    let (status, code, message) = match error {
        BoardError::InvalidEvidence(_) | BoardError::VoterMismatch => (
            StatusCode::BAD_REQUEST,
            "invalid_ballot",
            "Cédula inválida para a conta e a eleição.",
        ),
        BoardError::NotEligible => (
            StatusCode::FORBIDDEN,
            "voter_not_eligible",
            "Eleitor sem cadastro prévio ou vínculo vigente.",
        ),
        BoardError::NotFound => (
            StatusCode::NOT_FOUND,
            "election_not_found",
            "Eleição não encontrada.",
        ),
        BoardError::Closed | BoardError::VotingEnded => (
            StatusCode::CONFLICT,
            "voting_ended",
            "O período de votação terminou.",
        ),
        BoardError::VotingNotStarted => (
            StatusCode::CONFLICT,
            "voting_not_started",
            "O período de votação ainda não começou.",
        ),
        BoardError::AlreadyParticipated => (
            StatusCode::CONFLICT,
            "already_voted",
            "A conta já votou nesta eleição.",
        ),
        BoardError::SubmissionConflict => (
            StatusCode::CONFLICT,
            "submission_conflict",
            "O conteúdo diverge da cédula confirmada.",
        ),
        BoardError::LimitExceeded => (
            StatusCode::CONFLICT,
            "election_full",
            "A eleição atingiu o limite de participantes.",
        ),
        BoardError::SigningUnavailable | BoardError::Storage(_) => {
            return ApiError::service_unavailable();
        }
        BoardError::InvalidArtifact(_) | BoardError::AlreadyExists | BoardError::NotYetEnded => {
            tracing::error!(%error, "Estado inconsistente ao registrar cédula.");
            return ApiError::internal_server_error();
        }
    };
    ApiError::new(status, code, message, None)
}
