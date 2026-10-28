//! Camada de casos de uso do módulo de eventos.
//!
//! [`commands`] agrupa alterações de estado, [`queries`] consultas e
//! [`services`] a coordenação dessas operações sobre o modelo de domínio.

pub mod commands;
pub mod queries;
pub mod services;

pub use commands::{CreateEventCommand, PublishEventCommand, UpdateEventCommand};
pub use queries::{GetEventByIdQuery, ListCaEventsQuery};
pub use services::{
    AllowAllEventAuthorization, EventApplicationError, EventAuthorizationPort, EventRepository,
    EventService, StaticEventAuthorization,
};
