//! Camada de domínio do módulo de eventos.
//!
//! [`entities`], [`rules`] e [`events`] definem os modelos de eventos dos Centros Acadêmicos,
//! suas invariantes de criação, edição, publicação e inscrição, permanecendo independentes
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;

pub use entities::{
    Event, EventDomainError, EventPeriod, EventPublicationStatus, EventRegistration,
    EventRegistrationInfo, MAX_EVENT_DESCRIPTION_LENGTH, MAX_EVENT_LOCATION_LENGTH,
    MAX_EVENT_TITLE_LENGTH, RegistrationStatus,
};
pub use rules::{
    DIRECT_EVENT_MANAGEMENT_PERMISSION, validate_event_basic_fields, validate_event_period,
    validate_event_registration,
};
