//! Serviços de aplicação e contratos de repositório para o módulo de eventos.
//!
//! Orquestra a criação, edição, publicação direta e consulta de eventos,
//! aplicando as validações do domínio ([`Event`]) antes de persistir no repositório.

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::{
    commands::{CreateEventCommand, PublishEventCommand, UpdateEventCommand},
    queries::{GetEventByIdQuery, ListCaEventsQuery},
};
use crate::modules::events::domain::{Event, EventDomainError};

/// Erros produzidos pelos casos de uso da camada de aplicação de eventos.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EventApplicationError {
    /// Violação de uma regra de negócio ou validação do modelo de domínio.
    #[error(transparent)]
    Domain(#[from] EventDomainError),

    /// O evento solicitado não foi encontrado.
    #[error("evento '{0}' não encontrado")]
    NotFound(Uuid),

    /// O usuário não possui permissão para gerenciar eventos no Centro Acadêmico informado.
    #[error("operação recusada por falta de permissão no Centro Acadêmico")]
    PermissionDenied,

    /// Falha técnica na camada de persistência.
    #[error("erro ao acessar o repositório de eventos: {0}")]
    Repository(String),
}

/// Contrato de persistência para o repositório de eventos.
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

    /// Atualiza os dados de um evento existente no armazenamento.
    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError>;
}

/// Serviço de aplicação responsável por coordenar os casos de uso de eventos.
#[derive(Debug, Clone)]
pub struct EventService<R> {
    repository: R,
}

impl<R> EventService<R>
where
    R: EventRepository,
{
    /// Constrói o serviço de eventos injetando a implementação de [`EventRepository`].
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Cria um novo evento (como rascunho ou publicado diretamente) após validar todas as invariantes de domínio.
    ///
    /// # Erros
    ///
    /// Retorna [`EventApplicationError::Domain`] se os campos obrigatórios ou o período forem inválidos,
    /// ou [`EventApplicationError::Repository`] em caso de erro de persistência.
    pub async fn create_event(
        &self,
        command: CreateEventCommand,
    ) -> Result<Event, EventApplicationError> {
        let event = if command.publish_directly {
            Event::create_published(
                command.academic_center_id,
                command.author_id,
                command.title,
                command.description,
                command.location,
                command.starts_at,
                command.ends_at,
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
    /// - [`EventApplicationError::PermissionDenied`] se o evento pertencer a outro CA.
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
            && event.academic_center_id != ca_id
        {
            return Err(EventApplicationError::PermissionDenied);
        }

        event.update_details(
            command.title,
            command.description,
            command.location,
            command.starts_at,
            command.ends_at,
        )?;

        if let Some(registration) = command.registration {
            event.configure_registration(registration)?;
        }

        self.repository.update(&event).await
    }

    /// Publica diretamente um evento salvo em rascunho no Centro Acadêmico.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::NotFound`] se o evento não existir.
    /// - [`EventApplicationError::PermissionDenied`] se o evento pertencer a outro CA.
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
            && event.academic_center_id != ca_id
        {
            return Err(EventApplicationError::PermissionDenied);
        }

        event.publish_directly()?;
        self.repository.update(&event).await
    }

    /// Obtém os detalhes de um evento pelo seu identificador.
    ///
    /// Quando `query.only_published` for `true`, eventos não publicados retornam
    /// [`EventApplicationError::NotFound`].
    pub async fn get_event_by_id(
        &self,
        query: GetEventByIdQuery,
    ) -> Result<Event, EventApplicationError> {
        let event = self
            .repository
            .find_by_id(query.event_id)
            .await?
            .ok_or(EventApplicationError::NotFound(query.event_id))?;

        if query.only_published && !event.is_published() {
            return Err(EventApplicationError::NotFound(query.event_id));
        }

        Ok(event)
    }

    /// Lista os eventos de um Centro Acadêmico (todos ou somente os publicados na agenda).
    pub async fn list_ca_events(
        &self,
        query: ListCaEventsQuery,
    ) -> Result<Vec<Event>, EventApplicationError> {
        self.repository
            .list_by_academic_center(query.academic_center_id, query.only_published)
            .await
    }
}
