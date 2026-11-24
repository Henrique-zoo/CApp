//! Camada de infraestrutura do módulo de notícias.
//!
//! Contém os repositórios em memória e PostgreSQL, bem como os adaptadores
//! para resolução de permissões e governança nos Centros Acadêmicos.

pub mod repository;

pub use repository::{
    AllowAllNewsAuthorization, InMemoryNewsRepository, PgNewsRepository,
    StaticRoleNewsAuthorization,
};
