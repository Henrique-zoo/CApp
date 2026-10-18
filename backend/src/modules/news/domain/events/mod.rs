//! Eventos de domínio do módulo de notícias.
//!
//! Reúne os fatos relevantes e imutáveis ocorridos no ciclo de vida
//! editorial das notícias.

pub mod news_events;

pub use news_events::{
    NewsArchived, NewsContentUpdated, NewsCreated, NewsDomainEvent, NewsPublished, NewsRejected,
    NewsSubmittedForReview,
};
