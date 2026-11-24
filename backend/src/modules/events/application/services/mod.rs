//! Serviços de aplicação e contratos de repositório e autorização para o módulo de eventos.
//!
//! Orquestra a criação, edição, publicação direta e consulta de eventos,
//! verificando as permissões administrativas ([`DIRECT_EVENT_MANAGEMENT_PERMISSION`](crate::modules::events::domain::DIRECT_EVENT_MANAGEMENT_PERMISSION))
//! por meio de [`EventAuthorizationPort`] e aplicando as validações do domínio ([`Event`])
//! antes de persistir no repositório.

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::{
    commands::{
        CancelEventRegistrationCommand, CreateEventCommand, PublishEventCommand,
        RegisterInEventCommand, UpdateEventCommand,
    },
    queries::{
        GetEventByIdQuery, GetPublishedEventDetailQuery, ListCaEventsQuery,
        ListPublishedEventsByCaQuery,
    },
};
use crate::modules::events::domain::{Event, EventDomainError, EventRegistration};

/// Erros produzidos pelos casos de uso da camada de aplicação de eventos.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EventApplicationError {
    /// Violação de uma regra de negócio ou validação do modelo de domínio.
    #[error(transparent)]
    Domain(#[from] EventDomainError),

    /// O evento solicitado não foi encontrado ou não está disponível no contexto consultado.
    #[error("evento '{0}' não encontrado")]
    NotFound(Uuid),

    /// O evento não está disponível para receber inscrições (ex.: não publicado ou não homologado).
    #[error("o evento '{0}' não está disponível para inscrição")]
    EventNotAvailableForRegistration(Uuid),

    /// O usuário já possui uma inscrição ativa (confirmada) no evento informado.
    #[error("o usuário '{participant_id}' já está inscrito no evento '{event_id}'")]
    AlreadyRegistered {
        /// Identificador do evento.
        event_id: Uuid,
        /// Identificador do participante.
        participant_id: Uuid,
    },

    /// O usuário solicitou o cancelamento, mas não há inscrição ativa para cancelar no evento.
    #[error("não há inscrição ativa para cancelar no evento '{event_id}'")]
    ActiveRegistrationNotFound {
        /// Identificador do evento.
        event_id: Uuid,
        /// Identificador do participante.
        participant_id: Uuid,
    },

    /// O evento atingiu o limite máximo de vagas configurado e não aceita novas inscrições diretas.
    #[error("o evento '{event_id}' atingiu o limite máximo de {max_capacity} vagas e está lotado")]
    EventFull {
        /// Identificador do evento lotado.
        event_id: Uuid,
        /// Capacidade máxima configurada para o evento.
        max_capacity: u32,
    },

    /// O usuário não possui permissão para gerenciar ou publicar eventos diretamente no Centro Acadêmico informado.
    #[error("operação recusada por falta de permissão no Centro Acadêmico")]
    PermissionDenied,

    /// Falha técnica na camada de persistência ou autorização.
    #[error("erro ao acessar o repositório de eventos: {0}")]
    Repository(String),
}

