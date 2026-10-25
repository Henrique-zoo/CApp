//! Regras de validação e invariantes do módulo de notícias.

use crate::modules::news::domain::NewsError;

/// Tamanho mínimo permitido para o título de uma notícia (em caracteres).
pub const MIN_TITLE_LENGTH: usize = 3;

/// Tamanho máximo permitido para o título de uma notícia (em caracteres).
pub const MAX_TITLE_LENGTH: usize = 200;

/// Tamanho mínimo permitido para o corpo de uma notícia (em caracteres).
pub const MIN_BODY_LENGTH: usize = 1;

/// Tamanho máximo permitido para o corpo de uma notícia (em caracteres).
pub const MAX_BODY_LENGTH: usize = 50_000;

/// Valida o título de uma notícia segundo as invariantes de negócio.
pub fn validate_title(title: &str) -> Result<(), NewsError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(NewsError::EmptyTitle);
    }
    let count = trimmed.chars().count();
    if count < MIN_TITLE_LENGTH {
        return Err(NewsError::TitleTooShort {
            min: MIN_TITLE_LENGTH,
            actual: count,
        });
    }
    if count > MAX_TITLE_LENGTH {
        return Err(NewsError::TitleTooLong {
            max: MAX_TITLE_LENGTH,
            actual: count,
        });
    }
    Ok(())
}

/// Valida o corpo de uma notícia segundo as invariantes de negócio.
pub fn validate_body(body: &str) -> Result<(), NewsError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(NewsError::EmptyBody);
    }
    let count = trimmed.chars().count();
    if count < MIN_BODY_LENGTH {
        return Err(NewsError::EmptyBody);
    }
    if count > MAX_BODY_LENGTH {
        return Err(NewsError::BodyTooLong {
            max: MAX_BODY_LENGTH,
            actual: count,
        });
    }
    Ok(())
}
