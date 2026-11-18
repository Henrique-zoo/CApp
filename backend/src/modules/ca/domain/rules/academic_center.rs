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
/// São aceitos caracteres alfanuméricos ASCII, hífens (`-`), pontos (`.`) e sublinhados (`_`).
pub const fn is_valid_acronym_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_'
}

/// Valida o nome oficial do Centro Acadêmico conforme os critérios de negócio.
///
/// Espaços periféricos são desconsiderados (`trim`), garantindo consistência com
/// o construtor da entidade de domínio.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::EmptyName`] se o nome for vazio ou composto apenas por espaços.
/// Retorna [`AcademicCenterError::NameTooLong`] se o nome sanitizado exceder [`MAX_NAME_LENGTH`].
pub fn validate_name(name: &str) -> Result<(), AcademicCenterError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AcademicCenterError::EmptyName);
    }
    if trimmed.chars().count() > MAX_NAME_LENGTH {
        return Err(AcademicCenterError::NameTooLong {
            max: MAX_NAME_LENGTH,
        });
    }
    Ok(())
}

/// Valida a sigla do Centro Acadêmico conforme as regras de negócio e formato permitido.
///
/// Espaços periféricos são desconsiderados (`trim`), garantindo consistência com
/// o construtor da entidade de domínio.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::EmptyAcronym`] se a sigla for vazia ou composta apenas por espaços.
/// Retorna [`AcademicCenterError::AcronymTooLong`] se a sigla sanitizada exceder [`MAX_ACRONYM_LENGTH`].
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

/// Valida a sigla opcional do Centro Acadêmico.
///
/// Caso a sigla seja fornecida (`Some`), aplica as validações de formato e limites de [`validate_acronym`].
///
/// # Erros
///
/// Retorna [`AcademicCenterError`] se a sigla informada for inválida.
pub fn validate_optional_acronym(acronym: Option<&str>) -> Result<(), AcademicCenterError> {
    if let Some(acronym) = acronym {
        validate_acronym(acronym)?;
    }
    Ok(())
}

