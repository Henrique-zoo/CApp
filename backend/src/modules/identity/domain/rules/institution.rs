//! Regras de validação para instituições cadastradas.

use crate::modules::identity::domain::entities::institution::InstitutionError;

/// Valida e normaliza o nome da instituição.
pub fn validate_institution_name(name: &str) -> Result<String, InstitutionError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Err(InstitutionError::EmptyName)
    } else {
        Ok(trimmed.to_string())
    }
}

/// Valida e normaliza a sigla opcional da instituição.
pub fn validate_institution_acronym(
    acronym: Option<&str>,
) -> Result<Option<String>, InstitutionError> {
    match acronym {
        Some(acr) => {
            let trimmed = acr.trim();
            if trimmed.is_empty() {
                Err(InstitutionError::EmptyAcronym)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_institution_name_correctly() {
        assert_eq!(
            validate_institution_name(" Universidade de Brasília ").unwrap(),
            "Universidade de Brasília".to_string()
        );
        assert!(matches!(
            validate_institution_name("   "),
            Err(InstitutionError::EmptyName)
        ));
    }

    #[test]
    fn should_validate_institution_acronym_correctly() {
        assert_eq!(
            validate_institution_acronym(Some(" UnB ")).unwrap(),
            Some("UnB".to_string())
        );
        assert_eq!(validate_institution_acronym(None).unwrap(), None);
        assert!(matches!(
            validate_institution_acronym(Some("   ")),
            Err(InstitutionError::EmptyAcronym)
        ));
    }
}
