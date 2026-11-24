//! Modelos institucionais e evidências de elegibilidade persistidas.
//!
//! [`entities`] contém instituição e resultado de verificação; [`CatalogStatus`]
//! representa disponibilidade no catálogo. [`rules`] e [`events`] são reservados.
//!
//! Os tipos não executam SQL nem conhecem HTTP. Os derives `sqlx::FromRow` e
//! `sqlx::Type` constituem o acoplamento de mapeamento adotado pelo projeto;
//! as consultas e o gerenciamento de conexões continuam na infraestrutura.

pub mod entities;
pub mod events;
pub mod rules;
mod status;
pub use status::CatalogStatus;
pub use status::UserStatus;
pub use entities::AppUser;
pub use entities::AppUserError;
pub use entities::AppUserStatus;
pub use entities::CALOURO_DIDACTIC_ID;
pub use entities::MAX_DISPLAY_NAME_LENGTH;
pub use entities::MAX_EMAIL_LENGTH;
pub use entities::PROHIBITED_MATRICULA;
pub use entities::UserPreference;
pub use entities::UserPreferenceError;
pub use entities::VETERANO_1_DIDACTIC_ID;
pub use entities::VETERANO_2_DIDACTIC_ID;
pub use rules::assert_navigation_only_scope;
pub use rules::has_academic_enrollment_via_preference;
pub use rules::has_administrative_privileges_via_preference;
pub use rules::is_eligible_to_vote_via_preference;
pub use rules::validate_favorite_ca_selection;
