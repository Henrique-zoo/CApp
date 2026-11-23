//! Contratos de transferência de dados (DTOs) para consultas e enquetes estudantis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::ca::domain::entities::{PollOptionResult, PollResults, PollVote, StudentPoll};

/// Corpo da requisição para cadastrar uma nova consulta estudantil.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePollRequest {
    /// Título oficial da consulta/proposta.
    pub title: String,
    /// Descrição detalhada ou texto da proposta (opcional).
    #[serde(default)]
    pub description: Option<String>,
    /// Lista de opções de voto disponíveis (mínimo de 2 opções distintas).
    pub options: Vec<String>,
    /// Início do período de votação (opcional, padrão: instante da criação).
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    /// Término do período de votação (opcional).
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    /// Público-alvo / critério de elegibilidade (opcional, padrão: "all_students").
    #[serde(default)]
    pub target_audience: Option<String>,
    /// Modalidade de votação ('identified' ou 'secret', padrão: 'identified').
    #[serde(default)]
    pub vote_modality: Option<String>,
}

/// Corpo da requisição para registrar um voto na enquete.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VotePollRequest {
    /// Identificador único da opção de voto escolhida.
    pub option_id: Uuid,
}

/// Representação JSON de uma opção de voto com sua respectiva apuração.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollOptionResponse {
    /// Identificador único da opção.
    pub id: Uuid,
    /// Texto descritivo da opção de voto.
    pub title: String,
    /// Ordem de exibição da opção.
    pub order: i32,
    /// Quantidade de votos recebidos pela opção.
    pub vote_count: i64,
    /// Percentual apurado em relação ao total de votos (0.0 a 100.0).
    pub percentage: f64,
}

impl From<&PollOptionResult> for PollOptionResponse {
    fn from(r: &PollOptionResult) -> Self {
        Self {
            id: r.id,
            title: r.title.clone(),
            order: r.order,
            vote_count: r.vote_count,
            percentage: r.percentage,
        }
    }
}

/// Representação JSON de uma consulta estudantil com seus resultados apurados.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    /// Identificador único da consulta.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do gestor criador da consulta.
    pub created_by_id: Uuid,
    /// Título oficial da consulta.
    pub title: String,
    /// Descrição ou proposta (se houver).
    pub description: Option<String>,
    /// Público-alvo configurado.
    pub target_audience: String,
    /// Modalidade de voto ('identified' ou 'secret').
    pub vote_modality: String,
    /// Início do período de votação.
    pub starts_at: DateTime<Utc>,
    /// Término do período de votação (se definido).
    pub ends_at: Option<DateTime<Utc>>,
    /// Situação da consulta ('open' ou 'closed').
    pub status: String,
    /// Total de votos depositados na consulta.
    pub total_votes: i64,
    /// Opções de voto e apuração de resultados.
    pub options: Vec<PollOptionResponse>,
    /// Instante de criação da consulta.
    pub created_at: DateTime<Utc>,
    /// Instante da última atualização.
    pub updated_at: DateTime<Utc>,
    /// Instante de encerramento da consulta (se encerrada).
    pub closed_at: Option<DateTime<Utc>>,
    /// Gestor que encerrou a consulta (se aplicável).
    pub closed_by_id: Option<Uuid>,
}

impl PollResponse {
    /// Constrói o DTO a partir da entidade e dos resultados apurados.
    pub fn from_entity_and_results(poll: &StudentPoll, results: &PollResults) -> Self {
        let options = results
            .options
            .iter()
            .map(PollOptionResponse::from)
            .collect();

        Self {
            id: poll.id(),
            academic_center_id: poll.academic_center_id(),
            created_by_id: poll.created_by_id(),
            title: poll.title().to_string(),
            description: poll.description().map(str::to_string),
            target_audience: poll.target_audience().to_string(),
            vote_modality: poll.vote_modality().to_string(),
            starts_at: poll.starts_at(),
            ends_at: poll.ends_at(),
            status: poll.status().to_string(),
            total_votes: results.total_votes,
            options,
            created_at: poll.created_at(),
            updated_at: poll.updated_at(),
            closed_at: poll.closed_at(),
            closed_by_id: poll.closed_by_id(),
        }
    }
}

/// Representação JSON do comprovante de depósito de voto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteResponse {
    /// Identificador do registro de voto.
    pub id: Uuid,
    /// Identificador da consulta votada.
    pub poll_id: Uuid,
    /// Identificador da opção escolhida.
    pub option_id: Uuid,
    /// Identificador do estudante votante.
    pub user_id: Uuid,
    /// Instante do voto.
    pub voted_at: DateTime<Utc>,
}

impl From<&PollVote> for VoteResponse {
    fn from(v: &PollVote) -> Self {
        Self {
            id: v.id(),
            poll_id: v.poll_id(),
            option_id: v.option_id(),
            user_id: v.user_id(),
            voted_at: v.voted_at(),
        }
    }
}
