//! Camada HTTP do módulo de notícias.
//!
//! [`dto`] define os contratos de entrada e saída; [`handlers`] adapta
//! as requisições aos casos de uso da camada de aplicação.

pub mod dto;
pub mod handlers;

use axum::{
    Router,
    routing::{get, patch},
};

use crate::api::state::AppState;

/// Cria e configura o router Axum com todos os endpoints do módulo de notícias.
///
/// Rotas registradas:
/// - `PATCH /news/{id}/publish`: homologa e publica o artigo.
/// - `PATCH /news/{id}/archive`: arquiva o artigo.
/// - `PATCH /news/{id}/submit-for-review`: submete o artigo para revisão da gestão.
/// - `GET /news/{id}`: consulta os detalhes do artigo.
pub fn news_routes() -> Router<AppState> {
    Router::new()
        .route("/news/{id}/publish", patch(handlers::publish_news_handler))
        .route("/news/{id}/archive", patch(handlers::archive_news_handler))
        .route(
            "/news/{id}/submit-for-review",
            patch(handlers::submit_for_review_handler),
        )
        .route("/news/{id}", get(handlers::get_news_by_id_handler))
}
