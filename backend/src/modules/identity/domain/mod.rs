//! Camada de domínio do módulo de identidade institucional.
//!
//! [`entities`], [`rules`] e [`events`] definem os modelos, suas invariantes
//! e os fatos relevantes do negócio. Essa camada permanece independente
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;

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
