//! Eventos de domínio do ciclo de vida de notícias.
//!
//! Representam fatos imutáveis ocorridos no negócio e expressam
//! transições e ações relevantes sobre uma notícia.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::news::domain::entities::NewsStatus;

/// Evento emitido quando uma notícia é criada ou registrada como proposta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsCreated {
    /// Identificador da notícia recém-criada.
    pub news_id: Uuid,
    /// Centro Acadêmico proprietário da notícia.
    pub academic_center_id: Uuid,
    /// Autor do conteúdo.
    pub author_id: Uuid,
    /// Situação inicial da notícia (geralmente rascunho ou pendente).
    pub status: NewsStatus,
    /// Instante do registro da notícia.
    pub created_at: DateTime<Utc>,
}

/// Evento emitido quando uma notícia é publicada e disponibilizada no mural.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsPublished {
    /// Identificador da notícia publicada.
    pub news_id: Uuid,
    /// Centro Acadêmico proprietário da notícia.
    pub academic_center_id: Uuid,
    /// Autor do conteúdo.
    pub author_id: Uuid,
    /// Instante em que a publicação foi efetivada.
    pub published_at: DateTime<Utc>,
}

/// Evento emitido quando o título, corpo ou tags de uma notícia são atualizados.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsContentUpdated {
    /// Identificador da notícia modificada.
    pub news_id: Uuid,
    /// Centro Acadêmico proprietário da notícia.
    pub academic_center_id: Uuid,
    /// Instante da atualização do conteúdo.
    pub updated_at: DateTime<Utc>,
}

/// Evento emitido quando uma notícia é arquivada e retirada da listagem corrente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArchived {
    /// Identificador da notícia arquivada.
    pub news_id: Uuid,
    /// Centro Acadêmico proprietário da notícia.
    pub academic_center_id: Uuid,
    /// Instante do arquivamento.
    pub archived_at: DateTime<Utc>,
}

/// Evento emitido quando uma proposta de notícia é submetida para homologação de gestor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsSubmittedForReview {
    /// Identificador da notícia submetida.
    pub news_id: Uuid,
    /// Centro Acadêmico proprietário da notícia.
    pub academic_center_id: Uuid,
    /// Autor do conteúdo.
    pub author_id: Uuid,
    /// Instante da submissão para análise.
    pub submitted_at: DateTime<Utc>,
}

/// Evento emitido quando a proposta de notícia é recusada/rejeitada na análise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsRejected {
    /// Identificador da notícia rejeitada.
    pub news_id: Uuid,
    /// Centro Acadêmico proprietário da notícia.
    pub academic_center_id: Uuid,
    /// Instante da rejeição.
    pub rejected_at: DateTime<Utc>,
    /// Justificativa ou motivo da recusa (se fornecido).
    pub reason: Option<String>,
}

/// Enumeração de todos os eventos de domínio emitidos pelo ciclo de vida de uma notícia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewsDomainEvent {
    /// Notícia criada no sistema.
    Created(NewsCreated),
    /// Notícia publicada nos murais.
    Published(NewsPublished),
    /// Conteúdo da notícia atualizado.
    ContentUpdated(NewsContentUpdated),
    /// Notícia arquivada.
    Archived(NewsArchived),
    /// Notícia submetida para análise.
    SubmittedForReview(NewsSubmittedForReview),
    /// Notícia recusada na análise.
    Rejected(NewsRejected),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_instantiate_and_match_domain_events() {
        let now = Utc::now();
        let news_id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let event = NewsDomainEvent::Created(NewsCreated {
            news_id,
            academic_center_id: ca_id,
            author_id,
            status: NewsStatus::Draft,
            created_at: now,
        });

        match event {
            NewsDomainEvent::Created(e) => {
                assert_eq!(e.news_id, news_id);
                assert_eq!(e.status, NewsStatus::Draft);
            }
            _ => panic!("tipo de evento incorreto"),
        }
    }
}
