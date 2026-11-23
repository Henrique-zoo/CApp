//! Handlers do quadro público, do recibo e do cadastro da conta autenticada.
//!
//! Esta fronteira extrai dados HTTP e monta a resposta de cada operação. As
//! rotas e os limites de requisição são compostos no módulo de API.

use std::time::Duration;

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::IntoResponse,
};
use serde_json::Value;
use uuid::Uuid;

use super::{
    dto::RegistrationResponse,
    error::{ballot_error, public_read_error, receipt_not_found, receipt_read_error},
    signed_json::SignedJson,
};
use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::{
        elections::{
            application::{
                registration::{RegistrationStatus, VoterIdentity},
                services::{board, voter_registration},
            },
            infrastructure::WireVoterKeyProofVerifier,
        },
        identity::api::extractors::AuthenticatedAccount,
    },
};

pub(super) async fn snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let chunks = board::stream_snapshot(&state.pg_database, id)
        .await
        .map_err(public_read_error)?;

    Ok((
        [(CONTENT_TYPE, "application/json")],
        Body::from_stream(chunks),
    ))
}

pub(super) async fn checkpoint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    board::read_checkpoint(&state.pg_database, id)
        .await
        .map(Json)
        .map_err(public_read_error)
}

pub(super) async fn read_own_receipt(
    State(state): State<AppState>,
    Path(election_id): Path<Uuid>,
    account: AuthenticatedAccount,
) -> ApiResult<Json<Value>> {
    let receipt = tokio::time::timeout(
        Duration::from_secs(15),
        board::read_own_receipt(&state.pg_database, election_id, account.account_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(receipt_read_error)?
    .ok_or_else(receipt_not_found)?;

    Ok(Json(receipt))
}

pub(super) async fn read_own_registration(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let result = tokio::time::timeout(
        Duration::from_secs(15),
        voter_registration::status(&state.voter_registry, election_id, account.account_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())??;
    Ok(registration_response(result))
}

pub(super) async fn register_voter_key(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
    SignedJson(raw): SignedJson,
) -> ApiResult<impl IntoResponse> {
    let identity = VoterIdentity {
        account_id: account.account_id,
        microsoft_object_id: account.microsoft_object_id,
    };
    let result = tokio::time::timeout(
        Duration::from_secs(15),
        voter_registration::register(
            &state.voter_registry,
            &WireVoterKeyProofVerifier,
            election_id,
            identity,
            &raw,
        ),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())??;
    Ok(registration_response(result))
}

pub(super) async fn submit_ballot(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
    SignedJson(raw): SignedJson,
) -> ApiResult<impl IntoResponse> {
    let board = state
        .election_board
        .as_ref()
        .ok_or_else(ApiError::service_unavailable)?;
    let identity = VoterIdentity {
        account_id: account.account_id,
        microsoft_object_id: account.microsoft_object_id,
    };

    let recorded = tokio::time::timeout(
        Duration::from_secs(15),
        board::record_ballot(board, election_id, identity, &raw),
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

fn registration_response(status: RegistrationStatus) -> impl IntoResponse {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(RegistrationResponse::from(status)),
    )
}
