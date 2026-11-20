//! Camada de aplicação do módulo de demandas.
//!
//! Reúne casos de uso, comandos, contratos abstratos (ports) e erros
//! para gerenciamento de demandas acadêmicas e de espaço físico.

pub mod commands;
pub mod error;
pub mod ports;
pub mod queries;
pub mod services;

pub use commands::*;
pub use error::*;
pub use ports::*;
pub use queries::*;
pub use services::*;
