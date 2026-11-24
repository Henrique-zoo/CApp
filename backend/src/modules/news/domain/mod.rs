//! Camada de domínio do módulo de notícias.
//!
//! Encapsula entidades, regras de negócio e eventos de ciclo de vida
//! relacionados à publicação e gestão de notícias nos Centros Acadêmicos.
//!
//! Esta camada é estritamente isolada de detalhes de transporte (HTTP)
//! e de mecanismos de armazenamento (SQL / persistência).

pub mod entities;
pub mod events;
pub mod rules;

pub use entities::{NewsArticle, NewsStatus};
pub use events::{
    NewsArticleArchivedEvent, NewsArticlePublishedEvent, NewsArticleSubmittedForReviewEvent,
};
pub use rules::NewsDomainError;
