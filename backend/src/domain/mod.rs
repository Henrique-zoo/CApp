//! Modelos e entidades de domínio do backend CApp.
//!
//! Centraliza as definições de domínio do sistema para facilitar a importação
//! e manter compatibilidade com os módulos em [`crate::modules`].

pub mod eventos;
pub use crate::modules::ca::domain::entities::context_center::CACC_DIDACTIC_ID;
pub use crate::modules::ca::domain::entities::context_center::CAER_DIDACTIC_ID;
pub use crate::modules::ca::domain::entities::context_center::MAX_ACRONYM_LENGTH;
pub use crate::modules::ca::domain::entities::context_center::MAX_DESCRIPTION_LENGTH;
pub use crate::modules::ca::domain::entities::context_center::MAX_NAME_LENGTH;
pub use academic_center::AcademicCenter;
pub use academic_center::AcademicCenterError;
pub use academic_center::AcademicCenterStatus;
pub use active_context::ActiveCaContext;
pub use active_context::ActiveContextError;
pub use active_context::ActiveContextSource;
pub use active_context::ManagementRoleAssignment;
pub use app_user::AppUser;
pub use app_user::AppUserError;
pub use app_user::AppUserStatus;
pub use app_user::CALOURO_DIDACTIC_ID;
pub use app_user::MAX_DISPLAY_NAME_LENGTH;
pub use app_user::MAX_EMAIL_LENGTH;
pub use app_user::PROHIBITED_MATRICULA;
pub use app_user::VETERANO_1_DIDACTIC_ID;
pub use app_user::VETERANO_2_DIDACTIC_ID;
pub use eventos::DIRECT_EVENT_MANAGEMENT_PERMISSION;
pub use eventos::Event;
pub use eventos::EventDomainError;
pub use eventos::EventPeriod;
pub use eventos::EventPublicationStatus;
pub use eventos::EventRegistration;
pub use eventos::EventRegistrationInfo;
pub use eventos::EventRegistrationPolicy;
pub use eventos::MAX_EVENT_DESCRIPTION_LENGTH;
pub use eventos::MAX_EVENT_LOCATION_LENGTH;
pub use eventos::MAX_EVENT_TITLE_LENGTH;
pub use eventos::RegistrationPolicy;
pub use eventos::RegistrationStatus;
pub use eventos::validate_event_basic_fields;
pub use eventos::validate_event_period;
pub use eventos::validate_event_registration;
pub use user_preference::UserPreference;
pub use user_preference::UserPreferenceError;
pub mod academic_center;
pub mod active_context;
pub mod app_user;
pub mod user_preference;
