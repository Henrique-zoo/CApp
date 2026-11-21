//! Regras e primitivas do domínio de eleições.
//!
//! [`cryptography`] implementa as operações matemáticas do protocolo eleitoral
//! e usa as falhas definidas em [`error`]. [`entities`], [`rules`] e [`events`]
//! reservam espaço para os modelos, suas invariantes e os fatos relevantes
//! do negócio.

pub mod cryptography;
pub mod entities;
pub mod error;
pub mod events;
pub mod rules;
