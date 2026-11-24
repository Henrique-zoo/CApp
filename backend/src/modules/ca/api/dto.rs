//! Contratos JSON (DTOs) do módulo de Centros Acadêmicos.
//!
//! Define as estruturas de requisição e resposta para configuração inicial
//! e consulta do mandato de gestão do Centro Acadêmico.

use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use crate::modules::ca::domain::entities::Mandate;
use crate::modules::ca::domain::entities::MandateMember;
use crate::modules::ca::domain::entities::PollOptionResult;
use crate::modules::ca::domain::entities::PollResults;
use crate::modules::ca::domain::entities::PollVote;
use crate::modules::ca::domain::entities::StudentPoll;
/// Corpo da requisição de configuração inicial do mandato da gestão.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureInitialMandateRequest {
    /// Data de início do mandato da gestão (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do mandato da gestão (AAAA-MM-DD), estritamente posterior ao início.
    pub end_date: NaiveDate,
    /// Título opcional representativo da chapa ou gestão.
    #[serde(default)]
    pub title: Option<String>,
}
/// Corpo da requisição para transição de gestão com preservação do histórico.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransitionMandateRequest {
    /// Data de início do novo mandato (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do novo mandato (AAAA-MM-DD).
    pub end_date: NaiveDate,
    /// Título opcional representativo da nova chapa ou gestão.
    #[serde(default)]
    pub title: Option<String>,
}
/// Corpo da requisição para vincular um integrante à gestão.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddMandateMemberRequest {
    /// Nome do integrante.
    pub name: String,
    /// Cargo exercido na gestão.
    pub role: String,
    /// Identificador opcional da conta de usuário correspondente no sistema.
    #[serde(default)]
    pub user_id: Option<Uuid>,
}
/// Resposta pública representando um integrante da gestão e sua participação histórica.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MandateMemberResponse {
    /// Identificador único do vínculo do integrante.
    pub id: Uuid,
    /// Identificador opcional da conta de usuário no aplicativo.
    pub user_id: Option<Uuid>,
    /// Nome do integrante.
    pub name: String,
    /// Cargo exercido na gestão.
    pub role: String,
    /// Indica se o vínculo do integrante permanece ativo na gestão.
    pub is_active: bool,
    /// Instante UTC de início da vinculação.
    pub joined_at: DateTime<Utc>,
    /// Instante UTC de encerramento do vínculo, se aplicável.
    pub left_at: Option<DateTime<Utc>>,
}
impl From<MandateMember> for MandateMemberResponse {
    fn from(member: MandateMember) -> Self {
        Self {
            id: member.id(),
            user_id: member.user_id(),
            name: member.name().to_string(),
            role: member.role().to_string(),
            is_active: member.is_active(),
            joined_at: member.joined_at(),
            left_at: member.left_at(),
        }
    }
}
/// Resposta pública contendo as informações, duração e integrantes do mandato de gestão.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MandateResponse {
    /// Identificador único do mandato.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico correspondente.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que registrou a configuração inicial do mandato.
    pub created_by_id: Uuid,
    /// Título ou nome da chapa/gestão, se informado.
    pub title: Option<String>,
    /// Data de início do período do mandato (AAAA-MM-DD).
    pub start_date: NaiveDate,
    /// Data de término do período do mandato (AAAA-MM-DD).
    pub end_date: NaiveDate,
    /// Alias para a data de início (AAAA-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<NaiveDate>,
    /// Alias para a data de término (AAAA-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<NaiveDate>,
    /// Situação operacional do mandato ('active', 'concluded', 'archived').
    pub status: String,
    /// Instante UTC de encerramento/conclusão da gestão, se houver.
    pub concluded_at: Option<DateTime<Utc>>,
    /// Duração do mandato em dias calculada a partir das datas persistidas.
    pub duration_in_days: i64,
    /// Composição de integrantes e cargos associados ao mandato.
    #[serde(default)]
    pub members: Vec<MandateMemberResponse>,
    /// Instante UTC de criação do registro.
    pub created_at: DateTime<Utc>,
    /// Instante UTC da última modificação do registro.
    pub updated_at: DateTime<Utc>,
}
impl From<Mandate> for MandateResponse {
    fn from(mandate: Mandate) -> Self {
        let start = mandate.start_date();
        let end = mandate.end_date();
        let members = mandate
            .members()
            .iter()
            .cloned()
            .map(MandateMemberResponse::from)
            .collect();

        Self {
            id: mandate.id(),
            academic_center_id: mandate.academic_center_id(),
            created_by_id: mandate.created_by_id(),
            title: mandate.title().map(String::from),
            start_date: start,
            end_date: end,
            starts_at: Some(start),
            ends_at: Some(end),
            status: mandate.status().as_str().to_string(),
            concluded_at: mandate.concluded_at(),
            duration_in_days: mandate.duration_in_days(),
            members,
            created_at: mandate.created_at(),
            updated_at: mandate.updated_at(),
        }
    }
}
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
