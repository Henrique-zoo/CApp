//! Regras de negócio e validações para o domínio de notícias.
//!
//! Centraliza as constantes de limite, funções de validação de formato e
//! invariantes de ciclo de vida aplicáveis a notícias e publicações no mural.

use chrono::{DateTime, Utc};

use crate::modules::news::domain::entities::{NewsArticle, NewsError, NewsStatus};

/// Limite mínimo em caracteres para o título da notícia.
pub const MIN_TITLE_LENGTH: usize = 3;

/// Limite máximo em caracteres para o título da notícia.
pub const MAX_TITLE_LENGTH: usize = 255;

/// Limite mínimo em caracteres para o corpo/conteúdo da notícia.
pub const MIN_BODY_LENGTH: usize = 5;

/// Limite máximo em caracteres para o corpo/conteúdo da notícia.
pub const MAX_BODY_LENGTH: usize = 50_000;

/// Limite máximo de tags/categorias associadas a uma notícia.
pub const MAX_TAGS_COUNT: usize = 10;

/// Limite máximo em caracteres para cada tag/categoria individual.
pub const MAX_TAG_LENGTH: usize = 50;

/// Valida o título da notícia conforme os critérios e limites de domínio.
///
/// O título não pode ser vazio nem composto exclusivamente por espaços em branco,
/// devendo ter entre [`MIN_TITLE_LENGTH`] e [`MAX_TITLE_LENGTH`] caracteres.
///
/// # Erros
///
/// - [`NewsError::EmptyTitle`]: se o título for vazio após remoção de espaços em branco.
/// - [`NewsError::TitleTooShort`]: se o comprimento for inferior a [`MIN_TITLE_LENGTH`].
/// - [`NewsError::TitleTooLong`]: se o comprimento exceder [`MAX_TITLE_LENGTH`].
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

/// Valida o corpo/conteúdo da notícia conforme os limites de negócio.
///
/// O corpo não pode ser vazio nem composto exclusivamente por espaços em branco,
/// devendo ter entre [`MIN_BODY_LENGTH`] e [`MAX_BODY_LENGTH`] caracteres.
///
/// # Erros
///
/// - [`NewsError::EmptyBody`]: se o corpo for vazio após remoção de espaços em branco.
/// - [`NewsError::BodyTooShort`]: se o comprimento for inferior a [`MIN_BODY_LENGTH`].
/// - [`NewsError::BodyTooLong`]: se o comprimento exceder [`MAX_BODY_LENGTH`].
pub fn validate_body(body: &str) -> Result<(), NewsError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(NewsError::EmptyBody);
    }
    let count = trimmed.chars().count();
    if count < MIN_BODY_LENGTH {
        return Err(NewsError::BodyTooShort {
            min: MIN_BODY_LENGTH,
            actual: count,
        });
    }
    if count > MAX_BODY_LENGTH {
        return Err(NewsError::BodyTooLong {
            max: MAX_BODY_LENGTH,
            actual: count,
        });
    }
    Ok(())
}

/// Valida a lista de tags/categorias da notícia.
///
/// # Erros
///
/// - [`NewsError::TooManyTags`]: se o número de tags exceder [`MAX_TAGS_COUNT`].
/// - [`NewsError::EmptyTag`]: se qualquer tag for vazia ou contiver apenas espaços.
/// - [`NewsError::TagTooLong`]: se qualquer tag exceder [`MAX_TAG_LENGTH`] caracteres.
pub fn validate_tags(tags: &[String]) -> Result<(), NewsError> {
    if tags.len() > MAX_TAGS_COUNT {
        return Err(NewsError::TooManyTags {
            max: MAX_TAGS_COUNT,
            actual: tags.len(),
        });
    }
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return Err(NewsError::EmptyTag);
        }
        let count = trimmed.chars().count();
        if count > MAX_TAG_LENGTH {
            return Err(NewsError::TagTooLong {
                max: MAX_TAG_LENGTH,
                actual: count,
            });
        }
    }
    Ok(())
}

/// Valida a coerência temporal entre os instantes de criação e atualização.
///
/// # Erros
///
/// Retorna [`NewsError::InconsistentTimestamps`] se `updated_at` for anterior a `created_at`.
pub fn validate_timestamps(
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), NewsError> {
    if updated_at < created_at {
        return Err(NewsError::InconsistentTimestamps {
            created_at,
            updated_at,
        });
    }
    Ok(())
}

