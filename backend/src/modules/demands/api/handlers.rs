//! Handlers HTTP para o módulo de demandas.
//!
//! Adapta as requisições HTTP aos casos de uso de aplicação de demandas,
//! extraindo identificadores, validando autenticação e mapeando respostas.

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
        demands::{
            api::dto::{
                CreateDemandCommentRequest, CreateDemandRequest, DemandCommentResponse,
                DemandResponse, ListDemandsQueryParams,
            },
            application::{
                commands::{CreateDemandCommand, PostDemandCommentCommand},
                error::DemandApplicationError,
                queries::{ListDemandCommentsQuery, ListDemandsQuery},
                services::{
                    create_demand, list_demand_comments, list_demands, post_demand_comment,
                },
            },
            domain::entities::{DemandForum, DemandStatus},
        },
        identity::{
            api::extractors,
            application::{ports::UserRepository, services::authentication},
        },
    },
};

/// Extrai e valida a identidade do chamador autenticado a partir dos cabeçalhos.
async fn authenticate_caller(state: &AppState, headers: &HeaderMap) -> ApiResult<Uuid> {
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

        Ok(user.id())
    } else {
        #[cfg(feature = "test-support")]
        {
            if let Some(test_author_header) = headers.get("x-test-author-id") {
                test_author_header
                    .to_str()
                    .ok()
                    .and_then(|s| s.parse::<Uuid>().ok())
                    .ok_or_else(ApiError::bad_request)
            } else {
                Err(ApiError::unauthorized())
            }
        }

        #[cfg(not(feature = "test-support"))]
        {
            Err(ApiError::unauthorized())
        }
    }
}

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
    let author_id = match authenticate_caller(&state, &headers).await {
        Ok(authenticated_user_id) => {
            // Se o corpo informar author_id divergente do autenticado, recusa por segurança
            if payload
                .author_id
                .is_some_and(|body_author| body_author != authenticated_user_id)
            {
                return Err(ApiError::forbidden());
            }
            authenticated_user_id
        }
        Err(err) => {
            #[cfg(feature = "test-support")]
            {
                if let Some(body_author) = payload.author_id {
                    body_author
                } else {
                    return Err(err);
                }
            }

            #[cfg(not(feature = "test-support"))]
            {
                return Err(err);
            }
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

/// Lista e filtra as demandas de um Centro Acadêmico nos fóruns institucional e de espaço físico.
///
/// # Rota
///
/// `GET /api/v1/academic-centers/{academic_center_id}/demands`
///
/// # Query Params
///
/// - `forum`: Fórum opcional (`institutional` ou `physical_space`).
/// - `status`: Status opcional no ciclo de vida (`open`, `in_analysis`, etc.).
/// - `mine`: Quando `true`, restringe às demandas registradas pelo usuário autenticado.
/// - `limit`: Quantidade máxima de registros retornados.
/// - `offset`: Quantidade de registros a pular.
///
/// # Respostas
///
/// - `200 OK`: Lista de demandas correspondentes aos filtros aplicados.
/// - `400 Bad Request`: Parâmetro de query inválido (ex: `invalid_forum` ou `invalid_status`).
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem permissão no CA ou sem vínculo estudantil ativo.
/// - `404 Not Found`: Centro Acadêmico inexistente.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn handle_list_demands(
    State(state): State<AppState>,
    Path(academic_center_id): Path<Uuid>,
    headers: HeaderMap,
    Query(params): Query<ListDemandsQueryParams>,
) -> ApiResult<impl IntoResponse> {
    let user_id = authenticate_caller(&state, &headers).await?;

    let forum = match params.forum {
        Some(ref f) => Some(
            f.parse::<DemandForum>()
                .map_err(DemandApplicationError::from)?,
        ),
        None => None,
    };

    let status = match params.status {
        Some(ref s) => Some(
            s.parse::<DemandStatus>()
                .map_err(DemandApplicationError::from)?,
        ),
        None => None,
    };

    let limit = params.limit.or(params.page_size);
    let offset = params.offset.or_else(|| {
        params
            .page
            .map(|p| p.saturating_sub(1) * limit.unwrap_or(50))
    });

    let query = ListDemandsQuery::new(
        academic_center_id,
        user_id,
        forum,
        status,
        params.mine.unwrap_or(false),
        limit,
        offset,
    );

    let demands = list_demands(&state.pg_database, &state.pg_database, query).await?;

    let response: Vec<DemandResponse> = demands.into_iter().map(DemandResponse::from).collect();

    Ok((StatusCode::OK, Json(response)))
}

/// Publica um novo comentário ou interação em uma demanda.
///
/// # Rota
///
/// `POST /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/comments`
///
/// # Respostas
///
/// - `201 Created`: Comentário registrado com sucesso.
/// - `400 Bad Request`: Payload JSON malformado ou demanda pertencente a outro CA.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem permissão para interagir (terceiro não autorizado ou sem vínculo ativo).
/// - `404 Not Found`: Centro Acadêmico ou demanda inexistente.
/// - `422 Unprocessable Entity`: Conteúdo vazio, excedendo limite ou CA inativo.
pub async fn handle_post_comment(
    State(state): State<AppState>,
    Path((academic_center_id, demand_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: Result<Json<CreateDemandCommentRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(payload) = body.map_err(|_| ApiError::bad_request())?;

    let author_id = match authenticate_caller(&state, &headers).await {
        Ok(authenticated_user_id) => {
            if payload
                .author_id
                .is_some_and(|body_author| body_author != authenticated_user_id)
            {
                return Err(ApiError::forbidden());
            }
            authenticated_user_id
        }
        Err(err) => {
            #[cfg(feature = "test-support")]
            {
                if let Some(body_author) = payload.author_id {
                    body_author
                } else {
                    return Err(err);
                }
            }

            #[cfg(not(feature = "test-support"))]
            {
                return Err(err);
            }
        }
    };

    let command =
        PostDemandCommentCommand::new(academic_center_id, demand_id, author_id, payload.content);

    let comment = post_demand_comment(
        &state.pg_database,
        &state.pg_database,
        &state.pg_database,
        command,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(DemandCommentResponse::from(comment)),
    ))
}

/// Lista os comentários e interações de uma demanda em ordem cronológica ascendente.
///
/// # Rota
///
/// `GET /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/comments`
///
/// # Respostas
///
/// - `200 OK`: Lista de comentários em ordem cronológica (`created_at ASC, id ASC`).
/// - `400 Bad Request`: Demanda pertencente a outro Centro Acadêmico.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem permissão para visualizar as interações (terceiro não autorizado ou sem vínculo ativo).
/// - `404 Not Found`: Centro Acadêmico ou demanda inexistente.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn handle_list_comments(
    State(state): State<AppState>,
    Path((academic_center_id, demand_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let user_id = authenticate_caller(&state, &headers).await?;

    let query = ListDemandCommentsQuery::new(academic_center_id, demand_id, user_id);

    let comments = list_demand_comments(
        &state.pg_database,
        &state.pg_database,
        &state.pg_database,
        query,
    )
    .await?;

    let response: Vec<DemandCommentResponse> = comments
        .into_iter()
        .map(DemandCommentResponse::from)
        .collect();

    Ok((StatusCode::OK, Json(response)))
}
