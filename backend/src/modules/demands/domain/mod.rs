//! Camada de domínio prevista para o módulo de demandas.
//!
//! [`entities`], [`rules`] e [`events`] reservarão os modelos, suas invariantes
//! e os fatos relevantes do negócio. Ainda não há regras implementadas;
//! essa camada deve permanecer independente de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;
