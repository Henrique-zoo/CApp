//! Regras de negócio e validações para a entidade de Centro Acadêmico.
//!
//! Centraliza as constantes de limite, funções de validação de formato e
//! invariantes de ciclo de vida aplicáveis aos Centros Acadêmicos.

use chrono::{DateTime, Utc};

use crate::modules::ca::domain::entities::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus,
};

/// Limite máximo em caracteres para o nome oficial do Centro Acadêmico.
pub const MAX_NAME_LENGTH: usize = 255;

/// Limite máximo em caracteres para a sigla do Centro Acadêmico.
pub const MAX_ACRONYM_LENGTH: usize = 30;

/// Limite máximo em caracteres para a descrição institucional do Centro Acadêmico.
pub const MAX_DESCRIPTION_LENGTH: usize = 4000;

/// Determina se um caractere é válido na composição da sigla de um Centro Acadêmico.
///
/// São aceitos caracteres alfanuméricos, hífens (`-`), pontos (`.`) e sublinhados (`_`).
pub const fn is_valid_acronym_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_'
}

/// Valida o nome oficial do Centro Acadêmico conforme os critérios de negócio.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::EmptyName`] se o nome for vazio ou composto apenas por espaços.
/// Retorna [`AcademicCenterError::NameTooLong`] se o nome exceder [`MAX_NAME_LENGTH`].
pub fn validate_name(name: &str) -> Result<(), AcademicCenterError> {
    if name.trim().is_empty() {
        return Err(AcademicCenterError::EmptyName);
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(AcademicCenterError::NameTooLong {
            max: MAX_NAME_LENGTH,
        });
    }
    Ok(())
}

/// Valida a sigla do Centro Acadêmico conforme as regras de negócio e formato permitido.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::EmptyAcronym`] se a sigla for vazia ou composta apenas por espaços.
/// Retorna [`AcademicCenterError::AcronymTooLong`] se a sigla exceder [`MAX_ACRONYM_LENGTH`].
/// Retorna [`AcademicCenterError::InvalidAcronym`] se a sigla contiver caracteres não permitidos.
pub fn validate_acronym(acronym: &str) -> Result<(), AcademicCenterError> {
    let trimmed = acronym.trim();
    if trimmed.is_empty() {
        return Err(AcademicCenterError::EmptyAcronym);
    }
    if trimmed.chars().count() > MAX_ACRONYM_LENGTH {
        return Err(AcademicCenterError::AcronymTooLong {
            max: MAX_ACRONYM_LENGTH,
        });
    }
    if !trimmed.chars().all(is_valid_acronym_char) {
        return Err(AcademicCenterError::InvalidAcronym(trimmed.to_string()));
    }
    Ok(())
}

/// Valida a descrição institucional do Centro Acadêmico.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::DescriptionTooLong`] se a descrição exceder [`MAX_DESCRIPTION_LENGTH`].
pub fn validate_description(description: &str) -> Result<(), AcademicCenterError> {
    if description.chars().count() > MAX_DESCRIPTION_LENGTH {
        return Err(AcademicCenterError::DescriptionTooLong {
            max: MAX_DESCRIPTION_LENGTH,
        });
    }
    Ok(())
}

/// Valida a coerência temporal entre os instantes de criação e de última modificação.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::InconsistentTimestamps`] se `updated_at` for anterior a `created_at`.
pub fn validate_timestamps(
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), AcademicCenterError> {
    if updated_at < created_at {
        return Err(AcademicCenterError::InconsistentTimestamps {
            created_at,
            updated_at,
        });
    }
    Ok(())
}

