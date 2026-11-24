//! Entidades do módulo de notícias.
//!
//! Reúne os modelos de domínio de notícias, seu ciclo de vida editorial,
//! invariantes e integridade estrutural.

pub mod news_article;
pub use news_article::News;
pub use news_article::NewsArticle;
pub use news_article::NewsArticleError;
pub use news_article::NewsError;
pub use news_article::NewsPost;
pub use news_article::NewsStatus;
pub use news_article::Noticia;
pub use news_article::NoticiaError;
pub use news_article::StatusNoticia;
pub use article::NewsArticle;
pub use status::NewsStatus;
pub mod article;
pub mod status;
