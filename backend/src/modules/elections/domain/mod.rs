//! Camada de domínio do módulo de eleições.
//!
//! Contém os modelos, objetos de valor, regras e eventos do negócio.
//! Esta camada não deve depender de HTTP, banco de dados ou mecanismos
//! de transporte.

pub mod entities;
pub mod errors;
pub mod events;
pub mod rules;
pub mod value_objects;
