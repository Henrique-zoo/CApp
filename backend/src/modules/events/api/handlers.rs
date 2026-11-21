//! Handlers HTTP (Axum) para criação, edição, publicação direta e consulta de eventos.
//!
//! Os endpoints de escrita e publicação direta integram [`AuthenticatedUser`]
//! e [`PostgresEventAuthorizationService`] para garantir que apenas usuários com a
//! permissão [`DIRECT_EVENT_MANAGEMENT_PERMISSION`](crate::modules::events::domain::DIRECT_EVENT_MANAGEMENT_PERMISSION)
//! no respectivo Centro Acadêmico possam publicar diretamente (sem homologação) e editar eventos.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use super::dto::{
    CreateEventRequest, EventDetailResponse, EventErrorResponse, EventRegistrationResponse,
    EventResponse, ListEventsQueryParams, PublishEventRequest, UpdateEventRequest,
};
use crate::{
    api::state::AppState,
    modules::{
        events::{
            application::{
                CancelEventRegistrationCommand, CancelEventRegistrationUseCase, CreateEventCommand,
                EventApplicationError, EventRepository, EventService, GetEventByIdQuery,
                GetPublishedEventDetailQuery, GetPublishedEventDetailUseCase, ListCaEventsQuery,
                ListPublishedEventsByCaQuery, ListPublishedEventsByCaUseCase, PublishEventCommand,
                RegisterInEventCommand, RegisterParticipantInEventUseCase, UpdateEventCommand,
            },
            domain::{EventDomainError, EventRegistrationInfo},
            infrastructure::{PostgresEventAuthorizationService, PostgresEventRepository},
        },
        identity::api::AuthenticatedUser,
    },
};

