//! Handlers HTTP para consultas e enquetes estudantis do CA.
//!
//! Conecta os endpoints REST aos casos de uso de aplicação, realizando autenticação,
//! conversão de DTOs e mapeamento padronizado de respostas e erros HTTP.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::{
        ca::{
            api::dto::{CreatePollRequest, PollResponse, VotePollRequest, VoteResponse},
            application::{
                commands::{
                    ClosePollCommand, CreatePollCommand, VotePollCommand, execute_close_poll,
                    execute_create_poll, execute_vote_poll,
                },
                error::PollApplicationError,
                ports::{DefaultAcademicCenterAuthorization, PollRepository},
                queries::{GetPollQuery, ListPollsQuery, execute_get_poll, execute_list_polls},
            },
            domain::entities::PollError,
        },
        identity::{
            api::extractors::AuthenticatedInstitutionalIdentity, application::ports::UserRepository,
        },
    },
};

/// Converte erros da aplicação de consultas em [`ApiError`] com status e código adequados.
fn map_poll_app_error(err: PollApplicationError) -> ApiError {
    match err {
        PollApplicationError::AcademicCenterNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "ca_not_found",
            format!("Centro Acadêmico '{id}' não encontrado"),
            None,
        ),
        PollApplicationError::AcademicCenterInactive(id) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ca_inactive",
            format!("Centro Acadêmico '{id}' encontra-se inativo ou arquivado"),
            None,
        ),
        PollApplicationError::PollNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "poll_not_found",
            format!("Consulta estudantil '{id}' não encontrada"),
            None,
        ),
        PollApplicationError::UserNotPermittedInCa { .. } => ApiError::new(
            StatusCode::FORBIDDEN,
            "user_not_permitted_in_ca",
            "Usuário não possui permissão de gestão no Centro Acadêmico",
            None,
        ),
        PollApplicationError::StudentNotEligible { .. } => ApiError::new(
            StatusCode::FORBIDDEN,
            "student_not_eligible",
            "Estudante não é elegível para participar desta consulta",
            None,
        ),
        PollApplicationError::DuplicateVote { .. } => ApiError::new(
            StatusCode::CONFLICT,
            "duplicate_vote",
            "O estudante já registrou seu voto nesta consulta",
            None,
        ),
        PollApplicationError::VotingNotStarted { starts_at } => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "poll_voting_not_started",
            format!(
                "O período de votação desta consulta ainda não começou (início em: {starts_at})"
            ),
            None,
        ),
        PollApplicationError::VotingEnded { ends_at } => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "poll_voting_ended",
            format!("O período de votação desta consulta já terminou (término em: {ends_at})"),
            None,
        ),
        PollApplicationError::PollClosed => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "poll_closed",
            "Esta consulta estudantil encontra-se encerrada",
            None,
        ),
        PollApplicationError::OptionNotFound(id) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "option_not_found",
            format!("Opção de voto '{id}' não pertence a esta consulta"),
            None,
        ),
        PollApplicationError::InvalidPollOptions(msg) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_poll_options",
            format!("Opções de enquete inválidas: {msg}"),
            None,
        ),
        PollApplicationError::InvalidPollDates(msg) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_poll_dates",
            format!("Período de votação inválido: {msg}"),
            None,
        ),
        PollApplicationError::Domain(PollError::EmptyTitle)
        | PollApplicationError::Domain(PollError::TitleTooLong { .. }) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_poll_title",
            "Título da consulta inválido",
            None,
        ),
        PollApplicationError::Domain(PollError::DescriptionTooLong { .. }) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_poll_description",
            "Descrição da consulta excede o limite máximo permitido",
            None,
        ),
        PollApplicationError::Domain(PollError::TooFewOptions { .. })
        | PollApplicationError::Domain(PollError::EmptyOptionTitle)
        | PollApplicationError::Domain(PollError::OptionTitleTooLong { .. })
        | PollApplicationError::Domain(PollError::DuplicateOptionTitles(_)) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_poll_options",
            "Opções de enquete inválidas",
            None,
        ),
        PollApplicationError::Domain(PollError::InvalidPeriod { .. }) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_poll_dates",
            "Período da consulta inválido",
            None,
        ),
        PollApplicationError::Domain(err) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "business_rule_violation",
            err.to_string(),
            None,
        ),
        PollApplicationError::Repository(msg) => {
            tracing::error!(%msg, "Falha de persistência no módulo de consultas");
            ApiError::internal_server_error()
        }
    }
}

