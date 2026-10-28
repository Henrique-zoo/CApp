//! Camada de persistência e autorização do módulo de eventos.
//!
//! [`repository`] disponibiliza os adaptadores PostgreSQL [`PostgresEventRepository`]
//! e [`PostgresEventAuthorizationService`].

pub mod repository;

pub use repository::{PostgresEventAuthorizationService, PostgresEventRepository};