/// Valida a descrição institucional do Centro Acadêmico.
///
/// Espaços periféricos são desconsiderados (`trim`), garantindo consistência com
/// o construtor da entidade de domínio.
///
/// # Erros
///
/// Retorna [`AcademicCenterError::DescriptionTooLong`] se a descrição exceder [`MAX_DESCRIPTION_LENGTH`].
pub fn validate_description(description: &str) -> Result<(), AcademicCenterError> {
    let trimmed = description.trim();
    if trimmed.chars().count() > MAX_DESCRIPTION_LENGTH {
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
/// - Um CA arquivado não pode retornar para o estado ativo.
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
        (AcademicCenterStatus::Active, AcademicCenterStatus::Archived)
        | (AcademicCenterStatus::Active, AcademicCenterStatus::Active)
        | (AcademicCenterStatus::Archived, AcademicCenterStatus::Archived) => Ok(()),
        (AcademicCenterStatus::Archived, AcademicCenterStatus::Active) => {
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
    validate_name(academic_center.name())?;
    validate_optional_acronym(academic_center.acronym())?;
    validate_description(academic_center.description())?;
    validate_timestamps(academic_center.created_at(), academic_center.updated_at())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    const INSTITUTION_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0000);

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
    fn should_validate_optional_acronym_correctly() {
        assert!(validate_optional_acronym(None).is_ok());
        assert!(validate_optional_acronym(Some("CAER")).is_ok());
        assert_eq!(
            validate_optional_acronym(Some("")),
            Err(AcademicCenterError::EmptyAcronym)
        );
        assert_eq!(
            validate_optional_acronym(Some("   ")),
            Err(AcademicCenterError::EmptyAcronym)
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
        assert!(
            validate_status_transition(
                AcademicCenterStatus::Archived,
                AcademicCenterStatus::Archived
            )
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
    }

    #[test]
    fn should_validate_entire_academic_center_entity() {
        let now = Utc::now();
        let valid_center = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            "Centro Acadêmico de Computação".to_string(),
            Some("CAC".to_string()),
            "Descrição válida".to_string(),
            now,
        )
        .expect("should create valid academic center");

        assert!(validate_academic_center(&valid_center).is_ok());
    }

    // Testes de consistência estrita de trim entre construtor e validadores exportados
    #[test]
    fn should_have_identical_trim_behavior_between_validate_name_and_constructor() {
        let now = Utc::now();

        // Caso 1: Nome com exatamente 255 caracteres + espaços nas extremidades
        // Deve ser aceito tanto pelo validador exportado quanto pelo construtor
        let exact_max_name = format!("  {}  ", "A".repeat(MAX_NAME_LENGTH));
        assert!(
            validate_name(&exact_max_name).is_ok(),
            "validate_name deve aceitar 255 caracteres com espaços nas extremidades"
        );
        let ca = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            exact_max_name.clone(),
            None,
            "Descrição".to_string(),
            now,
        )
        .expect("construtor deve aceitar 255 caracteres com espaços nas extremidades");
        assert_eq!(ca.name().len(), MAX_NAME_LENGTH);

        // Caso 2: Nome com 256 caracteres + espaços nas extremidades
        // Deve ser rejeitado tanto pelo validador exportado quanto pelo construtor
        let over_max_name = format!("  {}  ", "A".repeat(MAX_NAME_LENGTH + 1));
        assert_eq!(
            validate_name(&over_max_name),
            Err(AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH
            }),
            "validate_name deve rejeitar 256 caracteres mesmo com espaços"
        );
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            over_max_name,
            None,
            "Descrição".to_string(),
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH
            }
        );
    }

    #[test]
    fn should_have_identical_trim_behavior_between_validate_acronym_and_constructor() {
        let now = Utc::now();

        // Caso 1: Sigla com exatamente 30 caracteres + espaços nas extremidades
        // Deve ser aceita tanto por validate_acronym quanto pelo construtor
        let exact_max_acronym = format!("  {}  ", "A".repeat(MAX_ACRONYM_LENGTH));
        assert!(
            validate_acronym(&exact_max_acronym).is_ok(),
            "validate_acronym deve aceitar 30 caracteres com espaços nas extremidades"
        );
        let ca = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            "Nome".to_string(),
            Some(exact_max_acronym.clone()),
            "Descrição".to_string(),
            now,
        )
        .expect("construtor deve aceitar 30 caracteres com espaços nas extremidades");
        assert_eq!(ca.acronym(), Some("A".repeat(MAX_ACRONYM_LENGTH).as_str()));

        // Caso 2: Sigla com 31 caracteres + espaços nas extremidades
        // Deve ser rejeitada tanto por validate_acronym quanto pelo construtor
        let over_max_acronym = format!("  {}  ", "A".repeat(MAX_ACRONYM_LENGTH + 1));
        assert_eq!(
            validate_acronym(&over_max_acronym),
            Err(AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH
            }),
            "validate_acronym deve rejeitar 31 caracteres mesmo com espaços"
        );
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            "Nome".to_string(),
            Some(over_max_acronym),
            "Descrição".to_string(),
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH
            }
        );
    }

    #[test]
    fn should_have_identical_trim_behavior_between_validate_description_and_constructor() {
        let now = Utc::now();

        // Caso 1: Descrição com exatamente 4000 caracteres + espaços nas extremidades
        // Deve ser aceita tanto por validate_description quanto pelo construtor
        let exact_max_desc = format!("  {}  ", "D".repeat(MAX_DESCRIPTION_LENGTH));
        assert!(
            validate_description(&exact_max_desc).is_ok(),
            "validate_description deve aceitar 4000 caracteres com espaços nas extremidades"
        );
        let ca = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            "Nome".to_string(),
            None,
            exact_max_desc.clone(),
            now,
        )
        .expect("construtor deve aceitar 4000 caracteres com espaços nas extremidades");
        assert_eq!(ca.description().len(), MAX_DESCRIPTION_LENGTH);

        // Caso 2: Descrição com 4001 caracteres + espaços nas extremidades
        // Deve ser rejeitada tanto por validate_description quanto pelo construtor
        let over_max_desc = format!("  {}  ", "D".repeat(MAX_DESCRIPTION_LENGTH + 1));
        assert_eq!(
            validate_description(&over_max_desc),
            Err(AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH
            }),
            "validate_description deve rejeitar 4001 caracteres mesmo com espaços"
        );
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_ID,
            None,
            "Nome".to_string(),
            None,
            over_max_desc,
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH
            }
        );
    }
}
