//! Regras de negócio e validação do módulo de Centros Acadêmicos.
//!
//! Centraliza as invariantes, limites estruturais e funções de validação
//! aplicáveis às entidades e operações deste domínio.

mod academic_center;
mod mandate;
pub use academic_center::*;
pub use mandate::*;
pub use poll::*;
mod poll;
