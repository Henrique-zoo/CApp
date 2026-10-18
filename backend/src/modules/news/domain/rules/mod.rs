//! Regras de negócio e validações do módulo de notícias.
//!
//! Centraliza as invariantes, limites estruturais e funções de validação
//! aplicáveis às notícias e suas transições de estado.

pub mod news_rules;

pub use news_rules::{
    MAX_BODY_LENGTH, MAX_TAG_LENGTH, MAX_TAGS_COUNT, MAX_TITLE_LENGTH, MIN_BODY_LENGTH,
    MIN_TITLE_LENGTH, validate_archive_date, validate_body, validate_news,
    validate_publication_date, validate_status_transition, validate_tags, validate_timestamps,
    validate_title,
};
