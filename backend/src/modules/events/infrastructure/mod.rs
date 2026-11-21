//! Camada de persistência e autorização do módulo de eventos.
//!
//! [`repository`] disponibiliza os adaptadores PostgreSQL [`PostgresEventRepository`]
//! e [`PostgresEventAuthorizationService`], além do repositório em memória
//! [`InMemoryEventRepository`](crate::modules::events::application::InMemoryEventRepository).

pub mod repository;

pub use crate::modules::events::application::InMemoryEventRepository;
pub use repository::{PostgresEventAuthorizationService, PostgresEventRepository};
