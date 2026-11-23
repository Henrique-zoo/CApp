//! Distribuição das evidências públicas para o eleitor e as autoridades.

use crate::api::state::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

mod account;
mod ballots;
pub mod dto;
pub mod handlers;
mod own_receipt;
mod registration;
mod signed_json;

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

/// Cadastro e consulta da chave da conta autenticada.
pub fn protected_registration_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/elections/{id}/registration/me",
            get(registration::read).put(registration::register),
        )
        .layer(DefaultBodyLimit::max(
            super::crypto_wire::MAX_SIGNED_MESSAGE_BYTES,
        ))
}

/// Recebe uma cédula cifrada sob a mesma proteção Firebase e App Check do cadastro.
pub fn protected_ballot_routes() -> Router<AppState> {
    Router::new()
        .route("/elections/{id}/ballots", post(ballots::submit))
        .layer(DefaultBodyLimit::max(
            super::crypto_wire::MAX_SIGNED_MESSAGE_BYTES,
        ))
}
