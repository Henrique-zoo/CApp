//! Regras de negócio e validações para o módulo de demandas.
//!
//! Centraliza as constantes de limite, validações estruturais de texto,
//! coerência temporal e a matriz de transições de status permitidas por fórum.

use chrono::{DateTime, Utc};

use crate::modules::demands::domain::entities::{DemandError, DemandForum, DemandStatus};

/// Limite máximo em caracteres para o título de uma demanda.
pub const MAX_TITLE_LENGTH: usize = 255;

/// Limite máximo em caracteres para a descrição ou conteúdo de uma demanda.
pub const MAX_DESCRIPTION_LENGTH: usize = 5000;

/// Limite máximo em caracteres para a categoria de uma demanda.
pub const MAX_CATEGORY_LENGTH: usize = 100;

/// Limite máximo em caracteres para o nome do curso vinculado a uma demanda institucional.
pub const MAX_COURSE_NAME_LENGTH: usize = 255;

/// Limite máximo em caracteres para a justificativa ou comentário de alteração de status.
pub const MAX_COMMENT_LENGTH: usize = 1000;

/// Valida a descrição ou conteúdo da demanda.
///
/// Espaços periféricos são desconsiderados (`trim`). O texto não pode ser vazio
/// nem exceder [`MAX_DESCRIPTION_LENGTH`].
///
/// # Erros
///
/// Retorna [`DemandError::EmptyDescription`] se a descrição for vazia ou só espaços.
/// Retorna [`DemandError::DescriptionTooLong`] se exceder o limite máximo.
pub fn validate_description(description: &str) -> Result<(), DemandError> {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return Err(DemandError::EmptyDescription);
    }
    if trimmed.chars().count() > MAX_DESCRIPTION_LENGTH {
        return Err(DemandError::DescriptionTooLong {
            max: MAX_DESCRIPTION_LENGTH,
        });
    }
    Ok(())
}

/// Valida o título de uma demanda quando informado.
///
/// Espaços periféricos são desconsiderados (`trim`). O título não pode ser vazio
/// nem exceder [`MAX_TITLE_LENGTH`].
///
/// # Erros
///
/// Retorna [`DemandError::EmptyTitle`] se for vazio ou composto apenas por espaços.
/// Retorna [`DemandError::TitleTooLong`] se exceder [`MAX_TITLE_LENGTH`].
pub fn validate_title(title: &str) -> Result<(), DemandError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DemandError::EmptyTitle);
    }
    if trimmed.chars().count() > MAX_TITLE_LENGTH {
        return Err(DemandError::TitleTooLong {
            max: MAX_TITLE_LENGTH,
        });
    }
    Ok(())
}

/// Valida o título opcional de uma demanda.
///
/// Caso esteja presente (`Some`), delega a validação para [`validate_title`].
///
/// # Erros
///
/// Retorna [`DemandError`] se o título informado for inválido.
pub fn validate_optional_title(title: Option<&str>) -> Result<(), DemandError> {
    if let Some(t) = title {
        validate_title(t)?;
    }
    Ok(())
}

/// Valida a categoria de uma demanda quando informada.
///
/// Espaços periféricos são desconsiderados (`trim`). A categoria não pode ser vazia
/// nem exceder [`MAX_CATEGORY_LENGTH`].
///
/// # Erros
///
/// Retorna [`DemandError::EmptyCategory`] se for vazia ou composta apenas por espaços.
/// Retorna [`DemandError::CategoryTooLong`] se exceder [`MAX_CATEGORY_LENGTH`].
pub fn validate_category(category: &str) -> Result<(), DemandError> {
    let trimmed = category.trim();
    if trimmed.is_empty() {
        return Err(DemandError::EmptyCategory);
    }
    if trimmed.chars().count() > MAX_CATEGORY_LENGTH {
        return Err(DemandError::CategoryTooLong {
            max: MAX_CATEGORY_LENGTH,
        });
    }
    Ok(())
}

