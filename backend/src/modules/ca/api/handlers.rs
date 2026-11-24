//! Handlers HTTP do módulo de Centros Acadêmicos.
//!
//! Implementa os endpoints REST para configuração inicial e consulta
//! de mandatos de gestão do Centro Acadêmico (Issue #296).

use std::time::Duration;

use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::{
        ca::{
            api::dto::{ConfigureInitialMandateRequest, MandateResponse},
            application::{
                commands::{
                    ConfigureInitialMandateCommand,
                    configure_initial_mandate as run_configure_initial_mandate,
                },
                error::MandateApplicationError,
                ports::DefaultAcademicCenterAuthorization,
                queries::{
                    get_current_mandate as run_get_current_mandate,
                    list_mandates as run_list_mandates,
                },
            },
        },
        identity::{
            api::extractors::firebase_bearer_token,
            application::ports::{IdentityTokenVerifier, UserRepository},
        },
    },
};

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
