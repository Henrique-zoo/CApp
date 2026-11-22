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

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::{
        elections::{application::board::BoardError, infrastructure::PgElectionBoard},
        identity::{
            api::extractors::firebase_bearer_token,
            application::{
                error::AuthenticationError,
                ports::{IdentityTokenVerifier, UserRepository},
            },
            domain::UserStatus,
        },
    },
};

pub(super) async fn read(
    State(state): State<AppState>,
    Path(election_id): Path<Uuid>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let token = firebase_bearer_token(&headers)?;
    let receipt = tokio::time::timeout(Duration::from_secs(15), async {
        let identity = state
            .clients
            .firebase
            .verify(token)
            .await
            .map_err(ApiError::from)?;
        let account = state
            .pg_database
            .get_by_firebase_uid(&identity.firebase_uid)
            .await
            .map_err(AuthenticationError::from)
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::from(AuthenticationError::AccountNotProvisioned))?;
        if account.status() == UserStatus::Inactive {
            return Err(ApiError::from(AuthenticationError::AccountInactive));
        }
        PgElectionBoard::read_receipt(&state.pg_database.pool, election_id, account.id())
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