/// Valida a categoria opcional de uma demanda.
///
/// Caso esteja presente (`Some`), delega para [`validate_category`].
///
/// # Erros
///
/// Retorna [`DemandError`] se a categoria informada for inválida.
pub fn validate_optional_category(category: Option<&str>) -> Result<(), DemandError> {
    if let Some(c) = category {
        validate_category(c)?;
    }
    Ok(())
}

/// Valida o nome do curso vinculado à demanda institucional.
///
/// Espaços periféricos são desconsiderados (`trim`). O nome do curso não pode ser vazio
/// nem exceder [`MAX_COURSE_NAME_LENGTH`].
///
/// # Erros
///
/// Retorna [`DemandError::EmptyCourseName`] se for vazio ou composto apenas por espaços.
/// Retorna [`DemandError::CourseNameTooLong`] se exceder [`MAX_COURSE_NAME_LENGTH`].
pub fn validate_course_name(course_name: &str) -> Result<(), DemandError> {
    let trimmed = course_name.trim();
    if trimmed.is_empty() {
        return Err(DemandError::EmptyCourseName);
    }
    if trimmed.chars().count() > MAX_COURSE_NAME_LENGTH {
        return Err(DemandError::CourseNameTooLong {
            max: MAX_COURSE_NAME_LENGTH,
        });
    }
    Ok(())
}

/// Valida o curso opcional associado à demanda conforme o fórum de negócio.
///
/// Demanda sobre o espaço físico nunca deve possuir curso associado.
///
/// # Erros
///
/// Retorna [`DemandError::CourseNotAllowedForPhysicalSpace`] se o curso for informado
/// para o fórum [`DemandForum::PhysicalSpace`].
/// Retorna [`DemandError`] se o nome do curso for inválido.
pub fn validate_optional_course_name(
    course_name: Option<&str>,
    forum: DemandForum,
) -> Result<(), DemandError> {
    match (course_name, forum) {
        (Some(_), DemandForum::PhysicalSpace) => Err(DemandError::CourseNotAllowedForPhysicalSpace),
        (Some(c), DemandForum::Institutional) => validate_course_name(c),
        (None, _) => Ok(()),
    }
}

/// Valida o comentário ou justificativa opcional de alteração de status.
///
/// Espaços periféricos são desconsiderados (`trim`). O comentário não pode ser vazio
/// nem exceder [`MAX_COMMENT_LENGTH`].
///
/// # Erros
///
/// Retorna [`DemandError::EmptyComment`] se informado apenas com espaços.
/// Retorna [`DemandError::CommentTooLong`] se exceder [`MAX_COMMENT_LENGTH`].
pub fn validate_comment(comment: &str) -> Result<(), DemandError> {
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        return Err(DemandError::EmptyComment);
    }
    if trimmed.chars().count() > MAX_COMMENT_LENGTH {
        return Err(DemandError::CommentTooLong {
            max: MAX_COMMENT_LENGTH,
        });
    }
    Ok(())
}

/// Valida o comentário opcional de alteração de status.
pub fn validate_optional_comment(comment: Option<&str>) -> Result<(), DemandError> {
    if let Some(c) = comment {
        validate_comment(c)?;
    }
    Ok(())
}

/// Valida a coerência temporal entre os instantes de criação e de atualização.
///
/// # Erros
///
/// Retorna [`DemandError::InconsistentTimestamps`] se `updated_at` for anterior a `created_at`.
pub fn validate_timestamps(
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), DemandError> {
    if updated_at < created_at {
        return Err(DemandError::InconsistentTimestamps {
            created_at,
            updated_at,
        });
    }
    Ok(())
}

