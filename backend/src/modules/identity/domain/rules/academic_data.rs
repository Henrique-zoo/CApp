//! Regras de negócio para associação e avaliação de dados acadêmicos.
//!
//! Conforme § 1.3 do Relatório de Features e a especificação de associação de dados
//! acadêmicos, o contexto estudantil pleno exige curso identificado e situação
//! acadêmica regular ativa confirmada pela instituição.

use crate::modules::identity::domain::entities::academic_data::AcademicStanding;

/// Determina a situação acadêmica a partir da participação no grupo estudantil e do curso informado.
pub fn determine_academic_standing(
    is_student_member: bool,
    course_name: Option<&str>,
) -> AcademicStanding {
    if is_student_member
        && course_name
            .map(str::trim)
            .is_some_and(|course| !course.is_empty())
    {
        AcademicStanding::ActiveStudent
    } else {
        AcademicStanding::NonStudent
    }
}

/// Verifica se o contexto acadêmico do estudante está completo.
///
/// Um contexto acadêmico só é considerado completo quando o curso está identificado
/// e a situação acadêmica confirma vínculo discente ativo.
pub fn is_academic_context_complete(course_name: Option<&str>, standing: AcademicStanding) -> bool {
    standing == AcademicStanding::ActiveStudent
        && course_name
            .map(str::trim)
            .is_some_and(|course| !course.is_empty())
}

/// Avalia se os dados acadêmicos conferem direitos e acesso a funcionalidades restritas a estudantes.
pub fn allows_student_access(standing: AcademicStanding, course_name: Option<&str>) -> bool {
    is_academic_context_complete(course_name, standing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_determine_active_student_only_with_member_and_course() {
        assert_eq!(
            determine_academic_standing(true, Some("Engenharia de Software")),
            AcademicStanding::ActiveStudent
        );
        assert_eq!(
            determine_academic_standing(true, None),
            AcademicStanding::NonStudent
        );
        assert_eq!(
            determine_academic_standing(true, Some("   ")),
            AcademicStanding::NonStudent
        );
        assert_eq!(
            determine_academic_standing(false, Some("Engenharia de Software")),
            AcademicStanding::NonStudent
        );
        assert_eq!(
            determine_academic_standing(false, None),
            AcademicStanding::NonStudent
        );
    }

    #[test]
    fn should_validate_academic_context_completeness() {
        assert!(is_academic_context_complete(
            Some("Ciência da Computação"),
            AcademicStanding::ActiveStudent
        ));
        assert!(!is_academic_context_complete(
            None,
            AcademicStanding::ActiveStudent
        ));
        assert!(!is_academic_context_complete(
            Some("Ciência da Computação"),
            AcademicStanding::NonStudent
        ));
    }

    #[test]
    fn should_allow_student_access_only_when_context_is_complete() {
        assert!(allows_student_access(
            AcademicStanding::ActiveStudent,
            Some("Engenharia de Redes")
        ));
        assert!(!allows_student_access(
            AcademicStanding::NonStudent,
            Some("Engenharia de Redes")
        ));
        assert!(!allows_student_access(
            AcademicStanding::ActiveStudent,
            None
        ));
    }
}
