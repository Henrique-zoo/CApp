//! Modelos e entidades de domínio do backend CApp.
//!
//! Centraliza as definições de domínio do sistema para facilitar a importação
//! e manter compatibilidade com os módulos em [`crate::modules`].

pub mod eventos;

pub use eventos::{
    DIRECT_EVENT_MANAGEMENT_PERMISSION, Event, EventDomainError, EventPeriod,
    EventPublicationStatus, EventRegistration, EventRegistrationInfo, MAX_EVENT_DESCRIPTION_LENGTH,
    MAX_EVENT_LOCATION_LENGTH, MAX_EVENT_TITLE_LENGTH, RegistrationStatus,
    validate_event_basic_fields, validate_event_period, validate_event_registration,
};
