//! Regras de negócio e validação do módulo de Centros Acadêmicos.
//!
//! Centraliza as invariantes, limites estruturais e funções de validação
//! aplicáveis às entidades e operações deste domínio.

mod academic_center;
mod mandate;
pub use academic_center::*;
pub use active_context_rules::active_permissions_in_context;
pub use active_context_rules::assert_favorite_preference_unaltered_by_context_switch;
pub use active_context_rules::assert_navigation_only_active_context_scope;
pub use active_context_rules::has_academic_enrollment_via_active_context;
pub use active_context_rules::has_administrative_permission_in_active_context;
pub use active_context_rules::has_administrative_privileges_via_active_context;
pub use active_context_rules::is_eligible_to_vote_via_active_context;
pub use active_context_rules::is_visitor_in_active_context;
pub use active_context_rules::validate_active_context_switch;
pub use authorization_rules::AuthorizationRuleError;
pub use authorization_rules::check_authorization_in_ca;
pub use authorization_rules::evaluate_effective_permissions;
pub use authorization_rules::filter_active_assignments;
pub use authorization_rules::is_authorized_in_ca;
pub use mandate::*;
pub use poll::*;
pub mod active_context_rules;
pub mod authorization_rules;
mod poll;
