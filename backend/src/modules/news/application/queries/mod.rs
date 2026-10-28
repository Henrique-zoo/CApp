//! Consultas do módulo de notícias.
//!
//! Organiza as operações de leitura que não alteram o estado do domínio.

use std::sync::Arc;

use uuid::Uuid;

use crate::modules::news::{
    application::{NewsApplicationError, ports::NewsRepositoryPort},
    domain::{NewsArticle, NewsStatus},
};

/// Consulta para recuperação de um artigo de notícia pelo identificador único.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetNewsArticleQuery {
    /// Identificador único da notícia.
    pub news_id: Uuid,
}

/// Consulta para listagem de notícias publicadas de um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPublishedNewsByCaQuery {
    /// Identificador do Centro Acadêmico ativo.
    pub ca_id: Uuid,
}

/// Consulta para recuperação detalhada de uma notícia publicada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetPublishedNewsDetailQuery {
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

/// Caso de uso para listar notícias publicadas do Centro Acadêmico ativo.
pub struct ListPublishedNewsByCaUseCase<R>
where
    R: NewsRepositoryPort + ?Sized,
{
    repository: Arc<R>,
}

impl<R> ListPublishedNewsByCaUseCase<R>
where
    R: NewsRepositoryPort + ?Sized,
{
    /// Constrói o caso de uso injetando o repositório de notícias.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa a listagem, retornando apenas as notícias publicadas em ordem cronológica decrescente.
    ///
    /// # Erros
    ///
    /// - [`NewsApplicationError::Repository`] em falhas de leitura no armazenamento.
    pub async fn execute(
        &self,
        query: ListPublishedNewsByCaQuery,
    ) -> Result<Vec<NewsArticle>, NewsApplicationError> {
        let articles = self.repository.find_published_by_ca(query.ca_id).await?;
        Ok(articles)
    }
}

/// Caso de uso para obtenção dos detalhes de uma notícia publicada.
pub struct GetPublishedNewsDetailUseCase<R>
where
    R: NewsRepositoryPort + ?Sized,
{
    repository: Arc<R>,
}

impl<R> GetPublishedNewsDetailUseCase<R>
where
    R: NewsRepositoryPort + ?Sized,
{
    /// Constrói o caso de uso injetando o repositório de notícias.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa a consulta garantindo que apenas conteúdos com status `Published` sejam disponibilizados.
    ///
    /// # Erros
    ///
    /// - [`NewsApplicationError::NotFound`] se a notícia não existir ou não estiver publicada.
    /// - [`NewsApplicationError::Repository`] em falhas de leitura no armazenamento.
    pub async fn execute(
        &self,
        query: GetPublishedNewsDetailQuery,
    ) -> Result<NewsArticle, NewsApplicationError> {
        let article = self
            .repository
            .find_by_id(query.news_id)
            .await?
            .ok_or(NewsApplicationError::NotFound(query.news_id))?;

        if article.status != NewsStatus::Published {
            return Err(NewsApplicationError::NotFound(query.news_id));
        }

        Ok(article)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;
    use crate::modules::news::infrastructure::InMemoryNewsRepository;

    #[tokio::test]
    async fn deve_listar_apenas_noticias_publicadas_do_ca_ordenadas_por_published_at_desc() {
        let ca_computacao = Uuid::new_v4();
        let ca_matematica = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let noticia_antiga = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Comunicado antigo".to_string(),
            "Corpo antigo".to_string(),
            NewsStatus::Published,
            1,
            now - Duration::days(5),
            Some(now - Duration::days(4)),
            None,
        );

        let noticia_recente = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Comunicado recente".to_string(),
            "Corpo recente".to_string(),
            NewsStatus::Published,
            1,
            now - Duration::days(2),
            Some(now - Duration::days(1)),
            None,
        );

        let noticia_meio = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Comunicado do meio".to_string(),
            "Corpo do meio".to_string(),
            NewsStatus::Published,
            1,
            now - Duration::days(3),
            Some(now - Duration::days(2)),
            None,
        );

        let noticia_rascunho = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_computacao,
            author_id,
            "Rascunho de evento".to_string(),
            "Corpo rascunho".to_string(),
            NewsStatus::Draft,
            1,
            now,
            None,
            None,
        );

        let noticia_outro_ca = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_matematica,
            author_id,
            "Semana de Matemática".to_string(),
            "Corpo matemática".to_string(),
            NewsStatus::Published,
            1,
            now,
            Some(now),
            None,
        );

        let repo = Arc::new(InMemoryNewsRepository::with_articles([
            noticia_antiga.clone(),
            noticia_recente.clone(),
            noticia_meio.clone(),
            noticia_rascunho,
            noticia_outro_ca,
        ]));

        let use_case = ListPublishedNewsByCaUseCase::new(repo);
        let result = use_case
            .execute(ListPublishedNewsByCaQuery {
                ca_id: ca_computacao,
            })
            .await
            .expect("A listagem de notícias deve ser bem-sucedida");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Comunicado recente");
        assert_eq!(result[1].title, "Comunicado do meio");
        assert_eq!(result[2].title, "Comunicado antigo");
    }

    #[tokio::test]
    async fn deve_retornar_lista_vazia_quando_nao_houver_noticias_publicadas_no_ca() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let noticia_rascunho = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Rascunho".to_string(),
            "Corpo".to_string(),
            NewsStatus::Draft,
            1,
            now,
            None,
            None,
        );

        let repo = Arc::new(InMemoryNewsRepository::with_articles([noticia_rascunho]));
        let use_case = ListPublishedNewsByCaUseCase::new(repo);

        let result = use_case
            .execute(ListPublishedNewsByCaQuery { ca_id })
            .await
            .expect("Deve retornar sucesso com lista vazia");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn deve_obter_detalhes_de_noticia_publicada() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let noticia = NewsArticle::reconstruct(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Mudança no calendário".to_string(),
            "O calendário acadêmico foi atualizado.".to_string(),
            NewsStatus::Published,
            1,
            now,
            Some(now),
            None,
        );

        let repo = Arc::new(InMemoryNewsRepository::with_articles([noticia.clone()]));
        let use_case = GetPublishedNewsDetailUseCase::new(repo);

        let result = use_case
            .execute(GetPublishedNewsDetailQuery {
                news_id: noticia.id,
            })
            .await
            .expect("Deve retornar a notícia publicada");

        assert_eq!(result.id, noticia.id);
        assert_eq!(result.title, "Mudança no calendário");
        assert_eq!(result.body, "O calendário acadêmico foi atualizado.");
    }

    #[tokio::test]
    async fn nao_deve_disponibilizar_detalhes_de_noticia_nao_publicada() {
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let status_nao_publicados = [
            NewsStatus::Draft,
            NewsStatus::PendingReview,
            NewsStatus::Rejected,
            NewsStatus::Archived,
        ];

        for status in status_nao_publicados {
            let noticia = NewsArticle::reconstruct(
                Uuid::new_v4(),
                ca_id,
                author_id,
                "Proposta confidencial".to_string(),
                "Conteúdo interno".to_string(),
                status,
                1,
                now,
                None,
                None,
            );

            let repo = Arc::new(InMemoryNewsRepository::with_articles([noticia.clone()]));
            let use_case = GetPublishedNewsDetailUseCase::new(repo);

            let result = use_case
                .execute(GetPublishedNewsDetailQuery {
                    news_id: noticia.id,
                })
                .await;

            match result {
                Err(NewsApplicationError::NotFound(id)) => assert_eq!(id, noticia.id),
                _ => panic!("Notícia em status '{status:?}' não pode ter seus detalhes expostos"),
            }
        }
    }
}
