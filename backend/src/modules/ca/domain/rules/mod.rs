//! Regras do módulo de Centros Acadêmicos.
//!
//! Reúne as invariantes e validações de regras de negócio relativas a
//! Centros Acadêmicos e ao contexto ativo de navegação em sessão.

pub mod active_context_rules;

pub use active_context_rules::{
    active_permissions_in_context, assert_favorite_preference_unaltered_by_context_switch,
    assert_navigation_only_active_context_scope, has_academic_enrollment_via_active_context,
    has_administrative_permission_in_active_context,
    has_administrative_privileges_via_active_context, is_eligible_to_vote_via_active_context,
    is_visitor_in_active_context, validate_active_context_switch,
};
