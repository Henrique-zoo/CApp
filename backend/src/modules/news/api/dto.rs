//! Contratos de dados (DTOs) da API HTTP do módulo de notícias.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::news::domain::NewsArticle;

/// Contrato de requisição para criar um artigo de notícia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateNewsArticleRequest {
    /// Identificador do Centro Acadêmico proprietário da notícia.
    pub ca_id: Uuid,
    /// Título da notícia.
    pub title: String,
    /// Conteúdo textual da notícia.
    pub body: String,
    /// Se verdadeiro, publica diretamente sem necessidade de homologação posterior.
    #[serde(default)]
    pub publish_directly: bool,
}

/// Contrato de requisição para atualizar um artigo de notícia existente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateNewsArticleRequest {
    /// Novo título para a notícia.
    pub title: String,
    /// Novo conteúdo textual para a notícia.
    pub body: String,
    /// Se verdadeiro, publica diretamente ou mantém em publicação caso autorizado.
    #[serde(default)]
    pub publish_directly: bool,
}

/// Contrato de resposta representando um artigo de notícia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleResponse {
    /// Identificador estável da notícia.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico proprietário.
    pub ca_id: Uuid,
    /// Identificador do autor responsável pela notícia.
    pub author_id: Uuid,
    /// Título apresentado.
    pub title: String,
    /// Texto do conteúdo.
    pub body: String,
    /// Situação editorial atual da notícia.
    pub status: String,
    /// Número da revisão atual do conteúdo.
    pub revision: u32,
    /// Instante em que a notícia foi criada.
    pub created_at: DateTime<Utc>,
    /// Instante em que o conteúdo foi publicado, se aplicável.
    pub published_at: Option<DateTime<Utc>>,
    /// Instante em que a notícia foi arquivada, se aplicável.
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<NewsArticle> for NewsArticleResponse {
    fn from(article: NewsArticle) -> Self {
        Self {
            id: article.id,
            ca_id: article.ca_id,
            author_id: article.author_id,
            title: article.title,
            body: article.body,
            status: article.status.as_str().to_string(),
            revision: article.revision,
            created_at: article.created_at,
            published_at: article.published_at,
            archived_at: article.archived_at,
        }
    }
}

impl From<&NewsArticle> for NewsArticleResponse {
    fn from(article: &NewsArticle) -> Self {
        Self {
            id: article.id,
            ca_id: article.ca_id,
            author_id: article.author_id,
            title: article.title.clone(),
            body: article.body.clone(),
            status: article.status.as_str().to_string(),
            revision: article.revision,
            created_at: article.created_at,
            published_at: article.published_at,
            archived_at: article.archived_at,
        }
    }
}

/// Parâmetros de consulta para listagem de notícias.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListNewsArticlesQuery {
    /// Identificador do Centro Acadêmico.
    pub ca_id: Uuid,
}

/// Contrato padronizado de resposta de erro da API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsErrorResponse {
    /// Mensagem descritiva do erro.
    pub error: String,
}
