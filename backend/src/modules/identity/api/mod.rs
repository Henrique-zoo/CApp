//! Camada HTTP do módulo de identidade institucional.
//!
//! [`dto`] reserva os contratos de entrada e saída, [`handlers`] adapta
//! as requisições aos casos de uso e [`extractor`] fornece [`AuthenticatedUser`]
//! para extração do usuário autenticado nos handlers protegidos.

pub mod dto;
pub mod extractor;
pub mod handlers;

pub use extractor::{AuthExtractionError, AuthenticatedUser};
