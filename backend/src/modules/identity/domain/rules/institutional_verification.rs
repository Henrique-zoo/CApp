//! Regras de validação e invariantes de verificação institucional.
//!
//! Centraliza as checagens de integridade estrutural da evidência institucional,
//! incluindo coerência de prazos e consistência entre vínculo e curso.

use chrono::{DateTime, Utc};

use crate::modules::identity::domain::entities::institutional_verification::InstitutionalVerificationError;

/// Valida a coerência temporal do período de validade da verificação.
///
/// O instante de expiração deve ser estritamente posterior ao instante de checagem.
pub fn validate_verification_period(
    checked_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<(), InstitutionalVerificationError> {
    if expires_at <= checked_at {
        return Err(InstitutionalVerificationError::InconsistentTimestamps {
            checked_at,
            expires_at,
        });
    }
    Ok(())
}

/// Valida e normaliza o curso em função do vínculo estudantil confirmado.
///
/// Para discentes regulares (`student_membership_confirmed = true`), o curso é obrigatório
/// e não pode ser vazio ou composto exclusivamente por espaços em branco.
/// Para não discentes (`student_membership_confirmed = false`), nenhum curso pode ser associado.
pub fn validate_student_course(
    student_membership_confirmed: bool,
    course_name: Option<&str>,
) -> Result<Option<String>, InstitutionalVerificationError> {
    if student_membership_confirmed {
        match course_name {
            Some(course) => {
                let trimmed = course.trim();
                if trimmed.is_empty() {
                    Err(InstitutionalVerificationError::EmptyCourseName)
                } else {
                    Ok(Some(trimmed.to_string()))
                }
            }
            None => Err(InstitutionalVerificationError::MissingCourseForStudent),
        }
    } else if course_name.is_some() {
        Err(InstitutionalVerificationError::CourseProvidedForNonStudent)
    } else {
        Ok(None)
    }
}

/// Normaliza o nome de perfil opcional, aparando espaços e descartando valores vazios.
pub fn validate_profile_name(profile_name: Option<&str>) -> Option<String> {
    profile_name
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn should_validate_verification_period_correctly() {
        let now = Utc::now();
        assert!(validate_verification_period(now, now + Duration::hours(1)).is_ok());
        assert!(validate_verification_period(now, now).is_err());
        assert!(validate_verification_period(now, now - Duration::hours(1)).is_err());
    }

    #[test]
    fn should_validate_student_course_correctly() {
        // Discente confirmado com curso válido
        assert_eq!(
            validate_student_course(true, Some("  Engenharia de Redes  ")).unwrap(),
            Some("Engenharia de Redes".to_string())
        );

        // Discente confirmado sem curso
        assert!(matches!(
            validate_student_course(true, None),
            Err(InstitutionalVerificationError::MissingCourseForStudent)
        ));

        // Discente confirmado com curso vazio ou apenas espaços
        assert!(matches!(
            validate_student_course(true, Some("   ")),
            Err(InstitutionalVerificationError::EmptyCourseName)
        ));

        // Não discente sem curso
        assert_eq!(validate_student_course(false, None).unwrap(), None);

        // Não discente com curso informado deve falhar
        assert!(matches!(
            validate_student_course(false, Some("Ciência da Computação")),
            Err(InstitutionalVerificationError::CourseProvidedForNonStudent)
        ));
    }

    #[test]
    fn should_normalize_profile_name() {
        assert_eq!(
            validate_profile_name(Some("  Nome do Estudante  ")),
            Some("Nome do Estudante".to_string())
        );
        assert_eq!(validate_profile_name(Some("   ")), None);
        assert_eq!(validate_profile_name(None), None);
    }
}
