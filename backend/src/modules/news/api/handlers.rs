//! Handlers HTTP para operações de notícias.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::modules::identity::api::AuthenticatedUser;
use crate::modules::news::api::dto::{
    CreateNewsArticleRequest, ListNewsArticlesQuery, NewsArticleResponse, NewsErrorResponse,
    UpdateNewsArticleRequest,
};
use crate::modules::news::application::commands::{
    CreateNewsArticleCommand, UpdateNewsArticleCommand,
};
use crate::modules::news::application::ports::{NewsAuthorizationPort, NewsRepository};
use crate::modules::news::application::services::NewsService;
use crate::modules::news::domain::NewsError;

impl IntoResponse for NewsError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            NewsError::EmptyTitle
            | NewsError::TitleTooShort { .. }
            | NewsError::TitleTooLong { .. }
            | NewsError::EmptyBody
            | NewsError::BodyTooLong { .. }
            | NewsError::InvalidStatusTransition { .. }
            | NewsError::InvalidStatusString(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            NewsError::CannotEditArchived => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            NewsError::ArticleNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            NewsError::UnauthorizedAuthor { .. } | NewsError::UnauthorizedPublish { .. } => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
            NewsError::RepositoryError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro interno de processamento na persistência de notícias.".to_string(),
            ),
        };

        let body = Json(NewsErrorResponse { error: message });
        (status, body).into_response()
    }
}

/// Cria um novo artigo de notícia para o Centro Acadêmico informado.
pub async fn create_news_article<R, A>(
    State(service): State<Arc<NewsService<R, A>>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CreateNewsArticleRequest>,
) -> Result<(StatusCode, Json<NewsArticleResponse>), NewsError>
where
    R: NewsRepository + Clone + 'static,
    A: NewsAuthorizationPort + Clone + 'static,
{
    let cmd = CreateNewsArticleCommand {
        ca_id: payload.ca_id,
        author_id: auth_user.user_id,
        title: payload.title,
        body: payload.body,
        publish_directly: payload.publish_directly,
    };

    let article = service.create_article(cmd).await?;
    Ok((
        StatusCode::CREATED,
        Json(NewsArticleResponse::from(article)),
    ))
}

/// Atualiza um artigo de notícia existente.
pub async fn update_news_article<R, A>(
    State(service): State<Arc<NewsService<R, A>>>,
    auth_user: AuthenticatedUser,
    Path(article_id): Path<Uuid>,
    Json(payload): Json<UpdateNewsArticleRequest>,
) -> Result<Json<NewsArticleResponse>, NewsError>
where
    R: NewsRepository + Clone + 'static,
    A: NewsAuthorizationPort + Clone + 'static,
{
    let cmd = UpdateNewsArticleCommand {
        article_id,
        author_id: auth_user.user_id,
        title: payload.title,
        body: payload.body,
        publish_directly: payload.publish_directly,
    };

    let article = service.update_article(cmd).await?;
    Ok(Json(NewsArticleResponse::from(article)))
}

/// Consulta os detalhes de uma notícia pelo seu identificador.
pub async fn get_news_article<R, A>(
    State(service): State<Arc<NewsService<R, A>>>,
    Path(article_id): Path<Uuid>,
) -> Result<Json<NewsArticleResponse>, NewsError>
where
    R: NewsRepository + Clone + 'static,
    A: NewsAuthorizationPort + Clone + 'static,
{
    let article = service.get_article(article_id).await?;
    Ok(Json(NewsArticleResponse::from(article)))
}

/// Lista todas as notícias vinculadas a um Centro Acadêmico.
pub async fn list_news_articles<R, A>(
    State(service): State<Arc<NewsService<R, A>>>,
    Query(query): Query<ListNewsArticlesQuery>,
) -> Result<Json<Vec<NewsArticleResponse>>, NewsError>
where
    R: NewsRepository + Clone + 'static,
    A: NewsAuthorizationPort + Clone + 'static,
{
    let articles = service.list_by_ca(query.ca_id).await?;
    let response: Vec<NewsArticleResponse> = articles
        .into_iter()
        .map(NewsArticleResponse::from)
        .collect();
    Ok(Json(response))
}
