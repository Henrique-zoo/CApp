//! Camada HTTP do módulo de Centros Acadêmicos.
//!
//! [`dto`] contém os contratos de entrada e saída JSON;
//! [`handlers`] adapta as requisições HTTP aos casos de uso de Centro Acadêmico e mandato.

use axum::Router;
use axum::routing::get;
use axum::routing::post;
use crate::api::state::AppState;
pub use crate::api::auth::ActiveCaContext;
pub use crate::api::auth::AuthorizationErrorResponse;
pub use crate::api::auth::RequirePermission;
pub use crate::api::auth::require_permission_middleware;
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
            "/academic-centers/{id}/mandates/history",
            get(handlers::get_mandates_history),
        )
        .route(
            "/academic-centers/{id}/mandates/conclude",
            post(handlers::conclude_active_mandate),
        )
        .route(
            "/academic-centers/{id}/mandates/transition",
            post(handlers::transition_mandate),
        )
        .route(
            "/academic-centers/{id}/mandates/{mandate_id}",
            get(handlers::get_mandate_by_id),
        )
        .route(
            "/academic-centers/{id}/mandates/{mandate_id}/members",
            post(handlers::add_mandate_member),
        )
        .route(
            "/academic-centers/{id}/mandates/{mandate_id}/members/{member_id}/disaffiliate",
            post(handlers::disaffiliate_mandate_member),
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
            "/api/v1/academic-centers/{id}/mandates/history",
            get(handlers::get_mandates_history),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates/conclude",
            post(handlers::conclude_active_mandate),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates/transition",
            post(handlers::transition_mandate),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates/{mandate_id}",
            get(handlers::get_mandate_by_id),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates/{mandate_id}/members",
            post(handlers::add_mandate_member),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandates/{mandate_id}/members/{member_id}/disaffiliate",
            post(handlers::disaffiliate_mandate_member),
        )
        .route(
            "/api/v1/academic-centers/{id}/mandate",
            post(handlers::configure_initial_mandate).get(handlers::get_current_mandate),
        )
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
