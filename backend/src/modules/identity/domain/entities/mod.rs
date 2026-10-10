//! Entidades usadas na verificação institucional.
//!
//! [`institution`] configura o tenant e o grupo de estudantes.
//! [`institutional_verification`] registra evidência por UID Firebase, independente
//! de provisionamento de usuário interno. Os derives SQLx mapeiam linhas para esses
//! tipos; não aplicam migrations nem consultam o banco.

pub mod institution;
pub mod institutional_verification;

pub mod app_user;
pub mod user_preference;

pub use app_user::{
    AppUser, AppUserError, AppUserStatus, CALOURO_DIDACTIC_ID, MAX_DISPLAY_NAME_LENGTH,
    MAX_EMAIL_LENGTH, PROHIBITED_MATRICULA, VETERANO_1_DIDACTIC_ID, VETERANO_2_DIDACTIC_ID,
};
pub use user_preference::{UserPreference, UserPreferenceError};