/// Valida se uma transição de status é permitida para o fórum da demanda.
///
/// Regras fundamentais de ciclo de vida:
/// - Transição para o mesmo status é proibida.
/// - Demandas de espaço físico NUNCA podem ser encaminhadas à coordenação
///   ([`DemandStatus::ForwardedToCoordination`]).
/// - Transições permitidas em espaço físico:
///   - `Open` -> `InAnalysis`, `Completed`
///   - `InAnalysis` -> `Completed`, `Open`
///   - `Completed` -> `Open`, `InAnalysis`
/// - Transições permitidas em fórum institucional:
///   - `Open` -> `InAnalysis`, `ForwardedToCoordination`, `Completed`
///   - `InAnalysis` -> `ForwardedToCoordination`, `Completed`, `Open`
///   - `ForwardedToCoordination` -> `InAnalysis`, `Completed`, `Open`
///   - `Completed` -> `Open`, `InAnalysis`, `ForwardedToCoordination`
///
/// # Erros
///
/// Retorna [`DemandError::CoordinationForwardingNotAllowed`] caso se tente encaminhar
/// à coordenação uma demanda de espaço físico.
/// Retorna [`DemandError::InvalidStatusTransition`] se a transição violar a matriz.
pub fn validate_status_transition(
    forum: DemandForum,
    from: DemandStatus,
    to: DemandStatus,
) -> Result<(), DemandError> {
    if from == to {
        return Err(DemandError::InvalidStatusTransition { from, to, forum });
    }

    if forum == DemandForum::PhysicalSpace && to == DemandStatus::ForwardedToCoordination {
        return Err(DemandError::CoordinationForwardingNotAllowed);
    }

    match (forum, from, to) {
        // Fórum do Espaço Físico
        (DemandForum::PhysicalSpace, DemandStatus::Open, DemandStatus::InAnalysis)
        | (DemandForum::PhysicalSpace, DemandStatus::Open, DemandStatus::Completed)
        | (DemandForum::PhysicalSpace, DemandStatus::InAnalysis, DemandStatus::Completed)
        | (DemandForum::PhysicalSpace, DemandStatus::InAnalysis, DemandStatus::Open)
        | (DemandForum::PhysicalSpace, DemandStatus::Completed, DemandStatus::Open)
        | (DemandForum::PhysicalSpace, DemandStatus::Completed, DemandStatus::InAnalysis) => Ok(()),

        // Fórum Institucional
        (DemandForum::Institutional, DemandStatus::Open, DemandStatus::InAnalysis)
        | (DemandForum::Institutional, DemandStatus::Open, DemandStatus::ForwardedToCoordination)
        | (DemandForum::Institutional, DemandStatus::Open, DemandStatus::Completed)
        | (
            DemandForum::Institutional,
            DemandStatus::InAnalysis,
            DemandStatus::ForwardedToCoordination,
        )
        | (DemandForum::Institutional, DemandStatus::InAnalysis, DemandStatus::Completed)
        | (DemandForum::Institutional, DemandStatus::InAnalysis, DemandStatus::Open)
        | (
            DemandForum::Institutional,
            DemandStatus::ForwardedToCoordination,
            DemandStatus::InAnalysis,
        )
        | (
            DemandForum::Institutional,
            DemandStatus::ForwardedToCoordination,
            DemandStatus::Completed,
        )
        | (DemandForum::Institutional, DemandStatus::ForwardedToCoordination, DemandStatus::Open)
        | (DemandForum::Institutional, DemandStatus::Completed, DemandStatus::InAnalysis)
        | (
            DemandForum::Institutional,
            DemandStatus::Completed,
            DemandStatus::ForwardedToCoordination,
        ) => Ok(()),

        _ => Err(DemandError::InvalidStatusTransition { from, to, forum }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_description_successfully() {
        assert!(validate_description("Porta da sala quebrada").is_ok());
        assert!(validate_description("  Lâmpada queimada  ").is_ok());
    }

    #[test]
    fn should_fail_validation_for_empty_or_whitespace_description() {
        assert_eq!(validate_description(""), Err(DemandError::EmptyDescription));
        assert_eq!(
            validate_description("   \t \n  "),
            Err(DemandError::EmptyDescription)
        );
    }

    #[test]
    fn should_fail_validation_when_description_exceeds_max_length() {
        let long_desc = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
        assert_eq!(
            validate_description(&long_desc),
            Err(DemandError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_title_successfully() {
        assert!(validate_title("Reparo da porta").is_ok());
        assert!(validate_title("  Troca da lâmpada  ").is_ok());
        assert!(validate_optional_title(None).is_ok());
        assert!(validate_optional_title(Some("Título")).is_ok());
    }

    #[test]
    fn should_fail_validation_for_empty_or_whitespace_title() {
        assert_eq!(validate_title(""), Err(DemandError::EmptyTitle));
        assert_eq!(validate_title("   "), Err(DemandError::EmptyTitle));
        assert_eq!(
            validate_optional_title(Some("  ")),
            Err(DemandError::EmptyTitle)
        );
    }

    #[test]
    fn should_fail_validation_when_title_exceeds_max_length() {
        let long_title = "t".repeat(MAX_TITLE_LENGTH + 1);
        assert_eq!(
            validate_title(&long_title),
            Err(DemandError::TitleTooLong {
                max: MAX_TITLE_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_category_successfully() {
        assert!(validate_category("Infraestrutura").is_ok());
        assert!(validate_optional_category(None).is_ok());
        assert!(validate_optional_category(Some("Acadêmico")).is_ok());
    }

    #[test]
    fn should_fail_validation_for_empty_or_whitespace_category() {
        assert_eq!(validate_category(""), Err(DemandError::EmptyCategory));
        assert_eq!(validate_category("   "), Err(DemandError::EmptyCategory));
        assert_eq!(
            validate_optional_category(Some(" ")),
            Err(DemandError::EmptyCategory)
        );
    }

    #[test]
    fn should_fail_validation_when_category_exceeds_max_length() {
        let long_cat = "c".repeat(MAX_CATEGORY_LENGTH + 1);
        assert_eq!(
            validate_category(&long_cat),
            Err(DemandError::CategoryTooLong {
                max: MAX_CATEGORY_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_course_name_rules() {
        assert!(validate_course_name("Ciência da Computação").is_ok());
        assert_eq!(validate_course_name(""), Err(DemandError::EmptyCourseName));
        assert_eq!(
            validate_course_name("   "),
            Err(DemandError::EmptyCourseName)
        );

        let long_course = "x".repeat(MAX_COURSE_NAME_LENGTH + 1);
        assert_eq!(
            validate_course_name(&long_course),
            Err(DemandError::CourseNameTooLong {
                max: MAX_COURSE_NAME_LENGTH
            })
        );

        // Espaço físico não pode ter curso
        assert_eq!(
            validate_optional_course_name(Some("Engenharia"), DemandForum::PhysicalSpace),
            Err(DemandError::CourseNotAllowedForPhysicalSpace)
        );
        assert!(validate_optional_course_name(None, DemandForum::PhysicalSpace).is_ok());
        assert!(
            validate_optional_course_name(Some("Engenharia"), DemandForum::Institutional).is_ok()
        );
    }

    #[test]
    fn should_validate_comment_rules() {
        assert!(validate_comment("Em análise com a equipe de patrimônio").is_ok());
        assert!(validate_optional_comment(None).is_ok());
        assert_eq!(validate_comment(""), Err(DemandError::EmptyComment));
        assert_eq!(validate_comment("   "), Err(DemandError::EmptyComment));

        let long_comment = "z".repeat(MAX_COMMENT_LENGTH + 1);
        assert_eq!(
            validate_comment(&long_comment),
            Err(DemandError::CommentTooLong {
                max: MAX_COMMENT_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_timestamps_coherence() {
        let now = Utc::now();
        let future = now + chrono::Duration::hours(1);
        let past = now - chrono::Duration::hours(1);

        assert!(validate_timestamps(now, now).is_ok());
        assert!(validate_timestamps(now, future).is_ok());
        assert_eq!(
            validate_timestamps(now, past),
            Err(DemandError::InconsistentTimestamps {
                created_at: now,
                updated_at: past
            })
        );
    }

    #[test]
    fn should_reject_transition_to_same_status() {
        assert_eq!(
            validate_status_transition(
                DemandForum::PhysicalSpace,
                DemandStatus::Open,
                DemandStatus::Open
            ),
            Err(DemandError::InvalidStatusTransition {
                from: DemandStatus::Open,
                to: DemandStatus::Open,
                forum: DemandForum::PhysicalSpace
            })
        );
    }

    #[test]
    fn should_strictly_forbid_forwarding_physical_space_demands_to_coordination() {
        assert_eq!(
            validate_status_transition(
                DemandForum::PhysicalSpace,
                DemandStatus::Open,
                DemandStatus::ForwardedToCoordination
            ),
            Err(DemandError::CoordinationForwardingNotAllowed)
        );
        assert_eq!(
            validate_status_transition(
                DemandForum::PhysicalSpace,
                DemandStatus::InAnalysis,
                DemandStatus::ForwardedToCoordination
            ),
            Err(DemandError::CoordinationForwardingNotAllowed)
        );
    }

    #[test]
    fn should_allow_valid_physical_space_transitions() {
        assert!(
            validate_status_transition(
                DemandForum::PhysicalSpace,
                DemandStatus::Open,
                DemandStatus::InAnalysis
            )
            .is_ok()
        );
        assert!(
            validate_status_transition(
                DemandForum::PhysicalSpace,
                DemandStatus::InAnalysis,
                DemandStatus::Completed
            )
            .is_ok()
        );
        assert!(
            validate_status_transition(
                DemandForum::PhysicalSpace,
                DemandStatus::Completed,
                DemandStatus::Open
            )
            .is_ok()
        );
    }

    #[test]
    fn should_allow_institutional_transitions_including_coordination() {
        assert!(
            validate_status_transition(
                DemandForum::Institutional,
                DemandStatus::Open,
                DemandStatus::ForwardedToCoordination
            )
            .is_ok()
        );
        assert!(
            validate_status_transition(
                DemandForum::Institutional,
                DemandStatus::InAnalysis,
                DemandStatus::ForwardedToCoordination
            )
            .is_ok()
        );
        assert!(
            validate_status_transition(
                DemandForum::Institutional,
                DemandStatus::ForwardedToCoordination,
                DemandStatus::Completed
            )
            .is_ok()
        );
    }

    #[test]
    fn should_have_identical_trim_behavior_between_rules_and_demand_constructor() {
        use crate::modules::demands::domain::entities::Demand;
        use uuid::Uuid;

        let raw_title = "   Reparo da fechadura   ";
        let raw_desc = "   A porta não fecha adequadamente   ";
        let raw_cat = "   Manutenção   ";
        let raw_course = "   Engenharia de Redes   ";

        assert!(validate_title(raw_title).is_ok());
        assert!(validate_description(raw_desc).is_ok());
        assert!(validate_category(raw_cat).is_ok());
        assert!(validate_course_name(raw_course).is_ok());

        let demand = Demand::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandForum::Institutional,
            Some(raw_title.to_string()),
            raw_desc.to_string(),
            Some(raw_cat.to_string()),
            Some(raw_course.to_string()),
            Utc::now(),
        )
        .expect("demanda deve ser criada com campos sanitizados");

        assert_eq!(demand.title(), Some("Reparo da fechadura"));
        assert_eq!(demand.description(), "A porta não fecha adequadamente");
        assert_eq!(demand.category(), Some("Manutenção"));
        assert_eq!(demand.course_name(), Some("Engenharia de Redes"));
    }
}
