//! Handlers HTTP para as operações e transições de notícias.
//!
//! Converte requisições Axum em comandos da camada de aplicação e serializa
//! os resultados ou erros padronizados em JSON.

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    api::state::AppState,
    modules::news::{
        api::dto::{
            ArchiveNewsRequestDto, ErrorResponseDto, NewsArticleResponseDto, PublishNewsRequestDto,
            SubmitForReviewRequestDto,
        },
        application::{
            ArchiveNewsArticleCommand, NewsApplicationError, PublishNewsArticleCommand,
            SubmitForReviewNewsArticleCommand,
        },
    },
};

/// Extrai o identificador do ator a partir dos cabeçalhos HTTP ou do corpo da requisição.
fn extract_actor_id(
    headers: &HeaderMap,
    body_actor_id: Option<Uuid>,
) -> Result<Uuid, (StatusCode, Json<ErrorResponseDto>)> {
    if let Some(header_val) = headers
        .get("x-user-id")
        .or_else(|| headers.get("x-actor-id"))
    {
        match header_val.to_str() {
            Ok(val_str) => match Uuid::parse_str(val_str) {
                Ok(parsed_uuid) => return Ok(parsed_uuid),
                Err(_) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponseDto::new(
                            "INVALID_USER_ID_HEADER",
                            "O cabeçalho 'x-user-id' deve conter um UUID válido.",
                        )),
                    ));
                }
            },
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponseDto::new(
                        "INVALID_USER_ID_HEADER",
                        "O cabeçalho 'x-user-id' contém caracteres inválidos.",
                    )),
                ));
            }
        }
    }

    if let Some(actor) = body_actor_id {
        return Ok(actor);
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponseDto::new(
            "MISSING_ACTOR_ID",
            "Identificador do usuário ausente. Forneça o cabeçalho 'x-user-id' ou o campo 'actor_id' no corpo da requisição.",
        )),
    ))
}

/// Mapeia erros da camada de aplicação para códigos de status e payloads HTTP semânticos.
fn map_app_error(err: NewsApplicationError) -> (StatusCode, Json<ErrorResponseDto>) {
    match err {
        NewsApplicationError::NotFound(id) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponseDto::new(
                "NEWS_NOT_FOUND",
                format!("A notícia com ID '{id}' não foi encontrada."),
            )),
        ),
        NewsApplicationError::Forbidden {
            user_id,
            ca_id,
            operation,
        } => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponseDto::new(
                "FORBIDDEN",
                format!(
                    "Usuário '{user_id}' não possui permissão para '{operation}' no Centro Acadêmico '{ca_id}'."
                ),
            )),
        ),
        NewsApplicationError::AlreadyArchived { id } => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponseDto::new(
                "ALREADY_ARCHIVED",
                format!("A notícia com ID '{id}' já se encontra arquivada."),
            )),
        ),
        NewsApplicationError::InvalidTransition(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponseDto::new(
                "INVALID_STATUS_TRANSITION",
                e.to_string(),
            )),
        ),
        NewsApplicationError::Repository(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                ErrorResponseDto::new(
                    "REPOSITORY_ERROR",
                    "Ocorreu um erro no armazenamento de notícias.",
                )
                .with_details(e.to_string()),
            ),
        ),
        NewsApplicationError::Authorization(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                ErrorResponseDto::new(
                    "AUTHORIZATION_SERVICE_ERROR",
                    "Falha ao consultar permissões institucionais.",
                )
                .with_details(e.to_string()),
            ),
        ),
    }
}

