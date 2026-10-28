//! Entidades do módulo de eventos.
//!
//! Define o modelo de domínio de [`Event`], suas informações de período
//! ([`EventPeriod`]), estado de publicação ([`EventPublicationStatus`]) e
//! controle de inscrições ([`EventRegistrationInfo`], [`EventRegistration`] e
//! [`RegistrationStatus`]).

pub mod event;

pub use event::{
    Event, EventDomainError, EventPeriod, EventPublicationStatus, EventRegistration,
    EventRegistrationInfo, MAX_EVENT_DESCRIPTION_LENGTH, MAX_EVENT_LOCATION_LENGTH,
    MAX_EVENT_TITLE_LENGTH, RegistrationStatus,
};
