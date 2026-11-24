//! Entrada HTTP para criação e edição no ciclo editorial de notícias.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{api::state::AppState, modules::identity::api::extractors::AuthenticatedAccount};

use super::{dto::NewsArticleResponseDto, handlers::map_app_error};

#[derive(Deserialize)]
pub(super) struct CreateNews {
    ca_id: Uuid,
    title: String,
    body: String,
    #[serde(default)]
    publish_directly: bool,
}

#[derive(Deserialize)]
pub(super) struct EditNews {
    title: String,
    body: String,
}

pub(super) async fn create(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Json(body): Json<CreateNews>,
) -> Response {
    match state
        .news_service()
        .create(
            account.account_id,
            body.ca_id,
            body.title,
            body.body,
            body.publish_directly,
        )
        .await
    {
        Ok(article) => (
            StatusCode::CREATED,
            Json(NewsArticleResponseDto::from(article)),
        )
            .into_response(),
        Err(error) => map_app_error(error).into_response(),
    }
}

pub(super) async fn edit(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(news_id): Path<Uuid>,
    Json(body): Json<EditNews>,
) -> Response {
    match state
        .news_service()
        .edit(news_id, account.account_id, body.title, body.body)
        .await
    {
        Ok(article) => Json(NewsArticleResponseDto::from(article)).into_response(),
        Err(error) => map_app_error(error).into_response(),
    }
}
