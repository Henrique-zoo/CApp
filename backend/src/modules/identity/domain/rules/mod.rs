//! Regras de validação e invariantes do módulo de identidade institucional.
//!
//! Encapsula as regras de integridade e consistência de negócio de forma pura,
//! livre de acoplamento com banco de dados, serialização externa ou frameworks.

pub mod academic_data;
pub mod institution;
pub mod institutional_verification;

pub use academic_data::*;
pub use institution::*;
pub use institutional_verification::*;
