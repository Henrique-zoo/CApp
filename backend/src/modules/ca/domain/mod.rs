//! Camada de domínio do módulo de Centros Acadêmicos.
//!
//! [`entities`], [`rules`] e [`events`] definem os modelos, suas invariantes
//! e os fatos relevantes do negócio. Essa camada permanece independente
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;

pub use entities::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus, ActiveCaContext, ActiveContextError,
    ActiveContextSource, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID, MAX_ACRONYM_LENGTH,
    MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH, ManagementRoleAssignment,
};
pub use rules::{
    active_permissions_in_context, assert_favorite_preference_unaltered_by_context_switch,
    assert_navigation_only_active_context_scope, has_academic_enrollment_via_active_context,
    has_administrative_permission_in_active_context,
    has_administrative_privileges_via_active_context, is_eligible_to_vote_via_active_context,
    is_visitor_in_active_context, validate_active_context_switch,
};
