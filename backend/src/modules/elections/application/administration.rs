//! Parâmetros tipados da administração de uma eleição pela sua comissão.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::modules::elections::domain::{
    entities::election::ElectionPeriod, identifiers::ElectionId,
};

/// Cancelamento decidido pela comissão e publicado com a eleição.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardCancellation {
    /// Instante da decisão no relógio do banco.
    pub cancelled_at: DateTime<Utc>,
    /// UUID da conta CApp designada para representar a comissão.
    pub cancelled_by: uuid::Uuid,
    /// Motivo informado pela comissão.
    pub reason: String,
}

/// Dados operacionais lidos sob bloqueio antes de alterar uma eleição.
pub struct AdministrationState {
    /// Eleição do quadro.
    pub election: ElectionId,
    /// Período vigente da preparação ou do quadro publicado.
    pub period: ElectionPeriod,
    /// Instante obtido após adquirir o bloqueio.
    pub now: DateTime<Utc>,
    /// Indica se o quadro já foi publicado.
    pub published: bool,
    /// Indica fechamento final confirmado.
    pub closed: bool,
    /// Cancelamento existente, em caso de repetição da consulta.
    pub cancellation: Option<BoardCancellation>,
}

/// Datas da eleição devolvidas ao concluir uma alteração de período.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedPeriod {
    /// Início vigente confirmado.
    pub starts_at: DateTime<Utc>,
    /// Encerramento vigente confirmado.
    pub ends_at: DateTime<Utc>,
}