/// Valida a transição entre estados no ciclo de vida de um Centro Acadêmico.
///
/// # Regras de transição
///
/// - Um CA arquivado não pode retornar para os estados ativo ou rascunho.
/// - Transições para o mesmo estado são idempotentes e válidas.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::InvalidStatusTransition`] quando a transição for proibida.
pub fn validate_status_transition(
    from: AcademicCenterStatus,
    to: AcademicCenterStatus,
) -> Result<(), AcademicCenterError> {
    match (from, to) {
        (AcademicCenterStatus::Draft, AcademicCenterStatus::Active)
        | (AcademicCenterStatus::Draft, AcademicCenterStatus::Archived)
        | (AcademicCenterStatus::Active, AcademicCenterStatus::Archived)
        | (AcademicCenterStatus::Draft, AcademicCenterStatus::Draft)
        | (AcademicCenterStatus::Active, AcademicCenterStatus::Active)
        | (AcademicCenterStatus::Archived, AcademicCenterStatus::Archived) => Ok(()),
        (AcademicCenterStatus::Archived, AcademicCenterStatus::Active)
        | (AcademicCenterStatus::Archived, AcademicCenterStatus::Draft)
        | (AcademicCenterStatus::Active, AcademicCenterStatus::Draft) => {
            Err(AcademicCenterError::InvalidStatusTransition { from, to })
        }
    }
}

