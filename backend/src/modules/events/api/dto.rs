//! Contratos HTTP (DTOs) de entrada e saída para os endpoints de eventos.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::events::domain::{
    Event, EventPublicationStatus, EventRegistration, EventRegistrationInfo, RegistrationStatus,
};

/// Dados do formulário para criação de um novo evento (rascunho ou publicação direta).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    /// Identificador do Centro Acadêmico responsável pelo evento.
    pub academic_center_id: Uuid,
    /// Identificador do gestor/usuário autor do evento.
    pub author_id: Uuid,
    /// Título do evento.
    pub title: String,
    /// Descrição detalhada do evento.
    pub description: String,
    /// Local de realização do evento.
    pub location: String,
    /// Data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Data e horário de término do evento.
    pub ends_at: DateTime<Utc>,
    /// Quando `true`, cria o evento já publicado na agenda; por padrão (`false`), salva como rascunho.
    #[serde(default)]
    pub publish_directly: bool,
    /// Indica se o evento aceita inscrições (padrão `true`).
    pub requires_registration: Option<bool>,
    /// Limite máximo opcional de participantes.
    pub max_capacity: Option<u32>,
    /// Prazo opcional limite para inscrição.
    pub registration_deadline: Option<DateTime<Utc>>,
}

impl CreateEventRequest {
    /// Constrói a configuração de inscrição correspondente aos campos opcionais informados.
    pub fn into_registration_info(&self) -> EventRegistrationInfo {
        EventRegistrationInfo {
            requires_registration: self.requires_registration.unwrap_or(true),
            max_capacity: self.max_capacity,
            confirmed_registrations: 0,
            registration_deadline: self.registration_deadline,
        }
    }
}

/// Dados do formulário para edição das informações de um evento existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    /// Identificador opcional do Centro Acadêmico onde a edição é solicitada.
    pub academic_center_id: Option<Uuid>,
    /// Identificador opcional do gestor que realiza a edição.
    pub actor_id: Option<Uuid>,
    /// Novo título do evento.
    pub title: String,
    /// Nova descrição do evento.
    pub description: String,
    /// Novo local de realização do evento.
    pub location: String,
    /// Nova data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Nova data e horário de término do evento.
    pub ends_at: DateTime<Utc>,
    /// Atualização opcional da necessidade de inscrição.
    pub requires_registration: Option<bool>,
    /// Atualização opcional do limite máximo de participantes.
    pub max_capacity: Option<u32>,
    /// Atualização opcional do prazo de inscrição.
    pub registration_deadline: Option<DateTime<Utc>>,
}

/// Corpo opcional para a ação de publicação direta de um rascunho de evento.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishEventRequest {
    /// Identificador opcional do Centro Acadêmico onde a publicação é solicitada.
    pub academic_center_id: Option<Uuid>,
    /// Identificador opcional do gestor que realiza a publicação direta.
    pub actor_id: Option<Uuid>,
}

/// Parâmetros de query string para listagem e consulta de detalhes de eventos de um Centro Acadêmico.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListEventsQueryParams {
    /// Identificador do Centro Acadêmico ativo (`?ca_id=...` ou `?academic_center_id=...`).
    #[serde(alias = "academic_center_id")]
    pub ca_id: Option<Uuid>,
    /// Define se apenas eventos publicados devem ser retornados.
    /// Quando omitido nas consultas públicas da agenda, assume `true` por padrão.
    pub only_published: Option<bool>,
}

impl ListEventsQueryParams {
    /// Retorna se a consulta deve filtrar exclusivamente eventos publicados (`true` por padrão).
    pub const fn should_filter_only_published(&self) -> bool {
        match self.only_published {
            Some(flag) => flag,
            None => true,
        }
    }
}

/// Representação resumida de um evento publicado na listagem da agenda do CA ativo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSummaryResponse {
    /// Identificador único do evento.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico responsável.
    pub academic_center_id: Uuid,
    /// Título do evento.
    pub title: String,
    /// Local do evento.
    pub location: String,
    /// Data/hora de início.
    pub starts_at: DateTime<Utc>,
    /// Data/hora de término.
    pub ends_at: DateTime<Utc>,
    /// Situação de publicação do evento.
    pub status: EventPublicationStatus,
    /// Indica se o evento aceita novas inscrições diretas no momento.
    pub has_available_spots: bool,
    /// Número de vagas restantes quando há limite configurado (`None` para eventos sem limite).
    pub remaining_spots: Option<u32>,
}

