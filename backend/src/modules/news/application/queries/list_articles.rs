//! Caso de uso: listagem de notícias por Centro Acadêmico.

use uuid::Uuid;

use crate::modules::news::application::ports::NewsRepository;
use crate::modules::news::domain::{NewsArticle, NewsError};

/// Consulta para listar notícias de um Centro Acadêmico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListNewsArticlesByCaQuery {
    /// Identificador do Centro Acadêmico.
    pub ca_id: Uuid,
}

/// Manipulador da consulta de listagem de notícias por Centro Acadêmico.
pub struct ListNewsArticlesByCaHandler<R> {
    repository: R,
}

impl<R> ListNewsArticlesByCaHandler<R>
where
    R: NewsRepository,
{
    /// Constrói o manipulador com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a listagem de notícias pertencentes ao Centro Acadêmico.
    pub async fn execute(
        &self,
        query: ListNewsArticlesByCaQuery,
    ) -> Result<Vec<NewsArticle>, NewsError> {
        self.repository.find_by_ca_id(query.ca_id).await
    }
}
