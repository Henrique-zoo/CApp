//! Entidades do módulo de identidade institucional.
//!
//! Reúne os modelos de domínio de usuários e preferências de navegação.

pub mod app_user;
pub mod user_preference;

pub use app_user::{
    AppUser, AppUserError, AppUserStatus, CALOURO_DIDACTIC_ID, MAX_DISPLAY_NAME_LENGTH,
    MAX_EMAIL_LENGTH, PROHIBITED_MATRICULA, VETERANO_1_DIDACTIC_ID, VETERANO_2_DIDACTIC_ID,
};
pub use user_preference::{UserPreference, UserPreferenceError};
