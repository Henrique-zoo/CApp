//! Camada de persistência para o módulo de Centros Acadêmicos.
//!
//! [`repository`] define os adaptadores PostgreSQL e em memória
//! para Centros Acadêmicos e mandatos de gestão.

pub mod repository;
pub use repository::*;
