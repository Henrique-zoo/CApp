//! Regras e primitivas do domínio de eleições.
//!
//! Contém os modelos, objetos de valor, regras e eventos do negócio.
//! Esta camada não deve depender de HTTP, banco de dados ou mecanismos
//! de transporte.
//! [`cryptography`] implementa as operações matemáticas do protocolo eleitoral
//! e usa as falhas definidas em [`error`].

pub mod cryptography;
pub mod entities;
pub mod error;
pub mod errors;
pub mod events;
pub mod rules;
pub mod value_objects;
