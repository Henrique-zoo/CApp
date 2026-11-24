//! Handlers HTTP para o módulo de demandas.
//!
//! Adapta as requisições HTTP aos casos de uso de aplicação de demandas,
//! extraindo identificadores, validando autenticação e mapeando respostas.

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
        demands::{
            api::dto::{CreateDemandRequest, DemandResponse},
            application::{
                commands::CreateDemandCommand, error::DemandApplicationError,
                services::create_demand,
            },
            domain::entities::DemandForum,
        },
        identity::{
            api::extractors,
            application::{ports::UserRepository, services::authentication},
        },
    },
};

/// Registra uma nova demanda em um Centro Acadêmico nos fóruns institucional ou de espaço físico.
///
/// # Rota
///
/// `POST /api/v1/academic-centers/{academic_center_id}/demands`
///
/// # Respostas
///
/// - `201 Created`: Demanda criada com sucesso.
/// - `400 Bad Request`: Payload JSON malformado ou fórum desconhecido.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem permissão no CA ou sem vínculo estudantil ativo.
/// - `404 Not Found`: Centro Acadêmico inexistente.
/// - `422 Unprocessable Entity`: Violação de regras de validação (ex: descrição vazia, curso não representado, curso ausente no institucional ou informado no espaço físico).
pub async fn handle_create_demand(
    State(state): State<AppState>,
    Path(academic_center_id): Path<Uuid>,
    headers: HeaderMap,
    body: Result<Json<CreateDemandRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(payload) = body.map_err(|_| ApiError::bad_request())?;

    // Autenticação do autor
    let author_id = if let Ok(token) = extractors::firebase_bearer_token(&headers) {
        let identity = timeout(
            Duration::from_secs(10),
            authentication::authenticate_institutional_request(
                &state.clients.firebase,
                &state.pg_database,
                token,
            ),
        )
        .await
        .map_err(|_| {
            tracing::warn!("Tempo limite de autenticação excedido");
            ApiError::service_unavailable()
        })??;

        let user = state
            .pg_database
            .get_by_firebase_uid(&identity.firebase_uid)
            .await
            .map_err(|_| ApiError::internal_server_error())?
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    "account_not_provisioned",
                    "Conta de usuário não encontrada para a identidade autenticada",
                    None,
                )
            })?;

        let authenticated_user_id = user.id();

        // Se o corpo informar author_id divergente do autenticado, recusa por segurança
        if payload
            .author_id
            .is_some_and(|body_author| body_author != authenticated_user_id)
        {
            return Err(ApiError::forbidden());
        }

        authenticated_user_id
    } else {
        #[cfg(feature = "test-support")]
        {
            if let Some(test_author_header) = headers.get("x-test-author-id") {
                test_author_header
                    .to_str()
                    .ok()
                    .and_then(|s| s.parse::<Uuid>().ok())
                    .ok_or_else(ApiError::bad_request)?
            } else if let Some(body_author) = payload.author_id {
                body_author
            } else {
                return Err(ApiError::unauthorized());
            }
        }

        #[cfg(not(feature = "test-support"))]
        {
            return Err(ApiError::unauthorized());
        }
    };

    let forum: DemandForum = payload
        .forum
        .parse()
        .map_err(DemandApplicationError::from)?;

    let command = CreateDemandCommand::new(
        academic_center_id,
        author_id,
        forum,
        payload.title,
        payload.description,
        payload.category,
        payload.course_name,
    );

    let demand = create_demand(&state.pg_database, &state.pg_database, command).await?;

    Ok((StatusCode::CREATED, Json(DemandResponse::from(demand))))
}
