//! Camada HTTP do módulo de identidade institucional.
//!
//! [`dto`] define os contratos de resposta e [`handlers`] adapta as requisições
//! aos casos de uso. O router global registra `GET /identity/me`.

pub mod dto;
pub mod handlers;
