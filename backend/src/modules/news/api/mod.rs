//! Camada HTTP do módulo de notícias.
//!
//! [`dto`] define os contratos de transporte (request/response);
//! [`handlers`] contém os adaptadores de requisições Axum;
//! [`news_router`] monta e expõe as rotas configuradas para o módulo.

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::modules::news::application::ports::{NewsAuthorizationPort, NewsRepository};
use crate::modules::news::application::services::NewsService;

pub mod dto;
pub mod handlers;

pub use dto::{
    CreateNewsArticleRequest, ListNewsArticlesQuery, NewsArticleResponse, NewsErrorResponse,
    UpdateNewsArticleRequest,
};

/// Cria um roteador Axum com todas as rotas do módulo de notícias.
///
/// Rotas registradas:
/// - `POST /news`: Criação de notícia.
/// - `GET /news`: Listagem de notícias por Centro Acadêmico (`?ca_id=...`).
/// - `GET /news/{id}`: Consulta de notícia por identificador.
/// - `PUT /news/{id}`: Atualização/edição de notícia existente.
pub fn news_router<R, A>(service: Arc<NewsService<R, A>>) -> Router
where
    R: NewsRepository + Clone + 'static,
    A: NewsAuthorizationPort + Clone + 'static,
{
    Router::new()
        .route(
            "/news",
            post(handlers::create_news_article::<R, A>).get(handlers::list_news_articles::<R, A>),
        )
        .route(
            "/news/{id}",
            get(handlers::get_news_article::<R, A>).put(handlers::update_news_article::<R, A>),
        )
        .with_state(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::modules::news::domain::NewsArticle;
    use crate::modules::news::infrastructure::{
        InMemoryNewsAuthorizationService, InMemoryNewsRepository,
    };

    const CAER_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000001");
    const VETERANO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000010");
    const CALOURO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000020");

    fn setup_test_router() -> (
        Router,
        InMemoryNewsRepository,
        InMemoryNewsAuthorizationService,
    ) {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        let service = Arc::new(NewsService::new(repo.clone(), auth.clone()));
        let router = news_router(service);
        (router, repo, auth)
    }

    #[tokio::test]
    async fn post_news_com_sucesso() {
        let (app, repo, auth) = setup_test_router();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let payload = serde_json::json!({
            "ca_id": CAER_ID,
            "title": "Aviso de Matrícula",
            "body": "Instruções para matrícula em disciplinas de Engenharia de Redes.",
            "publish_directly": false
        });

        let req = Request::builder()
            .method("POST")
            .uri("/news")
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-User-Id", VETERANO_ID.to_string())
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let resp: NewsArticleResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(resp.title, "Aviso de Matrícula");
        assert_eq!(resp.status, "draft");
        assert_eq!(resp.revision, 1);
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn post_news_sem_autenticacao_retorna_401() {
        let (app, _repo, _auth) = setup_test_router();

        let payload = serde_json::json!({
            "ca_id": CAER_ID,
            "title": "Aviso Anônimo",
            "body": "Tentativa sem credenciais de autenticação.",
        });

        let req = Request::builder()
            .method("POST")
            .uri("/news")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn post_news_usuario_sem_permissao_retorna_403() {
        let (app, _repo, _auth) = setup_test_router();
        // CALOURO_ID não tem permissão concedida

        let payload = serde_json::json!({
            "ca_id": CAER_ID,
            "title": "Tentativa Indevida",
            "body": "Corpo de notícia sem cargo de gestão.",
        });

        let req = Request::builder()
            .method("POST")
            .uri("/news")
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-User-Id", CALOURO_ID.to_string())
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn post_news_titulo_invalido_retorna_400() {
        let (app, _repo, auth) = setup_test_router();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let payload = serde_json::json!({
            "ca_id": CAER_ID,
            "title": "  ",
            "body": "Corpo válido.",
        });

        let req = Request::builder()
            .method("POST")
            .uri("/news")
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-User-Id", VETERANO_ID.to_string())
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn put_news_atualiza_noticia_com_sucesso() {
        let (app, repo, auth) = setup_test_router();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Título Inicial",
            "Texto inicial da notícia.",
        )
        .unwrap();
        let article_id = article.id;
        repo.save(&article).await.unwrap();

        let payload = serde_json::json!({
            "title": "Título Atualizado via API",
            "body": "Texto revisado via requisição HTTP.",
            "publish_directly": false
        });

        let req = Request::builder()
            .method("PUT")
            .uri(format!("/news/{article_id}"))
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-User-Id", VETERANO_ID.to_string())
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let resp: NewsArticleResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(resp.title, "Título Atualizado via API");
        assert_eq!(resp.revision, 2);
    }

    #[tokio::test]
    async fn get_news_by_id_retorna_noticia() {
        let (app, repo, _auth) = setup_test_router();

        let article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Notícia Consulta",
            "Texto para consulta por ID.",
        )
        .unwrap();
        let article_id = article.id;
        repo.save(&article).await.unwrap();

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news/{article_id}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let resp: NewsArticleResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(resp.id, article_id);
        assert_eq!(resp.title, "Notícia Consulta");
    }

    #[tokio::test]
    async fn get_news_list_retorna_noticias_do_ca() {
        let (app, repo, _auth) = setup_test_router();

        let a1 = NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia 1", "Texto 1.").unwrap();
        let a2 = NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia 2", "Texto 2.").unwrap();
        repo.save(&a1).await.unwrap();
        repo.save(&a2).await.unwrap();

        let req = Request::builder()
            .method("GET")
            .uri(format!("/news?ca_id={CAER_ID}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let list: Vec<NewsArticleResponse> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(list.len(), 2);
    }
}
