//! Contratos de transferência de dados (DTO) da API de demandas.
//!
//! Encapsula os payloads JSON de requisição e resposta do módulo de demandas,
//! mantendo o domínio estritamente livre de dependências de serialização.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::demands::domain::entities::Demand;

/// Payload de requisição para criação de uma nova demanda.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDemandRequest {
    /// Fórum alvo da demanda (`institutional` ou `physical_space`).
    pub forum: String,

    /// Título opcional identificador da solicitação.
    #[serde(default)]
    pub title: Option<String>,

    /// Conteúdo ou descrição detalhada da demanda.
    #[serde(alias = "content")]
    pub description: String,

    /// Categoria temática opcional.
    #[serde(default)]
    pub category: Option<String>,

    /// Nome do curso associado (obrigatório em demandas institucionais).
    #[serde(default, alias = "course_name")]
    pub course_name: Option<String>,

    /// Identificador opcional do autor no corpo da requisição.
    #[serde(default, alias = "author_id")]
    pub author_id: Option<Uuid>,
}

/// Representação serializada pública de uma demanda acadêmica ou de espaço físico.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DemandResponse {
    /// Identificador estável da demanda.
    pub id: Uuid,

    /// Identificador do Centro Acadêmico responsável.
    pub academic_center_id: Uuid,

    /// Identificador do estudante autor da solicitação.
    pub author_id: Uuid,

    /// Fórum ao qual a demanda pertence (`institutional` ou `physical_space`).
    pub forum: String,

    /// Título ou resumo identificador da demanda, se houver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Descrição detalhada do problema ou necessidade comunicada.
    pub description: String,

    /// Categoria temática, se informada.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Curso associado à demanda (presente somente em demandas institucionais).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_name: Option<String>,

    /// Status atual no ciclo de vida da demanda (`open`, `in_analysis`, etc.).
    pub status: String,

    /// Instante de registro da demanda em UTC.
    pub created_at: DateTime<Utc>,

    /// Instante da última modificação de estado em UTC.
    pub updated_at: DateTime<Utc>,
}

impl From<&Demand> for DemandResponse {
    fn from(demand: &Demand) -> Self {
        Self {
            id: demand.id(),
            academic_center_id: demand.academic_center_id(),
            author_id: demand.author_id(),
            forum: demand.forum().as_str().to_string(),
            title: demand.title().map(String::from),
            description: demand.description().to_string(),
            category: demand.category().map(String::from),
            course_name: demand.course_name().map(String::from),
            status: demand.status().as_str().to_string(),
            created_at: demand.created_at(),
            updated_at: demand.updated_at(),
        }
    }
}

impl From<Demand> for DemandResponse {
    fn from(demand: Demand) -> Self {
        Self::from(&demand)
    }
}
