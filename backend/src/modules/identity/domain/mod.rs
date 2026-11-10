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

pub use entities::{
    AppUser, AppUserError, AppUserStatus, CALOURO_DIDACTIC_ID, MAX_DISPLAY_NAME_LENGTH,
    MAX_EMAIL_LENGTH, PROHIBITED_MATRICULA, UserPreference, UserPreferenceError,
    VETERANO_1_DIDACTIC_ID, VETERANO_2_DIDACTIC_ID,
};
pub use rules::{
    assert_navigation_only_scope, has_academic_enrollment_via_preference,
    has_administrative_privileges_via_preference, is_eligible_to_vote_via_preference,
    validate_favorite_ca_selection,
};
