//! Regras de negócio e validação de mandatos de gestão do Centro Acadêmico.
//!
//! Centraliza as invariantes de consistência temporal, limites de título,
//! transições de estado e coerência entre criação e atualização.

use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};

use crate::modules::ca::domain::entities::{MandateError, MandateStatus};

/// Comprimento máximo permitido para o título descritivo do mandato/gestão.
pub const MAX_MANDATE_TITLE_LENGTH: usize = 120;

/// Comprimento máximo permitido para o nome de um integrante da gestão.
pub const MAX_MEMBER_NAME_LENGTH: usize = 150;

/// Comprimento máximo permitido para o cargo de um integrante da gestão.
pub const MAX_MEMBER_ROLE_LENGTH: usize = 100;

/// Valida o nome de um integrante da gestão.
pub fn validate_member_name(name: &str) -> Result<String, MandateError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(MandateError::EmptyMemberName);
    }
    if trimmed.chars().count() > MAX_MEMBER_NAME_LENGTH {
        return Err(MandateError::MemberNameTooLong {
            max: MAX_MEMBER_NAME_LENGTH,
        });
    }
    Ok(trimmed.to_string())
}

/// Valida o cargo de um integrante da gestão.
pub fn validate_member_role(role: &str) -> Result<String, MandateError> {
    let trimmed = role.trim();
    if trimmed.is_empty() {
        return Err(MandateError::EmptyMemberRole);
    }
    if trimmed.chars().count() > MAX_MEMBER_ROLE_LENGTH {
        return Err(MandateError::MemberRoleTooLong {
            max: MAX_MEMBER_ROLE_LENGTH,
        });
    }
    Ok(trimmed.to_string())
}

/// Valida a coerência cronológica das datas de vinculação e desvinculação de um integrante.
pub fn validate_member_dates(
    joined_at: DateTime<Utc>,
    left_at: Option<DateTime<Utc>>,
) -> Result<(), MandateError> {
    if let Some(left) = left_at.filter(|&left| left < joined_at) {
        return Err(MandateError::InconsistentMemberDates {
            joined_at,
            left_at: left,
        });
    }
    Ok(())
}

