//! Infraestrutura do módulo de identidade institucional.
//!
//! Contém adaptadores de banco de dados e persistência para as entidades do módulo.

pub mod repository;

pub use repository::{
    InMemoryUserPreferenceRepository, PgUserPreferenceRepository, UserPreferenceRepository,
    UserPreferenceRepositoryError,
};