/// Handler HTTP para a ação de publicação de notícia: `PATCH /news/{id}/publish`.
pub async fn publish_news_handler(
    State(state): State<AppState>,
    Path(news_id): Path<Uuid>,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> Response {
    let body_actor_id = if !body_bytes.is_empty() {
        match serde_json::from_slice::<PublishNewsRequestDto>(&body_bytes) {
            Ok(parsed) => parsed.actor_id,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponseDto::new("INVALID_JSON", e.to_string())),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let actor_id = match extract_actor_id(&headers, body_actor_id) {
        Ok(id) => id,
        Err(err_response) => return err_response.into_response(),
    };

    let service = state.news_service();
    let command = PublishNewsArticleCommand {
        news_id,
        actor_id,
        published_at: None,
    };

    match service.publish_news(command).await {
        Ok(article) => {
            (StatusCode::OK, Json(NewsArticleResponseDto::from(article))).into_response()
        }
        Err(err) => map_app_error(err).into_response(),
    }
}

/// Handler HTTP para a ação de arquivamento de notícia: `PATCH /news/{id}/archive`.
pub async fn archive_news_handler(
    State(state): State<AppState>,
    Path(news_id): Path<Uuid>,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> Response {
    let body_actor_id = if !body_bytes.is_empty() {
        match serde_json::from_slice::<ArchiveNewsRequestDto>(&body_bytes) {
            Ok(parsed) => parsed.actor_id,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponseDto::new("INVALID_JSON", e.to_string())),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let actor_id = match extract_actor_id(&headers, body_actor_id) {
        Ok(id) => id,
        Err(err_response) => return err_response.into_response(),
    };

    let service = state.news_service();
    let command = ArchiveNewsArticleCommand {
        news_id,
        actor_id,
        archived_at: None,
    };

    match service.archive_news(command).await {
        Ok(article) => {
            (StatusCode::OK, Json(NewsArticleResponseDto::from(article))).into_response()
        }
        Err(err) => map_app_error(err).into_response(),
    }
}

/// Handler HTTP para a ação de submissão para revisão: `PATCH /news/{id}/submit-for-review`.
pub async fn submit_for_review_handler(
    State(state): State<AppState>,
    Path(news_id): Path<Uuid>,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> Response {
    let body_actor_id = if !body_bytes.is_empty() {
        match serde_json::from_slice::<SubmitForReviewRequestDto>(&body_bytes) {
            Ok(parsed) => parsed.actor_id,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponseDto::new("INVALID_JSON", e.to_string())),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let actor_id = match extract_actor_id(&headers, body_actor_id) {
        Ok(id) => id,
        Err(err_response) => return err_response.into_response(),
    };

    let service = state.news_service();
    let command = SubmitForReviewNewsArticleCommand { news_id, actor_id };

    match service.submit_for_review(command).await {
        Ok(article) => {
            (StatusCode::OK, Json(NewsArticleResponseDto::from(article))).into_response()
        }
        Err(err) => map_app_error(err).into_response(),
    }
}

/// Handler HTTP para consulta de detalhes da notícia: `GET /news/{id}`.
pub async fn get_news_by_id_handler(
    State(state): State<AppState>,
    Path(news_id): Path<Uuid>,
) -> Response {
    let service = state.news_service();

    match service.get_news_by_id(news_id).await {
        Ok(article) => {
            (StatusCode::OK, Json(NewsArticleResponseDto::from(article))).into_response()
        }
        Err(err) => map_app_error(err).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;
    use crate::{
        api::router::create_router,
        modules::news::{
            domain::NewsArticle,
            infrastructure::{InMemoryNewsRepository, StaticRoleNewsAuthorization},
        },
    };

    fn preparar_app_teste(
        article: NewsArticle,
        manager_id: Uuid,
    ) -> (axum::Router, Arc<InMemoryNewsRepository>) {
        let repo = Arc::new(InMemoryNewsRepository::with_articles([article.clone()]));
        let mut auth = StaticRoleNewsAuthorization::new();
        auth.grant_manage(manager_id, article.ca_id);
        let auth_arc = Arc::new(auth);

        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .unwrap();

        let state = AppState::new(pool)
            .with_news_repository(repo.clone())
            .with_news_authorization(auth_arc);

        let app = create_router().with_state(state);
        (app, repo)
    }

    #[tokio::test]
    async fn endpoint_publish_deve_retornar_200_quando_autorizado() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/publish", article.id))
            .header("x-user-id", manager_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dto: NewsArticleResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(dto.status, "published");
        assert!(dto.published_at.is_some());
    }

    #[tokio::test]
    async fn endpoint_publish_deve_retornar_403_quando_sem_permissao() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);
        let usuario_sem_permissao = Uuid::new_v4();

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/publish", article.id))
            .header("x-user-id", usuario_sem_permissao.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn endpoint_publish_deve_retornar_404_para_noticia_inexistente() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let (app, _repo) = preparar_app_teste(article, manager_id);
        let id_inexistente = Uuid::new_v4();

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{id_inexistente}/publish"))
            .header("x-user-id", manager_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn endpoint_publish_deve_retornar_422_quando_artigo_ja_arquivado() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let mut article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();
        article.archive(Utc::now()).unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/publish", article.id))
            .header("x-user-id", manager_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let err_dto: ErrorResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(err_dto.code, "ALREADY_ARCHIVED");
    }

    #[tokio::test]
    async fn endpoint_archive_deve_retornar_200_quando_autorizado() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let mut article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();
        article.publish(Utc::now()).unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/archive", article.id))
            .header("x-user-id", manager_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dto: NewsArticleResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(dto.status, "archived");
        assert!(dto.archived_at.is_some());
    }

    #[tokio::test]
    async fn endpoint_archive_deve_retornar_422_ao_tentar_arquivar_ja_arquivada() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let mut article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();
        article.archive(Utc::now()).unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/archive", article.id))
            .header("x-user-id", manager_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn endpoint_submit_for_review_deve_retornar_200_para_autor() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/submit-for-review", article.id))
            .header("x-user-id", author_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dto: NewsArticleResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(dto.status, "pending_review");
    }

    #[tokio::test]
    async fn endpoint_get_news_by_id_deve_retornar_detalhes() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news/{}", article.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dto: NewsArticleResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(dto.id, article.id);
        assert_eq!(dto.title, "Semana de Integração");
    }

    #[tokio::test]
    async fn endpoint_deve_retornar_400_quando_nenhum_actor_for_fornecido() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Semana de Integração".to_string(),
            "Programação confirmada pelo CA.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let (app, _repo) = preparar_app_teste(article.clone(), manager_id);

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/news/{}/publish", article.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
