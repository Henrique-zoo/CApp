//! Casos de uso e contratos internos da identidade institucional.
//!
//! [`services::authentication`] coordena as operações de verificação e autenticação.
//! [`ports`] define dependências externas e dados autenticados; [`error`] representa
//! falhas sem conhecer códigos HTTP ou SQLx. Os serviços recebem referências às
//! traits e não acessam `AppState`, pool PostgreSQL ou DTOs HTTP.
//!
//! [`commands`] e [`queries`] permanecem reservados para outros casos de uso.

pub mod commands;
pub mod error;
pub mod ports;
pub mod queries;
pub mod services;
