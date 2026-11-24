//! Handlers HTTP para as operações e transições de notícias.
//!
//! Converte requisições Axum em comandos da camada de aplicação e serializa
//! os resultados ou erros padronizados em JSON.

use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    api::state::AppState,
    modules::news::{
        api::dto::{
            ArchiveNewsRequestDto, ErrorResponseDto, ListNewsQueryParams, NewsArticleResponseDto,
            PublishNewsRequestDto, SubmitForReviewRequestDto,
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

/// Handler HTTP para a listagem pública de notícias do Centro Acadêmico ativo: `GET /news?ca_id={ca_id}`.
///
/// Retorna apenas notícias publicadas (`published`) do Centro Acadêmico indicado,
/// ordenadas de forma decrescente pela data de publicação (`published_at DESC`).
/// Se não houver notícias publicadas, responde HTTP 200 com array vazio `[]`.
/// Se o parâmetro `ca_id` estiver ausente, responde HTTP 400 Bad Request.
pub async fn list_published_news_handler(
    State(state): State<AppState>,
    Query(params): Query<ListNewsQueryParams>,
) -> Response {
    let ca_id = match params.ca_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponseDto::new(
                    "MISSING_CA_ID",
                    "O parâmetro 'ca_id' é obrigatório para listar as notícias publicadas do Centro Acadêmico.",
                )),
            )
                .into_response();
        }
    };

    let service = state.news_service();
    match service.list_published_news_by_ca(ca_id).await {
        Ok(articles) => {
            let dtos: Vec<NewsArticleResponseDto> = articles
                .into_iter()
                .map(NewsArticleResponseDto::from)
                .collect();
            (StatusCode::OK, Json(dtos)).into_response()
        }
        Err(err) => map_app_error(err).into_response(),
    }
}

