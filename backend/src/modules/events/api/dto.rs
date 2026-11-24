//! Contratos HTTP (DTOs) de entrada e saída para os endpoints de eventos.

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use crate::modules::events::domain::Event;
use crate::modules::events::domain::EventDomainError;
use crate::modules::events::domain::EventPublicationStatus;
use crate::modules::events::domain::EventRegistrationPolicy;
use crate::modules::events::domain::EventRegistration;
use crate::modules::events::domain::RegistrationStatus;
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
    /// Instante opcional de abertura das inscrições.
    pub registration_opens_at: Option<DateTime<Utc>>,
    /// Prazo opcional limite para inscrição (`closes_at` da política).
    pub registration_deadline: Option<DateTime<Utc>>,
}
impl CreateEventRequest {
    /// Converte o limite HTTP (`u32`) para a capacidade de domínio (`i32`).
    pub fn capacity(&self) -> Result<Option<i32>, EventDomainError> {
        map_capacity(self.max_capacity)
    }

    /// Constrói a política de inscrição correspondente aos campos opcionais informados.
    pub fn registration_policy(&self) -> Result<EventRegistrationPolicy, EventDomainError> {
        EventRegistrationPolicy::new(
            self.requires_registration.unwrap_or(true),
            self.registration_opens_at,
            self.registration_deadline,
        )
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
    /// Atualização opcional do instante de abertura das inscrições.
    pub registration_opens_at: Option<DateTime<Utc>>,
    /// Atualização opcional do prazo de inscrição.
    pub registration_deadline: Option<DateTime<Utc>>,
}
impl UpdateEventRequest {
    /// Indica se o corpo solicita alteração de capacidade ou política de inscrição.
    pub fn touches_registration(&self) -> bool {
        self.requires_registration.is_some()
            || self.max_capacity.is_some()
            || self.registration_opens_at.is_some()
            || self.registration_deadline.is_some()
    }

    /// Converte o limite HTTP (`u32`) para a capacidade de domínio (`i32`).
    pub fn capacity(&self) -> Result<Option<i32>, EventDomainError> {
        map_capacity(self.max_capacity)
    }

    /// Constrói a política de inscrição a partir dos campos enviados e da política atual.
    pub fn registration_policy(
        &self,
        current: &EventRegistrationPolicy,
    ) -> Result<EventRegistrationPolicy, EventDomainError> {
        EventRegistrationPolicy::new(
            self.requires_registration
                .unwrap_or_else(|| current.requires_registration()),
            self.registration_opens_at.or_else(|| current.opens_at()),
            self.registration_deadline.or_else(|| current.closes_at()),
        )
    }
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
    /// Número da revisão do conteúdo.
    pub revision: i32,
    /// Indica se requer inscrição.
    pub requires_registration: bool,
    /// Limite máximo opcional de vagas.
    pub max_capacity: Option<u32>,
    /// Número de vagas restantes quando há limite e contagem conhecida (`None` sem limite).
    pub remaining_spots: Option<u32>,
    /// Indica se o evento aceita novas inscrições diretas considerando capacidade zero confirmada.
    pub has_available_spots: bool,
    /// Instante opcional de abertura das inscrições.
    pub registration_opens_at: Option<DateTime<Utc>>,
    /// Prazo opcional de inscrição.
    pub registration_deadline: Option<DateTime<Utc>>,
}
/// Alias semântico para a resposta de detalhes de um evento publicado.
pub type EventDetailResponse = EventResponse;
impl From<Event> for EventResponse {
    fn from(event: Event) -> Self {
        let registered_count = 0;
        let remaining_spots = event.remaining_spots(registered_count);
        let has_available_spots = event.has_available_spots(registered_count);
        let capacity = event.capacity().and_then(|value| u32::try_from(value).ok());
        let policy = event.registration();

        Self {
            id: event.id(),
            academic_center_id: event.academic_center_id(),
            author_id: event.author_id(),
            title: event.title().to_owned(),
            description: event.description().to_owned(),
            location: event.location().to_owned(),
            starts_at: event.starts_at(),
            ends_at: event.ends_at(),
            status: event.status(),
            revision: event.revision(),
            requires_registration: policy.requires_registration(),
            max_capacity: capacity,
            remaining_spots,
            has_available_spots,
            registration_opens_at: policy.opens_at(),
            registration_deadline: policy.closes_at(),
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
fn map_capacity(max_capacity: Option<u32>) -> Result<Option<i32>, EventDomainError> {
    max_capacity
        .map(|value| {
            i32::try_from(value)
                .map_err(|_| EventDomainError::InvalidMaxCapacity(value))
                .and_then(|capacity| {
                    if capacity <= 0 {
                        Err(EventDomainError::InvalidMaxCapacity(value))
                    } else {
                        Ok(capacity)
                    }
                })
        })
        .transpose()
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
