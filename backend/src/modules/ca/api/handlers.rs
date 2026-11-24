//! Handlers HTTP do módulo de Centros Acadêmicos.
//!
//! Implementa os endpoints REST para configuração inicial e consulta
//! de mandatos de gestão do Centro Acadêmico (Issue #296).

use crate::api::error::ApiError;
use crate::api::error::ApiResult;
use crate::api::state::AppState;
use crate::modules::ca::api::dto::AddMandateMemberRequest;
use crate::modules::ca::api::dto::ConfigureInitialMandateRequest;
use crate::modules::ca::api::dto::CreatePollRequest;
use crate::modules::ca::api::dto::MandateMemberResponse;
use crate::modules::ca::api::dto::MandateResponse;
use crate::modules::ca::api::dto::PollResponse;
use crate::modules::ca::api::dto::TransitionMandateRequest;
use crate::modules::ca::api::dto::VotePollRequest;
use crate::modules::ca::api::dto::VoteResponse;
use crate::modules::ca::application::commands::AddMandateMemberCommand;
use crate::modules::ca::application::commands::ClosePollCommand;
use crate::modules::ca::application::commands::ConcludeActiveMandateCommand;
use crate::modules::ca::application::commands::ConfigureInitialMandateCommand;
use crate::modules::ca::application::commands::CreatePollCommand;
use crate::modules::ca::application::commands::DisaffiliateMandateMemberCommand;
use crate::modules::ca::application::commands::TransitionMandateCommand;
use crate::modules::ca::application::commands::VotePollCommand;
use crate::modules::ca::application::commands::add_mandate_member as run_add_mandate_member;
use crate::modules::ca::application::commands::conclude_active_mandate as run_conclude_active_mandate;
use crate::modules::ca::application::commands::configure_initial_mandate as run_configure_initial_mandate;
use crate::modules::ca::application::commands::disaffiliate_mandate_member as run_disaffiliate_mandate_member;
use crate::modules::ca::application::commands::execute_close_poll;
use crate::modules::ca::application::commands::execute_create_poll;
use crate::modules::ca::application::commands::execute_vote_poll;
use crate::modules::ca::application::commands::transition_mandate as run_transition_mandate;
use crate::modules::ca::application::error::MandateApplicationError;
use crate::modules::ca::application::error::PollApplicationError;
use crate::modules::ca::application::ports::DefaultAcademicCenterAuthorization;
use crate::modules::ca::application::ports::DefaultPollAuthorization;
use crate::modules::ca::application::ports::PollRepository;
use crate::modules::ca::application::queries::GetPollQuery;
use crate::modules::ca::application::queries::ListPollsQuery;
use crate::modules::ca::application::queries::execute_get_poll;
use crate::modules::ca::application::queries::execute_list_polls;
use crate::modules::ca::application::queries::get_current_mandate as run_get_current_mandate;
use crate::modules::ca::application::queries::get_mandate_by_id as run_get_mandate_by_id;
use crate::modules::ca::application::queries::get_mandates_history as run_get_mandates_history;
use crate::modules::ca::application::queries::list_mandates as run_list_mandates;
use crate::modules::ca::domain::entities::PollError;
use crate::modules::identity::api::extractors::AuthenticatedInstitutionalIdentity;
use crate::modules::identity::api::extractors::firebase_bearer_token;
use crate::modules::identity::application::ports::IdentityTokenVerifier;
use crate::modules::identity::application::ports::UserRepository;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
/// Converte um erro de aplicação de mandato para a resposta [`ApiError`] apropriada.
fn map_mandate_application_error(error: MandateApplicationError) -> ApiError {
    match error {
        MandateApplicationError::AcademicCenterNotFound(_) => ApiError::new(
            StatusCode::NOT_FOUND,
            "ca_not_found",
            "Centro Acadêmico não encontrado",
            None,
        ),
        MandateApplicationError::AcademicCenterInactive(_) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ca_inactive",
            "Centro Acadêmico encontra-se inativo ou arquivado",
            None,
        ),
        MandateApplicationError::UserNotPermittedInCa { .. } => ApiError::new(
            StatusCode::FORBIDDEN,
            "user_not_permitted_in_ca",
            "Usuário não possui permissão para gerenciar este Centro Acadêmico",
            None,
        ),
        MandateApplicationError::InvalidMandateDates(msg) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_mandate_dates",
            msg,
            None,
        ),
        MandateApplicationError::ActiveMandateAlreadyExists(_) => ApiError::new(
            StatusCode::CONFLICT,
            "active_mandate_already_exists",
            "O Centro Acadêmico já possui um mandato de gestão ativo",
            None,
        ),
        MandateApplicationError::ActiveMandateNotFound(_) => ApiError::new(
            StatusCode::NOT_FOUND,
            "no_active_mandate",
            "Não existe gestão vigente registrada para este Centro Acadêmico",
            None,
        ),
        MandateApplicationError::MandateNotFound(_) => ApiError::new(
            StatusCode::NOT_FOUND,
            "mandate_not_found",
            "Mandato de gestão não encontrado",
            None,
        ),
        MandateApplicationError::MemberNotFound(_) => ApiError::new(
            StatusCode::NOT_FOUND,
            "member_not_found",
            "Integrante não encontrado na gestão",
            None,
        ),
        MandateApplicationError::Domain(err) => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_mandate_dates",
            err.to_string(),
            None,
        ),
        MandateApplicationError::Repository(msg) => {
            tracing::error!(%msg, "Erro interno de repositório de mandato");
            ApiError::internal_server_error()
        }
    }
}
/// Autentica o token Firebase da requisição e recupera o usuário interno provisionado correspondente.
async fn authenticate_user(
    state: &AppState,
    headers: &HeaderMap,
) -> ApiResult<crate::modules::identity::domain::entities::user::User> {
    let token = firebase_bearer_token(headers)?;

    let claims = timeout(
        Duration::from_secs(10),
        state.clients.firebase.verify(token),
    )
    .await
    .map_err(|_| {
        tracing::warn!("Timeout na verificação do token Firebase");
        ApiError::service_unavailable()
    })?
    .map_err(|_| ApiError::unauthorized())?;

    let user = state
        .pg_database
        .get_by_firebase_uid(&claims.firebase_uid)
        .await
        .map_err(|_| ApiError::internal_server_error())?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::FORBIDDEN,
                "user_not_permitted_in_ca",
                "Usuário não possui conta provisionada no sistema",
                None,
            )
        })?;

    Ok(user)
}
/// Endpoint `POST /academic-centers/{id}/mandates`: configura o mandato inicial da gestão.
///
/// # Erros e Códigos HTTP
///
/// - `400 bad_request`: JSON malformado ou UUID do CA inválido.
/// - `401 unauthorized`: token de autenticação ausente ou inválido.
/// - `403 user_not_permitted_in_ca`: usuário não possui permissão de gestão no CA.
/// - `404 ca_not_found`: Centro Acadêmico não encontrado.
/// - `409 active_mandate_already_exists`: CA já possui mandato ativo configurado.
/// - `422 invalid_mandate_dates`: inconsistência temporal (`end_date <= start_date`).
/// - `422 ca_inactive`: Centro Acadêmico está arquivado ou inativo.
pub async fn configure_initial_mandate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ca_id_str): Path<String>,
    body: Result<Json<ConfigureInitialMandateRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;

    let user = authenticate_user(&state, &headers).await?;

    let Json(body) = body.map_err(|_| ApiError::bad_request())?;

    let command = ConfigureInitialMandateCommand {
        academic_center_id: ca_id,
        actor_user_id: user.id(),
        title: body.title,
        start_date: body.start_date,
        end_date: body.end_date,
    };

    let auth = DefaultAcademicCenterAuthorization;

    let mandate = timeout(
        Duration::from_secs(15),
        run_configure_initial_mandate(&state.pg_database, &state.pg_database, &auth, command),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    Ok((StatusCode::CREATED, Json(MandateResponse::from(mandate))))
}
/// Endpoint `GET /academic-centers/{id}/mandates/current`: consulta o mandato vigente da gestão.
///
/// # Erros e Códigos HTTP
///
/// - `400 bad_request`: UUID do CA inválido.
/// - `401 unauthorized`: token ausente ou inválido.
/// - `404 ca_not_found`: Centro Acadêmico não encontrado.
/// - `404 no_active_mandate`: nenhuma gestão ativa registrada para o CA.
pub async fn get_current_mandate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ca_id_str): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;

    // Autenticação exigida pela especificação de segurança
    let _ = authenticate_user(&state, &headers).await?;

    let mandate = timeout(
        Duration::from_secs(15),
        run_get_current_mandate(&state.pg_database, &state.pg_database, ca_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?
    .ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "no_active_mandate",
            "Não existe gestão vigente registrada para este Centro Acadêmico",
            None,
        )
    })?;

    Ok((StatusCode::OK, Json(MandateResponse::from(mandate))))
}
/// Endpoint `GET /academic-centers/{id}/mandates`: lista todos os mandatos do Centro Acadêmico.
pub async fn list_mandates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ca_id_str): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;

    let _ = authenticate_user(&state, &headers).await?;

    let mandates = timeout(
        Duration::from_secs(15),
        run_list_mandates(&state.pg_database, &state.pg_database, ca_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    let response: Vec<MandateResponse> = mandates.into_iter().map(MandateResponse::from).collect();

    Ok((StatusCode::OK, Json(response)))
}
/// Endpoint `GET /academic-centers/{id}/mandates/history`: consulta o histórico completo de gestões.
///
/// # Erros e Códigos HTTP
///
/// - `400 bad_request`: UUID do CA inválido.
/// - `401 unauthorized`: token ausente ou inválido.
/// - `404 ca_not_found`: Centro Acadêmico não encontrado.
pub async fn get_mandates_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ca_id_str): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;

    let _ = authenticate_user(&state, &headers).await?;

    let mandates = timeout(
        Duration::from_secs(15),
        run_get_mandates_history(&state.pg_database, &state.pg_database, ca_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    let response: Vec<MandateResponse> = mandates.into_iter().map(MandateResponse::from).collect();

    Ok((StatusCode::OK, Json(response)))
}
/// Endpoint `GET /academic-centers/{id}/mandates/{mandate_id}`: consulta detalhes de uma gestão específica.
///
/// # Erros e Códigos HTTP
///
/// - `400 bad_request`: UUID do CA ou do mandato inválido.
/// - `401 unauthorized`: token ausente ou inválido.
/// - `404 ca_not_found`: Centro Acadêmico não encontrado.
/// - `404 mandate_not_found`: mandato não encontrado para o respectivo CA.
pub async fn get_mandate_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((ca_id_str, mandate_id_str)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;
    let mandate_id = Uuid::parse_str(&mandate_id_str).map_err(|_| ApiError::bad_request())?;

    let _ = authenticate_user(&state, &headers).await?;

    let mandate = timeout(
        Duration::from_secs(15),
        run_get_mandate_by_id(&state.pg_database, &state.pg_database, ca_id, mandate_id),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    Ok((StatusCode::OK, Json(MandateResponse::from(mandate))))
}
/// Endpoint `POST /academic-centers/{id}/mandates/conclude`: encerra e conclui o mandato ativo.
pub async fn conclude_active_mandate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ca_id_str): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;
    let user = authenticate_user(&state, &headers).await?;

    let command = ConcludeActiveMandateCommand {
        academic_center_id: ca_id,
        actor_user_id: user.id(),
    };

    let auth = DefaultAcademicCenterAuthorization;

    let mandate = timeout(
        Duration::from_secs(15),
        run_conclude_active_mandate(&state.pg_database, &state.pg_database, &auth, command),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    Ok((StatusCode::OK, Json(MandateResponse::from(mandate))))
}
/// Endpoint `POST /academic-centers/{id}/mandates/transition`: transiciona para uma nova gestão preservando a anterior.
pub async fn transition_mandate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ca_id_str): Path<String>,
    body: Result<Json<TransitionMandateRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;
    let user = authenticate_user(&state, &headers).await?;

    let Json(body) = body.map_err(|_| ApiError::bad_request())?;

    let command = TransitionMandateCommand {
        academic_center_id: ca_id,
        actor_user_id: user.id(),
        title: body.title,
        start_date: body.start_date,
        end_date: body.end_date,
    };

    let auth = DefaultAcademicCenterAuthorization;

    let mandate = timeout(
        Duration::from_secs(15),
        run_transition_mandate(&state.pg_database, &state.pg_database, &auth, command),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    Ok((StatusCode::CREATED, Json(MandateResponse::from(mandate))))
}
/// Endpoint `POST /academic-centers/{id}/mandates/{mandate_id}/members`: vincula um integrante à gestão.
pub async fn add_mandate_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((ca_id_str, mandate_id_str)): Path<(String, String)>,
    body: Result<Json<AddMandateMemberRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;
    let mandate_id = Uuid::parse_str(&mandate_id_str).map_err(|_| ApiError::bad_request())?;
    let user = authenticate_user(&state, &headers).await?;

    let Json(body) = body.map_err(|_| ApiError::bad_request())?;

    let command = AddMandateMemberCommand {
        academic_center_id: ca_id,
        mandate_id,
        actor_user_id: user.id(),
        member_user_id: body.user_id,
        name: body.name,
        role: body.role,
    };

    let auth = DefaultAcademicCenterAuthorization;

    let member = timeout(
        Duration::from_secs(15),
        run_add_mandate_member(&state.pg_database, &state.pg_database, &auth, command),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    Ok((
        StatusCode::CREATED,
        Json(MandateMemberResponse::from(member)),
    ))
}
/// Endpoint `POST /academic-centers/{id}/mandates/{mandate_id}/members/{member_id}/disaffiliate`: encerra vínculo de integrante mantendo participação histórica.
pub async fn disaffiliate_mandate_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((ca_id_str, mandate_id_str, member_id_str)): Path<(String, String, String)>,
) -> ApiResult<impl IntoResponse> {
    let ca_id = Uuid::parse_str(&ca_id_str).map_err(|_| ApiError::bad_request())?;
    let mandate_id = Uuid::parse_str(&mandate_id_str).map_err(|_| ApiError::bad_request())?;
    let member_id = Uuid::parse_str(&member_id_str).map_err(|_| ApiError::bad_request())?;
    let user = authenticate_user(&state, &headers).await?;

    let command = DisaffiliateMandateMemberCommand {
        academic_center_id: ca_id,
        mandate_id,
        member_id,
        actor_user_id: user.id(),
    };

    let auth = DefaultAcademicCenterAuthorization;

    let member = timeout(
        Duration::from_secs(15),
        run_disaffiliate_mandate_member(&state.pg_database, &state.pg_database, &auth, command),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())?
    .map_err(map_mandate_application_error)?;

    Ok((StatusCode::OK, Json(MandateMemberResponse::from(member))))
}
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
        &DefaultPollAuthorization,
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
        &DefaultPollAuthorization,
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
        &DefaultPollAuthorization,
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
