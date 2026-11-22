//! Contratos de transferência de dados (DTO) da API de perguntas frequentes (FAQ).
//!
//! Encapsula os payloads JSON de requisição e resposta do módulo de FAQ,
//! mantendo o domínio estritamente livre de dependências de serialização HTTP.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::faq::domain::entities::faq_entry::FaqEntry;

/// Payload de requisição para cadastro de uma nova pergunta e resposta de FAQ.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFaqEntryRequest {
    /// Texto da pergunta frequente.
    pub question: String,

    /// Resposta fornecida pelo Centro Acadêmico.
    pub answer: String,

    /// Categoria temática opcional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Identificador opcional do autor no corpo da requisição.
    #[serde(default, alias = "author_id", skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Uuid>,
}

/// Representação serializada pública de uma pergunta e resposta de FAQ.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaqEntryResponse {
    /// Identificador estável da entrada de FAQ.
    pub id: Uuid,

    /// Identificador do Centro Acadêmico vinculado.
    #[serde(alias = "academic_center_id")]
    pub academic_center_id: Uuid,

    /// Texto da pergunta frequente.
    pub question: String,

    /// Resposta oficial.
    pub answer: String,

    /// Categoria temática, se informada.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Identificador do usuário que realizou o cadastro inicial.
    #[serde(alias = "created_by")]
    pub created_by: Uuid,

    /// Identificador do usuário que realizou a última modificação.
    #[serde(alias = "updated_by")]
    pub updated_by: Uuid,

    /// Instante de registro em UTC.
    #[serde(alias = "created_at")]
    pub created_at: DateTime<Utc>,

    /// Instante da última modificação em UTC.
    #[serde(alias = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

impl From<&FaqEntry> for FaqEntryResponse {
    fn from(entry: &FaqEntry) -> Self {
        Self {
            id: entry.id(),
            academic_center_id: entry.academic_center_id(),
            question: entry.question().to_string(),
            answer: entry.answer().to_string(),
            category: entry.category().map(String::from),
            created_by: entry.created_by(),
            updated_by: entry.updated_by(),
            created_at: entry.created_at(),
            updated_at: entry.updated_at(),
        }
    }
}

impl From<FaqEntry> for FaqEntryResponse {
    fn from(entry: FaqEntry) -> Self {
        Self::from(&entry)
    }
}
