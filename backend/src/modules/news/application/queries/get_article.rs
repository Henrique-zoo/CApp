//! Caso de uso: consulta de notícia por identificador.

use uuid::Uuid;

use crate::modules::news::application::ports::NewsRepository;
use crate::modules::news::domain::{NewsArticle, NewsError};

/// Consulta para obter uma notícia por seu identificador único.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetNewsArticleQuery {
    /// Identificador da notícia procurada.
    pub id: Uuid,
}

/// Manipulador da consulta de notícia por identificador.
pub struct GetNewsArticleHandler<R> {
    repository: R,
}

impl<R> GetNewsArticleHandler<R>
where
    R: NewsRepository,
{
    /// Constrói o manipulador com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a busca da notícia, retornando [`NewsError::ArticleNotFound`] se ausente.
    pub async fn execute(&self, query: GetNewsArticleQuery) -> Result<NewsArticle, NewsError> {
        self.repository
            .find_by_id(query.id)
            .await?
            .ok_or(NewsError::ArticleNotFound { id: query.id })
    }
}
