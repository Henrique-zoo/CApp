//! Handlers HTTP (Axum) para criação, edição, publicação e consulta de eventos.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use super::dto::{
    CreateEventRequest, EventErrorResponse, EventResponse, ListEventsQueryParams,
    PublishEventRequest, UpdateEventRequest,
};
use crate::{
    api::state::AppState,
    modules::events::{
        application::{
            CreateEventCommand, EventApplicationError, EventService, GetEventByIdQuery,
            ListCaEventsQuery, PublishEventCommand, UpdateEventCommand,
        },
        domain::{EventDomainError, EventRegistrationInfo},
        infrastructure::PostgresEventRepository,
    },
};

/// Converte um [`EventApplicationError`] em resposta HTTP com status e código semântico adequados.
fn map_application_error(error: EventApplicationError) -> Response {
    let (status, code) = match &error {
        EventApplicationError::Domain(domain_err) => match domain_err {
            EventDomainError::EmptyTitle => (StatusCode::BAD_REQUEST, "missing:título"),
            EventDomainError::EmptyDescription => (StatusCode::BAD_REQUEST, "missing:descrição"),
            EventDomainError::EmptyLocation => (StatusCode::BAD_REQUEST, "missing:local"),
            EventDomainError::InvalidEventPeriod { .. } => {
                (StatusCode::BAD_REQUEST, "invalid_period")
            }
            _ => (StatusCode::BAD_REQUEST, "validation_error"),
        },
        EventApplicationError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
        EventApplicationError::PermissionDenied => (StatusCode::FORBIDDEN, "permission_denied"),
        EventApplicationError::Repository(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
        }
    };

    let body = Json(EventErrorResponse {
        code: code.to_string(),
        message: error.to_string(),
    });
    (status, body).into_response()
}

/// Handler `POST /api/v1/events` para criar um novo evento (rascunho ou publicado diretamente).
pub async fn create_event_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Response {
    let service = EventService::new(PostgresEventRepository::new(state.pool));
    let registration = payload.into_registration_info();

    let command = CreateEventCommand {
        academic_center_id: payload.academic_center_id,
        author_id: payload.author_id,
        title: payload.title,
        description: payload.description,
        location: payload.location,
        starts_at: payload.starts_at,
        ends_at: payload.ends_at,
        publish_directly: payload.publish_directly,
        registration,
    };

    match service.create_event(command).await {
        Ok(event) => (StatusCode::CREATED, Json(EventResponse::from(event))).into_response(),
        Err(error) => map_application_error(error),
    }
}

/// Handler `GET /api/v1/events/{id}` para consultar um evento pelo seu identificador.
pub async fn get_event_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListEventsQueryParams>,
) -> Response {
    let service = EventService::new(PostgresEventRepository::new(state.pool));
    let query = GetEventByIdQuery {
        event_id: id,
        only_published: params.only_published,
    };
    match service.get_event_by_id(query).await {
        Ok(event) => (StatusCode::OK, Json(EventResponse::from(event))).into_response(),
        Err(error) => map_application_error(error),
    }
}

/// Handler `PUT /api/v1/events/{id}` para editar as informações de um evento existente.
pub async fn update_event_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEventRequest>,
) -> Response {
    let service = EventService::new(PostgresEventRepository::new(state.pool));
    let registration = if payload.requires_registration.is_some()
        || payload.max_capacity.is_some()
        || payload.registration_deadline.is_some()
    {
        Some(EventRegistrationInfo {
            requires_registration: payload.requires_registration.unwrap_or(true),
            max_capacity: payload.max_capacity,
            confirmed_registrations: 0,
            registration_deadline: payload.registration_deadline,
        })
    } else {
        None
    };

    let command = UpdateEventCommand {
        event_id: id,
        academic_center_id: payload.academic_center_id,
        actor_id: payload.actor_id,
        title: payload.title,
        description: payload.description,
        location: payload.location,
        starts_at: payload.starts_at,
        ends_at: payload.ends_at,
        registration,
    };

    match service.update_event(command).await {
        Ok(event) => (StatusCode::OK, Json(EventResponse::from(event))).into_response(),
        Err(error) => map_application_error(error),
    }
}

/// Handler `POST /api/v1/events/{id}/publish` para publicar diretamente um evento em rascunho.
pub async fn publish_event_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    payload: Option<Json<PublishEventRequest>>,
) -> Response {
    let service = EventService::new(PostgresEventRepository::new(state.pool));
    let body = payload.map(|json| json.0).unwrap_or_default();

    let command = PublishEventCommand {
        event_id: id,
        academic_center_id: body.academic_center_id,
        actor_id: body.actor_id,
    };

    match service.publish_event_directly(command).await {
        Ok(event) => (StatusCode::OK, Json(EventResponse::from(event))).into_response(),
        Err(error) => map_application_error(error),
    }
}

/// Handler `GET /api/v1/academic-centers/{ca_id}/events` para listar eventos de um Centro Acadêmico.
pub async fn list_ca_events_handler(
    State(state): State<AppState>,
    Path(ca_id): Path<Uuid>,
    Query(params): Query<ListEventsQueryParams>,
) -> Response {
    let service = EventService::new(PostgresEventRepository::new(state.pool));
    let query = ListCaEventsQuery {
        academic_center_id: ca_id,
        only_published: params.only_published,
    };

    match service.list_ca_events(query).await {
        Ok(events) => {
            let response: Vec<EventResponse> =
                events.into_iter().map(EventResponse::from).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => map_application_error(error),
    }
}
