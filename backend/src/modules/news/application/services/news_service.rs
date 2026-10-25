//! Serviço de aplicação para orquestração das operações de notícias.

use uuid::Uuid;

use crate::modules::news::application::commands::{
    CreateNewsArticleCommand, CreateNewsArticleHandler, UpdateNewsArticleCommand,
    UpdateNewsArticleHandler,
};
use crate::modules::news::application::ports::{NewsAuthorizationPort, NewsRepository};
use crate::modules::news::application::queries::{
    GetNewsArticleHandler, GetNewsArticleQuery, ListNewsArticlesByCaHandler,
    ListNewsArticlesByCaQuery,
};
use crate::modules::news::domain::{NewsArticle, NewsError};

/// Serviço de aplicação que coordena casos de uso de notícias.
pub struct NewsService<R, A> {
    repository: R,
    authorization_port: A,
}

impl<R, A> NewsService<R, A>
where
    R: NewsRepository + Clone,
    A: NewsAuthorizationPort + Clone,
{
    /// Constrói o serviço com repositório e porta de autorização.
    pub fn new(repository: R, authorization_port: A) -> Self {
        Self {
            repository,
            authorization_port,
        }
    }

    /// Cria um novo artigo de notícia após validar permissões e regras de domínio.
    pub async fn create_article(
        &self,
        cmd: CreateNewsArticleCommand,
    ) -> Result<NewsArticle, NewsError> {
        let handler =
            CreateNewsArticleHandler::new(self.repository.clone(), self.authorization_port.clone());
        handler.execute(cmd).await
    }

    /// Atualiza um artigo de notícia existente, aplicando as regras de revisão e homologação.
    pub async fn update_article(
        &self,
        cmd: UpdateNewsArticleCommand,
    ) -> Result<NewsArticle, NewsError> {
        let handler =
            UpdateNewsArticleHandler::new(self.repository.clone(), self.authorization_port.clone());
        handler.execute(cmd).await
    }

    /// Obtém uma notícia por seu identificador.
    pub async fn get_article(&self, id: Uuid) -> Result<NewsArticle, NewsError> {
        let handler = GetNewsArticleHandler::new(self.repository.clone());
        handler.execute(GetNewsArticleQuery { id }).await
    }

    /// Lista as notícias de um Centro Acadêmico específico.
    pub async fn list_by_ca(&self, ca_id: Uuid) -> Result<Vec<NewsArticle>, NewsError> {
        let handler = ListNewsArticlesByCaHandler::new(self.repository.clone());
        handler.execute(ListNewsArticlesByCaQuery { ca_id }).await
    }
}
