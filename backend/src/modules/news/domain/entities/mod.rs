//! Entidades do módulo de notícias.
//!
//! Reúne os modelos de domínio de notícias, seu ciclo de vida editorial,
//! invariantes e integridade estrutural.

pub mod news_article;

pub use news_article::{
    News, NewsArticle, NewsArticleError, NewsError, NewsPost, NewsStatus, Noticia, NoticiaError,
    StatusNoticia,
};
