//! Adaptadores de persistência do módulo de notícias.

pub mod in_memory;
pub mod postgres;

pub use in_memory::InMemoryNewsRepository;
pub use postgres::PgNewsRepository;
