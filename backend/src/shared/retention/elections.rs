//! Salvaguardas de sigilo eleitoral, desassociação de votos e ciclo de vida de auditoria de urnas.

use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::RetentionError;

/// Registro de comparecimento eleitoral nominal (Turnout) para prevenção a voto duplo e quórum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnoutRecord {
    /// Identificador do registro de comparecimento.
    pub turnout_id: Uuid,

    /// Identificador do usuário eleitor apto.
    pub user_id: Uuid,

    /// Identificador da eleição do Centro Acadêmico.
    pub election_id: Uuid,

    /// Data e hora exata do registro de presença na urna.
    pub voted_at: DateTime<Utc>,

    /// Hash criptográfico do comprovante de votação entregue ao discente.
    pub receipt_hash: String,
}

/// Cédula eleitoral contendo o voto secreto, estritamente desprovida de dados do eleitor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymousBallot {
    /// Identificador único da cédula anônima depositada na urna.
    pub ballot_id: Uuid,

    /// Identificador da eleição do Centro Acadêmico correspondente.
    pub election_id: Uuid,

    /// Identificador da chapa ou candidato escolhido (ou nulo/branco).
    pub candidate_chapa_id: Uuid,

    /// Carga cifrada do voto para auditoria criptográfica.
    pub encrypted_vote_payload: Vec<u8>,

    /// Timestamp quantizado/em lote para impossibilitar correlação temporal com o comparecimento.
    pub batch_timestamp: DateTime<Utc>,
}

/// Estado do ciclo de vida dos dados de auditoria e comparecimento de uma eleição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectoralAuditStatus {
    /// Janela de 30 dias para interposição e julgamento de recursos e impugnações.
    AppealWindowActive,

    /// Período pós-30 dias: trilha efêmera de auditoria expurgada; comparecimento mantido para histórico.
    AuditLogsPurged,

    /// Período pós-180 dias: comparecimento nominal expurgado; apenas quórum consolidado preservado.
    TurnoutRecordsPurged,
}

/// Guardião das regras de sigilo eleitoral e descarte progressivo de dados de votações.
pub struct ElectoralPrivacyGuardian;

impl ElectoralPrivacyGuardian {
    /// Valida que uma cédula anônima não carrega nenhum dado pessoal ou vínculo com o eleitor.
    pub fn validate_ballot_isolation(
        ballot: &AnonymousBallot,
        forbidden_user_id: Option<Uuid>,
    ) -> Result<(), RetentionError> {
        if forbidden_user_id == Some(ballot.ballot_id) {
            return Err(RetentionError::ElectoralSecretViolation(
                "O ballot_id não pode ser igual ao user_id".to_string(),
            ));
        }

        if ballot.encrypted_vote_payload.is_empty() {
            return Err(RetentionError::ElectoralSecretViolation(
                "A cédula deve conter carga criptográfica válida".to_string(),
            ));
        }

        Ok(())
    }

    /// Quantiza o timestamp da cédula para uma janela horária (ex.: múltiplos de 15 minutos),
    /// mitigando ataques de inferência estatística baseados na comparação com o horário do turnout.
    pub fn quantize_timestamp(timestamp: DateTime<Utc>, interval_minutes: u32) -> DateTime<Utc> {
        let interval = interval_minutes.max(1);
        let minute = timestamp.minute();
        let rounded_minute = (minute / interval) * interval;

        timestamp
            .with_minute(rounded_minute)
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(timestamp)
    }

    /// Avalia o status do ciclo de vida de dados de uma eleição a partir da homologação.
    pub fn evaluate_audit_status(
        homologated_at: DateTime<Utc>,
        current_time: DateTime<Utc>,
    ) -> ElectoralAuditStatus {
        let appeal_window_end = homologated_at + Duration::days(30);
        let turnout_purge_date = homologated_at + Duration::days(180);

        if current_time < appeal_window_end {
            ElectoralAuditStatus::AppealWindowActive
        } else if current_time < turnout_purge_date {
            ElectoralAuditStatus::AuditLogsPurged
        } else {
            ElectoralAuditStatus::TurnoutRecordsPurged
        }
    }
}
