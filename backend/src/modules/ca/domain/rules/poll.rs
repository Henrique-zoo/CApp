//! Regras de negócio e validações para consultas e enquetes estudantis do CA.
//!
//! Centraliza as constantes de limite, funções de validação de formato e
//! invariantes de ciclo de vida aplicáveis às consultas e enquetes estudantis.

use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::modules::ca::domain::entities::PollError;

/// Limite máximo em caracteres para o título da consulta/enquete.
pub const MAX_POLL_TITLE_LENGTH: usize = 255;

/// Limite máximo em caracteres para a descrição ou proposta da consulta.
pub const MAX_POLL_DESCRIPTION_LENGTH: usize = 4000;

/// Número mínimo de opções distintas exigidas para uma enquete.
pub const MIN_POLL_OPTIONS: usize = 2;

/// Número máximo de opções permitidas em uma enquete.
pub const MAX_POLL_OPTIONS: usize = 50;

/// Limite máximo em caracteres para o texto de cada opção de voto.
pub const MAX_OPTION_TITLE_LENGTH: usize = 255;

/// Valida o título da consulta/enquete estudantil.
///
/// # Erros
///
/// Retorna [`PollError::EmptyTitle`] se o título for vazio ou composto apenas por espaços.
/// Retorna [`PollError::TitleTooLong`] se o título sanitizado exceder [`MAX_POLL_TITLE_LENGTH`].
pub fn validate_poll_title(title: &str) -> Result<(), PollError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(PollError::EmptyTitle);
    }
    if trimmed.chars().count() > MAX_POLL_TITLE_LENGTH {
        return Err(PollError::TitleTooLong {
            max: MAX_POLL_TITLE_LENGTH,
        });
    }
    Ok(())
}

/// Valida a descrição ou proposta da consulta estudantil.
///
/// # Erros
///
/// Retorna [`PollError::DescriptionTooLong`] se a descrição exceder [`MAX_POLL_DESCRIPTION_LENGTH`].
pub fn validate_poll_description(description: Option<&str>) -> Result<(), PollError> {
    if let Some(desc) = description {
        let trimmed = desc.trim();
        if trimmed.chars().count() > MAX_POLL_DESCRIPTION_LENGTH {
            return Err(PollError::DescriptionTooLong {
                max: MAX_POLL_DESCRIPTION_LENGTH,
            });
        }
    }
    Ok(())
}

/// Valida a lista de opções de uma enquete estudantil.
///
/// Exige ao menos [`MIN_POLL_OPTIONS`] opções não vazias e distintas entre si.
///
/// # Erros
///
/// Retorna [`PollError::TooFewOptions`] se houver menos de [`MIN_POLL_OPTIONS`].
/// Retorna [`PollError::EmptyOptionTitle`] se alguma opção for vazia.
/// Retorna [`PollError::OptionTitleTooLong`] se alguma opção exceder o limite.
/// Retorna [`PollError::DuplicateOptionTitles`] se houver opções com títulos idênticos.
pub fn validate_poll_options(options: &[String]) -> Result<(), PollError> {
    if options.len() < MIN_POLL_OPTIONS {
        return Err(PollError::TooFewOptions {
            min: MIN_POLL_OPTIONS,
            found: options.len(),
        });
    }

    let mut seen = HashSet::new();

    for option in options {
        let trimmed = option.trim();
        if trimmed.is_empty() {
            return Err(PollError::EmptyOptionTitle);
        }
        if trimmed.chars().count() > MAX_OPTION_TITLE_LENGTH {
            return Err(PollError::OptionTitleTooLong {
                max: MAX_OPTION_TITLE_LENGTH,
            });
        }
        let lower = trimmed.to_lowercase();
        if !seen.insert(lower) {
            return Err(PollError::DuplicateOptionTitles(trimmed.to_string()));
        }
    }

    Ok(())
}

/// Valida o período temporal de votação da consulta estudantil.
///
/// Quando `ends_at` for informado, deve ser estritamente posterior a `starts_at`.
///
/// # Erros
///
/// Retorna [`PollError::InvalidPeriod`] se `ends_at` for menor ou igual a `starts_at`.
pub fn validate_poll_period(
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
) -> Result<(), PollError> {
    if let Some(end) = ends_at.filter(|&end| end <= starts_at) {
        return Err(PollError::InvalidPeriod {
            starts_at,
            ends_at: end,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn should_validate_valid_title() {
        assert!(validate_poll_title("Consulta sobre horários").is_ok());
    }

    #[test]
    fn should_fail_when_title_is_empty_or_whitespace() {
        assert_eq!(validate_poll_title(""), Err(PollError::EmptyTitle));
        assert_eq!(validate_poll_title("   "), Err(PollError::EmptyTitle));
    }

    #[test]
    fn should_fail_when_title_is_too_long() {
        let long_title = "a".repeat(MAX_POLL_TITLE_LENGTH + 1);
        assert_eq!(
            validate_poll_title(&long_title),
            Err(PollError::TitleTooLong {
                max: MAX_POLL_TITLE_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_description_correctly() {
        assert!(validate_poll_description(None).is_ok());
        assert!(validate_poll_description(Some("Descrição")).is_ok());

        let long_desc = "a".repeat(MAX_POLL_DESCRIPTION_LENGTH + 1);
        assert_eq!(
            validate_poll_description(Some(&long_desc)),
            Err(PollError::DescriptionTooLong {
                max: MAX_POLL_DESCRIPTION_LENGTH
            })
        );
    }

    #[test]
    fn should_validate_options_correctly() {
        let valid_options = vec![
            "Apoio a proposta".to_string(),
            "Não apoio a proposta".to_string(),
        ];
        assert!(validate_poll_options(&valid_options).is_ok());

        let single_option = vec!["Apoio".to_string()];
        assert_eq!(
            validate_poll_options(&single_option),
            Err(PollError::TooFewOptions { min: 2, found: 1 })
        );

        let with_empty = vec!["Apoio".to_string(), "  ".to_string()];
        assert_eq!(
            validate_poll_options(&with_empty),
            Err(PollError::EmptyOptionTitle)
        );

        let duplicate = vec!["Apoio".to_string(), "apoio".to_string()];
        assert!(matches!(
            validate_poll_options(&duplicate),
            Err(PollError::DuplicateOptionTitles(_))
        ));
    }

    #[test]
    fn should_validate_period_correctly() {
        let now = Utc::now();
        assert!(validate_poll_period(now, None).is_ok());
        assert!(validate_poll_period(now, Some(now + Duration::days(5))).is_ok());
        assert!(matches!(
            validate_poll_period(now, Some(now - Duration::days(1))),
            Err(PollError::InvalidPeriod { .. })
        ));
        assert!(matches!(
            validate_poll_period(now, Some(now)),
            Err(PollError::InvalidPeriod { .. })
        ));
    }
}
