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

/// Parâmetros de consulta via query string para listagem e filtragem de demandas.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListDemandsQueryParams {
    /// Fórum alvo da demanda (`institutional` ou `physical_space`).
    pub forum: Option<String>,

    /// Status atual no ciclo de vida da demanda (`open`, `in_analysis`, etc.).
    pub status: Option<String>,

    /// Quando verdadeiro, restringe os resultados às demandas registradas pelo usuário autenticado.
    #[serde(default)]
    pub mine: Option<bool>,

    /// Quantidade máxima de registros retornados (paginação).
    pub limit: Option<usize>,

    /// Deslocamento inicial de registros para paginação.
    pub offset: Option<usize>,

    /// Número da página baseada em 1 (conveniência para clientes).
    pub page: Option<usize>,

    /// Tamanho da página (alias para limite de registros).
    #[serde(alias = "page_size")]
    pub page_size: Option<usize>,
}

/// Payload de requisição para atualização de status de uma demanda pela gestão.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDemandStatusRequest {
    /// Novo status pretendido para a demanda no ciclo de vida.
    pub status: String,

    /// Parecer ou justificativa obrigatória da alteração de status.
    #[serde(
        default,
        alias = "reason",
        alias = "comment",
        alias = "justificativa",
        alias = "parecer"
    )]
    pub opinion: Option<String>,
}

/// Representação serializada pública de um registro de histórico de status da demanda.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DemandStatusHistoryResponse {
    /// Identificador do registro de histórico.
    pub id: Uuid,

    /// Identificador da demanda vinculada.
    pub demand_id: Uuid,

    /// Status anterior à alteração, se houver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_status: Option<String>,

    /// Novo status assumido pela demanda.
    pub to_status: String,

    /// Identificador do usuário que realizou a alteração.
    pub changed_by_id: Uuid,

    /// Parecer ou justificativa registrada (comentário da transição).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Parecer registrado (alias semântico para comment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opinion: Option<String>,

    /// Instante UTC da alteração de status.
    pub created_at: DateTime<Utc>,
}

impl From<&crate::modules::demands::domain::entities::DemandStatusHistoryEntry>
    for DemandStatusHistoryResponse
{
    fn from(entry: &crate::modules::demands::domain::entities::DemandStatusHistoryEntry) -> Self {
        Self {
            id: entry.id(),
            demand_id: entry.demand_id(),
            from_status: entry.from_status().map(|s| s.as_str().to_string()),
            to_status: entry.to_status().as_str().to_string(),
            changed_by_id: entry.changed_by_id(),
            comment: entry.comment().map(String::from),
            opinion: entry.comment().map(String::from),
            created_at: entry.created_at(),
        }
    }
}

impl From<crate::modules::demands::domain::entities::DemandStatusHistoryEntry>
    for DemandStatusHistoryResponse
{
    fn from(entry: crate::modules::demands::domain::entities::DemandStatusHistoryEntry) -> Self {
        Self::from(&entry)
    }
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

    /// Histórico de alterações de status da demanda, presente em consultas detalhadas e após transição.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_history: Option<Vec<DemandStatusHistoryResponse>>,
}

impl From<&Demand> for DemandResponse {
    fn from(demand: &Demand) -> Self {
        let status_history = if demand.status_history().is_empty() {
            None
        } else {
            Some(
                demand
                    .status_history()
                    .iter()
                    .map(DemandStatusHistoryResponse::from)
                    .collect(),
            )
        };

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
            status_history,
        }
    }
}

impl From<Demand> for DemandResponse {
    fn from(demand: Demand) -> Self {
        Self::from(&demand)
    }
}
