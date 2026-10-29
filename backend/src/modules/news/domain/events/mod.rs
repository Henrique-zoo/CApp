//! Eventos de domínio do módulo de notícias.
//!
//! Representam fatos de negócio imutáveis ocorridos no ciclo de vida editorial,
//! úteis para notificação, integração e auditoria.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Evento disparado quando um artigo de notícia é submetido para revisão da gestão.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleSubmittedForReviewEvent {
    /// Identificador único da notícia submetida.
    pub news_id: Uuid,
    /// Centro Acadêmico contextual da notícia.
    pub ca_id: Uuid,
    /// Usuário autor que submeteu o artigo.
    pub author_id: Uuid,
    /// Versão da revisão submetida.
    pub revision: i32,
    /// Instante do evento.
    pub occurred_at: DateTime<Utc>,
}

/// Evento disparado quando um artigo de notícia é publicado no mural.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticlePublishedEvent {
    /// Identificador único da notícia publicada.
    pub news_id: Uuid,
    /// Centro Acadêmico contextual da notícia.
    pub ca_id: Uuid,
    /// Usuário com papel de gestão responsável pela publicação.
    pub published_by: Uuid,
    /// Instante oficial da publicação.
    pub published_at: DateTime<Utc>,
}

/// Evento disparado quando um artigo de notícia é arquivado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleArchivedEvent {
    /// Identificador único da notícia arquivada.
    pub news_id: Uuid,
    /// Centro Acadêmico contextual da notícia.
    pub ca_id: Uuid,
    /// Usuário que realizou a ação de arquivamento.
    pub archived_by: Uuid,
    /// Instante oficial do arquivamento.
    pub archived_at: DateTime<Utc>,
}
