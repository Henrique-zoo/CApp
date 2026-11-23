//! Camada HTTP do módulo de Centros Acadêmicos.
//!
//! [`dto`] contém os contratos de entrada e saída JSON;
//! [`handlers`] adapta as requisições HTTP aos casos de uso de consultas e enquetes estudantis.

use axum::{
    Router,
    routing::{get, post},
};

use crate::api::state::AppState;

pub mod dto;
pub mod handlers;

/// Monta o roteador Axum com os endpoints de Centros Acadêmicos e consultas/enquetes estudantis.
pub fn router() -> Router<AppState> {
    Router::new()
        // Rotas com prefixo padrão da API /api/v1
        .route(
            "/api/v1/academic-centers/{academic_center_id}/polls",
            post(handlers::create_poll).get(handlers::list_polls),
        )
        .route(
            "/api/v1/academic-centers/{academic_center_id}/polls/{poll_id}",
            get(handlers::get_poll),
        )
        .route(
            "/api/v1/academic-centers/{academic_center_id}/polls/{poll_id}/votes",
            post(handlers::vote_poll),
        )
        .route(
            "/api/v1/academic-centers/{academic_center_id}/polls/{poll_id}/close",
            post(handlers::close_poll),
        )
        // Aliases diretos sem /api/v1
        .route(
            "/academic-centers/{academic_center_id}/polls",
            post(handlers::create_poll).get(handlers::list_polls),
        )
        .route(
            "/academic-centers/{academic_center_id}/polls/{poll_id}",
            get(handlers::get_poll),
        )
        .route(
            "/academic-centers/{academic_center_id}/polls/{poll_id}/votes",
            post(handlers::vote_poll),
        )
        .route(
            "/academic-centers/{academic_center_id}/polls/{poll_id}/close",
            post(handlers::close_poll),
        )
}
