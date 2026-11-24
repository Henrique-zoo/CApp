//! Serviços de aplicação do módulo de Centros Acadêmicos.
//!
//! Coordena casos de uso, regras de domínio e persistência de sessões de navegação.

pub mod active_context_service;

pub use active_context_service::ActiveContextService;
