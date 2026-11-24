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
        queries::{GetNewsArticleQuery, GetNewsArticleUseCase},
    },
    domain::NewsArticle,
};

/// Serviço coordenador das operações da aplicação de notícias.
pub struct NewsApplicationService<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    publish_use_case: PublishNewsArticleUseCase<R, A>,
    archive_use_case: ArchiveNewsArticleUseCase<R, A>,
    submit_use_case: SubmitForReviewNewsArticleUseCase<R, A>,
    get_use_case: GetNewsArticleUseCase<R>,
}

impl<R, A> NewsApplicationService<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    /// Constrói o serviço com as instâncias do repositório e autorizador.
    pub fn new(repository: Arc<R>, authorization: Arc<A>) -> Self {
        Self {
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
}
