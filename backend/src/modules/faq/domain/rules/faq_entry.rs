//! Regras de validação e limites estruturais para perguntas e respostas de FAQ.

use chrono::{DateTime, Utc};

use crate::modules::faq::domain::entities::faq_entry::FaqError;

/// Limite máximo em caracteres para o texto de uma pergunta frequente.
pub const MAX_QUESTION_LENGTH: usize = 500;

/// Limite máximo em caracteres para a resposta de uma pergunta frequente.
pub const MAX_ANSWER_LENGTH: usize = 5000;

/// Limite máximo em caracteres para a categoria de uma pergunta frequente.
pub const MAX_CATEGORY_LENGTH: usize = 100;

/// Valida o texto da pergunta frequente.
///
/// Espaços periféricos são desconsiderados (`trim`).
///
/// # Erros
///
/// Retorna [`FaqError::EmptyQuestion`] se o texto for vazio.
/// Retorna [`FaqError::QuestionTooLong`] se exceder [`MAX_QUESTION_LENGTH`].
pub fn validate_question(question: &str) -> Result<(), FaqError> {
    let trimmed = question.trim();
    if trimmed.is_empty() {
        return Err(FaqError::EmptyQuestion);
    }
    if trimmed.chars().count() > MAX_QUESTION_LENGTH {
        return Err(FaqError::QuestionTooLong {
            max: MAX_QUESTION_LENGTH,
        });
    }
    Ok(())
}

/// Valida a resposta da pergunta frequente.
///
/// Espaços periféricos são desconsiderados (`trim`).
///
/// # Erros
///
/// Retorna [`FaqError::EmptyAnswer`] se a resposta for vazia.
/// Retorna [`FaqError::AnswerTooLong`] se exceder [`MAX_ANSWER_LENGTH`].
pub fn validate_answer(answer: &str) -> Result<(), FaqError> {
    let trimmed = answer.trim();
    if trimmed.is_empty() {
        return Err(FaqError::EmptyAnswer);
    }
    if trimmed.chars().count() > MAX_ANSWER_LENGTH {
        return Err(FaqError::AnswerTooLong {
            max: MAX_ANSWER_LENGTH,
        });
    }
    Ok(())
}

/// Valida a categoria temática opcional.
///
/// Se informada, não pode ser vazia após `trim` e não pode exceder [`MAX_CATEGORY_LENGTH`].
///
/// # Erros
///
/// Retorna [`FaqError::EmptyCategory`] se a categoria fornecida for composta apenas por espaços.
/// Retorna [`FaqError::CategoryTooLong`] se exceder [`MAX_CATEGORY_LENGTH`].
pub fn validate_category(category: Option<&str>) -> Result<Option<String>, FaqError> {
    match category {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(FaqError::EmptyCategory);
            }
            if trimmed.chars().count() > MAX_CATEGORY_LENGTH {
                return Err(FaqError::CategoryTooLong {
                    max: MAX_CATEGORY_LENGTH,
                });
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

/// Valida a coerência temporal entre os instantes de criação e última atualização.
///
/// # Erros
///
/// Retorna [`FaqError::InconsistentTimestamps`] se `updated_at` for anterior a `created_at`.
pub fn validate_timestamps(
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), FaqError> {
    if updated_at < created_at {
        return Err(FaqError::InconsistentTimestamps {
            created_at,
            updated_at,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_question_correctly() {
        assert!(validate_question("Como funciona?").is_ok());
        assert_eq!(validate_question("   "), Err(FaqError::EmptyQuestion));
        assert_eq!(
            validate_question(&"A".repeat(MAX_QUESTION_LENGTH + 1)),
            Err(FaqError::QuestionTooLong {
                max: MAX_QUESTION_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_answer_correctly() {
        assert!(validate_answer("Resposta explicativa.").is_ok());
        assert_eq!(validate_answer("   "), Err(FaqError::EmptyAnswer));
        assert_eq!(
            validate_answer(&"B".repeat(MAX_ANSWER_LENGTH + 1)),
            Err(FaqError::AnswerTooLong {
                max: MAX_ANSWER_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_category_correctly() {
        assert_eq!(validate_category(None), Ok(None));
        assert_eq!(
            validate_category(Some("  Geral  ")),
            Ok(Some("Geral".to_string()))
        );
        assert_eq!(validate_category(Some("   ")), Err(FaqError::EmptyCategory));
        assert_eq!(
            validate_category(Some(&"C".repeat(MAX_CATEGORY_LENGTH + 1))),
            Err(FaqError::CategoryTooLong {
                max: MAX_CATEGORY_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_timestamps_correctly() {
        let now = Utc::now();
        assert!(validate_timestamps(now, now).is_ok());
        let later = now + chrono::Duration::seconds(5);
        assert!(validate_timestamps(now, later).is_ok());
        let earlier = now - chrono::Duration::seconds(5);
        assert_eq!(
            validate_timestamps(now, earlier),
            Err(FaqError::InconsistentTimestamps {
                created_at: now,
                updated_at: earlier
            })
        );
    }
}
