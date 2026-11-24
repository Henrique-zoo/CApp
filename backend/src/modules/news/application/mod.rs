//! Camada de aplicação do módulo de notícias.
//!
//! Organiza os casos de uso de alteração de estado ([`commands`]),
//! consultas ([`queries`]), contratos abstratos ([`ports`]) e
//! a coordenação desses fluxos ([`services`]).

pub mod commands;
pub mod ports;
pub mod queries;
pub mod services;

pub use commands::{
    CreateNewsArticleCommand, CreateNewsArticleHandler, UpdateNewsArticleCommand,
    UpdateNewsArticleHandler,
};
pub use ports::{NewsAuthorizationPort, NewsRepository};
pub use queries::{
    GetNewsArticleHandler, GetNewsArticleQuery, ListNewsArticlesByCaHandler,
    ListNewsArticlesByCaQuery,
};
pub use services::NewsService;
