//! Regras e invariantes de negócio do módulo de eventos.
//!
//! Expõe validadores de domínio independentes de transporte (HTTP) e persistência (SQL)
//! para garantir que todo evento criado ou editado possua informações obrigatórias
//! válidas e período consistente.

use chrono::{DateTime, Utc};

use super::entities::{Event, EventDomainError, EventPeriod, EventRegistrationInfo};

/// Permissão administrativa exigida para criar eventos já publicados, publicar rascunhos
/// diretamente e editar eventos do Centro Acadêmico.
pub const DIRECT_EVENT_MANAGEMENT_PERMISSION: &str = "Publicar e gerenciar eventos diretamente";

/// Valida os campos textuais obrigatórios de um evento (`title`, `description` e `location`).
///
/// # Erros
///
/// Retorna [`EventDomainError`] caso algum campo obrigatório esteja vazio ou exceda o limite.
pub fn validate_event_basic_fields(
    title: &str,
    description: &str,
    location: &str,
) -> Result<(), EventDomainError> {
    Event::validate_basic_fields(title, description, location)
}

/// Valida a consistência do período do evento, exigindo que o término seja posterior ao início.
///
/// # Erros
///
/// Retorna [`EventDomainError::InvalidEventPeriod`] se `ends_at <= starts_at`.
pub fn validate_event_period(
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
) -> Result<EventPeriod, EventDomainError> {
    EventPeriod::new(starts_at, ends_at)
}

/// Valida as regras de inscrição de um evento frente à data/hora de término.
///
/// # Erros
///
/// Retorna [`EventDomainError`] se o limite máximo for zero, se houver mais inscritos
/// confirmados que o limite ou se o prazo de inscrição ultrapassar o término do evento.
pub fn validate_event_registration(
    registration: &EventRegistrationInfo,
    ends_at: DateTime<Utc>,
) -> Result<(), EventDomainError> {
    registration.validate(ends_at)
}
