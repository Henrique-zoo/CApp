//! Serviços de aplicação do módulo de notícias.
//!
//! Coordena os casos de uso de leitura, publicação, submissão e arquivamento,
//! fornecendo uma interface unificada para a camada de apresentação HTTP.

use std::sync::Arc;

use uuid::Uuid;

use crate::modules::news::{
    application::{
        NewsApplicationError,
        commands::{
            ArchiveNewsArticleCommand, ArchiveNewsArticleUseCase, PublishNewsArticleCommand,
            PublishNewsArticleUseCase, SubmitForReviewNewsArticleCommand,
            SubmitForReviewNewsArticleUseCase,
        },
        ports::{NewsAuthorizationPort, NewsRepositoryPort},
        queries::{
            GetNewsArticleQuery, GetNewsArticleUseCase, GetPublishedNewsDetailQuery,
            GetPublishedNewsDetailUseCase, ListPublishedNewsByCaQuery,
            ListPublishedNewsByCaUseCase,
        },
    },
    domain::NewsArticle,
};

/// Serviço coordenador das operações da aplicação de notícias.
pub struct NewsApplicationService<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    repository: Arc<R>,
    authorization: Arc<A>,
    publish_use_case: PublishNewsArticleUseCase<R, A>,
    archive_use_case: ArchiveNewsArticleUseCase<R, A>,
    submit_use_case: SubmitForReviewNewsArticleUseCase<R, A>,
    get_use_case: GetNewsArticleUseCase<R>,
    list_published_use_case: ListPublishedNewsByCaUseCase<R>,
    get_published_detail_use_case: GetPublishedNewsDetailUseCase<R>,
}

impl<R, A> NewsApplicationService<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    /// Constrói o serviço com as instâncias do repositório e autorizador.
    pub fn new(repository: Arc<R>, authorization: Arc<A>) -> Self {
        Self {
            repository: Arc::clone(&repository),
            authorization: Arc::clone(&authorization),
            publish_use_case: PublishNewsArticleUseCase::new(
                Arc::clone(&repository),
                Arc::clone(&authorization),
            ),
            archive_use_case: ArchiveNewsArticleUseCase::new(
                Arc::clone(&repository),
                Arc::clone(&authorization),
            ),
            submit_use_case: SubmitForReviewNewsArticleUseCase::new(
                Arc::clone(&repository),
                Arc::clone(&authorization),
            ),
            get_use_case: GetNewsArticleUseCase::new(Arc::clone(&repository)),
            list_published_use_case: ListPublishedNewsByCaUseCase::new(Arc::clone(&repository)),
            get_published_detail_use_case: GetPublishedNewsDetailUseCase::new(Arc::clone(
                &repository,
            )),
        }
    }

    /// Executa o comando de publicação de notícia.
    pub async fn publish_news(
        &self,
        command: PublishNewsArticleCommand,
    ) -> Result<NewsArticle, NewsApplicationError> {
        self.publish_use_case.execute(command).await
    }

    /// Executa o comando de arquivamento de notícia.
    pub async fn archive_news(
        &self,
        command: ArchiveNewsArticleCommand,
    ) -> Result<NewsArticle, NewsApplicationError> {
        self.archive_use_case.execute(command).await
    }

    /// Executa o comando de submissão para homologação.
    pub async fn submit_for_review(
        &self,
        command: SubmitForReviewNewsArticleCommand,
    ) -> Result<NewsArticle, NewsApplicationError> {
        self.submit_use_case.execute(command).await
    }

    /// Executa a busca de uma notícia pelo identificador.
    pub async fn get_news_by_id(&self, news_id: Uuid) -> Result<NewsArticle, NewsApplicationError> {
        self.get_use_case
            .execute(GetNewsArticleQuery { news_id })
            .await
    }

    /// Executa a listagem pública de notícias publicadas para o Centro Acadêmico informado.
    pub async fn list_published_news_by_ca(
        &self,
        ca_id: Uuid,
    ) -> Result<Vec<NewsArticle>, NewsApplicationError> {
        self.list_published_use_case
            .execute(ListPublishedNewsByCaQuery { ca_id })
            .await
    }

    /// Executa a consulta de detalhes de uma notícia publicada.
    ///
    /// Retorna [`NewsApplicationError::NotFound`] se o artigo não existir ou não estiver publicado.
    pub async fn get_published_news_detail(
        &self,
        news_id: Uuid,
    ) -> Result<NewsArticle, NewsApplicationError> {
        self.get_published_detail_use_case
            .execute(GetPublishedNewsDetailQuery { news_id })
            .await
    }

    /// Cria o rascunho e publica diretamente quando a operação foi autorizada.
    pub async fn create(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        title: String,
        body: String,
        direct: bool,
    ) -> Result<NewsArticle, NewsApplicationError> {
        if !self.authorization.can_publish_news(user_id, ca_id).await? {
            return Err(NewsApplicationError::Forbidden {
                user_id,
                ca_id,
                operation: "create",
            });
        }
        let now = chrono::Utc::now();
        let mut article = NewsArticle::new(Uuid::new_v4(), ca_id, user_id, title, body, now)?;
        if direct {
            article.publish(now)?;
        }
        self.repository.save(&article).await?;
        Ok(article)
    }

    /// Edita o conteúdo e devolve a publicação anterior à revisão.
    pub async fn edit(
        &self,
        news_id: Uuid,
        user_id: Uuid,
        title: String,
        body: String,
    ) -> Result<NewsArticle, NewsApplicationError> {
        let mut article = self.get_news_by_id(news_id).await?;
        let ca_id = article.ca_id;
        if !self.authorization.can_publish_news(user_id, ca_id).await? {
            return Err(NewsApplicationError::Forbidden {
                user_id,
                ca_id,
                operation: "edit",
            });
        }
        article.update_content(title, body)?;
        if article.status.is_published() {
            article.status = crate::modules::news::domain::NewsStatus::PendingReview;
            article.published_at = None;
        }
        self.repository.update(&article).await?;
        Ok(article)
    }
}
