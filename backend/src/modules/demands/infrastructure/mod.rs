//! Camada de persistência do módulo de demandas.
//!
//! Contém implementações PostgreSQL e adaptadores em memória para repositórios
//! e portas de integração.

mod repository;

pub use repository::*;