/// Valida o título descritivo opcional da chapa ou gestão.
///
/// Espaços nas extremidades são removidos. Caso o texto resultante seja vazio,
/// retorna `Err(MandateError::EmptyTitle)`. Exceder [`MAX_MANDATE_TITLE_LENGTH`]
/// retorna `Err(MandateError::TitleTooLong)`.
pub fn validate_mandate_title(title: Option<&str>) -> Result<Option<String>, MandateError> {
    match title {
        None => Ok(None),
        Some(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                return Err(MandateError::EmptyTitle);
            }
            if trimmed.chars().count() > MAX_MANDATE_TITLE_LENGTH {
                return Err(MandateError::TitleTooLong {
                    max: MAX_MANDATE_TITLE_LENGTH,
                });
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// Valida a consistência temporal das datas de início e término do mandato.
///
/// Invariante estrita: a data de término deve ser estritamente posterior à
/// data de início (`end_date > start_date`).
pub fn validate_mandate_dates(
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(), MandateError> {
    if end_date <= start_date {
        return Err(MandateError::InvalidDateRange {
            start_date,
            end_date,
        });
    }
    Ok(())
}

/// Valida a ordem cronológica entre os timestamps de criação e atualização.
pub fn validate_mandate_timestamps(
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), MandateError> {
    if updated_at < created_at {
        return Err(MandateError::InconsistentTimestamps {
            created_at,
            updated_at,
        });
    }
    Ok(())
}

/// Valida se a transição entre estados do mandato é permitida pelo ciclo de vida.
///
/// - `Active` pode transicionar para `Concluded` ou `Archived`.
/// - `Concluded` pode transicionar para `Archived`.
/// - `Archived` é terminal e não permite transição para outro estado.
pub fn validate_mandate_status_transition(
    from: MandateStatus,
    to: MandateStatus,
) -> Result<(), MandateError> {
    if from == to {
        return Ok(());
    }

    match (from, to) {
        (MandateStatus::Active, MandateStatus::Concluded | MandateStatus::Archived) => Ok(()),
        (MandateStatus::Concluded, MandateStatus::Archived) => Ok(()),
        _ => Err(MandateError::InvalidStatusTransition { from, to }),
    }
}

/// Valida e converte uma string para [`MandateStatus`].
pub fn validate_mandate_status(status_str: &str) -> Result<MandateStatus, MandateError> {
    MandateStatus::from_str(status_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_dates_when_end_is_strictly_after_start() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert!(validate_mandate_dates(start, end).is_ok());
    }

    #[test]
    fn should_fail_validation_when_end_date_is_equal_to_start_date() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let err = validate_mandate_dates(date, date).unwrap_err();
        assert_eq!(
            err,
            MandateError::InvalidDateRange {
                start_date: date,
                end_date: date,
            }
        );
    }

    #[test]
    fn should_fail_validation_when_end_date_is_before_start_date() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let err = validate_mandate_dates(start, end).unwrap_err();
        assert_eq!(
            err,
            MandateError::InvalidDateRange {
                start_date: start,
                end_date: end,
            }
        );
    }

    #[test]
    fn should_trim_and_validate_title_correctly() {
        assert_eq!(validate_mandate_title(None).unwrap(), None);
        assert_eq!(
            validate_mandate_title(Some("  Gestão 2026  ")).unwrap(),
            Some("Gestão 2026".to_string())
        );
        assert_eq!(
            validate_mandate_title(Some("   ")).unwrap_err(),
            MandateError::EmptyTitle
        );
    }

    #[test]
    fn should_fail_title_when_exceeding_max_length() {
        let long_title = "a".repeat(MAX_MANDATE_TITLE_LENGTH + 1);
        assert_eq!(
            validate_mandate_title(Some(&long_title)).unwrap_err(),
            MandateError::TitleTooLong {
                max: MAX_MANDATE_TITLE_LENGTH
            }
        );
    }

    #[test]
    fn should_validate_status_transitions() {
        assert!(
            validate_mandate_status_transition(MandateStatus::Active, MandateStatus::Concluded)
                .is_ok()
        );
        assert!(
            validate_mandate_status_transition(MandateStatus::Active, MandateStatus::Archived)
                .is_ok()
        );
        assert!(
            validate_mandate_status_transition(MandateStatus::Concluded, MandateStatus::Archived)
                .is_ok()
        );
        assert!(
            validate_mandate_status_transition(MandateStatus::Active, MandateStatus::Active)
                .is_ok()
        );

        assert_eq!(
            validate_mandate_status_transition(MandateStatus::Concluded, MandateStatus::Active)
                .unwrap_err(),
            MandateError::InvalidStatusTransition {
                from: MandateStatus::Concluded,
                to: MandateStatus::Active,
            }
        );
        assert_eq!(
            validate_mandate_status_transition(MandateStatus::Archived, MandateStatus::Active)
                .unwrap_err(),
            MandateError::InvalidStatusTransition {
                from: MandateStatus::Archived,
                to: MandateStatus::Active,
            }
        );
    }

    #[test]
    fn should_validate_member_name_and_role() {
        assert_eq!(validate_member_name("  Ana Silva  ").unwrap(), "Ana Silva");
        assert_eq!(
            validate_member_name("   ").unwrap_err(),
            MandateError::EmptyMemberName
        );

        let long_name = "a".repeat(MAX_MEMBER_NAME_LENGTH + 1);
        assert_eq!(
            validate_member_name(&long_name).unwrap_err(),
            MandateError::MemberNameTooLong {
                max: MAX_MEMBER_NAME_LENGTH
            }
        );

        assert_eq!(
            validate_member_role("  Presidente  ").unwrap(),
            "Presidente"
        );
        assert_eq!(
            validate_member_role("   ").unwrap_err(),
            MandateError::EmptyMemberRole
        );

        let long_role = "r".repeat(MAX_MEMBER_ROLE_LENGTH + 1);
        assert_eq!(
            validate_member_role(&long_role).unwrap_err(),
            MandateError::MemberRoleTooLong {
                max: MAX_MEMBER_ROLE_LENGTH
            }
        );
    }

    #[test]
    fn should_validate_member_dates() {
        let now = Utc::now();
        let later = now + chrono::Duration::hours(2);
        let earlier = now - chrono::Duration::hours(2);

        assert!(validate_member_dates(now, None).is_ok());
        assert!(validate_member_dates(now, Some(later)).is_ok());
        assert_eq!(
            validate_member_dates(now, Some(earlier)).unwrap_err(),
            MandateError::InconsistentMemberDates {
                joined_at: now,
                left_at: earlier
            }
        );
    }
}