impl From<Event> for EventSummaryResponse {
    fn from(event: Event) -> Self {
        let remaining_spots = event.registration.remaining_spots();
        let has_available_spots = event.registration.has_available_spots();
        Self {
            id: event.id,
            academic_center_id: event.academic_center_id,
            title: event.title,
            location: event.location,
            starts_at: event.starts_at,
            ends_at: event.ends_at,
            status: event.status,
            has_available_spots,
            remaining_spots,
        }
    }
}

/// Representação completa JSON de um evento retornada pela API (listagem e detalhes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventResponse {
    /// Identificador único do evento.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico responsável.
    pub academic_center_id: Uuid,
    /// Identificador do autor responsável.
    pub author_id: Uuid,
    /// Título do evento.
    pub title: String,
    /// Descrição detalhada do evento.
    pub description: String,
    /// Local do evento.
    pub location: String,
    /// Data/hora de início.
    pub starts_at: DateTime<Utc>,
    /// Data/hora de término.
    pub ends_at: DateTime<Utc>,
    /// Situação de publicação do evento.
    pub status: EventPublicationStatus,
    /// Indica se requer inscrição.
    pub requires_registration: bool,
    /// Limite máximo opcional de vagas.
    pub max_capacity: Option<u32>,
    /// Total atual de inscrições confirmadas.
    pub confirmed_registrations: u32,
    /// Número de vagas restantes quando há limite configurado (`None` para eventos sem limite).
    pub remaining_spots: Option<u32>,
    /// Indica se o evento aceita novas inscrições diretas no momento.
    pub has_available_spots: bool,
    /// Prazo opcional de inscrição.
    pub registration_deadline: Option<DateTime<Utc>>,
    /// Data/hora de publicação.
    pub published_at: Option<DateTime<Utc>>,
    /// Data/hora de criação.
    pub created_at: DateTime<Utc>,
    /// Data/hora da última atualização.
    pub updated_at: DateTime<Utc>,
}

/// Alias semântico para a resposta de detalhes de um evento publicado.
pub type EventDetailResponse = EventResponse;

impl From<Event> for EventResponse {
    fn from(event: Event) -> Self {
        let remaining_spots = event.registration.remaining_spots();
        let has_available_spots = event.registration.has_available_spots();
        Self {
            id: event.id,
            academic_center_id: event.academic_center_id,
            author_id: event.author_id,
            title: event.title,
            description: event.description,
            location: event.location,
            starts_at: event.starts_at,
            ends_at: event.ends_at,
            status: event.status,
            requires_registration: event.registration.requires_registration,
            max_capacity: event.registration.max_capacity,
            confirmed_registrations: event.registration.confirmed_registrations,
            remaining_spots,
            has_available_spots,
            registration_deadline: event.registration.registration_deadline,
            published_at: event.published_at,
            created_at: event.created_at,
            updated_at: event.updated_at,
        }
    }
}

/// Representação JSON de uma inscrição (ativa ou cancelada) de um participante em um evento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRegistrationResponse {
    /// Identificador único da inscrição.
    pub id: Uuid,
    /// Identificador do evento.
    pub event_id: Uuid,
    /// Identificador do participante inscrito.
    pub participant_id: Uuid,
    /// Situação atual da inscrição (`confirmed`, `cancelled`, `waitlisted`).
    pub status: RegistrationStatus,
    /// Data/hora em que a inscrição foi realizada.
    pub registered_at: DateTime<Utc>,
    /// Data/hora do cancelamento, quando aplicável.
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Mensagem informativa sobre o resultado da operação.
    pub message: String,
}

impl EventRegistrationResponse {
    /// Constrói a resposta para uma inscrição confirmada com sucesso.
    pub fn confirmed(registration: EventRegistration) -> Self {
        Self {
            id: registration.id,
            event_id: registration.event_id,
            participant_id: registration.participant_id,
            status: registration.status,
            registered_at: registration.registered_at,
            cancelled_at: registration.cancelled_at,
            message: "Inscrição realizada com sucesso.".to_string(),
        }
    }

    /// Constrói a resposta para o cancelamento de inscrição realizado com sucesso.
    pub fn cancelled(registration: EventRegistration) -> Self {
        Self {
            id: registration.id,
            event_id: registration.event_id,
            participant_id: registration.participant_id,
            status: registration.status,
            registered_at: registration.registered_at,
            cancelled_at: registration.cancelled_at,
            message: "Cancelamento realizado com sucesso.".to_string(),
        }
    }
}

impl From<EventRegistration> for EventRegistrationResponse {
    fn from(registration: EventRegistration) -> Self {
        if registration.is_active() {
            Self::confirmed(registration)
        } else {
            Self::cancelled(registration)
        }
    }
}

/// Resposta padronizada de erro para operações da API de eventos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventErrorResponse {
    /// Código semântico do erro para inspeção programática.
    pub code: String,
    /// Mensagem descritiva em português.
    pub message: String,
}
