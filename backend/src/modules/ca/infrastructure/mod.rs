//! Camada de persistência do módulo de Centros Acadêmicos.
//!
//! [`repository`] contém os adaptadores de persistência PostgreSQL e em memória.

pub mod repository;

pub use repository::*;
