//! Camada de infraestrutura e persistência do módulo de notícias.

pub mod authorization;
pub mod repository;

pub use authorization::InMemoryNewsAuthorizationService;
pub use repository::{InMemoryNewsRepository, PgNewsRepository};
