//! Rotas HTTP do quadro eleitoral e dos recursos privados da própria conta.

use crate::api::state::AppState;
use axum::{Router, extract::DefaultBodyLimit, routing::get};

pub mod dto;
mod error;
pub mod handlers;

/// Publica o quadro e sua âncora assinada para verificação em outro dispositivo.
pub fn public_board_routes() -> Router<AppState> {
    Router::new()
        .route("/elections/{id}/board", get(handlers::snapshot))
        .route(
            "/elections/{id}/board/checkpoint",
            get(handlers::checkpoint),
        )
}

/// Consulta do recibo da conta autenticada, inclusive após vencer seu vínculo.
/// O router raiz registra esta rota sob o middleware de App Check.
pub fn protected_receipt_routes() -> Router<AppState> {
    Router::new().route(
        "/elections/{id}/receipts/me",
        get(handlers::read_own_receipt),
    )
}

/// Permite cadastrar a chave e consultar a participação somente da conta autenticada.
pub fn protected_registration_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/elections/{id}/registration/me",
            get(handlers::read_own_registration).put(handlers::register_voter_key),
        )
        .layer(DefaultBodyLimit::max(
            super::crypto_wire::MAX_SIGNED_MESSAGE_BYTES,
        ))
}
