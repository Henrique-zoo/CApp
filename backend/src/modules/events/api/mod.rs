//! Camada HTTP do módulo de eventos.
//!
//! [`dto`] define os contratos de entrada e saída e [`handlers`] adapta as
//! requisições HTTP aos casos de uso de [`crate::modules::events::application`].

use axum::{
    Router,
    routing::{get, post},
};

use crate::api::state::AppState;

pub mod dto;
pub mod handlers;

pub use dto::{
    CreateEventRequest, EventErrorResponse, EventResponse, ListEventsQueryParams,
    PublishEventRequest, UpdateEventRequest,
};
pub use handlers::{
    create_event_handler, get_event_handler, list_ca_events_handler, publish_event_handler,
    update_event_handler,
};

/// Constrói o roteador HTTP do módulo de eventos sob `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/events", post(create_event_handler))
        .route(
            "/api/v1/events/{id}",
            get(get_event_handler).put(update_event_handler),
        )
        .route("/api/v1/events/{id}/publish", post(publish_event_handler))
        .route(
            "/api/v1/academic-centers/{ca_id}/events",
            get(list_ca_events_handler),
        )
}
