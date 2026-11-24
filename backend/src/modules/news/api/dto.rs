//! Contratos de transferência de dados (DTOs) da API de notícias.
//!
//! Separa o modelo público exposto pelas rotas HTTP das entidades
//! internas de domínio e persistência.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::news::domain::NewsArticle;

/// Payload para a ação de publicação de uma notícia.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishNewsRequestDto {
    /// Identificador do usuário operador (opcional se fornecido via header `x-user-id`).
    pub actor_id: Option<Uuid>,
}

/// Payload para a ação de arquivamento de uma notícia.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiveNewsRequestDto {
    /// Identificador do usuário operador (opcional se fornecido via header `x-user-id`).
    pub actor_id: Option<Uuid>,
}

/// Payload para a ação de submissão de uma notícia para homologação.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubmitForReviewRequestDto {
    /// Identificador do usuário operador (opcional se fornecido via header `x-user-id`).
    pub actor_id: Option<Uuid>,
}

/// Parâmetros de consulta via query string para a listagem pública de notícias: `GET /news`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListNewsQueryParams {
    /// Identificador do Centro Acadêmico ativo.
    pub ca_id: Option<Uuid>,
}

/// Representação pública de uma notícia no mural do Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticleResponseDto {
    /// Identificador único da notícia.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico associado.
    pub ca_id: Uuid,
    /// Identificador do autor original.
    pub author_id: Uuid,
    /// Título do artigo.
    pub title: String,
    /// Conteúdo textual do artigo.
    pub body: String,
    /// Estado atual do ciclo editorial (`draft`, `pending_review`, `published`, `rejected`, `archived`).
    pub status: String,
    /// Número sequencial da revisão do artigo.
    pub revision: i32,
    /// Instante em que o registro inicial foi criado.
    pub created_at: DateTime<Utc>,
    /// Instante de publicação efetiva, se publicado.
    pub published_at: Option<DateTime<Utc>>,
    /// Instante de arquivamento, se arquivado.
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<NewsArticle> for NewsArticleResponseDto {
    fn from(article: NewsArticle) -> Self {
        Self {
            id: article.id,
            ca_id: article.ca_id,
            author_id: article.author_id,
            title: article.title,
            body: article.body,
            status: article.status.to_string(),
            revision: article.revision,
            created_at: article.created_at,
            published_at: article.published_at,
            archived_at: article.archived_at,
        }
    }
}

/// Formato padronizado para respostas de erro da API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponseDto {
    /// Mensagem descritiva da falha para exibição amigável.
    pub error: String,
    /// Código semântico do erro para tratamento programático no cliente.
    pub code: String,
    /// Detalhes técnicos adicionais, quando disponíveis.
    pub details: Option<String>,
}

impl ErrorResponseDto {
    /// Constrói uma nova resposta de erro padronizada.
    pub fn new(code: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            error: error.into(),
            details: None,
        }
    }

    /// Acrescenta detalhes diagnósticos à resposta de erro.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
