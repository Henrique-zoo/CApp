//! Rotas autenticadas de contextos, publicação e consulta de materiais acadêmicos.

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, header::CACHE_CONTROL},
    middleware,
    response::Response,
    routing::get,
};

use crate::{api::state::AppState, modules::materials::application::services::MAX_PDF_BYTES};

mod dto;
mod handlers;

/// Monta as rotas que recebem App Check no router global.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/materials/contexts", get(handlers::centers))
        .route(
            "/academic-centers/{ca_id}/materials",
            get(handlers::list).post(handlers::publish),
        )
        .route(
            "/academic-centers/{ca_id}/materials/disciplines",
            get(handlers::disciplines),
        )
        .route(
            "/academic-centers/{ca_id}/materials/{id}",
            get(handlers::get),
        )
        .route(
            "/academic-centers/{ca_id}/materials/{id}/file",
            get(handlers::download),
        )
        .layer(DefaultBodyLimit::max(
            MAX_PDF_BYTES + handlers::METADATA_LIMIT + 16 * 1024,
        ))
        .layer(middleware::map_response(
            |mut response: Response| async move {
                response
                    .headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
                response
            },
        ))
}
