//! Camada HTTP do módulo de demandas.
//!
//! [`dto`] define os contratos de transferência; [`handlers`] adapta as requisições;
//! [`error`] traduz as falhas de aplicação para os códigos HTTP adequados.

use axum::Router;

use crate::api::state::AppState;

pub mod dto;
pub mod error;
pub mod handlers;

/// Monta o roteador da API para o módulo de demandas.
///
/// # Rotas
///
/// - `POST /api/v1/academic-centers/{academic_center_id}/demands`: Criação de demanda nos dois fóruns.
/// - `GET /api/v1/academic-centers/{academic_center_id}/demands`: Listagem e filtragem de demandas nos fóruns.
/// - `GET /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}`: Detalhes e histórico da demanda.
/// - `PATCH /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/status`: Atualização de status com parecer.
/// - `GET /api/v1/demands/notifications`: Consulta das notificações de demandas do estudante autenticado.
/// - `PATCH /api/v1/demands/notifications/{notification_id}/read`: Marcação de notificação como lida.
/// - Rotas equivalentes sem o prefixo `/api/v1` para conveniência e retrocompatibilidade.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/academic-centers/{academic_center_id}/demands",
            axum::routing::post(handlers::handle_create_demand).get(handlers::handle_list_demands),
        )
        .route(
            "/academic-centers/{academic_center_id}/demands",
            axum::routing::post(handlers::handle_create_demand).get(handlers::handle_list_demands),
        )
        .route(
            "/api/v1/academic-centers/{academic_center_id}/demands/{demand_id}",
            axum::routing::get(handlers::handle_get_demand),
        )
        .route(
            "/academic-centers/{academic_center_id}/demands/{demand_id}",
            axum::routing::get(handlers::handle_get_demand),
        )
        .route(
            "/api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/status",
            axum::routing::patch(handlers::handle_update_demand_status),
        )
        .route(
            "/academic-centers/{academic_center_id}/demands/{demand_id}/status",
            axum::routing::patch(handlers::handle_update_demand_status),
        )
        .route(
            "/api/v1/demands/notifications",
            axum::routing::get(handlers::handle_list_notifications),
        )
        .route(
            "/demands/notifications",
            axum::routing::get(handlers::handle_list_notifications),
        )
        .route(
            "/api/v1/demands/notifications/{notification_id}/read",
            axum::routing::patch(handlers::handle_mark_notification_as_read),
        )
        .route(
            "/demands/notifications/{notification_id}/read",
            axum::routing::patch(handlers::handle_mark_notification_as_read),
        )
}
