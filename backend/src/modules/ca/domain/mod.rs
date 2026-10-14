//! Camada de domínio do módulo de Centros Acadêmicos.
//!
//! [`rules`] e [`events`] definem as regras, limites e fatos relevantes
//! do negócio. Essa camada permanece independente de HTTP e banco de dados.

mod entities;
pub mod events;
pub mod rules;

pub use entities::*;
