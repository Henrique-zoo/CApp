//! Eventos de domínio do módulo de notícias.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::news::domain::entities::status::NewsStatus;

/// Fato relevante: artigo de notícia criado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleCreated {
    /// Identificador do artigo criado.
    pub article_id: Uuid,
    /// Identificador do Centro Acadêmico.
    pub ca_id: Uuid,
    /// Identificador do autor da notícia.
    pub author_id: Uuid,
    /// Status inicial da notícia.
    pub status: NewsStatus,
    /// Instante em que o evento ocorreu.
    pub occurred_at: DateTime<Utc>,
}

/// Fato relevante: artigo de notícia atualizado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleUpdated {
    /// Identificador do artigo atualizado.
    pub article_id: Uuid,
    /// Nova revisão resultante da edição.
    pub revision: u32,
    /// Status resultante da edição.
    pub status: NewsStatus,
    /// Instante em que o evento ocorreu.
    pub occurred_at: DateTime<Utc>,
}

/// Fato relevante: artigo de notícia publicado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticlePublished {
    /// Identificador do artigo publicado.
    pub article_id: Uuid,
    /// Instante em que o artigo foi publicado.
    pub published_at: DateTime<Utc>,
}

/// Fato relevante: artigo de notícia arquivado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleArchived {
    /// Identificador do artigo arquivado.
    pub article_id: Uuid,
    /// Instante em que o artigo foi arquivado.
    pub archived_at: DateTime<Utc>,
}