/// Contrato de persistência para o repositório de eventos e inscrições.
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Persiste um novo evento no armazenamento.
    async fn create(&self, event: &Event) -> Result<Event, EventApplicationError>;

    /// Busca um evento pelo seu identificador único.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, EventApplicationError>;

    /// Lista os eventos de um Centro Acadêmico, podendo filtrar apenas os publicados.
    async fn list_by_academic_center(
        &self,
        academic_center_id: Uuid,
        only_published: bool,
    ) -> Result<Vec<Event>, EventApplicationError>;

    /// Lista exclusivamente os eventos publicados na agenda oficial de um Centro Acadêmico.
    async fn find_published_by_ca(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<Event>, EventApplicationError> {
        let events = self
            .list_by_academic_center(academic_center_id, true)
            .await?;
        Ok(events
            .into_iter()
            .filter(|event| {
                event.academic_center_id() == academic_center_id && event.is_published()
            })
            .collect())
    }

    /// Atualiza os dados de um evento existente no armazenamento.
    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError>;

    /// Busca a inscrição de um participante em um evento específico.
    async fn find_registration(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
    ) -> Result<Option<EventRegistration>, EventApplicationError>;

    /// Persiste (cria ou atualiza) a inscrição de um participante em um evento.
    async fn save_registration(
        &self,
        registration: &EventRegistration,
    ) -> Result<EventRegistration, EventApplicationError>;

    /// Lista todas as inscrições associadas a um evento.
    async fn list_registrations_by_event(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<EventRegistration>, EventApplicationError>;

    /// Registra ou reativa de forma atômica a inscrição de um participante em um evento publicado,
    /// verificando isolamento por CA, unicidade da inscrição ativa e limite opcional de capacidade
    /// sob controle de concorrência.
    async fn register_participant_atomically(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
        academic_center_id: Option<Uuid>,
    ) -> Result<EventRegistration, EventApplicationError>;

    /// Cancela de forma atômica a inscrição ativa de um participante em um evento,
    /// liberando a vaga ocupada sob controle de concorrência.
    async fn cancel_registration_atomically(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
        academic_center_id: Option<Uuid>,
    ) -> Result<EventRegistration, EventApplicationError>;
}

/// Repositório de eventos e inscrições em memória para testes unitários e de contrato HTTP.
#[derive(Debug, Clone, Default)]
pub struct InMemoryEventRepository {
    events: Arc<Mutex<Vec<Event>>>,
    registrations: Arc<Mutex<Vec<EventRegistration>>>,
}

impl InMemoryEventRepository {
    /// Cria um repositório em memória vazio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cria um repositório em memória pré-populado com a coleção de eventos informada.
    pub fn with_events(events: impl IntoIterator<Item = Event>) -> Self {
        Self {
            events: Arc::new(Mutex::new(events.into_iter().collect())),
            registrations: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl EventRepository for InMemoryEventRepository {
    async fn create(&self, event: &Event) -> Result<Event, EventApplicationError> {
        self.events.lock().unwrap().push(event.clone());
        Ok(event.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, EventApplicationError> {
        Ok(self
            .events
            .lock()
            .unwrap()
            .iter()
            .find(|event| event.id() == id)
            .cloned())
    }

    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError> {
        let mut guard = self.events.lock().unwrap();
        if let Some(existing) = guard.iter_mut().find(|item| item.id() == event.id()) {
            *existing = event.clone();
            Ok(event.clone())
        } else {
            Err(EventApplicationError::NotFound(event.id()))
        }
    }

    async fn list_by_academic_center(
        &self,
        academic_center_id: Uuid,
        only_published: bool,
    ) -> Result<Vec<Event>, EventApplicationError> {
        let mut results: Vec<Event> = self
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| {
                event.academic_center_id() == academic_center_id
                    && (!only_published || event.is_published())
            })
            .cloned()
            .collect();
        results.sort_by(|a, b| {
            a.starts_at()
                .cmp(&b.starts_at())
                .then_with(|| a.id().cmp(&b.id()))
        });
        Ok(results)
    }

    async fn find_registration(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
    ) -> Result<Option<EventRegistration>, EventApplicationError> {
        Ok(self
            .registrations
            .lock()
            .unwrap()
            .iter()
            .find(|item| item.event_id() == event_id && item.participant_id() == participant_id)
            .cloned())
    }

    async fn save_registration(
        &self,
        registration: &EventRegistration,
    ) -> Result<EventRegistration, EventApplicationError> {
        let mut guard = self.registrations.lock().unwrap();
        if let Some(existing) = guard.iter_mut().find(|item| {
            item.event_id() == registration.event_id()
                && item.participant_id() == registration.participant_id()
        }) {
            *existing = registration.clone();
            Ok(existing.clone())
        } else {
            guard.push(registration.clone());
            Ok(registration.clone())
        }
    }

    async fn list_registrations_by_event(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<EventRegistration>, EventApplicationError> {
        Ok(self
            .registrations
            .lock()
            .unwrap()
            .iter()
            .filter(|item| item.event_id() == event_id)
            .cloned()
            .collect())
    }

    async fn register_participant_atomically(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
        academic_center_id: Option<Uuid>,
    ) -> Result<EventRegistration, EventApplicationError> {
        if participant_id.is_nil() {
            return Err(EventApplicationError::Domain(
                EventDomainError::InvalidParticipantId,
            ));
        }

        let mut events_guard = self.events.lock().unwrap();
        let mut registrations_guard = self.registrations.lock().unwrap();

        let event = events_guard
            .iter_mut()
            .find(|item| item.id() == event_id)
            .ok_or(EventApplicationError::NotFound(event_id))?;

        if let Some(active_ca_id) = academic_center_id
            && event.academic_center_id() != active_ca_id
        {
            return Err(EventApplicationError::NotFound(event_id));
        }

        if !event.is_published() {
            return Err(EventApplicationError::EventNotAvailableForRegistration(
                event_id,
            ));
        }

        if let Some(existing) = registrations_guard
            .iter()
            .find(|item| item.event_id() == event_id && item.participant_id() == participant_id)
            && existing.is_active()
        {
            return Err(EventApplicationError::AlreadyRegistered {
                event_id,
                participant_id,
            });
        }

        let active_count = u32::try_from(
            registrations_guard
                .iter()
                .filter(|item| item.event_id() == event_id && item.is_active())
                .count(),
        )
        .unwrap_or(u32::MAX);
        event
            .register_participant(chrono::Utc::now(), active_count)
            .map_err(|err| match err {
                EventDomainError::CapacityExceeded { max_capacity, .. } => {
                    EventApplicationError::EventFull {
                        event_id,
                        max_capacity,
                    }
                }
                other => EventApplicationError::Domain(other),
            })?;

        if let Some(existing) = registrations_guard
            .iter_mut()
            .find(|item| item.event_id() == event_id && item.participant_id() == participant_id)
        {
            existing.reactivate_registered(chrono::Utc::now())?;
            Ok(existing.clone())
        } else {
            let registration = EventRegistration::new_confirmed(event_id, participant_id)?;
            registrations_guard.push(registration.clone());
            Ok(registration)
        }
    }

    async fn cancel_registration_atomically(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
        academic_center_id: Option<Uuid>,
    ) -> Result<EventRegistration, EventApplicationError> {
        if participant_id.is_nil() {
            return Err(EventApplicationError::Domain(
                EventDomainError::InvalidParticipantId,
            ));
        }

        let mut events_guard = self.events.lock().unwrap();
        let mut registrations_guard = self.registrations.lock().unwrap();

        let event = events_guard
            .iter_mut()
            .find(|item| item.id() == event_id)
            .ok_or(EventApplicationError::NotFound(event_id))?;

        if let Some(active_ca_id) = academic_center_id
            && event.academic_center_id() != active_ca_id
        {
            return Err(EventApplicationError::NotFound(event_id));
        }

        let cancelled = {
            let existing = registrations_guard
                .iter_mut()
                .find(|item| {
                    item.event_id() == event_id
                        && item.participant_id() == participant_id
                        && item.is_active()
                })
                .ok_or(EventApplicationError::ActiveRegistrationNotFound {
                    event_id,
                    participant_id,
                })?;
            existing.cancel();
            existing.clone()
        };

        Ok(cancelled)
    }
}

/// Contrato de controle de acesso para verificar permissões administrativas sobre eventos no CA.
#[async_trait]
pub trait EventAuthorizationPort: Send + Sync {
    /// Verifica se `user_id` detém a permissão administrativa
    /// [`DIRECT_EVENT_MANAGEMENT_PERMISSION`](crate::modules::events::domain::DIRECT_EVENT_MANAGEMENT_PERMISSION)
    /// na gestão do Centro Acadêmico `academic_center_id`.
    async fn can_manage_and_publish_directly(
        &self,
        user_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<bool, EventApplicationError>;
}

/// Adaptador permissivo padrão para contextos onde o controle de acesso é avaliado externamente.
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAllEventAuthorization;

#[async_trait]
impl EventAuthorizationPort for AllowAllEventAuthorization {
    async fn can_manage_and_publish_directly(
        &self,
        _user_id: Uuid,
        _academic_center_id: Uuid,
    ) -> Result<bool, EventApplicationError> {
        Ok(true)
    }
}

/// Adaptador determinístico em memória para controle de permissões `(user_id, academic_center_id)`.
#[derive(Debug, Default, Clone)]
pub struct StaticEventAuthorization {
    authorized_pairs: HashSet<(Uuid, Uuid)>,
}

impl StaticEventAuthorization {
    /// Cria um autorizador em memória inicialmente vazio (nega por padrão).
    pub fn new() -> Self {
        Self::default()
    }

    /// Concede a permissão de publicação direta e gerenciamento de eventos para `user_id` em `academic_center_id`.
    pub fn grant_direct_management(&mut self, user_id: Uuid, academic_center_id: Uuid) {
        self.authorized_pairs.insert((user_id, academic_center_id));
    }
}

#[async_trait]
impl EventAuthorizationPort for StaticEventAuthorization {
    async fn can_manage_and_publish_directly(
        &self,
        user_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<bool, EventApplicationError> {
        Ok(self
            .authorized_pairs
            .contains(&(user_id, academic_center_id)))
    }
}

/// Serviço de aplicação responsável por coordenar os casos de uso de eventos.
#[derive(Debug, Clone)]
pub struct EventService<R, A = AllowAllEventAuthorization> {
    repository: R,
    authorizer: A,
}

impl<R> EventService<R, AllowAllEventAuthorization>
where
    R: EventRepository,
{
    /// Constrói o serviço de eventos injetando o repositório com autorizador permissivo padrão.
    pub const fn new(repository: R) -> Self {
        Self {
            repository,
            authorizer: AllowAllEventAuthorization,
        }
    }
}

impl<R, A> EventService<R, A>
where
    R: EventRepository,
    A: EventAuthorizationPort,
{
    /// Constrói o serviço de eventos injetando o repositório e a porta de autorização institucional.
    pub const fn with_authorizer(repository: R, authorizer: A) -> Self {
        Self {
            repository,
            authorizer,
        }
    }

    /// Cria um novo evento (como rascunho ou publicado diretamente) após validar a permissão do autor
    /// no CA e todas as invariantes de domínio.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::PermissionDenied`] se o autor não tiver permissão no CA.
    /// - [`EventApplicationError::Domain`] se os campos obrigatórios ou o período forem inválidos.
    /// - [`EventApplicationError::Repository`] em caso de erro de persistência.
    pub async fn create_event(
        &self,
        command: CreateEventCommand,
    ) -> Result<Event, EventApplicationError> {
        let authorized = self
            .authorizer
            .can_manage_and_publish_directly(command.author_id, command.academic_center_id)
            .await?;
        if !authorized {
            return Err(EventApplicationError::PermissionDenied);
        }

        let event = if command.publish_directly {
            Event::create_published(
                command.academic_center_id,
                command.author_id,
                command.title,
                command.description,
                command.location,
                command.starts_at,
                command.ends_at,
                command.capacity,
                command.registration,
            )?
        } else {
            Event::create_draft(
                command.academic_center_id,
                command.author_id,
                command.title,
                command.description,
                command.location,
                command.starts_at,
                command.ends_at,
                command.capacity,
                command.registration,
            )?
        };

        self.repository.create(&event).await
    }

    /// Atualiza as informações de um evento existente preservando sua validade e situação de publicação.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::NotFound`] se o evento não existir.
    /// - [`EventApplicationError::PermissionDenied`] se o evento pertencer a outro CA ou o ator não tiver permissão.
    /// - [`EventApplicationError::Domain`] se os novos dados violarem regras de negócio.
    pub async fn update_event(
        &self,
        command: UpdateEventCommand,
    ) -> Result<Event, EventApplicationError> {
        let mut event = self
            .repository
            .find_by_id(command.event_id)
            .await?
            .ok_or(EventApplicationError::NotFound(command.event_id))?;

        if let Some(ca_id) = command.academic_center_id
            && event.academic_center_id() != ca_id
        {
            return Err(EventApplicationError::PermissionDenied);
        }

        if let Some(actor_id) = command.actor_id {
            let authorized = self
                .authorizer
                .can_manage_and_publish_directly(actor_id, event.academic_center_id())
                .await?;
            if !authorized {
                return Err(EventApplicationError::PermissionDenied);
            }
        }

        event.update_details(
            command.title,
            command.description,
            command.location,
            command.starts_at,
            command.ends_at,
        )?;

        if command.capacity.is_some() || command.registration.is_some() {
            let capacity = command.capacity.unwrap_or_else(|| event.capacity());
            let registration = command
                .registration
                .unwrap_or_else(|| event.registration().clone());
            event.configure_registration(capacity, registration)?;
        }

        self.repository.update(&event).await
    }

    /// Publica diretamente um evento salvo em rascunho no Centro Acadêmico, sem exigir homologação,
    /// validando que o ator autenticado possui [`DIRECT_EVENT_MANAGEMENT_PERMISSION`](crate::modules::events::domain::DIRECT_EVENT_MANAGEMENT_PERMISSION) no CA do evento.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::NotFound`] se o evento não existir.
    /// - [`EventApplicationError::PermissionDenied`] se o ator não estiver identificado, não tiver permissão no CA ou o CA divergir.
    /// - [`EventApplicationError::Domain`] se a transição de estado for inválida.
    pub async fn publish_event_directly(
        &self,
        command: PublishEventCommand,
    ) -> Result<Event, EventApplicationError> {
        let mut event = self
            .repository
            .find_by_id(command.event_id)
            .await?
            .ok_or(EventApplicationError::NotFound(command.event_id))?;

        if let Some(ca_id) = command.academic_center_id
            && event.academic_center_id() != ca_id
        {
            return Err(EventApplicationError::PermissionDenied);
        }

        let actor_id = command
            .actor_id
            .ok_or(EventApplicationError::PermissionDenied)?;
        let authorized = self
            .authorizer
            .can_manage_and_publish_directly(actor_id, event.academic_center_id())
            .await?;
        if !authorized {
            return Err(EventApplicationError::PermissionDenied);
        }

        event.publish_directly()?;
        self.repository.update(&event).await
    }

    /// Obtém os detalhes de um evento pelo seu identificador.
    ///
    /// Quando `query.only_published` for `true`, eventos não publicados retornam
    /// [`EventApplicationError::NotFound`]. Quando `query.academic_center_id` for
    /// informado, eventos pertencentes a outro CA também retornam [`EventApplicationError::NotFound`].
    pub async fn get_event_by_id(
        &self,
        query: GetEventByIdQuery,
    ) -> Result<Event, EventApplicationError> {
        let event = self
            .repository
            .find_by_id(query.event_id)
            .await?
            .ok_or(EventApplicationError::NotFound(query.event_id))?;

        if let Some(active_ca_id) = query.academic_center_id
            && event.academic_center_id() != active_ca_id
        {
            return Err(EventApplicationError::NotFound(query.event_id));
        }

        if query.only_published && !event.is_published() {
            return Err(EventApplicationError::NotFound(query.event_id));
        }

        Ok(event)
    }

    /// Obtém os detalhes de um evento publicado, garantindo que eventos não publicados
    /// (rascunho, pendente de homologação, rejeitado ou cancelado) ou pertencentes a
    /// outro CA não sejam disponibilizados.
    pub async fn get_published_event_detail(
        &self,
        query: GetPublishedEventDetailQuery,
    ) -> Result<Event, EventApplicationError> {
        self.get_event_by_id(GetEventByIdQuery {
            event_id: query.event_id,
            academic_center_id: query.academic_center_id,
            only_published: true,
        })
        .await
    }

    /// Lista os eventos de um Centro Acadêmico (todos ou somente os publicados na agenda).
    pub async fn list_ca_events(
        &self,
        query: ListCaEventsQuery,
    ) -> Result<Vec<Event>, EventApplicationError> {
        let events = self
            .repository
            .list_by_academic_center(query.academic_center_id, query.only_published)
            .await?;

        Ok(events
            .into_iter()
            .filter(|event| {
                event.academic_center_id() == query.academic_center_id
                    && (!query.only_published || event.is_published())
            })
            .collect())
    }

    /// Lista exclusivamente os eventos publicados do Centro Acadêmico ativo.
    pub async fn list_published_events_by_ca(
        &self,
        query: ListPublishedEventsByCaQuery,
    ) -> Result<Vec<Event>, EventApplicationError> {
        self.repository
            .find_published_by_ca(query.academic_center_id)
            .await
    }

    /// Inscreve um usuário autenticado em um evento publicado.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::Domain`] se `participant_id` for nulo.
    /// - [`EventApplicationError::NotFound`] se o evento não existir ou pertencer a outro CA.
    /// - [`EventApplicationError::EventNotAvailableForRegistration`] se o evento não estiver publicado.
    /// - [`EventApplicationError::AlreadyRegistered`] se o usuário já possuir inscrição ativa no evento.
    /// - [`EventApplicationError::EventFull`] se o evento tiver atingido o limite máximo de vagas configurado.
    pub async fn register_in_event(
        &self,
        command: RegisterInEventCommand,
    ) -> Result<EventRegistration, EventApplicationError> {
        self.repository
            .register_participant_atomically(
                command.event_id,
                command.participant_id,
                command.academic_center_id,
            )
            .await
    }

    /// Cancela a inscrição ativa de um usuário autenticado em um evento.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::Domain`] se `participant_id` for nulo.
    /// - [`EventApplicationError::NotFound`] se o evento não existir ou pertencer a outro CA.
    /// - [`EventApplicationError::ActiveRegistrationNotFound`] se o usuário não possuir inscrição ativa no evento.
    pub async fn cancel_registration(
        &self,
        command: CancelEventRegistrationCommand,
    ) -> Result<EventRegistration, EventApplicationError> {
        self.repository
            .cancel_registration_atomically(
                command.event_id,
                command.participant_id,
                command.academic_center_id,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;
    use crate::modules::events::{
        application::queries::{GetPublishedEventDetailUseCase, ListPublishedEventsByCaUseCase},
        domain::{EventPublicationStatus, EventRegistrationInfo},
    };

    fn make_event_with_status(
        ca_id: Uuid,
        author_id: Uuid,
        title: &str,
        description: &str,
        status: EventPublicationStatus,
    ) -> Event {
        let now = Utc::now();
        let starts_at = now + Duration::days(3);
        let ends_at = starts_at + Duration::hours(2);
        Event::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            title,
            description,
            "Auditório 2",
            starts_at,
            ends_at,
            None,
            status,
            1,
            EventRegistrationInfo::unrestricted(),
        )
        .expect("evento de teste deve ser válido")
    }

    #[tokio::test]
    async fn allows_direct_publication_only_for_user_with_permission_in_ca() {
        let ca_id = Uuid::new_v4();
        let other_ca_id = Uuid::new_v4();
        let authorized_user = Uuid::new_v4();
        let unauthorized_user = Uuid::new_v4();

        let mut authorizer = StaticEventAuthorization::new();
        authorizer.grant_direct_management(authorized_user, ca_id);
        authorizer.grant_direct_management(unauthorized_user, other_ca_id);

        let repo = InMemoryEventRepository::default();
        let service = EventService::with_authorizer(repo, authorizer);

        let starts_at = Utc::now() + Duration::days(3);
        let ends_at = starts_at + Duration::hours(2);

        // Criação em rascunho por usuário autorizado deve funcionar.
        let draft = service
            .create_event(CreateEventCommand {
                capacity: None,
                academic_center_id: ca_id,
                author_id: authorized_user,
                title: "Oficina de Git".to_string(),
                description: "Controle de versão na prática".to_string(),
                location: "Lab B".to_string(),
                starts_at,
                ends_at,
                publish_directly: false,
                registration: EventRegistrationInfo::unrestricted(),
            })
            .await
            .expect("autorizado deve criar rascunho");
        assert_eq!(draft.status(), EventPublicationStatus::Draft);

        // Publicação direta por usuário sem permissão no CA deve ser negada.
        let denied = service
            .publish_event_directly(PublishEventCommand {
                event_id: draft.id(),
                academic_center_id: Some(ca_id),
                actor_id: Some(unauthorized_user),
            })
            .await;
        assert!(matches!(
            denied,
            Err(EventApplicationError::PermissionDenied)
        ));

        // Publicação direta pelo usuário autorizado deve publicar sem homologação.
        let published = service
            .publish_event_directly(PublishEventCommand {
                event_id: draft.id(),
                academic_center_id: Some(ca_id),
                actor_id: Some(authorized_user),
            })
            .await
            .expect("autorizado deve publicar diretamente");
        assert_eq!(published.status(), EventPublicationStatus::Published);
    }

    #[tokio::test]
    async fn lists_only_published_events_from_active_academic_center() {
        let ca_computacao = Uuid::new_v4();
        let ca_matematica = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let semana_academica = make_event_with_status(
            ca_computacao,
            author_id,
            "Semana Acadêmica",
            "Ciclo de palestras de computação",
            EventPublicationStatus::Published,
        );
        let palestra_ia = make_event_with_status(
            ca_computacao,
            author_id,
            "Palestra de IA",
            "Introdução a modelos fundacionais",
            EventPublicationStatus::Published,
        );
        let hackathon_em_analise = make_event_with_status(
            ca_computacao,
            author_id,
            "Hackathon em análise",
            "Proposta aguardando homologação",
            EventPublicationStatus::PendingReview,
        );
        let encontro_rascunho = make_event_with_status(
            ca_computacao,
            author_id,
            "Encontro em rascunho",
            "Rascunho interno da gestão",
            EventPublicationStatus::Draft,
        );
        let evento_rejeitado = make_event_with_status(
            ca_computacao,
            author_id,
            "Evento rejeitado",
            "Proposta recusada",
            EventPublicationStatus::Rejected,
        );
        let semana_matematica = make_event_with_status(
            ca_matematica,
            author_id,
            "Semana de Matemática",
            "Evento de outro CA",
            EventPublicationStatus::Published,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events([
            semana_academica,
            palestra_ia,
            hackathon_em_analise,
            encontro_rascunho,
            evento_rejeitado,
            semana_matematica,
        ]));

        let use_case = ListPublishedEventsByCaUseCase::new(repo.clone());
        let events_computacao = use_case
            .execute(ListPublishedEventsByCaQuery::new(ca_computacao))
            .await
            .expect("deve listar eventos publicados do CA de Computação");

        let titles: Vec<&str> = events_computacao.iter().map(|e| e.title()).collect();
        assert_eq!(titles, vec!["Semana Acadêmica", "Palestra de IA"]);

        // Alternar o CA ativo para CA de Matemática deve retornar apenas os eventos desse CA.
        let events_matematica = use_case
            .execute(ListPublishedEventsByCaQuery::new(ca_matematica))
            .await
            .expect("deve listar eventos publicados do CA de Matemática");

        let titles_mat: Vec<&str> = events_matematica.iter().map(|e| e.title()).collect();
        assert_eq!(titles_mat, vec!["Semana de Matemática"]);
    }

    #[tokio::test]
    async fn returns_empty_list_when_active_ca_has_no_published_events() {
        let ca_computacao = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let rascunho = make_event_with_status(
            ca_computacao,
            author_id,
            "Encontro em rascunho",
            "Não publicado",
            EventPublicationStatus::Draft,
        );
        let pendente = make_event_with_status(
            ca_computacao,
            author_id,
            "Hackathon em análise",
            "Aguardando homologação",
            EventPublicationStatus::PendingReview,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events([rascunho, pendente]));
        let use_case = ListPublishedEventsByCaUseCase::new(repo);

        let events = use_case
            .execute(ListPublishedEventsByCaQuery::new(ca_computacao))
            .await
            .expect("deve retornar sucesso com lista vazia");

        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn returns_published_event_details_and_rejects_unpublished_or_other_ca() {
        let ca_computacao = Uuid::new_v4();
        let ca_matematica = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let publicado = make_event_with_status(
            ca_computacao,
            author_id,
            "Mudança de sala",
            "A palestra acontecerá no auditório 2.",
            EventPublicationStatus::Published,
        );
        let pendente = make_event_with_status(
            ca_computacao,
            author_id,
            "Hackathon em análise",
            "Proposta não homologada",
            EventPublicationStatus::PendingReview,
        );
        let rascunho = make_event_with_status(
            ca_computacao,
            author_id,
            "Rascunho interno",
            "Ainda em edição",
            EventPublicationStatus::Draft,
        );
        let cancelado = make_event_with_status(
            ca_computacao,
            author_id,
            "Evento cancelado",
            "Cancelado pela organização",
            EventPublicationStatus::Cancelled,
        );

        let repo = Arc::new(InMemoryEventRepository::with_events([
            publicado.clone(),
            pendente.clone(),
            rascunho.clone(),
            cancelado.clone(),
        ]));
        let detail_use_case = GetPublishedEventDetailUseCase::new(repo);

        // Detalhes de evento publicado no CA ativo devem ser retornados.
        let detail = detail_use_case
            .execute(GetPublishedEventDetailQuery::for_academic_center(
                publicado.id(),
                ca_computacao,
            ))
            .await
            .expect("deve retornar detalhes do evento publicado");
        assert_eq!(detail.title(), "Mudança de sala");
        assert_eq!(
            detail.description(),
            "A palestra acontecerá no auditório 2."
        );

        // Detalhes do mesmo evento quando o CA ativo é outro devem ser recusados (isolamento de CA).
        let wrong_ca = detail_use_case
            .execute(GetPublishedEventDetailQuery::for_academic_center(
                publicado.id(),
                ca_matematica,
            ))
            .await;
        assert_eq!(
            wrong_ca,
            Err(EventApplicationError::NotFound(publicado.id()))
        );

        // Eventos não publicados não devem ter seus detalhes disponibilizados.
        for unpublished in [&pendente, &rascunho, &cancelado] {
            let res = detail_use_case
                .execute(GetPublishedEventDetailQuery::for_academic_center(
                    unpublished.id(),
                    ca_computacao,
                ))
                .await;
            assert_eq!(res, Err(EventApplicationError::NotFound(unpublished.id())));
        }
    }

    #[tokio::test]
    async fn registers_and_cancels_participation_in_published_event() {
        let ca_computacao = Uuid::new_v4();
        let author = Uuid::new_v4();
        let student = Uuid::new_v4();

        let palestra_ia = make_event_with_status(
            ca_computacao,
            author,
            "Palestra de IA",
            "Debate sobre modelos generativos",
            EventPublicationStatus::Published,
        );

        let repo = InMemoryEventRepository::with_events(vec![palestra_ia.clone()]);
        let service = EventService::new(repo.clone());

        // 1. Inscrição em evento publicado é registrada como confirmada
        let registration = service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                palestra_ia.id(),
                student,
                ca_computacao,
            ))
            .await
            .expect("deve registrar inscrição em evento publicado");
        assert_eq!(registration.event_id(), palestra_ia.id());
        assert_eq!(registration.participant_id(), student);
        assert!(registration.is_active());
        assert!(registration.cancelled_at().is_none());

        let updated_event = repo.find_by_id(palestra_ia.id()).await.unwrap().unwrap();
        assert_eq!(
            repo.list_registrations_by_event(updated_event.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            1
        );

        // 2. Tentativa de inscrição duplicada no mesmo evento é recusada mantendo inscrição única
        let duplicate = service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                palestra_ia.id(),
                student,
                ca_computacao,
            ))
            .await;
        assert_eq!(
            duplicate,
            Err(EventApplicationError::AlreadyRegistered {
                event_id: palestra_ia.id(),
                participant_id: student,
            })
        );
        let registrations = repo
            .list_registrations_by_event(palestra_ia.id())
            .await
            .unwrap();
        assert_eq!(registrations.len(), 1);

        // 3. Cancelamento da inscrição ativa pelo próprio estudante desativa a inscrição
        let cancelled = service
            .cancel_registration(CancelEventRegistrationCommand::for_academic_center(
                palestra_ia.id(),
                student,
                ca_computacao,
            ))
            .await
            .expect("deve cancelar inscrição ativa");
        assert!(!cancelled.is_active());
        assert!(cancelled.cancelled_at().is_some());

        let event_after_cancel = repo.find_by_id(palestra_ia.id()).await.unwrap().unwrap();
        assert_eq!(
            repo.list_registrations_by_event(event_after_cancel.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            0
        );

        // 4. Tentativa de cancelar novamente sem inscrição ativa é informada com erro semântico
        let second_cancel = service
            .cancel_registration(CancelEventRegistrationCommand::for_academic_center(
                palestra_ia.id(),
                student,
                ca_computacao,
            ))
            .await;
        assert_eq!(
            second_cancel,
            Err(EventApplicationError::ActiveRegistrationNotFound {
                event_id: palestra_ia.id(),
                participant_id: student,
            })
        );
    }

    #[tokio::test]
    async fn rejects_registration_for_unpublished_or_other_ca_and_cancellation_without_active_registration()
     {
        let ca_computacao = Uuid::new_v4();
        let ca_fisica = Uuid::new_v4();
        let author = Uuid::new_v4();
        let student = Uuid::new_v4();

        let publicado_cc = make_event_with_status(
            ca_computacao,
            author,
            "Palestra de IA",
            "Debate sobre modelos generativos",
            EventPublicationStatus::Published,
        );
        let pendente_cc = make_event_with_status(
            ca_computacao,
            author,
            "Hackathon em análise",
            "Proposta aguardando homologação",
            EventPublicationStatus::PendingReview,
        );
        let rascunho_cc = make_event_with_status(
            ca_computacao,
            author,
            "Oficina interna",
            "Rascunho ainda não publicado",
            EventPublicationStatus::Draft,
        );

        let repo = InMemoryEventRepository::with_events(vec![
            publicado_cc.clone(),
            pendente_cc.clone(),
            rascunho_cc.clone(),
        ]);
        let service = EventService::new(repo.clone());

        // 1. Não permite inscrição em proposta não homologada ou rascunho
        for unpublished in [&pendente_cc, &rascunho_cc] {
            let res = service
                .register_in_event(RegisterInEventCommand::for_academic_center(
                    unpublished.id(),
                    student,
                    ca_computacao,
                ))
                .await;
            assert_eq!(
                res,
                Err(EventApplicationError::EventNotAvailableForRegistration(
                    unpublished.id()
                ))
            );
            assert!(
                repo.find_registration(unpublished.id(), student)
                    .await
                    .unwrap()
                    .is_none()
            );
        }

        // 2. Não permite inscrição quando o evento pertence a outro CA
        let wrong_ca = service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                publicado_cc.id(),
                student,
                ca_fisica,
            ))
            .await;
        assert_eq!(
            wrong_ca,
            Err(EventApplicationError::NotFound(publicado_cc.id()))
        );

        // 3. Informa tentativa de cancelar quando o estudante nunca se inscreveu no evento
        let cancel_without_registration = service
            .cancel_registration(CancelEventRegistrationCommand::for_academic_center(
                publicado_cc.id(),
                student,
                ca_computacao,
            ))
            .await;
        assert_eq!(
            cancel_without_registration,
            Err(EventApplicationError::ActiveRegistrationNotFound {
                event_id: publicado_cc.id(),
                participant_id: student,
            })
        );
    }

    #[tokio::test]
    async fn configures_and_validates_optional_max_capacity_on_create_and_update() {
        let ca_computacao = Uuid::new_v4();
        let author = Uuid::new_v4();
        let starts_at = Utc::now() + Duration::days(2);
        let ends_at = starts_at + Duration::hours(3);

        let repo = InMemoryEventRepository::new();
        let service = EventService::new(repo.clone());

        // 1. Criação com limite de 3 vagas persiste max_capacity = Some(3)
        let event_with_limit = service
            .create_event(CreateEventCommand {
                capacity: Some(3),
                academic_center_id: ca_computacao,
                author_id: author,
                title: "Palestra de IA".to_string(),
                description: "Controle opcional de vagas".to_string(),
                location: "Auditório 1".to_string(),
                starts_at,
                ends_at,
                publish_directly: true,
                registration: EventRegistrationInfo::unrestricted(),
            })
            .await
            .expect("deve criar evento com limite de 3 vagas");
        assert_eq!(event_with_limit.capacity(), Some(3));

        // 2. Criação com limite inválido (0) no comando é rejeitada pelo domínio
        let invalid_create = service
            .create_event(CreateEventCommand {
                capacity: Some(0),
                academic_center_id: ca_computacao,
                author_id: author,
                title: "Evento Inválido".to_string(),
                description: "Capacidade zero".to_string(),
                location: "Sala 2".to_string(),
                starts_at,
                ends_at,
                publish_directly: false,
                registration: EventRegistrationInfo::unrestricted(),
            })
            .await;
        assert_eq!(
            invalid_create,
            Err(EventApplicationError::Domain(
                EventDomainError::InvalidMaxCapacity(0)
            ))
        );

        // 3. Edição para remover o limite (None) atualiza max_capacity para None
        let unlimited = service
            .update_event(UpdateEventCommand {
                capacity: Some(None),
                event_id: event_with_limit.id(),
                academic_center_id: Some(ca_computacao),
                actor_id: Some(author),
                title: "Palestra de IA".to_string(),
                description: "Controle opcional de vagas".to_string(),
                location: "Auditório 1".to_string(),
                starts_at,
                ends_at,
                registration: Some(EventRegistrationInfo::unrestricted()),
            })
            .await
            .expect("deve permitir remover o limite de vagas");
        assert_eq!(unlimited.capacity(), None);

        // 4. Edição com limite inválido (0) é rejeitada mantendo o evento intacto
        let invalid_update = service
            .update_event(UpdateEventCommand {
                capacity: Some(Some(0)),
                event_id: event_with_limit.id(),
                academic_center_id: Some(ca_computacao),
                actor_id: Some(author),
                title: "Palestra de IA".to_string(),
                description: "Controle opcional de vagas".to_string(),
                location: "Auditório 1".to_string(),
                starts_at,
                ends_at,
                registration: Some(EventRegistrationInfo::unrestricted()),
            })
            .await;
        assert_eq!(
            invalid_update,
            Err(EventApplicationError::Domain(
                EventDomainError::InvalidMaxCapacity(0)
            ))
        );
    }

    #[tokio::test]
    async fn enforces_max_capacity_releases_spot_on_cancel_and_allows_unlimited_events() {
        let ca_computacao = Uuid::new_v4();
        let author = Uuid::new_v4();
        let starts_at = Utc::now() + Duration::days(2);
        let ends_at = starts_at + Duration::hours(2);

        let limited_event = Event::create_published(
            ca_computacao,
            author,
            "Palestra de IA",
            "Capacidade limitada a 2 vagas",
            "Auditório 1",
            starts_at,
            ends_at,
            Some(2),
            EventRegistrationInfo::unrestricted(),
        )
        .unwrap();

        let unlimited_event = Event::create_published(
            ca_computacao,
            author,
            "Semana Acadêmica",
            "Evento sem limite de vagas",
            "Anfiteatro",
            starts_at,
            ends_at,
            None,
            EventRegistrationInfo::unrestricted(),
        )
        .unwrap();

        let repo = InMemoryEventRepository::with_events(vec![
            limited_event.clone(),
            unlimited_event.clone(),
        ]);
        let service = EventService::new(repo.clone());

        let student_1 = Uuid::new_v4();
        let student_2 = Uuid::new_v4();
        let student_3 = Uuid::new_v4();

        // 1. Primeiros 2 participantes ocupam as 2 vagas disponíveis
        service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                limited_event.id(),
                student_1,
                ca_computacao,
            ))
            .await
            .expect("deve inscrever primeiro participante");

        // Repetir inscrição para o mesmo participante antes de lotar não consome a segunda vaga
        let repeat_before_full = service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                limited_event.id(),
                student_1,
                ca_computacao,
            ))
            .await;
        assert_eq!(
            repeat_before_full,
            Err(EventApplicationError::AlreadyRegistered {
                event_id: limited_event.id(),
                participant_id: student_1,
            })
        );

        service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                limited_event.id(),
                student_2,
                ca_computacao,
            ))
            .await
            .expect("deve inscrever segundo participante na última vaga");

        let event_full_state = repo.find_by_id(limited_event.id()).await.unwrap().unwrap();
        assert_eq!(
            repo.list_registrations_by_event(event_full_state.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            2
        );
        assert_eq!(
            event_full_state.remaining_spots(
                repo.list_registrations_by_event(event_full_state.id())
                    .await
                    .unwrap()
                    .iter()
                    .filter(|r| r.is_active())
                    .count() as u32
            ),
            Some(0)
        );
        assert!(
            !event_full_state.has_available_spots(
                repo.list_registrations_by_event(event_full_state.id())
                    .await
                    .unwrap()
                    .iter()
                    .filter(|r| r.is_active())
                    .count() as u32
            )
        );

        // 2. Quando todas as vagas estão ocupadas, nova inscrição direta é recusada com EventFull
        let rejected_full = service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                limited_event.id(),
                student_3,
                ca_computacao,
            ))
            .await;
        assert_eq!(
            rejected_full,
            Err(EventApplicationError::EventFull {
                event_id: limited_event.id(),
                max_capacity: 2,
            })
        );

        // Repetir inscrição de quem já está inscrito mesmo com evento lotado retorna AlreadyRegistered
        let repeat_when_full = service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                limited_event.id(),
                student_1,
                ca_computacao,
            ))
            .await;
        assert_eq!(
            repeat_when_full,
            Err(EventApplicationError::AlreadyRegistered {
                event_id: limited_event.id(),
                participant_id: student_1,
            })
        );

        // 3. Cancelamento de student_1 libera 1 vaga e permite que student_3 se inscreva
        service
            .cancel_registration(CancelEventRegistrationCommand::for_academic_center(
                limited_event.id(),
                student_1,
                ca_computacao,
            ))
            .await
            .expect("deve cancelar inscrição de student_1 e liberar uma vaga");

        let after_cancel = repo.find_by_id(limited_event.id()).await.unwrap().unwrap();
        assert_eq!(
            repo.list_registrations_by_event(after_cancel.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            1
        );
        assert_eq!(
            after_cancel.remaining_spots(
                repo.list_registrations_by_event(after_cancel.id())
                    .await
                    .unwrap()
                    .iter()
                    .filter(|r| r.is_active())
                    .count() as u32
            ),
            Some(1)
        );

        service
            .register_in_event(RegisterInEventCommand::for_academic_center(
                limited_event.id(),
                student_3,
                ca_computacao,
            ))
            .await
            .expect("deve permitir inscrição de student_3 na vaga liberada");

        let after_reoccupy = repo.find_by_id(limited_event.id()).await.unwrap().unwrap();
        assert_eq!(
            repo.list_registrations_by_event(after_reoccupy.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            2
        );

        // 4. Evento sem limite de vagas (max_capacity = None) aceita todos sem restrição artificial
        for _ in 0..10 {
            service
                .register_in_event(RegisterInEventCommand::for_academic_center(
                    unlimited_event.id(),
                    Uuid::new_v4(),
                    ca_computacao,
                ))
                .await
                .expect("evento sem limite não deve recusar por capacidade");
        }
        let unlimited_after = repo
            .find_by_id(unlimited_event.id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            repo.list_registrations_by_event(unlimited_after.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            10
        );
        assert_eq!(unlimited_after.capacity(), None);
    }

    #[tokio::test]
    async fn concurrent_registrations_never_exceed_configured_max_capacity() {
        let ca_computacao = Uuid::new_v4();
        let author = Uuid::new_v4();
        let starts_at = Utc::now() + Duration::days(2);
        let ends_at = starts_at + Duration::hours(2);

        // Cenário BDD: Evento publicado "Palestra de IA" possui limite de 3 vagas e já possui 2 inscritos
        let event = Event::create_published(
            ca_computacao,
            author,
            "Palestra de IA",
            "Concorrência pela última vaga",
            "Auditório 1",
            starts_at,
            ends_at,
            Some(3),
            EventRegistrationInfo::unrestricted(),
        )
        .unwrap();

        let repo = InMemoryEventRepository::with_events(vec![event.clone()]);
        let service = EventService::new(repo.clone());

        for _ in 0..2 {
            service
                .register_in_event(RegisterInEventCommand::for_academic_center(
                    event.id(),
                    Uuid::new_v4(),
                    ca_computacao,
                ))
                .await
                .expect("deve registrar participante inicial");
        }

        // 10 pessoas sem inscrição solicitam simultaneamente a última vaga disponível
        let barrier = Arc::new(tokio::sync::Barrier::new(10));
        let mut handles = Vec::with_capacity(10);

        for _ in 0..10 {
            let svc = service.clone();
            let bar = barrier.clone();
            let event_id = event.id();
            handles.push(tokio::spawn(async move {
                let candidate = Uuid::new_v4();
                bar.wait().await;
                svc.register_in_event(RegisterInEventCommand::for_academic_center(
                    event_id,
                    candidate,
                    ca_computacao,
                ))
                .await
            }));
        }

        let mut successes = 0usize;
        let mut full_errors = 0usize;

        for handle in handles {
            match handle.await.expect("tarefa concorrente deve concluir") {
                Ok(reg) => {
                    assert!(reg.is_active());
                    successes += 1;
                }
                Err(EventApplicationError::EventFull {
                    event_id,
                    max_capacity,
                }) => {
                    assert_eq!(event_id, event.id());
                    assert_eq!(max_capacity, 3);
                    full_errors += 1;
                }
                Err(other) => panic!("erro inesperado na disputa concorrente de vaga: {other:?}"),
            }
        }

        // Somente 1 nova inscrição ativa é registrada e o evento fica com 3 participantes inscritos
        assert_eq!(successes, 1);
        assert_eq!(full_errors, 9);

        let updated_event = repo.find_by_id(event.id()).await.unwrap().unwrap();
        assert_eq!(
            repo.list_registrations_by_event(updated_event.id())
                .await
                .unwrap()
                .iter()
                .filter(|r| r.is_active())
                .count() as u32,
            3
        );

        let active_registrations = repo
            .list_registrations_by_event(event.id())
            .await
            .unwrap()
            .into_iter()
            .filter(EventRegistration::is_active)
            .count();
        assert_eq!(active_registrations, 3);
    }
}