/// Cria uma nova consulta estudantil para o Centro Acadêmico especificado.
///
/// `POST /api/v1/academic-centers/{academic_center_id}/polls`
pub(crate) async fn create_poll(
    State(state): State<AppState>,
    identity: AuthenticatedInstitutionalIdentity,
    Path(academic_center_id): Path<Uuid>,
    Json(payload): Json<CreatePollRequest>,
) -> ApiResult<(StatusCode, Json<PollResponse>)> {
    let user = state
        .pg_database
        .get_by_firebase_uid(&identity.0.firebase_uid)
        .await
        .map_err(|_| ApiError::internal_server_error())?
        .ok_or_else(ApiError::unauthorized)?;

    let starts_at = payload.starts_at.unwrap_or_else(Utc::now);

    let cmd = CreatePollCommand {
        academic_center_id,
        created_by_id: user.id(),
        title: payload.title,
        description: payload.description,
        options: payload.options,
        starts_at,
        ends_at: payload.ends_at,
        target_audience: payload.target_audience,
        vote_modality: payload.vote_modality,
    };

    let poll = execute_create_poll(
        &state.pg_database,
        &state.pg_database,
        &DefaultAcademicCenterAuthorization,
        cmd,
    )
    .await
    .map_err(map_poll_app_error)?;

    let vote_counts = std::collections::HashMap::new();
    let results = poll.tally_results(&vote_counts);
    let response = PollResponse::from_entity_and_results(&poll, &results);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Lista todas as consultas estudantis de um Centro Acadêmico.
///
/// `GET /api/v1/academic-centers/{academic_center_id}/polls`
pub(crate) async fn list_polls(
    State(state): State<AppState>,
    _identity: AuthenticatedInstitutionalIdentity,
    Path(academic_center_id): Path<Uuid>,
) -> ApiResult<Json<Vec<PollResponse>>> {
    let query = ListPollsQuery { academic_center_id };

    let pairs = execute_list_polls(&state.pg_database, &state.pg_database, query)
        .await
        .map_err(map_poll_app_error)?;

    let responses = pairs
        .iter()
        .map(|(poll, results)| PollResponse::from_entity_and_results(poll, results))
        .collect();

    Ok(Json(responses))
}

/// Recupera os detalhes e resultados apurados de uma consulta estudantil.
///
/// `GET /api/v1/academic-centers/{academic_center_id}/polls/{poll_id}`
pub(crate) async fn get_poll(
    State(state): State<AppState>,
    _identity: AuthenticatedInstitutionalIdentity,
    Path((academic_center_id, poll_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<PollResponse>> {
    let query = GetPollQuery {
        academic_center_id,
        poll_id,
    };

    let (poll, results) = execute_get_poll(&state.pg_database, &state.pg_database, query)
        .await
        .map_err(map_poll_app_error)?;

    let response = PollResponse::from_entity_and_results(&poll, &results);
    Ok(Json(response))
}

/// Registra o voto de um estudante em uma consulta estudantil.
///
/// `POST /api/v1/academic-centers/{academic_center_id}/polls/{poll_id}/votes`
pub(crate) async fn vote_poll(
    State(state): State<AppState>,
    identity: AuthenticatedInstitutionalIdentity,
    Path((academic_center_id, poll_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<VotePollRequest>,
) -> ApiResult<(StatusCode, Json<VoteResponse>)> {
    let user = state
        .pg_database
        .get_by_firebase_uid(&identity.0.firebase_uid)
        .await
        .map_err(|_| ApiError::internal_server_error())?
        .ok_or_else(ApiError::unauthorized)?;

    let cmd = VotePollCommand {
        academic_center_id,
        poll_id,
        option_id: payload.option_id,
        user_id: user.id(),
        voted_at: Utc::now(),
    };

    let vote = execute_vote_poll(
        &state.pg_database,
        &state.pg_database,
        &DefaultAcademicCenterAuthorization,
        cmd,
    )
    .await
    .map_err(map_poll_app_error)?;

    Ok((StatusCode::CREATED, Json(VoteResponse::from(&vote))))
}

/// Encerra antecipadamente uma consulta estudantil.
///
/// `POST /api/v1/academic-centers/{academic_center_id}/polls/{poll_id}/close`
pub(crate) async fn close_poll(
    State(state): State<AppState>,
    identity: AuthenticatedInstitutionalIdentity,
    Path((academic_center_id, poll_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<PollResponse>> {
    let user = state
        .pg_database
        .get_by_firebase_uid(&identity.0.firebase_uid)
        .await
        .map_err(|_| ApiError::internal_server_error())?
        .ok_or_else(ApiError::unauthorized)?;

    let cmd = ClosePollCommand {
        academic_center_id,
        poll_id,
        closed_by_id: user.id(),
        closed_at: Utc::now(),
    };

    let closed_poll = execute_close_poll(
        &state.pg_database,
        &state.pg_database,
        &DefaultAcademicCenterAuthorization,
        cmd,
    )
    .await
    .map_err(map_poll_app_error)?;

    let vote_counts = state
        .pg_database
        .count_votes_by_option(poll_id)
        .await
        .map_err(|e| {
            tracing::error!(%e, "Falha ao apurar votos após encerramento");
            ApiError::internal_server_error()
        })?;

    let results = closed_poll.tally_results(&vote_counts);
    let response = PollResponse::from_entity_and_results(&closed_poll, &results);

    Ok(Json(response))
}
