//! Handlers HTTP para o módulo de perguntas frequentes (FAQ).

use std::time::Duration;

use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
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
            api::dto::{
                CreateFaqEntryRequest, FaqEntryResponse, FaqQueryParams, UpdateFaqEntryRequest,
            },
            application::{
                commands::{
                    create_faq_entry::CreateFaqEntryCommand,
                    delete_faq_entry::DeleteFaqEntryCommand,
                    update_faq_entry::UpdateFaqEntryCommand,
                },
                queries::{
                    get_faq_entry::GetFaqEntryQuery, list_faq_categories::ListFaqCategoriesQuery,
                    list_faq_entries::ListFaqEntriesQuery,
                },
                services::{
                    create_faq_entry::CreateFaqEntryUseCase,
                    delete_faq_entry::DeleteFaqEntryUseCase, get_faq_entry::GetFaqEntryUseCase,
                    list_faq_categories::ListFaqCategoriesUseCase,
                    list_faq_entries::ListFaqEntriesUseCase,
                    update_faq_entry::UpdateFaqEntryUseCase,
                },
            },
        },
        identity::{
            api::extractors,
            application::{ports::UserRepository, services::authentication},
        },
    },
};

/// Extrai e valida a identidade autenticada do usuário a partir dos headers da requisição.
async fn resolve_authenticated_user_id(
    state: &AppState,
    headers: &HeaderMap,
    fallback_author_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    if let Ok(token) = extractors::firebase_bearer_token(headers) {
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

        if fallback_author_id.is_some_and(|body_author| body_author != authenticated_user_id) {
            return Err(ApiError::forbidden());
        }

        Ok(authenticated_user_id)
    } else {
        #[cfg(feature = "test-support")]
        {
            if let Some(test_author_header) = headers.get("x-test-author-id") {
                test_author_header
                    .to_str()
                    .ok()
                    .and_then(|s| s.parse::<Uuid>().ok())
                    .ok_or_else(ApiError::bad_request)
            } else if let Some(body_author) = fallback_author_id {
                Ok(body_author)
            } else {
                Err(ApiError::unauthorized())
            }
        }

        #[cfg(not(feature = "test-support"))]
        {
            let _ = fallback_author_id;
            Err(ApiError::unauthorized())
        }
    }
}

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

    let author_id = resolve_authenticated_user_id(&state, &headers, payload.author_id).await?;

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