/// Valida todas as invariantes e integridade estrutural de um Centro Acadêmico.
///
/// # Erros
///
/// Retorna [`AcademicCenterError`] se qualquer uma das regras de nome, sigla,
/// descrição ou coerência temporal for violada.
pub fn validate_academic_center(
    academic_center: &AcademicCenter,
) -> Result<(), AcademicCenterError> {
    validate_name(&academic_center.name)?;
    validate_acronym(&academic_center.acronym)?;
    validate_description(&academic_center.description)?;
    validate_timestamps(academic_center.created_at, academic_center.updated_at)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn should_validate_name_successfully() {
        assert!(validate_name("Centro Acadêmico de Engenharia").is_ok());
        assert!(validate_name("CA").is_ok());
        assert!(validate_name(&"A".repeat(MAX_NAME_LENGTH)).is_ok());
    }

    #[test]
    fn should_fail_validation_for_empty_or_whitespace_name() {
        assert_eq!(validate_name(""), Err(AcademicCenterError::EmptyName));
        assert_eq!(validate_name("   "), Err(AcademicCenterError::EmptyName));
        assert_eq!(validate_name("\t\n"), Err(AcademicCenterError::EmptyName));
    }

    #[test]
    fn should_fail_validation_when_name_exceeds_max_length() {
        let long_name = "A".repeat(MAX_NAME_LENGTH + 1);
        assert_eq!(
            validate_name(&long_name),
            Err(AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_acronym_successfully() {
        assert!(validate_acronym("CAER").is_ok());
        assert!(validate_acronym("CACC").is_ok());
        assert!(validate_acronym("CA-REDES").is_ok());
        assert!(validate_acronym("C.A.E.R").is_ok());
        assert!(validate_acronym("CA_COMP").is_ok());
        assert!(validate_acronym(&"A".repeat(MAX_ACRONYM_LENGTH)).is_ok());
    }

    #[test]
    fn should_fail_validation_for_empty_or_whitespace_acronym() {
        assert_eq!(validate_acronym(""), Err(AcademicCenterError::EmptyAcronym));
        assert_eq!(
            validate_acronym("   "),
            Err(AcademicCenterError::EmptyAcronym)
        );
        assert_eq!(
            validate_acronym("\n\t"),
            Err(AcademicCenterError::EmptyAcronym)
        );
    }

    #[test]
    fn should_fail_validation_when_acronym_exceeds_max_length() {
        let long_acronym = "A".repeat(MAX_ACRONYM_LENGTH + 1);
        assert_eq!(
            validate_acronym(&long_acronym),
            Err(AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH
            })
        );
    }

    #[test]
    fn should_fail_validation_when_acronym_contains_invalid_characters() {
        assert_eq!(
            validate_acronym("CA ER"),
            Err(AcademicCenterError::InvalidAcronym("CA ER".to_string()))
        );
        assert_eq!(
            validate_acronym("CAER!"),
            Err(AcademicCenterError::InvalidAcronym("CAER!".to_string()))
        );
        assert_eq!(
            validate_acronym("CA@UnB"),
            Err(AcademicCenterError::InvalidAcronym("CA@UnB".to_string()))
        );
    }

    #[test]
    fn should_validate_description_successfully() {
        assert!(validate_description("").is_ok());
        assert!(validate_description("Descrição institucional detalhada").is_ok());
        assert!(validate_description(&"D".repeat(MAX_DESCRIPTION_LENGTH)).is_ok());
    }

    #[test]
    fn should_fail_validation_when_description_exceeds_max_length() {
        let long_description = "D".repeat(MAX_DESCRIPTION_LENGTH + 1);
        assert_eq!(
            validate_description(&long_description),
            Err(AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_timestamps_successfully() {
        let now = Utc::now();
        assert!(validate_timestamps(now, now).is_ok());
        let later = now + chrono::Duration::seconds(10);
        assert!(validate_timestamps(now, later).is_ok());
    }

    #[test]
    fn should_fail_validation_when_updated_at_is_before_created_at() {
        let now = Utc::now();
        let earlier = now - chrono::Duration::seconds(10);
        assert_eq!(
            validate_timestamps(now, earlier),
            Err(AcademicCenterError::InconsistentTimestamps {
                created_at: now,
                updated_at: earlier,
            })
        );
    }

    #[test]
    fn should_validate_status_transitions() {
        assert!(
            validate_status_transition(AcademicCenterStatus::Draft, AcademicCenterStatus::Active)
                .is_ok()
        );
        assert!(
            validate_status_transition(
                AcademicCenterStatus::Active,
                AcademicCenterStatus::Archived
            )
            .is_ok()
        );
        assert!(
            validate_status_transition(AcademicCenterStatus::Active, AcademicCenterStatus::Active)
                .is_ok()
        );

        assert_eq!(
            validate_status_transition(
                AcademicCenterStatus::Archived,
                AcademicCenterStatus::Active
            ),
            Err(AcademicCenterError::InvalidStatusTransition {
                from: AcademicCenterStatus::Archived,
                to: AcademicCenterStatus::Active,
            })
        );
        assert_eq!(
            validate_status_transition(AcademicCenterStatus::Active, AcademicCenterStatus::Draft),
            Err(AcademicCenterError::InvalidStatusTransition {
                from: AcademicCenterStatus::Active,
                to: AcademicCenterStatus::Draft,
            })
        );
    }

    #[test]
    fn should_validate_entire_academic_center_entity() {
        let now = Utc::now();
        let valid_center = AcademicCenter {
            id: Uuid::new_v4(),
            created_by_id: None,
            name: "Centro Acadêmico de Computação".to_string(),
            acronym: "CAC".to_string(),
            description: "Descrição válida".to_string(),
            status: AcademicCenterStatus::Active,
            created_at: now,
            updated_at: now,
        };
        assert!(validate_academic_center(&valid_center).is_ok());

        let mut invalid_name = valid_center.clone();
        invalid_name.name = "".to_string();
        assert_eq!(
            validate_academic_center(&invalid_name),
            Err(AcademicCenterError::EmptyName)
        );

        let mut invalid_acronym = valid_center.clone();
        invalid_acronym.acronym = "INVALID@".to_string();
        assert_eq!(
            validate_academic_center(&invalid_acronym),
            Err(AcademicCenterError::InvalidAcronym("INVALID@".to_string()))
        );

        let mut invalid_timestamps = valid_center;
        invalid_timestamps.updated_at = now - chrono::Duration::hours(1);
        assert_eq!(
            validate_academic_center(&invalid_timestamps),
            Err(AcademicCenterError::InconsistentTimestamps {
                created_at: now,
                updated_at: invalid_timestamps.updated_at,
            })
        );
    }
}
