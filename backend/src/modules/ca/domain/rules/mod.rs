//! Regras de negócio do módulo de Centros Acadêmicos.
//!
//! Centraliza validações de domínio para autorização, isolamento multi-CA
//! e integridade de cargos.

pub mod authorization_rules;

pub use authorization_rules::{
    AuthorizationRuleError, check_authorization_in_ca, evaluate_effective_permissions,
    filter_active_assignments, is_authorized_in_ca,
};
