//! Camada de domínio do módulo de notícias.
//!
//! [`entities`], [`rules`] e [`events`] definem os modelos, suas invariantes
//! e os fatos relevantes do negócio. Essa camada permanece independente de HTTP
//! e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;
