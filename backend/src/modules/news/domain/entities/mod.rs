//! Entidades do módulo de notícias.
//!
//! Contém as definições do modelo de dados do domínio, incluindo
//! os artigos de notícia e seus respectivos estados editoriais.

pub mod news_article;
pub mod news_status;

pub use news_article::NewsArticle;
pub use news_status::NewsStatus;
