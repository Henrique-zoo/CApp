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
    CreateEventRequest, EventDetailResponse, EventErrorResponse, EventRegistrationResponse,
    EventResponse, EventSummaryResponse, ListEventsQueryParams, PublishEventRequest,
    UpdateEventRequest,
};
pub use handlers::{
    cancel_ca_event_registration_handler, cancel_event_registration_handler, create_event_handler,
    get_ca_event_detail_handler, get_event_handler, list_ca_events_handler,
    list_published_events_handler, publish_event_handler, register_in_ca_event_handler,
    register_in_event_handler, update_event_handler,
};

/// Constrói o roteador HTTP do módulo de eventos sob `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/events",
            get(list_published_events_handler).post(create_event_handler),
        )
        .route(
            "/api/v1/events/{id}",
            get(get_event_handler).put(update_event_handler),
        )
        .route("/api/v1/events/{id}/publish", post(publish_event_handler))
        .route(
            "/api/v1/events/{id}/registrations",
            post(register_in_event_handler).delete(cancel_event_registration_handler),
        )
        .route(
            "/api/v1/academic-centers/{ca_id}/events",
            get(list_ca_events_handler),
        )
        .route(
            "/api/v1/academic-centers/{ca_id}/events/{id}",
            get(get_ca_event_detail_handler),
        )
        .route(
            "/api/v1/academic-centers/{ca_id}/events/{id}/registrations",
            post(register_in_ca_event_handler).delete(cancel_ca_event_registration_handler),
        )
}
