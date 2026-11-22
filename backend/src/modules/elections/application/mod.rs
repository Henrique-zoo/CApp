//! Camada de aplicação do módulo eleitoral.
//!
//! Coordena casos de uso por meio de portas, sem depender de HTTP,
//! mecanismos de persistência ou implementações criptográficas.

pub mod board;
pub mod commands;
pub mod queries;
pub mod services;
