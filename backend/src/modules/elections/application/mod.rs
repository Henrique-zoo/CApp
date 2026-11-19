//! Camada de aplicação do módulo eleitoral.
//!
//! Coordena casos de uso por meio de portas, sem depender de HTTP,
//! mecanismos de persistência ou implementações criptográficas.

pub mod ballot_ports;
pub mod commands;
pub mod error;
pub mod ports;
pub mod queries;
pub mod services;

/// Porta do quadro público transacional.
pub mod board;
