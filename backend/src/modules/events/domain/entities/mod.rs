//! Entidades do módulo de eventos.
//!
//! Define o modelo de domínio de [`Event`], suas informações de período
//! ([`EventPeriod`]), estado de publicação ([`EventPublicationStatus`] / [`EventStatus`]),
//! avaliação de homologação ([`EventReview`] e [`PublicationReviewDecision`]) e
//! controle de inscrições ([`EventRegistrationInfo`] / [`EventRegistrationPolicy`],
//! [`EventRegistration`] e [`RegistrationStatus`] / [`EventRegistrationStatus`]).

pub mod event;

pub use event::{
    Event, EventDomainError, EventPeriod, EventPublicationStatus, EventRegistration,
    EventRegistrationInfo, EventRegistrationPolicy, EventRegistrationStatus, EventReview,
    EventStatus, MAX_EVENT_DESCRIPTION_LENGTH, MAX_EVENT_LOCATION_LENGTH, MAX_EVENT_TITLE_LENGTH,
    PublicationReviewDecision, RegistrationStatus,
};
