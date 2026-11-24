//! Handlers HTTP para o módulo de demandas.
//!
//! Adapta as requisições HTTP aos casos de uso de aplicação de demandas,
//! extraindo identificadores, validando autenticação e mapeando respostas.

use std::time::Duration;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tokio::time::timeout;
use uuid::Uuid;
use crate::api::error::ApiError;
use crate::api::error::ApiResult;
use crate::api::state::AppState;
use crate::modules::demands::api::dto::CreateDemandRequest;
use crate::modules::demands::api::dto::DemandResponse;
use crate::modules::demands::api::dto::ListDemandsQueryParams;
use crate::modules::demands::api::dto::UpdateDemandStatusRequest;
use crate::modules::demands::application::commands::CreateDemandCommand;
use crate::modules::demands::application::commands::UpdateDemandStatusCommand;
use crate::modules::demands::application::error::DemandApplicationError;
use crate::modules::demands::application::queries::GetDemandQuery;
use crate::modules::demands::application::queries::ListDemandsQuery;
use crate::modules::demands::application::services::create_demand;
use crate::modules::demands::application::services::get_demand;
use crate::modules::demands::application::services::list_demands;
use crate::modules::demands::application::services::update_demand_status;
use crate::modules::demands::domain::entities::DemandForum;
use crate::modules::demands::domain::entities::DemandStatus;
use crate::modules::identity::api::extractors;
use crate::modules::identity::application::ports::UserRepository;
use crate::modules::identity::application::services::authentication;
use crate::modules::demands::api::dto::CreateDemandCommentRequest;
use crate::modules::demands::api::dto::DemandCommentResponse;
use crate::modules::demands::application::commands::PostDemandCommentCommand;
use crate::modules::demands::application::queries::ListDemandCommentsQuery;
use crate::modules::demands::application::services::list_demand_comments;
use crate::modules::demands::application::services::post_demand_comment;
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
/// Consulta os detalhes e o histórico completo de uma demanda específica.
///
/// # Rota
///
/// `GET /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}`
///
/// # Respostas
///
/// - `200 OK`: Detalhes e histórico da demanda solicitada.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem permissão no CA/curso ou sem vínculo estudantil ativo.
/// - `404 Not Found`: Centro Acadêmico ou demanda inexistente.
/// - `422 Unprocessable Entity`: Centro Acadêmico inativo.
pub async fn handle_get_demand(
    State(state): State<AppState>,
    Path((academic_center_id, demand_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let user_id = authenticate_caller(&state, &headers).await?;

    let query = GetDemandQuery::new(academic_center_id, demand_id, user_id);

    let demand = get_demand(&state.pg_database, &state.pg_database, query).await?;

    Ok((StatusCode::OK, Json(DemandResponse::from(demand))))
}
/// Atualiza o status de uma demanda pela gestão do Centro Acadêmico com parecer obrigatório.
///
/// # Rota
///
/// `PATCH /api/v1/academic-centers/{academic_center_id}/demands/{demand_id}/status`
///
/// # Respostas
///
/// - `200 OK`: Demanda com status atualizado e novo registro no histórico.
/// - `400 Bad Request`: Payload JSON malformado ou status inválido/desconhecido.
/// - `401 Unauthorized`: Credenciais ausentes ou inválidas.
/// - `403 Forbidden`: Usuário sem cargo de gestão ativo no Centro Acadêmico.
/// - `404 Not Found`: Centro Acadêmico ou demanda inexistente.
/// - `422 Unprocessable Entity`: Violação de regras (parecer vazio, encaminhamento proibido para espaço físico, transição inválida ou CA inativo).
pub async fn handle_update_demand_status(
    State(state): State<AppState>,
    Path((academic_center_id, demand_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: Result<Json<UpdateDemandStatusRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(payload) = body.map_err(|_| ApiError::bad_request())?;

    let manager_id = authenticate_caller(&state, &headers).await?;

    let new_status: DemandStatus = payload
        .status
        .parse()
        .map_err(DemandApplicationError::from)?;

    let opinion = payload.opinion.unwrap_or_default();

    let command = UpdateDemandStatusCommand::new(
        academic_center_id,
        demand_id,
        manager_id,
        new_status,
        opinion,
    );

    let demand = update_demand_status(&state.pg_database, &state.pg_database, command).await?;

    Ok((StatusCode::OK, Json(DemandResponse::from(demand))))
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
