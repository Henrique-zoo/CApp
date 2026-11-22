//! Handlers HTTP para o módulo de perguntas frequentes (FAQ).

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
        faq::{
            api::dto::{CreateFaqEntryRequest, FaqEntryResponse},
            application::{
                commands::create_faq_entry::CreateFaqEntryCommand,
                services::create_faq_entry::CreateFaqEntryUseCase,
            },
        },
        identity::{
            api::extractors,
            application::{ports::UserRepository, services::authentication},
        },
    },
};

/// Cadastra uma nova pergunta e resposta de FAQ no Centro Acadêmico especificado.
///
/// # Rotas
///
/// - `POST /api/v1/academic-centers/{academic_center_id}/faq`
/// - `POST /academic-centers/{academic_center_id}/faq`
///
/// # Respostas
///
/// - `201 Created`: Pergunta e resposta cadastradas com sucesso.
/// - `400 Bad Request`: Payload JSON malformado ou inválido.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem cargo de gestão ativo no Centro Acadêmico.
/// - `404 Not Found`: Centro Acadêmico não encontrado.
/// - `422 Unprocessable Entity`: Violação de regras de validação (pergunta/resposta vazia ou excedendo limites, CA inativo).
pub async fn create_faq_entry(
    State(state): State<AppState>,
    Path(academic_center_id): Path<Uuid>,
    headers: HeaderMap,
    body: Result<Json<CreateFaqEntryRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(payload) = body.map_err(|_| ApiError::bad_request())?;

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

    let command = CreateFaqEntryCommand::new(
        academic_center_id,
        author_id,
        payload.question,
        payload.answer,
        payload.category,
    );

    let use_case = CreateFaqEntryUseCase::new(&state.pg_database);
    let entry = use_case.execute(command).await?;

    Ok((StatusCode::CREATED, Json(FaqEntryResponse::from(entry))))
}
