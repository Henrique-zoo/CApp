//! Leitura de evidências eleitorais já confirmadas pelo PostgreSQL.

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
};
use serde_json::Value;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::api::{
    error::{ApiError, ApiResult},
    state::AppState,
};
use crate::modules::elections::{application::board::BoardError, infrastructure::PgElectionBoard};

pub(super) async fn snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let chunks = PgElectionBoard::stream_snapshot(&state.pg_database.pool, id)
        .await
        .map_err(public_read_error)?;
    Ok((
        [(CONTENT_TYPE, "application/json")],
        Body::from_stream(ReceiverStream::new(chunks)),
    ))
}

pub(super) async fn checkpoint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    PgElectionBoard::read_checkpoint(&state.pg_database.pool, id)
        .await
        .map(Json)
        .map_err(public_read_error)
}

fn public_read_error(error: BoardError) -> ApiError {
    match error {
        BoardError::NotFound => ApiError::new(
            StatusCode::NOT_FOUND,
            "board_not_found",
            "Quadro eleitoral não encontrado.",
            None,
        ),
        BoardError::Storage(error) => {
            tracing::warn!(%error, "Falha na leitura do quadro público.");
            ApiError::service_unavailable()
        }
        error => {
            tracing::error!(%error, "Falha de consistência no quadro público.");
            ApiError::internal_server_error()
        }
    }
}
