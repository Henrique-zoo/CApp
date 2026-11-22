//! Recuperação do recibo pela conta autenticada, inclusive após expirar o vínculo.
//! Não recebe identificador de eleitor, conta ou UID declarado pelo cliente.

use std::time::Duration;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde_json::Value;
use uuid::Uuid;

use super::account::AuthenticatedAccount;
use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::elections::{application::board::BoardError, infrastructure::PgElectionBoard},
};

pub(super) async fn read(
    State(state): State<AppState>,
    Path(election_id): Path<Uuid>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let receipt = tokio::time::timeout(Duration::from_secs(15), async {
        let AuthenticatedAccount(identity) =
            AuthenticatedAccount::authenticate(&headers, &state).await?;
        PgElectionBoard::read_receipt(&state.pg_database.pool, election_id, identity.account_id)
            .await
            .map_err(read_error)?
            .map(|recorded| recorded.receipt)
            .ok_or_else(not_found)
    })
    .await
    .map_err(|_| ApiError::service_unavailable())??;
    Ok(Json(receipt))
}

fn not_found() -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "receipt_not_found",
        "Recibo eleitoral não encontrado.",
        None,
    )
}

fn read_error(error: BoardError) -> ApiError {
    match error {
        BoardError::NotFound => not_found(),
        BoardError::Storage(error) => {
            tracing::warn!(%error, "Falha na leitura do próprio recibo eleitoral.");
            ApiError::service_unavailable()
        }
        error => {
            tracing::error!(%error, "Falha de consistência no recibo eleitoral.");
            ApiError::internal_server_error()
        }
    }
}
