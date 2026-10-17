//! Camada de aplicação do módulo de identidade institucional.
//!
//! [`queries::current_identity`] define a identidade autenticada, o contrato
//! para verificação de tokens e os erros esperados desse fluxo.
//! [`commands`] e [`services`] permanecem reservados para comportamentos
//! ainda não implementados.

pub mod commands;
pub mod queries;
pub mod services;
