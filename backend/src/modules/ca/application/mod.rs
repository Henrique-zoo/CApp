//! Camada de aplicação do módulo de Centros Acadêmicos.
//!
//! Orquestra casos de uso de cargos, integrantes e autorização por meio de portas e serviços.

pub mod commands;
pub mod ports;
pub mod queries;
pub mod services;

pub use ports::*;
pub use services::*;
