//! Camada de domínio prevista para o módulo de Centros Acadêmicos.
//!
//! [`entities`], [`rules`] e [`events`] reservam os modelos, suas invariantes,
//! limites e fatos relevantes do negócio. Essa camada permanece independente
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;
pub use entities::AcademicCenter;
pub use entities::AcademicCenterError;
pub use entities::AcademicCenterStatus;
pub use entities::CACC_DIDACTIC_ID;
pub use entities::CAER_DIDACTIC_ID;
pub use entities::MAX_ACRONYM_LENGTH;
pub use entities::MAX_DESCRIPTION_LENGTH;
pub use entities::MAX_NAME_LENGTH;
pub use entities::ActiveCaContext;
pub use entities::ActiveContextError;
pub use entities::ActiveContextSource;
pub use entities::ManagementRoleAssignment;
pub use rules::active_permissions_in_context;
pub use rules::assert_favorite_preference_unaltered_by_context_switch;
pub use rules::assert_navigation_only_active_context_scope;
pub use rules::has_academic_enrollment_via_active_context;
pub use rules::has_administrative_permission_in_active_context;
pub use rules::has_administrative_privileges_via_active_context;
pub use rules::is_eligible_to_vote_via_active_context;
pub use rules::is_visitor_in_active_context;
pub use rules::validate_active_context_switch;
pub use entities::*;
pub use rules::*;
