//! Consultas do módulo de notícias.
//!
//! Organiza as operações de leitura que não alteram o estado do domínio.

use std::sync::Arc;

use uuid::Uuid;

use crate::modules::news::{
    application::{NewsApplicationError, ports::NewsRepositoryPort},
    domain::NewsArticle,
};

/// Consulta para recuperação de um artigo de notícia pelo identificador único.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetNewsArticleQuery {
    /// Identificador único da notícia.
    pub news_id: Uuid,
}

/// Caso de uso para obtenção de detalhes de uma notícia.
pub struct GetNewsArticleUseCase<R>
where
    R: NewsRepositoryPort + ?Sized,
{
    repository: Arc<R>,
}

impl<R> GetNewsArticleUseCase<R>
where
    R: NewsRepositoryPort + ?Sized,
{
    /// Constrói o caso de uso injetando o repositório de notícias.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa a consulta, retornando o artigo ou erro se não for encontrado.
    ///
    /// # Erros
    ///
    /// - [`NewsApplicationError::NotFound`] se o artigo não existir.
    /// - [`NewsApplicationError::Repository`] em falhas de leitura no armazenamento.
    pub async fn execute(
        &self,
        query: GetNewsArticleQuery,
    ) -> Result<NewsArticle, NewsApplicationError> {
        self.repository
            .find_by_id(query.news_id)
            .await?
            .ok_or(NewsApplicationError::NotFound(query.news_id))
    }
}
