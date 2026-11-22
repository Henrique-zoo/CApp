//! Camada HTTP do módulo de Centros Acadêmicos.
//!
//! [`dto`] contém os contratos de entrada e saída JSON;
//! [`handlers`] adapta as requisições HTTP aos casos de uso de Centro Acadêmico e mandato.

use axum::{
    Router,
    routing::{get, post},
};

use crate::api::state::AppState;

pub mod dto;
pub mod handlers;

/// Monta o roteador Axum com os endpoints de Centros Acadêmicos e mandatos de gestão.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/academic-centers/{id}/mandates",
            post(handlers::configure_initial_mandate).get(handlers::list_mandates),
        )
        .route(
            "/academic-centers/{id}/mandates/current",
            get(handlers::get_current_mandate),
        )
        .route(
            "/academic-centers/{id}/mandate",
            post(handlers::configure_initial_mandate).get(handlers::get_current_mandate),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates",
            post(handlers::configure_initial_mandate).get(handlers::list_mandates),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates/current",
            get(handlers::get_current_mandate),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandate",
            post(handlers::configure_initial_mandate).get(handlers::get_current_mandate),
        )
}
