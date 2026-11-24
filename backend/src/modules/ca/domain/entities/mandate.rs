//! Entidade de domínio representando o mandato de gestão de um Centro Acadêmico.
//!
//! O mandato define o período cronológico e o ciclo de vida da administração
//! do Centro Acadêmico, servindo de base para a atuação da gestão e para
//! transições eleitorais.

use std::{error::Error, fmt, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::modules::ca::domain::rules::{
    validate_mandate_dates, validate_mandate_status_transition, validate_mandate_timestamps,
    validate_mandate_title,
};

/// Erros decorrentes de violações de regras de negócio ou validação de mandato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateError {
    /// O título da gestão/mandato não pode ser vazio quando fornecido.
    EmptyTitle,

    /// O título da gestão ultrapassou o comprimento máximo permitido.
    TitleTooLong {
        /// Limite máximo de caracteres.
        max: usize,
    },

    /// As datas de mandato violam a coerência cronológica (`end_date <= start_date`).
    InvalidDateRange {
        /// Data de início informada.
        start_date: NaiveDate,
        /// Data de término informada.
        end_date: NaiveDate,
    },

    /// O status informado não é reconhecido pelo domínio.
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada não é permitida.
    InvalidStatusTransition {
        /// Status atual do mandato.
        from: MandateStatus,
        /// Status pretendido.
        to: MandateStatus,
    },

    /// Os timestamps de criação e atualização violam a coerência temporal.
    InconsistentTimestamps {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de atualização.
        updated_at: DateTime<Utc>,
    },
}

impl fmt::Display for MandateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => {
                write!(f, "o título da gestão não pode ser vazio quando informado")
            }
            Self::TitleTooLong { max } => {
                write!(f, "o título da gestão excede o limite de {max} caracteres")
            }
            Self::InvalidDateRange {
                start_date,
                end_date,
            } => {
                write!(
                    f,
                    "a data de término ({end_date}) deve ser estritamente posterior à data de início ({start_date})"
                )
            }
            Self::InvalidStatus(status) => {
                write!(f, "status de mandato inválido ou desconhecido: '{status}'")
            }
            Self::InvalidStatusTransition { from, to } => {
                write!(f, "transição de status inválida de '{from}' para '{to}'")
            }
            Self::InconsistentTimestamps {
                created_at,
                updated_at,
            } => {
                write!(
                    f,
                    "data de atualização ({updated_at}) não pode ser anterior à data de criação ({created_at})"
                )
            }
        }
    }
}

impl Error for MandateError {}

/// Situação do mandato no ciclo de vida da gestão do Centro Acadêmico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MandateStatus {
    /// Mandato atualmente em exercício e referência ativa do CA.
    Active,
    /// Mandato regularmente finalizado após o encerramento do período ou posse subsequente.
    Concluded,
    /// Mandato arquivado para retenção histórica.
    Archived,
}

impl MandateStatus {
    /// Retorna a representação textual canônica do status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Concluded => "concluded",
            Self::Archived => "archived",
        }
    }
}

impl fmt::Display for MandateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MandateStatus {
    type Err = MandateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "concluded" => Ok(Self::Concluded),
            "archived" => Ok(Self::Archived),
            other => Err(MandateError::InvalidStatus(other.to_string())),
        }
    }
}

/// Entidade de domínio que representa o mandato de gestão de um Centro Acadêmico.
///
/// Mantém as invariantes de consistência temporal, titularidade e ciclo de vida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mandate {
    /// Identificador único do mandato.
    id: Uuid,
    /// Identificador do Centro Acadêmico ao qual o mandato pertence.
    academic_center_id: Uuid,
    /// Identificador do usuário que registrou a configuração inicial do mandato.
    created_by_id: Uuid,
    /// Título opcional representativo da chapa ou gestão (ex.: "Gestão 2026-2027").
    title: Option<String>,
    /// Data de início do período do mandato.
    start_date: NaiveDate,
    /// Data de término do período do mandato (`end_date > start_date`).
    end_date: NaiveDate,
    /// Situação operacional do mandato no ciclo de vida.
    status: MandateStatus,
    /// Instante UTC de registro da configuração do mandato.
    created_at: DateTime<Utc>,
    /// Instante UTC da última modificação do registro.
    updated_at: DateTime<Utc>,
}