/// Extrai o identificador do Centro Acadêmico ativo a partir dos parâmetros de query
/// (`ca_id` / `academic_center_id`) ou dos cabeçalhos HTTP (`X-Academic-Center-Id`, `X-Active-Ca-Id`).
fn extract_active_ca_id(params: &ListEventsQueryParams, headers: &HeaderMap) -> Option<Uuid> {
    if let Some(ca_id) = params.ca_id {
        return Some(ca_id);
    }

    for header_name in ["x-academic-center-id", "x-active-ca-id"] {
        if let Some(value) = headers.get(header_name).and_then(|v| v.to_str().ok())
            && let Ok(parsed) = Uuid::parse_str(value.trim())
        {
            return Some(parsed);
        }
    }

    None
}

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
            EventDomainError::EventNotPublished => {
                (StatusCode::UNPROCESSABLE_ENTITY, "event_not_available")
            }
            _ => (StatusCode::BAD_REQUEST, "validation_error"),
        },
        EventApplicationError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
        EventApplicationError::EventNotAvailableForRegistration(_) => {
            (StatusCode::UNPROCESSABLE_ENTITY, "event_not_available")
        }
        EventApplicationError::AlreadyRegistered { .. } => {
            (StatusCode::CONFLICT, "already_registered")
        }
        EventApplicationError::ActiveRegistrationNotFound { .. } => {
            (StatusCode::CONFLICT, "no_active_registration")
        }
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
///
/// Verifica se o usuário autenticado (ou autor informado) possui a permissão
/// `DIRECT_EVENT_MANAGEMENT_PERMISSION` no Centro Acadêmico antes de criar/publicar diretamente.
pub async fn create_event_handler(
    State(state): State<AppState>,
    authenticated_user: Option<AuthenticatedUser>,
    Json(payload): Json<CreateEventRequest>,
) -> Response {
    let service = EventService::with_authorizer(
        PostgresEventRepository::new(state.pool.clone()),
        PostgresEventAuthorizationService::new(state.pool),
    );
    let registration = payload.into_registration_info();
    let author_id = authenticated_user
        .map(|user| user.user_id)
        .unwrap_or(payload.author_id);

    let command = CreateEventCommand {
        academic_center_id: payload.academic_center_id,
        author_id,
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

/// Handler `GET /api/v1/events?ca_id={ca_id}` para listar eventos publicados filtrados pelo CA ativo.
pub async fn list_published_events_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListEventsQueryParams>,
) -> Response {
    let repo = Arc::new(PostgresEventRepository::new(state.pool));
    execute_list_published_events(repo, &params, &headers).await
}

async fn execute_list_published_events<R: EventRepository>(
    repo: Arc<R>,
    params: &ListEventsQueryParams,
    headers: &HeaderMap,
) -> Response {
    let Some(ca_id) = extract_active_ca_id(params, headers) else {
        let body = Json(EventErrorResponse {
            code: "missing:ca_id".to_string(),
            message: "O identificador do Centro Acadêmico ativo (ca_id) é obrigatório".to_string(),
        });
        return (StatusCode::BAD_REQUEST, body).into_response();
    };

    if params.should_filter_only_published() {
        let use_case = ListPublishedEventsByCaUseCase::new(repo);
        let query = ListPublishedEventsByCaQuery::new(ca_id);
        match use_case.execute(query).await {
            Ok(events) => {
                let response: Vec<EventResponse> =
                    events.into_iter().map(EventResponse::from).collect();
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(error) => map_application_error(error),
        }
    } else {
        match repo.list_by_academic_center(ca_id, false).await {
            Ok(events) => {
                let response: Vec<EventResponse> =
                    events.into_iter().map(EventResponse::from).collect();
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(error) => map_application_error(error),
        }
    }
}

/// Handler `GET /api/v1/events/{id}` para consultar os detalhes de um evento pelo seu identificador.
///
/// Respeita o isolamento por CA ativo (quando informado via query `ca_id` ou cabeçalho
/// `X-Academic-Center-Id`) e filtra apenas eventos publicados por padrão (`only_published = true`).
pub async fn get_event_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<ListEventsQueryParams>,
) -> Response {
    let repo = PostgresEventRepository::new(state.pool);
    execute_get_event_by_id(repo, id, &params, &headers).await
}

async fn execute_get_event_by_id<R: EventRepository>(
    repo: R,
    id: Uuid,
    params: &ListEventsQueryParams,
    headers: &HeaderMap,
) -> Response {
    let active_ca_id = extract_active_ca_id(params, headers);
    let only_published = params.should_filter_only_published();

    if only_published {
        let use_case = GetPublishedEventDetailUseCase::new(Arc::new(repo));
        let query = match active_ca_id {
            Some(ca_id) => GetPublishedEventDetailQuery::for_academic_center(id, ca_id),
            None => GetPublishedEventDetailQuery::new(id),
        };
        match use_case.execute(query).await {
            Ok(event) => (StatusCode::OK, Json(EventDetailResponse::from(event))).into_response(),
            Err(error) => map_application_error(error),
        }
    } else {
        let service = EventService::new(repo);
        let query = GetEventByIdQuery {
            event_id: id,
            academic_center_id: active_ca_id,
            only_published: false,
        };
        match service.get_event_by_id(query).await {
            Ok(event) => (StatusCode::OK, Json(EventDetailResponse::from(event))).into_response(),
            Err(error) => map_application_error(error),
        }
    }
}

/// Handler `PUT /api/v1/events/{id}` para editar as informações de um evento existente.
pub async fn update_event_handler(
    State(state): State<AppState>,
    authenticated_user: Option<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEventRequest>,
) -> Response {
    let service = EventService::with_authorizer(
        PostgresEventRepository::new(state.pool.clone()),
        PostgresEventAuthorizationService::new(state.pool),
    );
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

    let actor_id = authenticated_user
        .map(|user| user.user_id)
        .or(payload.actor_id);

    let command = UpdateEventCommand {
        event_id: id,
        academic_center_id: payload.academic_center_id,
        actor_id,
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
///
/// Extrai o usuário autenticado via [`AuthenticatedUser`] e verifica por meio de
/// [`PostgresEventAuthorizationService`] se ele possui [`DIRECT_EVENT_MANAGEMENT_PERMISSION`](crate::modules::events::domain::DIRECT_EVENT_MANAGEMENT_PERMISSION)
/// no Centro Acadêmico do evento antes de efetivar a publicação direta sem homologação.
pub async fn publish_event_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    payload: Option<Json<PublishEventRequest>>,
) -> Response {
    let service = EventService::with_authorizer(
        PostgresEventRepository::new(state.pool.clone()),
        PostgresEventAuthorizationService::new(state.pool),
    );
    let body = payload.map(|json| json.0).unwrap_or_default();

    let command = PublishEventCommand {
        event_id: id,
        academic_center_id: body.academic_center_id,
        actor_id: Some(authenticated_user.user_id),
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
    let repo = PostgresEventRepository::new(state.pool);
    execute_list_ca_events(repo, ca_id, &params).await
}

async fn execute_list_ca_events<R: EventRepository>(
    repo: R,
    ca_id: Uuid,
    params: &ListEventsQueryParams,
) -> Response {
    let only_published = params.should_filter_only_published();
    if only_published {
        let use_case = ListPublishedEventsByCaUseCase::new(Arc::new(repo));
        let query = ListPublishedEventsByCaQuery::new(ca_id);
        match use_case.execute(query).await {
            Ok(events) => {
                let response: Vec<EventResponse> =
                    events.into_iter().map(EventResponse::from).collect();
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(error) => map_application_error(error),
        }
    } else {
        let service = EventService::new(repo);
        let query = ListCaEventsQuery {
            academic_center_id: ca_id,
            only_published: false,
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
}

/// Handler `GET /api/v1/academic-centers/{ca_id}/events/{id}` para consultar detalhes de um evento publicado de um CA específico.
pub async fn get_ca_event_detail_handler(
    State(state): State<AppState>,
    Path((ca_id, id)): Path<(Uuid, Uuid)>,
) -> Response {
    let repo = Arc::new(PostgresEventRepository::new(state.pool));
    execute_get_ca_event_detail(repo, ca_id, id).await
}

async fn execute_get_ca_event_detail<R: EventRepository>(
    repo: Arc<R>,
    ca_id: Uuid,
    id: Uuid,
) -> Response {
    let use_case = GetPublishedEventDetailUseCase::new(repo);
    let query = GetPublishedEventDetailQuery::for_academic_center(id, ca_id);

    match use_case.execute(query).await {
        Ok(event) => (StatusCode::OK, Json(EventDetailResponse::from(event))).into_response(),
        Err(error) => map_application_error(error),
    }
}

/// Handler `POST /api/v1/events/{id}/registrations` para inscrever o usuário autenticado em um evento publicado.
pub async fn register_in_event_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<ListEventsQueryParams>,
) -> Response {
    let repo = Arc::new(PostgresEventRepository::new(state.pool));
    let active_ca_id = extract_active_ca_id(&params, &headers);
    execute_register_in_event(repo, id, authenticated_user.user_id, active_ca_id).await
}

/// Handler `POST /api/v1/academic-centers/{ca_id}/events/{id}/registrations` para inscrever o usuário autenticado em um evento publicado de um CA específico.
pub async fn register_in_ca_event_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path((ca_id, id)): Path<(Uuid, Uuid)>,
) -> Response {
    let repo = Arc::new(PostgresEventRepository::new(state.pool));
    execute_register_in_event(repo, id, authenticated_user.user_id, Some(ca_id)).await
}

async fn execute_register_in_event<R: EventRepository>(
    repo: Arc<R>,
    event_id: Uuid,
    participant_id: Uuid,
    academic_center_id: Option<Uuid>,
) -> Response {
    let use_case = RegisterParticipantInEventUseCase::new(repo);
    let command = RegisterInEventCommand {
        event_id,
        participant_id,
        academic_center_id,
    };

    match use_case.execute(command).await {
        Ok(registration) => (
            StatusCode::CREATED,
            Json(EventRegistrationResponse::confirmed(registration)),
        )
            .into_response(),
        Err(error) => map_application_error(error),
    }
}

/// Handler `DELETE /api/v1/events/{id}/registrations` para cancelar a inscrição ativa do usuário autenticado em um evento.
pub async fn cancel_event_registration_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<ListEventsQueryParams>,
) -> Response {
    let repo = Arc::new(PostgresEventRepository::new(state.pool));
    let active_ca_id = extract_active_ca_id(&params, &headers);
    execute_cancel_event_registration(repo, id, authenticated_user.user_id, active_ca_id).await
}

/// Handler `DELETE /api/v1/academic-centers/{ca_id}/events/{id}/registrations` para cancelar a inscrição ativa do usuário autenticado em um evento de um CA específico.
pub async fn cancel_ca_event_registration_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path((ca_id, id)): Path<(Uuid, Uuid)>,
) -> Response {
    let repo = Arc::new(PostgresEventRepository::new(state.pool));
    execute_cancel_event_registration(repo, id, authenticated_user.user_id, Some(ca_id)).await
}

async fn execute_cancel_event_registration<R: EventRepository>(
    repo: Arc<R>,
    event_id: Uuid,
    participant_id: Uuid,
    academic_center_id: Option<Uuid>,
) -> Response {
    let use_case = CancelEventRegistrationUseCase::new(repo);
    let command = CancelEventRegistrationCommand {
        event_id,
        participant_id,
        academic_center_id,
    };

    match use_case.execute(command).await {
        Ok(registration) => (
            StatusCode::OK,
            Json(EventRegistrationResponse::cancelled(registration)),
        )
            .into_response(),
        Err(error) => map_application_error(error),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::Request,
        routing::{get, post},
    };
    use chrono::{Duration, Utc};
    use tower::ServiceExt;

    use super::*;
    use crate::modules::events::{
        application::InMemoryEventRepository,
        domain::{Event, EventPublicationStatus, RegistrationStatus},
    };

    fn test_read_router(repo: Arc<InMemoryEventRepository>) -> Router {
        Router::new()
            .route(
                "/api/v1/events",
                get(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     headers: HeaderMap,
                     Query(params): Query<ListEventsQueryParams>| async move {
                        execute_list_published_events(repo, &params, &headers).await
                    },
                ),
            )
            .route(
                "/api/v1/events/{id}",
                get(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     headers: HeaderMap,
                     Path(id): Path<Uuid>,
                     Query(params): Query<ListEventsQueryParams>| async move {
                        execute_get_event_by_id(repo.as_ref().clone(), id, &params, &headers).await
                    },
                ),
            )
            .route(
                "/api/v1/events/{id}/registrations",
                post(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     authenticated_user: AuthenticatedUser,
                     headers: HeaderMap,
                     Path(id): Path<Uuid>,
                     Query(params): Query<ListEventsQueryParams>| async move {
                        let active_ca_id = extract_active_ca_id(&params, &headers);
                        execute_register_in_event(
                            repo,
                            id,
                            authenticated_user.user_id,
                            active_ca_id,
                        )
                        .await
                    },
                )
                .delete(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     authenticated_user: AuthenticatedUser,
                     headers: HeaderMap,
                     Path(id): Path<Uuid>,
                     Query(params): Query<ListEventsQueryParams>| async move {
                        let active_ca_id = extract_active_ca_id(&params, &headers);
                        execute_cancel_event_registration(
                            repo,
                            id,
                            authenticated_user.user_id,
                            active_ca_id,
                        )
                        .await
                    },
                ),
            )
            .route(
                "/api/v1/academic-centers/{ca_id}/events",
                get(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     Path(ca_id): Path<Uuid>,
                     Query(params): Query<ListEventsQueryParams>| async move {
                        execute_list_ca_events(repo.as_ref().clone(), ca_id, &params).await
                    },
                ),
            )
            .route(
                "/api/v1/academic-centers/{ca_id}/events/{id}",
                get(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     Path((ca_id, id)): Path<(Uuid, Uuid)>| async move {
                        execute_get_ca_event_detail(repo, ca_id, id).await
                    },
                ),
            )
            .route(
                "/api/v1/academic-centers/{ca_id}/events/{id}/registrations",
                post(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     authenticated_user: AuthenticatedUser,
                     Path((ca_id, id)): Path<(Uuid, Uuid)>| async move {
                        execute_register_in_event(repo, id, authenticated_user.user_id, Some(ca_id))
                            .await
                    },
                )
                .delete(
                    |State(repo): State<Arc<InMemoryEventRepository>>,
                     authenticated_user: AuthenticatedUser,
                     Path((ca_id, id)): Path<(Uuid, Uuid)>| async move {
                        execute_cancel_event_registration(
                            repo,
                            id,
                            authenticated_user.user_id,
                            Some(ca_id),
                        )
                        .await
                    },
                ),
            )
            .with_state(repo)
    }

    fn build_event(
        ca_id: Uuid,
        title: &str,
        status: EventPublicationStatus,
        starts_in_hours: i64,
    ) -> Event {
        let now = Utc::now();
        let starts_at = now + Duration::hours(starts_in_hours);
        let ends_at = starts_at + Duration::hours(2);
        let published_at = if status == EventPublicationStatus::Published {
            Some(now)
        } else {
            None
        };

        Event {
            id: Uuid::new_v4(),
            academic_center_id: ca_id,
            author_id: Uuid::new_v4(),
            title: title.to_string(),
            description: format!("Descrição detalhada de {title}"),
            location: "Auditório Principal".to_string(),
            starts_at,
            ends_at,
            status,
            registration: EventRegistrationInfo::with_capacity(100)
                .expect("configuração de inscrição válida"),
            published_at,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn get_events_lists_only_published_events_for_active_ca() {
        let ca_cc = Uuid::new_v4();
        let ca_fisica = Uuid::new_v4();

        let published_1 = build_event(
            ca_cc,
            "Semana da Computação",
            EventPublicationStatus::Published,
            24,
        );
        let published_2 = build_event(
            ca_cc,
            "Torneio de Programação",
            EventPublicationStatus::Published,
            48,
        );
        let draft = build_event(ca_cc, "Rascunho Interno", EventPublicationStatus::Draft, 72);
        let pending = build_event(
            ca_cc,
            "Proposta Pendente",
            EventPublicationStatus::PendingApproval,
            96,
        );
        let other_ca = build_event(
            ca_fisica,
            "Simpósio de Física",
            EventPublicationStatus::Published,
            12,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events(vec![
            published_2.clone(),
            draft,
            published_1.clone(),
            pending,
            other_ca,
        ]));
        let app = test_read_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/events?ca_id={ca_cc}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let listed: Vec<EventResponse> = serde_json::from_slice(&body).unwrap();

        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, published_1.id);
        assert_eq!(listed[0].title, "Semana da Computação");
        assert_eq!(listed[1].id, published_2.id);
        assert_eq!(listed[1].title, "Torneio de Programação");
    }

    #[tokio::test]
    async fn get_events_returns_empty_list_when_ca_has_no_published_events() {
        let ca_cc = Uuid::new_v4();
        let draft = build_event(ca_cc, "Rascunho", EventPublicationStatus::Draft, 24);
        let repo = Arc::new(InMemoryEventRepository::with_events(vec![draft]));
        let app = test_read_router(repo);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/academic-centers/{ca_cc}/events"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let listed: Vec<EventResponse> = serde_json::from_slice(&body).unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn get_event_details_enforces_published_status_and_ca_isolation() {
        let ca_cc = Uuid::new_v4();
        let ca_fisica = Uuid::new_v4();

        let published_cc = build_event(
            ca_cc,
            "Hackathon de Inverno",
            EventPublicationStatus::Published,
            24,
        );
        let draft_cc = build_event(
            ca_cc,
            "Hackathon Rascunho",
            EventPublicationStatus::Draft,
            48,
        );
        let pending_cc = build_event(
            ca_cc,
            "Proposta Não Homologada",
            EventPublicationStatus::PendingApproval,
            72,
        );
        let published_fisica = build_event(
            ca_fisica,
            "Colóquio de Física Quântica",
            EventPublicationStatus::Published,
            24,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events(vec![
            published_cc.clone(),
            draft_cc.clone(),
            pending_cc.clone(),
            published_fisica.clone(),
        ]));
        let app = test_read_router(repo);

        // 1. Consulta de detalhes de evento publicado do CA ativo retorna 200 OK
        let ok_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/academic-centers/{ca_cc}/events/{}",
                        published_cc.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok_response.status(), StatusCode::OK);
        let body = to_bytes(ok_response.into_body(), usize::MAX).await.unwrap();
        let detail: EventDetailResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(detail.id, published_cc.id);
        assert_eq!(detail.title, "Hackathon de Inverno");
        assert_eq!(detail.max_capacity, Some(100));

        // 2. Consulta de evento de outro CA a partir do contexto do CA de Computação retorna 404
        let cross_ca_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/events/{}?ca_id={ca_cc}",
                        published_fisica.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cross_ca_response.status(), StatusCode::NOT_FOUND);

        // 3. Consulta de evento em rascunho retorna 404 Not Found por padrão
        let draft_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/events/{}?ca_id={ca_cc}", draft_cc.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(draft_response.status(), StatusCode::NOT_FOUND);

        // 4. Consulta de proposta pendente de homologação retorna 404 Not Found
        let pending_response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/academic-centers/{ca_cc}/events/{}",
                        pending_cc.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pending_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn post_and_delete_event_registration_succeeds_for_published_event() {
        let ca_cc = Uuid::new_v4();
        let student = Uuid::new_v4();
        let published = build_event(
            ca_cc,
            "Palestra de IA",
            EventPublicationStatus::Published,
            24,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events(vec![
            published.clone(),
        ]));
        let app = test_read_router(repo);

        // 1. POST /api/v1/academic-centers/{ca_id}/events/{id}/registrations -> 201 Created
        let register_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/academic-centers/{ca_cc}/events/{}/registrations",
                        published.id
                    ))
                    .header("X-User-Id", student.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(register_res.status(), StatusCode::CREATED);
        let body = to_bytes(register_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let registration: EventRegistrationResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(registration.event_id, published.id);
        assert_eq!(registration.participant_id, student);
        assert_eq!(registration.status, RegistrationStatus::Confirmed);
        assert!(registration.cancelled_at.is_none());

        // 2. DELETE /api/v1/academic-centers/{ca_id}/events/{id}/registrations -> 200 OK
        let cancel_res = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/api/v1/academic-centers/{ca_cc}/events/{}/registrations",
                        published.id
                    ))
                    .header("X-User-Id", student.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cancel_res.status(), StatusCode::OK);
        let body = to_bytes(cancel_res.into_body(), usize::MAX).await.unwrap();
        let cancelled: EventRegistrationResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(cancelled.event_id, published.id);
        assert_eq!(cancelled.participant_id, student);
        assert_eq!(cancelled.status, RegistrationStatus::Cancelled);
        assert!(cancelled.cancelled_at.is_some());
    }

    #[tokio::test]
    async fn post_event_registration_rejects_duplicate_and_unpublished_or_other_ca() {
        let ca_cc = Uuid::new_v4();
        let ca_fisica = Uuid::new_v4();
        let student = Uuid::new_v4();

        let published_cc = build_event(
            ca_cc,
            "Palestra de IA",
            EventPublicationStatus::Published,
            24,
        );
        let pending_cc = build_event(
            ca_cc,
            "Hackathon em análise",
            EventPublicationStatus::PendingApproval,
            48,
        );
        let draft_cc = build_event(ca_cc, "Oficina Rascunho", EventPublicationStatus::Draft, 72);

        let repo = Arc::new(InMemoryEventRepository::with_events(vec![
            published_cc.clone(),
            pending_cc.clone(),
            draft_cc.clone(),
        ]));
        let app = test_read_router(repo);

        // 1. Primeira inscrição em evento publicado retorna 201 Created
        let first_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/events/{}/registrations?ca_id={ca_cc}",
                        published_cc.id
                    ))
                    .header("X-User-Id", student.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first_res.status(), StatusCode::CREATED);

        // 2. Segunda inscrição no mesmo evento retorna 409 Conflict ("already_registered")
        let duplicate_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/events/{}/registrations?ca_id={ca_cc}",
                        published_cc.id
                    ))
                    .header("X-User-Id", student.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(duplicate_res.status(), StatusCode::CONFLICT);
        let body = to_bytes(duplicate_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let err: EventErrorResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(err.code, "already_registered");

        // 3. Inscrição em proposta não homologada ou rascunho retorna 422 ("event_not_available")
        for unpublished in [&pending_cc, &draft_cc] {
            let unpub_res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!(
                            "/api/v1/events/{}/registrations?ca_id={ca_cc}",
                            unpublished.id
                        ))
                        .header("X-User-Id", student.to_string())
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(unpub_res.status(), StatusCode::UNPROCESSABLE_ENTITY);
            let body = to_bytes(unpub_res.into_body(), usize::MAX).await.unwrap();
            let err: EventErrorResponse = serde_json::from_slice(&body).unwrap();
            assert_eq!(err.code, "event_not_available");
        }

        // 4. Inscrição em evento de outro CA retorna 404 Not Found
        let cross_ca_res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/academic-centers/{ca_fisica}/events/{}/registrations",
                        published_cc.id
                    ))
                    .header("X-User-Id", student.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cross_ca_res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_event_registration_rejects_when_no_active_registration_exists() {
        let ca_cc = Uuid::new_v4();
        let student = Uuid::new_v4();
        let published = build_event(
            ca_cc,
            "Palestra de IA",
            EventPublicationStatus::Published,
            24,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events(vec![
            published.clone(),
        ]));
        let app = test_read_router(repo);

        // Tentativa de cancelar sem inscrição ativa retorna 409 Conflict ("no_active_registration")
        let cancel_res = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/api/v1/events/{}/registrations?ca_id={ca_cc}",
                        published.id
                    ))
                    .header("X-User-Id", student.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cancel_res.status(), StatusCode::CONFLICT);
        let body = to_bytes(cancel_res.into_body(), usize::MAX).await.unwrap();
        let err: EventErrorResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(err.code, "no_active_registration");
    }
}
