//! Consultas e casos de uso de leitura da camada de aplicação do módulo de eventos.
//!
//! Implementa as operações de consulta da agenda pública de eventos de um Centro Acadêmico
//! ativo e de visualização de detalhes de um evento publicado (§ 4.4 do relatório de features),
//! garantindo o isolamento entre CAs e a exclusão de rascunhos e propostas não homologadas.

use std::sync::Arc;

use uuid::Uuid;

use crate::modules::events::{
    application::services::{EventApplicationError, EventRepository},
    domain::Event,
};

/// Consulta para buscar um evento específico pelo seu identificador único.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetEventByIdQuery {
    /// Identificador único do evento.
    pub event_id: Uuid,
    /// Identificador opcional do Centro Acadêmico ativo para validar isolamento de contexto.
    pub academic_center_id: Option<Uuid>,
    /// Quando `true`, oculta eventos que ainda não estejam publicados (ex.: visão pública de estudantes).
    pub only_published: bool,
}

/// Consulta para listar os eventos pertencentes a um Centro Acadêmico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListCaEventsQuery {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Quando `true`, retorna apenas os eventos publicados na agenda oficial do CA.
    pub only_published: bool,
}

/// Consulta para listar os eventos publicados na agenda pública do Centro Acadêmico ativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListPublishedEventsByCaQuery {
    /// Identificador do Centro Acadêmico ativo selecionado pelo usuário.
    pub academic_center_id: Uuid,
}

impl ListPublishedEventsByCaQuery {
    /// Constrói uma consulta de listagem de eventos publicados para o CA ativo informado.
    pub const fn new(academic_center_id: Uuid) -> Self {
        Self { academic_center_id }
    }
}

/// Consulta para obter os detalhes de um evento publicado no contexto de um Centro Acadêmico ativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetPublishedEventDetailQuery {
    /// Identificador único do evento consultado.
    pub event_id: Uuid,
    /// Identificador opcional do Centro Acadêmico ativo para garantir isolamento entre CAs.
    pub academic_center_id: Option<Uuid>,
}

impl GetPublishedEventDetailQuery {
    /// Constrói uma consulta de detalhes de evento publicado sem filtro explícito de CA.
    pub const fn new(event_id: Uuid) -> Self {
        Self {
            event_id,
            academic_center_id: None,
        }
    }

    /// Constrói uma consulta de detalhes de evento publicado restrita ao CA ativo informado.
    pub const fn for_academic_center(event_id: Uuid, academic_center_id: Uuid) -> Self {
        Self {
            event_id,
            academic_center_id: Some(academic_center_id),
        }
    }
}

/// Caso de uso para listar os eventos publicados do Centro Acadêmico ativo.
///
/// Retorna somente eventos cujo status seja [`EventPublicationStatus::Published`](crate::modules::events::domain::EventPublicationStatus::Published)
/// pertencentes ao `academic_center_id` informado, excluindo rascunhos, propostas pendentes,
/// rejeitadas, canceladas e eventos de outros CAs.
#[derive(Debug, Clone)]
pub struct ListPublishedEventsByCaUseCase<R>
where
    R: EventRepository + ?Sized,
{
    repository: Arc<R>,
}

impl<R> ListPublishedEventsByCaUseCase<R>
where
    R: EventRepository + ?Sized,
{
    /// Constrói o caso de uso injetando o repositório de eventos.
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa a listagem de eventos publicados do CA ativo.
    ///
    /// # Erros
    ///
    /// Retorna [`EventApplicationError::Repository`] caso ocorra falha ao consultar o repositório.
    pub async fn execute(
        &self,
        query: ListPublishedEventsByCaQuery,
    ) -> Result<Vec<Event>, EventApplicationError> {
        let events = self
            .repository
            .list_by_academic_center(query.academic_center_id, true)
            .await?;

        Ok(events
            .into_iter()
            .filter(|event| {
                event.academic_center_id() == query.academic_center_id && event.is_published()
            })
            .collect())
    }
}

/// Caso de uso para obter as informações detalhadas de um evento publicado.
///
/// Garante que:
/// - rascunhos, propostas pendentes, eventos rejeitados ou cancelados não tenham detalhes expostos;
/// - quando um CA ativo (`academic_center_id`) for informado, eventos de outros CAs não sejam retornados.
#[derive(Debug, Clone)]
pub struct GetPublishedEventDetailUseCase<R>
where
    R: EventRepository + ?Sized,
{
    repository: Arc<R>,
}

impl<R> GetPublishedEventDetailUseCase<R>
where
    R: EventRepository + ?Sized,
{
    /// Constrói o caso de uso injetando o repositório de eventos.
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Executa a consulta de detalhes de um evento publicado.
    ///
    /// # Erros
    ///
    /// - [`EventApplicationError::NotFound`] se o evento não existir, não estiver publicado ou pertencer a outro CA.
    /// - [`EventApplicationError::Repository`] em caso de falha de leitura no repositório.
    pub async fn execute(
        &self,
        query: GetPublishedEventDetailQuery,
    ) -> Result<Event, EventApplicationError> {
        let event = self
            .repository
            .find_by_id(query.event_id)
            .await?
            .ok_or(EventApplicationError::NotFound(query.event_id))?;

        if !event.is_published() {
            return Err(EventApplicationError::NotFound(query.event_id));
        }

        if let Some(active_ca_id) = query.academic_center_id
            && event.academic_center_id() != active_ca_id
        {
            return Err(EventApplicationError::NotFound(query.event_id));
        }

        Ok(event)
    }
}
