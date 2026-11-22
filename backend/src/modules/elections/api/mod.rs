//! Distribuição das evidências públicas para o eleitor e as autoridades.

use crate::api::state::AppState;
use axum::{Router, routing::get};

pub mod dto;
pub mod handlers;
mod own_receipt;

/// Publica o quadro e sua âncora assinada para verificação em outro dispositivo.
pub fn public_board_routes() -> Router<AppState> {
    Router::new()
        .route("/elections/{id}/board", get(handlers::snapshot))
        .route(
            "/elections/{id}/board/checkpoint",
            get(handlers::checkpoint),
        )
}

/// Consulta da própria conta; compor sob o middleware App Check das rotas protegidas.
/// A sessão Firebase é validada no handler, sem exigir elegibilidade atual.
pub fn protected_receipt_routes() -> Router<AppState> {
    Router::new().route("/elections/{id}/receipts/me", get(own_receipt::read))
}
