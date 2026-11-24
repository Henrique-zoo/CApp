//! Camada HTTP do módulo de perguntas frequentes (FAQ).

use axum::{
    Router,
    routing::{patch, post},
};

use crate::api::state::AppState;

pub mod dto;
pub mod error;
pub mod handlers;

/// Constrói o router com os endpoints do módulo de FAQ.
///
/// Registra as rotas canônicas e seus aliases com prefixo de versionamento.
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
        .route(
            "/api/v1/academic-centers/{academic_center_id}/faq/{faq_entry_id}",
            patch(handlers::update_faq_entry).delete(handlers::delete_faq_entry),
        )
        .route(
            "/academic-centers/{academic_center_id}/faq/{faq_entry_id}",
            patch(handlers::update_faq_entry).delete(handlers::delete_faq_entry),
        )
}