/// Atualiza uma pergunta e resposta de FAQ existente no Centro Acadêmico especificado.
///
/// # Rotas
///
/// - `PATCH /api/v1/academic-centers/{academic_center_id}/faq/{faq_entry_id}`
/// - `PATCH /academic-centers/{academic_center_id}/faq/{faq_entry_id}`
///
/// # Respostas
///
/// - `200 OK`: Pergunta e resposta atualizadas com sucesso.
/// - `400 Bad Request`: Payload JSON malformado, vazio ou sem campos a atualizar.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem cargo de gestão ativo no Centro Acadêmico.
/// - `404 Not Found`: Centro Acadêmico ou pergunta frequente não encontrados.
/// - `422 Unprocessable Entity`: Violação de regras de validação (pergunta/resposta vazia ou excedendo limites, CA inativo).
pub async fn update_faq_entry(
    State(state): State<AppState>,
    Path((academic_center_id, faq_entry_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: Result<Json<UpdateFaqEntryRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(payload) = body.map_err(|_| ApiError::bad_request())?;

    if payload.question.is_none() && payload.answer.is_none() && payload.category.is_none() {
        return Err(ApiError::bad_request());
    }

    let author_id = resolve_authenticated_user_id(&state, &headers, None).await?;

    let command = UpdateFaqEntryCommand::new(
        academic_center_id,
        faq_entry_id,
        author_id,
        payload.question,
        payload.answer,
        payload.category,
    );

    let use_case = UpdateFaqEntryUseCase::new(&state.pg_database);
    let entry = use_case.execute(command).await?;

    Ok((StatusCode::OK, Json(FaqEntryResponse::from(entry))))
}

/// Remove uma pergunta e resposta de FAQ existente no Centro Acadêmico especificado.
///
/// # Rotas
///
/// - `DELETE /api/v1/academic-centers/{academic_center_id}/faq/{faq_entry_id}`
/// - `DELETE /academic-centers/{academic_center_id}/faq/{faq_entry_id}`
///
/// # Respostas
///
/// - `204 No Content`: Pergunta e resposta removidas com sucesso.
/// - `400 Bad Request`: Identificadores inválidos na rota.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem cargo de gestão ativo no Centro Acadêmico.
/// - `404 Not Found`: Centro Acadêmico ou pergunta frequente não encontrados.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn delete_faq_entry(
    State(state): State<AppState>,
    Path((academic_center_id, faq_entry_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let author_id = resolve_authenticated_user_id(&state, &headers, None).await?;

    let command = DeleteFaqEntryCommand::new(academic_center_id, faq_entry_id, author_id);

    let use_case = DeleteFaqEntryUseCase::new(&state.pg_database);
    use_case.execute(command).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Consulta e busca perguntas e respostas de FAQ do Centro Acadêmico especificado.
///
/// Suporta filtros por categoria temática e busca textual case-insensitive por palavra-chave.
///
/// # Rotas
///
/// - `GET /api/v1/academic-centers/{academic_center_id}/faq`
/// - `GET /academic-centers/{academic_center_id}/faq`
///
/// # Query Params
///
/// - `category`: Filtro opcional por categoria temática.
/// - `q` / `search`: Termo de busca textual opcional (procura em pergunta e resposta).
///
/// # Respostas
///
/// - `200 OK`: Lista de entradas de FAQ encontradas (ordenadas por `created_at DESC`).
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `404 Not Found`: Centro Acadêmico não encontrado.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn list_faq_entries(
    State(state): State<AppState>,
    Path(academic_center_id): Path<Uuid>,
    Query(params): Query<FaqQueryParams>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _user_id = resolve_authenticated_user_id(&state, &headers, None).await?;

    let query = ListFaqEntriesQuery::new(academic_center_id, params.category, params.q);
    let use_case = ListFaqEntriesUseCase::new(&state.pg_database);
    let entries = use_case.execute(query).await?;

    let response: Vec<FaqEntryResponse> = entries.into_iter().map(FaqEntryResponse::from).collect();
    Ok((StatusCode::OK, Json(response)))
}

/// Lista as categorias temáticas distintas cadastradas no FAQ do Centro Acadêmico especificado.
///
/// # Rotas
///
/// - `GET /api/v1/academic-centers/{academic_center_id}/faq/categories`
/// - `GET /academic-centers/{academic_center_id}/faq/categories`
///
/// # Respostas
///
/// - `200 OK`: Lista de nomes de categorias temáticas ordenadas alfabeticamente.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `404 Not Found`: Centro Acadêmico não encontrado.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn list_faq_categories(
    State(state): State<AppState>,
    Path(academic_center_id): Path<Uuid>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _user_id = resolve_authenticated_user_id(&state, &headers, None).await?;

    let query = ListFaqCategoriesQuery::new(academic_center_id);
    let use_case = ListFaqCategoriesUseCase::new(&state.pg_database);
    let categories = use_case.execute(query).await?;

    Ok((StatusCode::OK, Json(categories)))
}

/// Consulta os detalhes de uma pergunta e resposta de FAQ individual pelo seu identificador.
///
/// # Rotas
///
/// - `GET /api/v1/academic-centers/{academic_center_id}/faq/{faq_entry_id}`
/// - `GET /academic-centers/{academic_center_id}/faq/{faq_entry_id}`
///
/// # Respostas
///
/// - `200 OK`: Dados da pergunta frequente.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `404 Not Found`: Centro Acadêmico ou pergunta frequente não encontrados.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn get_faq_entry(
    State(state): State<AppState>,
    Path((academic_center_id, faq_entry_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _user_id = resolve_authenticated_user_id(&state, &headers, None).await?;

    let query = GetFaqEntryQuery::new(academic_center_id, faq_entry_id);
    let use_case = GetFaqEntryUseCase::new(&state.pg_database);
    let entry = use_case.execute(query).await?;

    Ok((StatusCode::OK, Json(FaqEntryResponse::from(entry))))
}
