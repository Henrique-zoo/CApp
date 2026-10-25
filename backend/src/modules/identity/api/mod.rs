//! Camada de API HTTP do módulo de identidade institucional.
//!
//! Disponibiliza extractors e adaptadores HTTP para autenticação.

pub mod extractor;

pub use extractor::AuthenticatedUser;
