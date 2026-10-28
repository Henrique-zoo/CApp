//! Camada de persistência do módulo de eventos.
//!
//! [`repository`] disponibiliza o adaptador PostgreSQL ([`PostgresEventRepository`])
//! que implementa [`EventRepository`](crate::modules::events::application::EventRepository).

pub mod repository;

pub use repository::PostgresEventRepository;
