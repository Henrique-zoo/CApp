//! Camada de infraestrutura e persistência do módulo de Centros Acadêmicos.
//!
//! [`repository`] define os contratos e implementações de armazenamento para
//! Centros Acadêmicos e contexto ativo de sessão em memória.

pub mod repository;

pub use repository::{
    AcademicCenterRepository, ActiveContextSessionStore, InMemoryAcademicCenterRepository,
    InMemoryActiveContextSessionStore,
};
