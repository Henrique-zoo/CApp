//! Modelos e entidades de domínio do backend.
//!
//! Centraliza as definições de domínio do sistema para facilitar a importação
//! e manter a compatibilidade com a estrutura do projeto.

pub mod academic_center;
pub mod app_user;
pub mod user_preference;

pub use academic_center::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};
pub use app_user::{
    AppUser, AppUserError, AppUserStatus, CALOURO_DIDACTIC_ID, MAX_DISPLAY_NAME_LENGTH,
    MAX_EMAIL_LENGTH, PROHIBITED_MATRICULA, VETERANO_1_DIDACTIC_ID, VETERANO_2_DIDACTIC_ID,
};
pub use user_preference::{UserPreference, UserPreferenceError};