/// Valida a coerência do instante de publicação com o status da notícia.
///
/// Notícias com status [`NewsStatus::Published`] exigem `published_at` definido e
/// igual ou posterior a `created_at`.
///
/// Notícias com status [`NewsStatus::Draft`] ou [`NewsStatus::PendingReview`] não podem
/// possuir data de publicação associada.
///
/// # Erros
///
/// - [`NewsError::PublishedWithoutDate`]: se a notícia estiver como publicada sem `published_at`.
/// - [`NewsError::UnpublishedWithDate`]: se rascunho ou pendente de revisão possuir `published_at`.
/// - [`NewsError::InconsistentPublicationDate`]: se `published_at` for anterior a `created_at`.
pub fn validate_publication_date(
    status: NewsStatus,
    created_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
) -> Result<(), NewsError> {
    match status {
        NewsStatus::Published => match published_at {
            Some(published) => {
                if published < created_at {
                    return Err(NewsError::InconsistentPublicationDate {
                        created_at,
                        published_at: published,
                    });
                }
                Ok(())
            }
            None => Err(NewsError::PublishedWithoutDate),
        },
        NewsStatus::Draft | NewsStatus::PendingReview => {
            if published_at.is_some() {
                return Err(NewsError::UnpublishedWithDate);
            }
            Ok(())
        }
        NewsStatus::Rejected | NewsStatus::Archived => {
            if let Some(published) = published_at
                && published < created_at
            {
                return Err(NewsError::InconsistentPublicationDate {
                    created_at,
                    published_at: published,
                });
            }
            Ok(())
        }
    }
}

/// Valida a coerência do instante de arquivamento com o status da notícia.
///
/// # Erros
///
/// - [`NewsError::ArchivedWithoutDate`]: se a notícia for arquivada sem `archived_at`.
/// - [`NewsError::InconsistentArchiveDate`]: se `archived_at` for anterior a `created_at`.
pub fn validate_archive_date(
    status: NewsStatus,
    created_at: DateTime<Utc>,
    archived_at: Option<DateTime<Utc>>,
) -> Result<(), NewsError> {
    match status {
        NewsStatus::Archived => match archived_at {
            Some(archived) => {
                if archived < created_at {
                    return Err(NewsError::InconsistentArchiveDate {
                        created_at,
                        archived_at: archived,
                    });
                }
                Ok(())
            }
            None => Err(NewsError::ArchivedWithoutDate),
        },
        _ => {
            if let Some(archived) = archived_at
                && archived < created_at
            {
                return Err(NewsError::InconsistentArchiveDate {
                    created_at,
                    archived_at: archived,
                });
            }
            Ok(())
        }
    }
}

/// Valida se uma transição de status no ciclo de vida editorial é permitida.
///
/// Transições para o mesmo estado são consideradas idempotentes e válidas.
///
/// # Regras de transição
///
/// - `Draft` -> `PendingReview`, `Published`, `Archived`
/// - `PendingReview` -> `Published`, `Rejected`, `Draft`, `Archived`
/// - `Published` -> `Archived`
/// - `Rejected` -> `Draft`, `Archived`
/// - `Archived` é terminal: nenhuma transição a partir de `Archived` para outro estado é permitida.
///
/// # Erros
///
/// Retorna [`NewsError::InvalidStatusTransition`] quando a transição for proibida.
pub fn validate_status_transition(from: NewsStatus, to: NewsStatus) -> Result<(), NewsError> {
    if from == to {
        return Ok(());
    }

    match (from, to) {
        (NewsStatus::Draft, NewsStatus::PendingReview)
        | (NewsStatus::Draft, NewsStatus::Published)
        | (NewsStatus::Draft, NewsStatus::Archived) => Ok(()),

        (NewsStatus::PendingReview, NewsStatus::Published)
        | (NewsStatus::PendingReview, NewsStatus::Rejected)
        | (NewsStatus::PendingReview, NewsStatus::Draft)
        | (NewsStatus::PendingReview, NewsStatus::Archived) => Ok(()),

        (NewsStatus::Published, NewsStatus::Archived) => Ok(()),

        (NewsStatus::Rejected, NewsStatus::Draft)
        | (NewsStatus::Rejected, NewsStatus::Archived) => Ok(()),

        _ => Err(NewsError::InvalidStatusTransition { from, to }),
    }
}

