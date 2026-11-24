//! Camada HTTP do módulo de perguntas frequentes (FAQ).

use axum::{Router, routing::post};

use crate::api::state::AppState;

pub mod dto;
pub mod error;
pub mod handlers;

/// Constrói o router com os endpoints do módulo de FAQ.
///
/// Registra a rota canônica e seu alias com prefixo de versionamento.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/academic-centers/{academic_center_id}/faq",
            post(handlers::create_faq_entry),
        )
        .route(
            "/academic-centers/{academic_center_id}/faq",
            post(handlers::create_faq_entry),
        )
}
