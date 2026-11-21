//! Comandos e casos de uso de escrita da camada de aplicação para criação, edição,
//! publicação direta, inscrição e cancelamento de inscrição em eventos.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::services::{EventApplicationError, EventRepository};
use crate::modules::events::domain::{EventRegistration, EventRegistrationInfo};

/// Comando para criar um novo evento (em rascunho ou publicado diretamente) em um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateEventCommand {
    /// Identificador do Centro Acadêmico responsável pelo evento.
    pub academic_center_id: Uuid,
    /// Identificador do gestor/usuário autor do evento.
    pub author_id: Uuid,
    /// Título do evento.
    pub title: String,
    /// Descrição do evento.
    pub description: String,
    /// Local de realização do evento.
    pub location: String,
    /// Data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Data e horário de término do evento.
    pub ends_at: DateTime<Utc>,
    /// Quando `true`, publica o evento diretamente sem exigir homologação; quando `false`, salva como rascunho.
    pub publish_directly: bool,
    /// Configuração de inscrição e limite opcional de vagas.
    pub registration: EventRegistrationInfo,
}

/// Comando para editar as informações de um evento existente no Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEventCommand {
    /// Identificador do evento a ser editado.
    pub event_id: Uuid,
    /// Identificador opcional do Centro Acadêmico no qual a operação está sendo solicitada.
    pub academic_center_id: Option<Uuid>,
    /// Identificador opcional do usuário que solicita a edição.
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
    /// Nova configuração opcional de inscrição.
    pub registration: Option<EventRegistrationInfo>,
}

/// Comando para publicar diretamente um evento salvo em rascunho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishEventCommand {
    /// Identificador do evento a ser publicado.
    pub event_id: Uuid,
    /// Identificador opcional do Centro Acadêmico no qual a publicação é solicitada.
    pub academic_center_id: Option<Uuid>,
    /// Identificador opcional do gestor que executa a publicação direta.
    pub actor_id: Option<Uuid>,
}

/// Comando para inscrever um usuário autenticado em um evento publicado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterInEventCommand {
    /// Identificador do evento no qual o estudante deseja se inscrever.
    pub event_id: Uuid,
    /// Identificador do usuário autenticado solicitante da inscrição.
    pub participant_id: Uuid,
    /// Identificador opcional do Centro Acadêmico ativo para validação de isolamento.
    pub academic_center_id: Option<Uuid>,
}

impl RegisterInEventCommand {
    /// Constrói um comando de inscrição em um evento.
    pub const fn new(event_id: Uuid, participant_id: Uuid) -> Self {
        Self {
            event_id,
            participant_id,
            academic_center_id: None,
        }
    }

    /// Constrói um comando de inscrição exigindo que o evento pertença ao Centro Acadêmico ativo informado.
    pub const fn for_academic_center(
        event_id: Uuid,
        participant_id: Uuid,
        academic_center_id: Uuid,
    ) -> Self {
        Self {
            event_id,
            participant_id,
            academic_center_id: Some(academic_center_id),
        }
    }
}

/// Comando para cancelar a inscrição ativa de um usuário autenticado em um evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelEventRegistrationCommand {
    /// Identificador do evento cuja inscrição será cancelada.
    pub event_id: Uuid,
    /// Identificador do usuário autenticado titular da inscrição.
    pub participant_id: Uuid,
    /// Identificador opcional do Centro Acadêmico ativo para validação de isolamento.
    pub academic_center_id: Option<Uuid>,
}

impl CancelEventRegistrationCommand {
    /// Constrói um comando de cancelamento de inscrição em um evento.
    pub const fn new(event_id: Uuid, participant_id: Uuid) -> Self {
        Self {
            event_id,
            participant_id,
            academic_center_id: None,
        }
    }

    /// Constrói um comando de cancelamento exigindo que o evento pertença ao Centro Acadêmico ativo informado.
    pub const fn for_academic_center(
        event_id: Uuid,
        participant_id: Uuid,
        academic_center_id: Uuid,
    ) -> Self {
        Self {
            event_id,
            participant_id,
            academic_center_id: Some(academic_center_id),
        }
    }
}

/// Caso de uso para registrar a inscrição de um usuário autenticado em um evento publicado.
#[derive(Debug, Clone)]
pub struct RegisterParticipantInEventUseCase<R>
where
    R: EventRepository + ?Sized,
{
    repository: Arc<R>,
}

impl<R> RegisterParticipantInEventUseCase<R>
where
    R: EventRepository + ?Sized,
{
    /// Constrói o caso de uso de inscrição injetando o repositório de eventos.
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa a inscrição do participante no evento publicado.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::Domain`] se `participant_id` for nulo.
    /// - [`EventApplicationError::NotFound`] se o evento não existir ou pertencer a outro CA.
    /// - [`EventApplicationError::EventNotAvailableForRegistration`] se o evento não estiver publicado.
    /// - [`EventApplicationError::AlreadyRegistered`] se o usuário já possuir inscrição ativa no evento.
    /// - [`EventApplicationError::EventFull`] se o evento tiver atingido o limite máximo de vagas configurado.
    pub async fn execute(
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
}

/// Caso de uso para cancelar a inscrição ativa de um usuário autenticado em um evento.
#[derive(Debug, Clone)]
pub struct CancelEventRegistrationUseCase<R>
where
    R: EventRepository + ?Sized,
{
    repository: Arc<R>,
}

impl<R> CancelEventRegistrationUseCase<R>
where
    R: EventRepository + ?Sized,
{
    /// Constrói o caso de uso de cancelamento de inscrição injetando o repositório de eventos.
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa o cancelamento da inscrição ativa do participante no evento.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::Domain`] se `participant_id` for nulo.
    /// - [`EventApplicationError::NotFound`] se o evento não existir ou pertencer a outro CA.
    /// - [`EventApplicationError::ActiveRegistrationNotFound`] se o participante não possuir inscrição ativa no evento.
    pub async fn execute(
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
