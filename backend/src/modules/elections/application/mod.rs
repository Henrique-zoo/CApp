//! Camada de aplicação do módulo eleitoral.
//!
//! Coordena casos de uso por meio de portas, sem depender de HTTP,
//! mecanismos de persistência ou implementações criptográficas.

pub mod administration;
pub mod board;
pub mod error;
pub mod ports;
pub mod preparation;
pub mod registration;
pub mod services;
pub mod tally;
pub mod verification;