/// Handler HTTP para consulta de detalhes de uma notícia publicada: `GET /news/{id}`.
///
/// Retorna HTTP 200 com os detalhes completos do artigo se a notícia existir e estiver publicada.
/// Se a notícia não existir ou seu status não for `published`, responde HTTP 404 Not Found.
pub async fn get_news_by_id_handler(
    State(state): State<AppState>,
    Path(news_id): Path<Uuid>,
) -> Response {
    let service = state.news_service();

    match service.get_published_news_detail(news_id).await {
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
            domain::{NewsArticle, NewsStatus},
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

    fn preparar_app_teste_com_artigos(
        articles: impl IntoIterator<Item = NewsArticle>,
    ) -> axum::Router {
        let repo = Arc::new(InMemoryNewsRepository::with_articles(articles));
        let auth = Arc::new(StaticRoleNewsAuthorization::new());

        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .unwrap();

        let state = AppState::new(pool)
            .with_news_repository(repo)
            .with_news_authorization(auth);

        create_router().with_state(state)
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
        assert_eq!(dto.body, "Programação confirmada pelo CA.");
        assert_eq!(dto.status, "published");
    }

    #[tokio::test]
    async fn endpoint_get_news_by_id_deve_retornar_404_para_noticia_nao_publicada() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let status_nao_publicados = [
            (NewsStatus::Draft, "Rascunho"),
            (NewsStatus::PendingReview, "Pendente"),
            (NewsStatus::Rejected, "Rejeitada"),
            (NewsStatus::Archived, "Arquivada"),
        ];

        for (status, title) in status_nao_publicados {
            let article = NewsArticle::reconstruct(
                Uuid::new_v4(),
                ca_id,
                author_id,
                title.to_string(),
                "Conteúdo interno".to_string(),
                status,
                1,
                now,
                None,
                None,
            );

            let app = preparar_app_teste_com_artigos([article.clone()]);

            let req = Request::builder()
                .method("GET")
                .uri(format!("/news/{}", article.id))
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(req).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Notícia com status '{status:?}' deve responder 404"
            );
        }
    }

    #[tokio::test]
    async fn endpoint_get_news_by_id_deve_retornar_404_para_noticia_inexistente() {
        let app = preparar_app_teste_com_artigos([]);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news/{}", Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn endpoint_list_published_news_deve_exibir_somente_publicadas_do_ca_ativo() {
        let ca_computacao = Uuid::new_v4();
        let ca_matematica = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let semana_academica = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Semana Acadêmica".to_string(),
            "Programação da semana acadêmica.".to_string(),
            NewsStatus::Published,
            1,
            now - chrono::Duration::days(3),
            Some(now - chrono::Duration::days(2)),
            None,
        );

        let edital_bolsas = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Edital de bolsas".to_string(),
            "Informações sobre bolsas de pesquisa.".to_string(),
            NewsStatus::Published,
            1,
            now - chrono::Duration::days(2),
            Some(now - chrono::Duration::days(1)),
            None,
        );

        let rascunho = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Rascunho de evento".to_string(),
            "Rascunho de evento que não deve aparecer.".to_string(),
            NewsStatus::Draft,
            1,
            now,
            None,
            None,
        );

        let pendente = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Proposta de palestra".to_string(),
            "Proposta aguardando homologação.".to_string(),
            NewsStatus::PendingReview,
            1,
            now,
            None,
            None,
        );

        let rejeitado = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Comunicado rejeitado".to_string(),
            "Comunicado não aprovado na revisão.".to_string(),
            NewsStatus::Rejected,
            1,
            now,
            None,
            None,
        );

        let arquivado = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Aviso arquivado".to_string(),
            "Aviso antigo que foi arquivado.".to_string(),
            NewsStatus::Archived,
            1,
            now - chrono::Duration::days(10),
            Some(now - chrono::Duration::days(8)),
            Some(now - chrono::Duration::days(1)),
        );

        let outro_ca = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_matematica,
            author_id,
            "Semana de Matemática".to_string(),
            "Atividade de outro Centro Acadêmico.".to_string(),
            NewsStatus::Published,
            1,
            now,
            Some(now),
            None,
        );

        let app = preparar_app_teste_com_artigos([
            semana_academica,
            edital_bolsas,
            rascunho,
            pendente,
            rejeitado,
            arquivado,
            outro_ca,
        ]);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news?ca_id={ca_computacao}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dtos: Vec<NewsArticleResponseDto> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].title, "Edital de bolsas");
        assert_eq!(dtos[1].title, "Semana Acadêmica");
    }

    #[tokio::test]
    async fn endpoint_list_published_news_deve_retornar_array_vazio_quando_nao_houver_publicadas() {
        let ca_id = Uuid::new_v4();
        let app = preparar_app_teste_com_artigos([]);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news?ca_id={ca_id}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dtos: Vec<NewsArticleResponseDto> = serde_json::from_slice(&body).unwrap();
        assert!(dtos.is_empty());
    }

    #[tokio::test]
    async fn endpoint_list_published_news_deve_ordenar_da_mais_recente_para_mais_antiga() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let dt_antigo = chrono::DateTime::parse_from_rfc3339("2026-10-10T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let dt_recente = chrono::DateTime::parse_from_rfc3339("2026-10-15T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let dt_meio = chrono::DateTime::parse_from_rfc3339("2026-10-12T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let comunicado_antigo = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Comunicado antigo".to_string(),
            "Corpo".to_string(),
            NewsStatus::Published,
            1,
            dt_antigo,
            Some(dt_antigo),
            None,
        );

        let comunicado_recente = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Comunicado recente".to_string(),
            "Corpo".to_string(),
            NewsStatus::Published,
            1,
            dt_recente,
            Some(dt_recente),
            None,
        );

        let comunicado_meio = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Comunicado do meio".to_string(),
            "Corpo".to_string(),
            NewsStatus::Published,
            1,
            dt_meio,
            Some(dt_meio),
            None,
        );

        let app = preparar_app_teste_com_artigos([
            comunicado_antigo,
            comunicado_recente,
            comunicado_meio,
        ]);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news?ca_id={ca_id}"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let dtos: Vec<NewsArticleResponseDto> = serde_json::from_slice(&body).unwrap();

        assert_eq!(dtos.len(), 3);
        assert_eq!(dtos[0].title, "Comunicado recente");
        assert_eq!(dtos[1].title, "Comunicado do meio");
        assert_eq!(dtos[2].title, "Comunicado antigo");
    }

    #[tokio::test]
    async fn endpoint_list_published_news_deve_isolar_noticias_por_ca() {
        let ca_computacao = Uuid::new_v4();
        let ca_matematica = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let noticia_comp = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Hackathon de Computação".to_string(),
            "Evento do CA de Computação.".to_string(),
            NewsStatus::Published,
            1,
            now,
            Some(now),
            None,
        );

        let noticia_mat = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_matematica,
            author_id,
            "Semana de Matemática".to_string(),
            "Evento do CA de Matemática.".to_string(),
            NewsStatus::Published,
            1,
            now,
            Some(now),
            None,
        );

        let app = preparar_app_teste_com_artigos([noticia_comp, noticia_mat]);

        // Consulta notícias do CA de Computação
        let req_comp = Request::builder()
            .method("GET")
            .uri(format!("/news?ca_id={ca_computacao}"))
            .body(Body::empty())
            .unwrap();

        let res_comp = app.clone().oneshot(req_comp).await.unwrap();
        assert_eq!(res_comp.status(), StatusCode::OK);
        let body_comp = to_bytes(res_comp.into_body(), usize::MAX).await.unwrap();
        let dtos_comp: Vec<NewsArticleResponseDto> = serde_json::from_slice(&body_comp).unwrap();
        assert_eq!(dtos_comp.len(), 1);
        assert_eq!(dtos_comp[0].title, "Hackathon de Computação");

        // Consulta notícias do CA de Matemática
        let req_mat = Request::builder()
            .method("GET")
            .uri(format!("/news?ca_id={ca_matematica}"))
            .body(Body::empty())
            .unwrap();

        let res_mat = app.oneshot(req_mat).await.unwrap();
        assert_eq!(res_mat.status(), StatusCode::OK);
        let body_mat = to_bytes(res_mat.into_body(), usize::MAX).await.unwrap();
        let dtos_mat: Vec<NewsArticleResponseDto> = serde_json::from_slice(&body_mat).unwrap();
        assert_eq!(dtos_mat.len(), 1);
        assert_eq!(dtos_mat[0].title, "Semana de Matemática");
    }

    #[tokio::test]
    async fn endpoint_list_published_news_deve_retornar_400_quando_ca_id_nao_fornecido() {
        let app = preparar_app_teste_com_artigos([]);

        let req = Request::builder()
            .method("GET")
            .uri("/news")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let err_dto: ErrorResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(err_dto.code, "MISSING_CA_ID");
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
