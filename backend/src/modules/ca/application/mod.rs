//! Camada de casos de uso do módulo de Centros Acadêmicos.
//!
//! [`commands`] organiza as operações de alteração de estado;
//! [`queries`] organiza as operações de leitura e apuração;
//! [`ports`] e [`error`] definem os contratos e falhas de negócio da aplicação.

pub mod commands;
pub mod error;
pub mod ports;
pub mod queries;
pub mod services;

#[cfg(test)]
mod tests;

pub use error::*;
pub use ports::*;
