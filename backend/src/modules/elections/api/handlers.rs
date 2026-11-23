//! Handlers do quadro público, do recibo e do cadastro da conta autenticada.
//!
//! Esta fronteira extrai dados HTTP e monta a resposta de cada operação. As
//! rotas e os limites de requisição são compostos no módulo de API.

use std::time::Duration;

use axum::{
    Json,
    body::{Body, Bytes},
    extract::{Path, State, rejection::BytesRejection},
    http::{
        HeaderMap, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::IntoResponse,
};
use serde_json::Value;
use uuid::Uuid;

use super::{
    dto::RegistrationResponse,
    error::{public_read_error, receipt_not_found, receipt_read_error},
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

fn registration_response(status: RegistrationStatus) -> impl IntoResponse {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(RegistrationResponse::from(status)),
    )
}
