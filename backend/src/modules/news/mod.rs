//! Módulo de notícias do CApp.
//!
//! Fornece recursos para publicação, homologação, edição, consulta e
//! compartilhamento de notícias entre Centros Acadêmicos.
//!
//! # Arquitetura
//!
//! - [`domain`]: modelos puros de negócio ([`domain::NewsArticle`], [`domain::NewsStatus`]),
//!   regras de validação e eventos de domínio.
//! - [`application`]: casos de uso de escrita ([`application::CreateNewsArticleCommand`],
//!   [`application::UpdateNewsArticleCommand`]), consultas, portas abstratas
//!   ([`application::NewsRepository`], [`application::NewsAuthorizationPort`])
//!   e o serviço orquestrador ([`application::NewsService`]).
//! - [`infrastructure`]: implementações concretas de persistência
//!   ([`infrastructure::InMemoryNewsRepository`], [`infrastructure::PgNewsRepository`])
//!   e adaptadores de autorização.
//! - [`api`]: contratos de transporte HTTP ([`api::dto`]), manipuladores de rotas
//!   ([`api::handlers`]) e montagem do roteador ([`api::news_router`]).

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::{
    CreateNewsArticleCommand, CreateNewsArticleHandler, NewsAuthorizationPort, NewsRepository,
    NewsService, UpdateNewsArticleCommand, UpdateNewsArticleHandler,
};
pub use domain::{NewsArticle, NewsError, NewsStatus};
