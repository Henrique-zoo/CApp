//! Consultas e manipuladores de leitura do módulo de notícias.

pub mod get_article;
pub mod list_articles;

pub use get_article::{GetNewsArticleHandler, GetNewsArticleQuery};
pub use list_articles::{ListNewsArticlesByCaHandler, ListNewsArticlesByCaQuery};
