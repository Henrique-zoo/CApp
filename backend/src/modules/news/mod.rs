//! Estrutura do módulo de notícias do mural administrado.
//!
//! O escopo abrange proposição, homologação, publicação, arquivamento e compartilhamento
//! de notícias entre Centros Acadêmicos.
//!
//! As camadas estão estruturadas conforme a arquitetura limpa:
//! - [`domain`]: entidades centrais ([`domain::NewsArticle`], [`domain::NewsStatus`]), invariantes e eventos.
//! - [`application`]: casos de uso, comandos, consultas e portas de persistência e autorização.
//! - [`infrastructure`]: adaptadores em memória e PostgreSQL para dados e controle de acesso.
//! - [`api`]: rotas HTTP, controladores (handlers) e DTOs de transferência.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use domain::{NewsArticle, NewsStatus};
