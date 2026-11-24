//! Camada de casos de uso do módulo de Centros Acadêmicos.
//!
//! [`commands`] agrupa alterações de estado, [`queries`] consultas,
//! [`ports`] as interfaces de persistência e integração, e [`error`]
//! os erros da aplicação.

pub mod commands;
pub mod error;
pub mod ports;
pub mod queries;
pub mod services;
