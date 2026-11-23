//! Rotas HTTP do quadro eleitoral e dos recursos privados da própria conta.

use crate::api::state::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

pub mod dto;
mod error;
pub mod handlers;
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

/// Consulta do recibo da conta autenticada, inclusive após vencer seu vínculo.
/// O router raiz registra esta rota sob o middleware de App Check.
pub fn protected_receipt_routes() -> Router<AppState> {
    Router::new().route(
        "/elections/{id}/receipts/me",
        get(handlers::read_own_receipt),
    )
}

/// Cadastro e consulta da chave da conta autenticada.
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

/// Recebe uma cédula cifrada sob a mesma proteção Firebase e App Check do cadastro.
pub fn protected_ballot_routes() -> Router<AppState> {
    Router::new()
        .route("/elections/{id}/ballots", post(handlers::submit_ballot))
        .layer(DefaultBodyLimit::max(
            super::crypto_wire::MAX_SIGNED_MESSAGE_BYTES,
        ))
}