/// Valida todas as invariantes e a integridade estrutural de uma notícia.
///
/// # Erros
///
/// Retorna [`NewsError`] se qualquer regra de título, corpo, tags, timestamps,
/// publicação ou arquivamento for violada.
pub fn validate_news(news: &NewsArticle) -> Result<(), NewsError> {
    validate_title(&news.title)?;
    validate_body(&news.body)?;
    validate_tags(&news.tags)?;
    validate_timestamps(news.created_at, news.updated_at)?;
    validate_publication_date(news.status, news.created_at, news.published_at)?;
    validate_archive_date(news.status, news.created_at, news.archived_at)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_title_bounds() {
        assert!(validate_title("UnB").is_ok());
        assert!(validate_title("Assembleia geral discente").is_ok());
        assert!(validate_title(&"A".repeat(MAX_TITLE_LENGTH)).is_ok());

        assert_eq!(validate_title(""), Err(NewsError::EmptyTitle));
        assert_eq!(validate_title("   "), Err(NewsError::EmptyTitle));
        assert_eq!(
            validate_title("AB"),
            Err(NewsError::TitleTooShort {
                min: MIN_TITLE_LENGTH,
                actual: 2
            })
        );
        let too_long = "A".repeat(MAX_TITLE_LENGTH + 1);
        assert_eq!(
            validate_title(&too_long),
            Err(NewsError::TitleTooLong {
                max: MAX_TITLE_LENGTH,
                actual: MAX_TITLE_LENGTH + 1
            })
        );
    }

    #[test]
    fn should_validate_body_bounds() {
        assert!(validate_body("Texto").is_ok());
        assert!(validate_body("Pauta detalhada da próxima assembleia geral").is_ok());
        assert!(validate_body(&"B".repeat(MAX_BODY_LENGTH)).is_ok());

        assert_eq!(validate_body(""), Err(NewsError::EmptyBody));
        assert_eq!(validate_body("   "), Err(NewsError::EmptyBody));
        assert_eq!(
            validate_body("1234"),
            Err(NewsError::BodyTooShort {
                min: MIN_BODY_LENGTH,
                actual: 4
            })
        );
        let too_long = "B".repeat(MAX_BODY_LENGTH + 1);
        assert_eq!(
            validate_body(&too_long),
            Err(NewsError::BodyTooLong {
                max: MAX_BODY_LENGTH,
                actual: MAX_BODY_LENGTH + 1
            })
        );
    }

    #[test]
    fn should_validate_tags() {
        let valid_tags = vec!["informativo".to_string(), "editais".to_string()];
        assert!(validate_tags(&valid_tags).is_ok());

        let empty_tags: Vec<String> = Vec::new();
        assert!(validate_tags(&empty_tags).is_ok());

        let too_many_tags: Vec<String> = (0..=MAX_TAGS_COUNT).map(|i| format!("tag{i}")).collect();
        assert_eq!(
            validate_tags(&too_many_tags),
            Err(NewsError::TooManyTags {
                max: MAX_TAGS_COUNT,
                actual: MAX_TAGS_COUNT + 1
            })
        );

        let with_empty_tag = vec!["tag1".to_string(), "   ".to_string()];
        assert_eq!(validate_tags(&with_empty_tag), Err(NewsError::EmptyTag));

        let tag_too_long = vec!["T".repeat(MAX_TAG_LENGTH + 1)];
        assert_eq!(
            validate_tags(&tag_too_long),
            Err(NewsError::TagTooLong {
                max: MAX_TAG_LENGTH,
                actual: MAX_TAG_LENGTH + 1
            })
        );
    }

    #[test]
    fn should_validate_timestamps() {
        let now = Utc::now();
        let later = now + chrono::Duration::minutes(5);
        let past = now - chrono::Duration::minutes(5);

        assert!(validate_timestamps(now, now).is_ok());
        assert!(validate_timestamps(now, later).is_ok());
        assert_eq!(
            validate_timestamps(now, past),
            Err(NewsError::InconsistentTimestamps {
                created_at: now,
                updated_at: past
            })
        );
    }

    #[test]
    fn should_validate_publication_date() {
        let now = Utc::now();
        let past = now - chrono::Duration::minutes(10);
        let later = now + chrono::Duration::minutes(10);

        // Publicada precisa de data válida
        assert!(validate_publication_date(NewsStatus::Published, now, Some(now)).is_ok());
        assert!(validate_publication_date(NewsStatus::Published, now, Some(later)).is_ok());
        assert_eq!(
            validate_publication_date(NewsStatus::Published, now, None),
            Err(NewsError::PublishedWithoutDate)
        );
        assert_eq!(
            validate_publication_date(NewsStatus::Published, now, Some(past)),
            Err(NewsError::InconsistentPublicationDate {
                created_at: now,
                published_at: past
            })
        );

        // Rascunho e Pendente de Revisão não podem ter data de publicação
        assert!(validate_publication_date(NewsStatus::Draft, now, None).is_ok());
        assert_eq!(
            validate_publication_date(NewsStatus::Draft, now, Some(now)),
            Err(NewsError::UnpublishedWithDate)
        );
        assert!(validate_publication_date(NewsStatus::PendingReview, now, None).is_ok());
        assert_eq!(
            validate_publication_date(NewsStatus::PendingReview, now, Some(now)),
            Err(NewsError::UnpublishedWithDate)
        );
    }

    #[test]
    fn should_validate_archive_date() {
        let now = Utc::now();
        let past = now - chrono::Duration::minutes(10);
        let later = now + chrono::Duration::minutes(10);

        assert!(validate_archive_date(NewsStatus::Archived, now, Some(now)).is_ok());
        assert!(validate_archive_date(NewsStatus::Archived, now, Some(later)).is_ok());
        assert_eq!(
            validate_archive_date(NewsStatus::Archived, now, None),
            Err(NewsError::ArchivedWithoutDate)
        );
        assert_eq!(
            validate_archive_date(NewsStatus::Archived, now, Some(past)),
            Err(NewsError::InconsistentArchiveDate {
                created_at: now,
                archived_at: past
            })
        );

        assert!(validate_archive_date(NewsStatus::Draft, now, None).is_ok());
    }

    #[test]
    fn should_validate_status_transitions() {
        // Idempotentes
        for status in [
            NewsStatus::Draft,
            NewsStatus::PendingReview,
            NewsStatus::Published,
            NewsStatus::Rejected,
            NewsStatus::Archived,
        ] {
            assert!(validate_status_transition(status, status).is_ok());
        }

        // Válidas
        assert!(validate_status_transition(NewsStatus::Draft, NewsStatus::PendingReview).is_ok());
        assert!(validate_status_transition(NewsStatus::Draft, NewsStatus::Published).is_ok());
        assert!(validate_status_transition(NewsStatus::Draft, NewsStatus::Archived).is_ok());

        assert!(
            validate_status_transition(NewsStatus::PendingReview, NewsStatus::Published).is_ok()
        );
        assert!(
            validate_status_transition(NewsStatus::PendingReview, NewsStatus::Rejected).is_ok()
        );
        assert!(validate_status_transition(NewsStatus::PendingReview, NewsStatus::Draft).is_ok());
        assert!(
            validate_status_transition(NewsStatus::PendingReview, NewsStatus::Archived).is_ok()
        );

        assert!(validate_status_transition(NewsStatus::Published, NewsStatus::Archived).is_ok());

        assert!(validate_status_transition(NewsStatus::Rejected, NewsStatus::Draft).is_ok());
        assert!(validate_status_transition(NewsStatus::Rejected, NewsStatus::Archived).is_ok());

        // Proibidas a partir de arquivada
        assert_eq!(
            validate_status_transition(NewsStatus::Archived, NewsStatus::Draft),
            Err(NewsError::InvalidStatusTransition {
                from: NewsStatus::Archived,
                to: NewsStatus::Draft
            })
        );
        assert_eq!(
            validate_status_transition(NewsStatus::Archived, NewsStatus::Published),
            Err(NewsError::InvalidStatusTransition {
                from: NewsStatus::Archived,
                to: NewsStatus::Published
            })
        );

        // Outras proibidas
        assert_eq!(
            validate_status_transition(NewsStatus::Published, NewsStatus::Draft),
            Err(NewsError::InvalidStatusTransition {
                from: NewsStatus::Published,
                to: NewsStatus::Draft
            })
        );
        assert_eq!(
            validate_status_transition(NewsStatus::Rejected, NewsStatus::Published),
            Err(NewsError::InvalidStatusTransition {
                from: NewsStatus::Rejected,
                to: NewsStatus::Published
            })
        );
    }
}
