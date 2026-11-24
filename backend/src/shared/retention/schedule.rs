//! Regras de cálculo de prazos de retenção, expiração e agendamento de descarte.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::categories::{DataCategory, DisposalMethod, LegalBasis, SensitivityLevel};

/// Duração temporal ou condição necessária para a expiração e o descarte de dados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionPeriod {
    /// Quantidade de dias corridos a partir da data de criação ou homologação.
    Days(i64),

    /// Quantidade de horas a partir da emissão (ex.: access tokens de vida curta).
    Hours(i64),

    /// Durante a vigência do vínculo acadêmico ativo na UnB, acrescido de dias de tolerância recursal.
    ActivePlusToleranceDays(i64),

    /// Preservação perene (ex.: ata final consolidada com apuração e totalizadores de eleição).
    Permanent,
}

/// Orquestrador de prazos, sensibilidade e políticas de expiração de dados do CApp.
pub struct RetentionSchedule;

impl RetentionSchedule {
    /// Determina o período de retenção regulamentar para a categoria informada.
    pub const fn period_for(category: &DataCategory) -> RetentionPeriod {
        match category {
            DataCategory::StudentPersonalData => RetentionPeriod::ActivePlusToleranceDays(180),
            DataCategory::ManagementMandate => RetentionPeriod::Days(1826), // 5 anos (1826 dias)
            DataCategory::FinancialAccountability => RetentionPeriod::Days(1826), // 5 anos
            DataCategory::ElectoralTurnout => RetentionPeriod::Days(180),
            DataCategory::ElectoralBallot => RetentionPeriod::Days(90),
            DataCategory::ElectoralAudit => RetentionPeriod::Days(30),
            DataCategory::SessionAndToken => RetentionPeriod::Days(30),
            DataCategory::AuditLog => RetentionPeriod::Days(180), // 6 meses (Marco Civil Art. 15)
            DataCategory::Telemetry => RetentionPeriod::Days(30),
        }
    }

    /// Determina o nível de sensibilidade associado à categoria de dado.
    pub const fn sensitivity_for(category: &DataCategory) -> SensitivityLevel {
        match category {
            DataCategory::StudentPersonalData => SensitivityLevel::Restricted,
            DataCategory::ManagementMandate => SensitivityLevel::Internal,
            DataCategory::FinancialAccountability => SensitivityLevel::Restricted,
            DataCategory::ElectoralTurnout => SensitivityLevel::Restricted,
            DataCategory::ElectoralBallot => SensitivityLevel::CriticalSensitive,
            DataCategory::ElectoralAudit => SensitivityLevel::CriticalSensitive,
            DataCategory::SessionAndToken => SensitivityLevel::CriticalSensitive,
            DataCategory::AuditLog => SensitivityLevel::Restricted,
            DataCategory::Telemetry => SensitivityLevel::Internal,
        }
    }

    /// Retorna a base legal que justifica o tratamento e a retenção da categoria.
    pub const fn legal_basis_for(category: &DataCategory) -> LegalBasis {
        match category {
            DataCategory::StudentPersonalData => LegalBasis::ContractExecution,
            DataCategory::ManagementMandate => LegalBasis::RegularExerciseOfRights,
            DataCategory::FinancialAccountability => LegalBasis::RegularExerciseOfRights,
            DataCategory::ElectoralTurnout => LegalBasis::ContractExecution,
            DataCategory::ElectoralBallot => LegalBasis::ElectoralStatutoryDuty,
            DataCategory::ElectoralAudit => LegalBasis::RegularExerciseOfRights,
            DataCategory::SessionAndToken => LegalBasis::ContractExecution,
            DataCategory::AuditLog => LegalBasis::LegalOrRegulatoryObligation,
            DataCategory::Telemetry => LegalBasis::LegitimateInterest,
        }
    }

    /// Retorna o método de descarte seguro recomendado para a categoria.
    pub const fn disposal_method_for(category: &DataCategory) -> DisposalMethod {
        match category {
            DataCategory::StudentPersonalData => DisposalMethod::IrreversibleAnonymization,
            DataCategory::ManagementMandate => DisposalMethod::IrreversibleAnonymization,
            DataCategory::FinancialAccountability => DisposalMethod::IrreversibleAnonymization,
            DataCategory::ElectoralTurnout => DisposalMethod::PhysicalPurge,
            DataCategory::ElectoralBallot => DisposalMethod::CryptoShredding,
            DataCategory::ElectoralAudit => DisposalMethod::PhysicalPurge,
            DataCategory::SessionAndToken => DisposalMethod::PhysicalPurge,
            DataCategory::AuditLog => DisposalMethod::PhysicalPurge,
            DataCategory::Telemetry => DisposalMethod::PhysicalPurge,
        }
    }

    /// Calcula o instante exato de expiração de um registro a partir da data-base de referência.
    ///
    /// Para categorias com retenção permanente, retorna `None`.
    pub fn calculate_expiration(
        category: &DataCategory,
        base_date: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        match Self::period_for(category) {
            RetentionPeriod::Days(days) => Some(base_date + Duration::days(days)),
            RetentionPeriod::Hours(hours) => Some(base_date + Duration::hours(hours)),
            RetentionPeriod::ActivePlusToleranceDays(tolerance_days) => {
                Some(base_date + Duration::days(tolerance_days))
            }
            RetentionPeriod::Permanent => None,
        }
    }

    /// Verifica se um registro associado à data-base já ultrapassou o período regulamentar de retenção.
    pub fn is_expired(
        category: &DataCategory,
        base_date: DateTime<Utc>,
        current_time: DateTime<Utc>,
    ) -> bool {
        match Self::calculate_expiration(category, base_date) {
            Some(expiration_date) => current_time >= expiration_date,
            None => false,
        }
    }
}
