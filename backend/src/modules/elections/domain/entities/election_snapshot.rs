//! Dados de entrada para reconstituição validada do agregado eleitoral.
//!
//! Este snapshot não define um contrato público de serialização.
//! Adaptadores de persistência devem convertê-lo a partir de seus registros.

use chrono::{DateTime, Utc};

use crate::modules::elections::domain::{
    entities::{
        closure::ElectionClosure,
        configuration::ElectionConfiguration,
        election::{ElectionStatus, TallyStatus},
        tally::VerifiedTally,
    },
    value_objects::{
        calendar_origin::CalendarOrigin,
        election_period::ElectionPeriod,
        fingerprint::Fingerprint,
        identifiers::{AcademicCenterId, ActorId, ElectionId},
        modality::ElectionModality,
        schedule_change::ScheduleChange,
        voter_count::VoterCount,
    },
};

/// Revisão de calendário e hash resultante armazenados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedScheduleChangeSnapshot {
    /// Registro imutável da alteração aplicada.
    pub change: ScheduleChange,
    /// Hash do calendário produzido pela alteração.
    pub resulting_schedule_hash: Fingerprint,
}

/// Dados persistidos de um cancelamento eleitoral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionCancellationSnapshot {
    /// Identificador da eleição cancelada.
    pub election_id: ElectionId,
    /// Hash do manifesto, se publicado.
    pub manifest_hash: Option<Fingerprint>,
    /// Hash do calendário vigente, se disponível.
    pub schedule_hash: Option<Fingerprint>,
    /// Hash da cabeça do quadro de votos, se disponível.
    pub head_hash: Option<Fingerprint>,
    /// Quantidade de eleitores registrada no cancelamento.
    pub voter_count: VoterCount,
    /// Identificador de quem solicitou o cancelamento.
    pub requested_by: ActorId,
    /// Instante em que o cancelamento foi registrado.
    pub cancelled_at: DateTime<Utc>,
    /// Justificativa do cancelamento.
    pub reason: String,
}

/// Estado persistido necessário para reconstituir uma eleição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionSnapshot {
    /// Identificador da eleição.
    pub id: ElectionId,
    /// Centro Acadêmico responsável pela eleição.
    pub academic_center_id: AcademicCenterId,
    /// Modalidade escolhida na criação.
    pub modality: ElectionModality,
    /// Configuração pronta, quando existente.
    pub configuration: Option<ElectionConfiguration>,
    /// Estado operacional persistido.
    pub status: ElectionStatus,
    /// Situação persistida da apuração.
    pub tally_status: TallyStatus,
    /// Calendário original imutável.
    pub calendar_origin: CalendarOrigin,
    /// Hash do calendário original.
    pub calendar_origin_hash: Fingerprint,
    /// Período atualmente vigente.
    pub current_period: ElectionPeriod,
    /// Instante de abertura, quando a votação foi aberta.
    pub opened_at: Option<DateTime<Utc>>,
    /// Revisão vigente quando a votação foi aberta; separa preparação de prorrogação.
    pub opening_schedule_revision: Option<u32>,
    /// Hash do calendário atualmente vigente.
    pub current_schedule_hash: Fingerprint,
    /// Número da última revisão aplicada.
    pub schedule_revision: u32,
    /// Histórico ordenado das revisões e respectivos hashes.
    pub schedule_changes: Vec<AppliedScheduleChangeSnapshot>,
    /// Registro de cancelamento, quando existente.
    pub cancellation: Option<ElectionCancellationSnapshot>,
    /// Registro de encerramento, quando existente.
    pub closure: Option<ElectionClosure>,
    /// Digest do encerramento conferido pelo codec, independente do resultado.
    pub closure_hash: Option<Fingerprint>,
    /// Último resultado de apuração publicado, quando existente.
    pub tally_result: Option<VerifiedTally>,
}

/// Erros encontrados ao reconstituir um agregado eleitoral persistido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionReconstitutionError {
    /// A modalidade diverge da configuração pública persistida.
    #[error("Configuração incompatível com a modalidade")]
    InvalidConfiguration,

    /// A origem do calendário pertence a outra eleição.
    #[error("A origem do calendário pertence a outra eleição")]
    CalendarOriginElectionMismatch,

    /// O manifesto não corresponde à eleição, ao centro ou ao calendário.
    #[error("O manifesto não corresponde aos dados da eleição")]
    ManifestReferenceMismatch,

    /// A revisão persistida não corresponde ao tamanho do histórico.
    #[error("A revisão atual diverge do histórico de alterações")]
    ScheduleRevisionMismatch,

    /// A cadeia de hashes do calendário está inconsistente.
    #[error("A cadeia de hashes do calendário é inconsistente")]
    ScheduleHashChainMismatch,

    /// O histórico contém alteração inválida.
    #[error("O histórico de alterações do calendário é inválido")]
    InvalidScheduleHistory,

    /// O período vigente diverge do histórico.
    #[error("O período vigente diverge do histórico de calendário")]
    CurrentPeriodMismatch,

    /// O instante de abertura é incompatível com o estado ou calendário.
    #[error("O instante de abertura é incompatível com o estado eleitoral")]
    InvalidOpening,

    /// Os registros de fechamento são incompatíveis com a eleição.
    #[error("O registro de fechamento é incompatível com a eleição")]
    InvalidClosure,

    /// Os registros de cancelamento são incompatíveis com a eleição.
    #[error("O registro de cancelamento é incompatível com a eleição")]
    InvalidCancellation,

    /// O estado operacional não corresponde aos registros persistidos.
    #[error("O estado operacional diverge dos registros persistidos")]
    InvalidLifecycleState,

    /// O estado da apuração não corresponde ao resultado persistido.
    #[error("O estado da apuração diverge do resultado persistido")]
    InvalidTallyState,

    /// O resultado não satisfaz as invariantes do agregado.
    #[error("O resultado persistido viola as invariantes da eleição")]
    InvalidTallyResult,
}