impl Mandate {
    /// Cria uma nova entidade de mandato em estado ativo para configuração inicial.
    ///
    /// # Erros
    ///
    /// Retorna [`MandateError`] se `end_date <= start_date` ou se o título for inválido.
    pub fn new(
        academic_center_id: Uuid,
        created_by_id: Uuid,
        title: Option<&str>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Self, MandateError> {
        let validated_title = validate_mandate_title(title)?;
        validate_mandate_dates(start_date, end_date)?;

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            academic_center_id,
            created_by_id,
            title: validated_title,
            start_date,
            end_date,
            status: MandateStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstrói uma entidade de mandato persistida, validando todas as invariantes.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        academic_center_id: Uuid,
        created_by_id: Uuid,
        title: Option<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        status: MandateStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, MandateError> {
        let validated_title = validate_mandate_title(title.as_deref())?;
        validate_mandate_dates(start_date, end_date)?;
        validate_mandate_timestamps(created_at, updated_at)?;

        Ok(Self {
            id,
            academic_center_id,
            created_by_id,
            title: validated_title,
            start_date,
            end_date,
            status,
            created_at,
            updated_at,
        })
    }

    /// Retorna o identificador único do mandato.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador do Centro Acadêmico correspondente.
    pub fn academic_center_id(&self) -> Uuid {
        self.academic_center_id
    }

    /// Retorna o identificador do usuário responsável pela configuração do mandato.
    pub fn created_by_id(&self) -> Uuid {
        self.created_by_id
    }

    /// Retorna o título opcional da gestão/chapa.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Retorna a data de início do mandato.
    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    /// Retorna a data de término do mandato.
    pub fn end_date(&self) -> NaiveDate {
        self.end_date
    }

    /// Retorna o status atual do mandato.
    pub fn status(&self) -> MandateStatus {
        self.status
    }

    /// Retorna se o mandato está atualmente ativo.
    pub fn is_active(&self) -> bool {
        self.status == MandateStatus::Active
    }

    /// Calcula a duração total do mandato em dias correspondente às datas persistidas.
    pub fn duration_in_days(&self) -> i64 {
        (self.end_date - self.start_date).num_days()
    }

    /// Retorna o instante UTC de criação.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Retorna o instante UTC da última modificação.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Transiciona o mandato para concluído.
    pub fn conclude(&mut self, now: DateTime<Utc>) -> Result<(), MandateError> {
        validate_mandate_status_transition(self.status, MandateStatus::Concluded)?;
        validate_mandate_timestamps(self.created_at, now)?;
        self.status = MandateStatus::Concluded;
        self.updated_at = now;
        Ok(())
    }

    /// Transiciona o mandato para arquivado.
    pub fn archive(&mut self, now: DateTime<Utc>) -> Result<(), MandateError> {
        validate_mandate_status_transition(self.status, MandateStatus::Archived)?;
        validate_mandate_timestamps(self.created_at, now)?;
        self.status = MandateStatus::Archived;
        self.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_mandate_with_active_status() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

        let mandate =
            Mandate::new(ca_id, user_id, Some("Gestão 2026"), start, end).expect("valid mandate");

        assert_eq!(mandate.academic_center_id(), ca_id);
        assert_eq!(mandate.created_by_id(), user_id);
        assert_eq!(mandate.title(), Some("Gestão 2026"));
        assert_eq!(mandate.start_date(), start);
        assert_eq!(mandate.end_date(), end);
        assert_eq!(mandate.status(), MandateStatus::Active);
        assert!(mandate.is_active());
        assert_eq!(mandate.duration_in_days(), 364);
    }

    #[test]
    fn should_create_valid_mandate_without_title() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2027, 1, 31).unwrap();

        let mandate = Mandate::new(ca_id, user_id, None, start, end).expect("valid mandate");

        assert_eq!(mandate.title(), None);
        assert_eq!(mandate.duration_in_days(), 364);
    }

    #[test]
    fn should_fail_creation_when_end_date_is_not_strictly_after_start_date() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let err = Mandate::new(ca_id, user_id, None, date, date).unwrap_err();
        assert_eq!(
            err,
            MandateError::InvalidDateRange {
                start_date: date,
                end_date: date,
            }
        );
    }

    #[test]
    fn should_reconstitute_persisted_mandate() {
        let id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let created_at = Utc::now();
        let updated_at = created_at;

        let mandate = Mandate::reconstitute(
            id,
            ca_id,
            user_id,
            Some("Gestão Anterior".into()),
            start,
            end,
            MandateStatus::Concluded,
            created_at,
            updated_at,
        )
        .expect("valid reconstitution");

        assert_eq!(mandate.id(), id);
        assert_eq!(mandate.status(), MandateStatus::Concluded);
        assert!(!mandate.is_active());
    }

    #[test]
    fn should_fail_reconstitution_when_updated_at_is_before_created_at() {
        let id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let now = Utc::now();
        let before = now - chrono::Duration::hours(1);

        let err = Mandate::reconstitute(
            id,
            ca_id,
            user_id,
            None,
            start,
            end,
            MandateStatus::Active,
            now,
            before,
        )
        .unwrap_err();

        assert_eq!(
            err,
            MandateError::InconsistentTimestamps {
                created_at: now,
                updated_at: before,
            }
        );
    }

    #[test]
    fn should_conclude_and_archive_mandate() {
        let ca_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

        let mut mandate = Mandate::new(ca_id, user_id, None, start, end).unwrap();
        let now = Utc::now();

        mandate.conclude(now).expect("should conclude");
        assert_eq!(mandate.status(), MandateStatus::Concluded);

        let later = now + chrono::Duration::hours(1);
        mandate.archive(later).expect("should archive");
        assert_eq!(mandate.status(), MandateStatus::Archived);
    }

    #[test]
    fn should_parse_and_format_status() {
        assert_eq!(
            MandateStatus::from_str("active").unwrap(),
            MandateStatus::Active
        );
        assert_eq!(
            MandateStatus::from_str("concluded").unwrap(),
            MandateStatus::Concluded
        );
        assert_eq!(
            MandateStatus::from_str("archived").unwrap(),
            MandateStatus::Archived
        );
        assert!(MandateStatus::from_str("invalid").is_err());
        assert_eq!(MandateStatus::Active.to_string(), "active");
    }

    #[test]
    fn should_implement_std_error_trait_for_mandate_error() {
        let err: Box<dyn Error> = Box::new(MandateError::EmptyTitle);
        assert!(err.to_string().contains("não pode ser vazio"));
    }
}
