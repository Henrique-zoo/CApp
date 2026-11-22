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
/// - `POST /academic-centers/{academic_center_id}/demands`: Rota de conveniência sem prefixo de versão.
/// - `GET /academic-centers/{academic_center_id}/demands`: Rota de conveniência sem prefixo de versão.
/// - `POST /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/comments`: Publicação de comentário/resposta na demanda.
/// - `GET /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/comments`: Listagem cronológica dos comentários da demanda.
/// - `POST /academic-centers/{academic_center_id}/demands/{demand_id}/comments`: Rota de conveniência sem prefixo de versão.
/// - `GET /academic-centers/{academic_center_id}/demands/{demand_id}/comments`: Rota de conveniência sem prefixo de versão.
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
            "/api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/comments",
            axum::routing::post(handlers::handle_post_comment).get(handlers::handle_list_comments),
        )
        .route(
            "/academic-centers/{academic_center_id}/demands/{demand_id}/comments",
            axum::routing::post(handlers::handle_post_comment).get(handlers::handle_list_comments),
        )
}
