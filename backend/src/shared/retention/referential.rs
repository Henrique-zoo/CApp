//! Garantia de integridade referencial de chaves estrangeiras e orquestração de descarte.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::RetentionError;

/// Tipos de salvaguarda legal e estatutária que impedem a exclusão física direta de um usuário.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionHold {
    /// Mandato estatutário de diretoria de Centro Acadêmico sujeito a comprovação por 5 anos.
    ManagementMandate {
        /// Identificador da gestão (`management_term`).
        term_id: Uuid,
        /// Data oficial de encerramento do mandato.
        term_ended_at: DateTime<Utc>,
    },

    /// Relatório financeiro e prestação de contas sujeito a guarda legal por 5 anos.
    FinancialReport {
        /// Identificador da prestação de contas (`financial_report`).
        report_id: Uuid,
        /// Data de aprovação formal pela assembleia de estudantes.
        approved_at: DateTime<Utc>,
    },
}

impl RetentionHold {
    /// Verifica se a salvaguarda de retenção ainda se encontra ativa na data indicada.
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        let retention_years = Duration::days(1826); // 5 anos regulamentares
        match self {
            Self::ManagementMandate { term_ended_at, .. } => {
                now < (*term_ended_at + retention_years)
            }
            Self::FinancialReport { approved_at, .. } => now < (*approved_at + retention_years),
        }
    }
}

/// Estratégia de descarte determinada pelo guardião de integridade referencial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisposalStrategy {
    /// Anonimização irreversível: preserva a chave estrangeira (`user_id`) e redige todos os dados pessoais.
    AnonymizeInPlace,

    /// Expurgo físico completo: exclusão física permitida por ausência de dependências legais ativas.
    PhysicalPurge,
}

/// Guardião da integridade referencial e da resolução de conflitos entre exclusão e histórico estatutário.
pub struct ReferentialIntegrityGuardian;

impl ReferentialIntegrityGuardian {
    /// Avalia a estratégia correta de descarte para um discente com base nas suas salvaguardas históricas.
    pub fn evaluate_user_disposal(
        holds: &[RetentionHold],
        current_time: DateTime<Utc>,
    ) -> DisposalStrategy {
        let has_active_holds = holds.iter().any(|hold| hold.is_active(current_time));

        if has_active_holds {
            DisposalStrategy::AnonymizeInPlace
        } else {
            DisposalStrategy::PhysicalPurge
        }
    }

    /// Tenta autorizar a exclusão física direta de um usuário, bloqueando caso existam salvaguardas ativas.
    pub fn authorize_physical_purge(
        user_id: Uuid,
        holds: &[RetentionHold],
        current_time: DateTime<Utc>,
    ) -> Result<(), RetentionError> {
        let active_holds = holds
            .iter()
            .filter(|hold| hold.is_active(current_time))
            .count();

        if active_holds > 0 {
            return Err(RetentionError::ReferentialIntegrityBlocked(format!(
                "Usuário {user_id} possui {active_holds} salvaguarda(s) ativa(s) de prestação de contas/mandato (guarda obrigatória de 5 anos)"
            )));
        }

        Ok(())
    }
}
