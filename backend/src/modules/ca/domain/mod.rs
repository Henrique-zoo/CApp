//! Camada de domínio prevista para o módulo de Centros Acadêmicos.
//!
//! [`entities`], [`rules`] e [`events`] reservam os modelos, suas invariantes,
//! limites e fatos relevantes do negócio. Essa camada permanece independente
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;
