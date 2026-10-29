//! Camada de domínio do módulo de Centros Acadêmicos.
//!
//! Reúne entidades, regras de negócio e eventos de governança e autorização do CA.

pub mod entities;
pub mod events;
pub mod rules;

pub use entities::*;
pub use rules::*;
