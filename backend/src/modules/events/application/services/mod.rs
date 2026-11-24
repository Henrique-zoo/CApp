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
    commands::{CreateEventCommand, PublishEventCommand, UpdateEventCommand},
    queries::{
        GetEventByIdQuery, GetPublishedEventDetailQuery, ListCaEventsQuery,
        ListPublishedEventsByCaQuery,
    },
};
use crate::modules::events::domain::{Event, EventDomainError};

/// Erros produzidos pelos casos de uso da camada de aplicação de eventos.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EventApplicationError {
    /// Violação de uma regra de negócio ou validação do modelo de domínio.
    #[error(transparent)]
    Domain(#[from] EventDomainError),

    /// O evento solicitado não foi encontrado ou não está disponível no contexto consultado.
    #[error("evento '{0}' não encontrado")]
    NotFound(Uuid),

    /// O usuário não possui permissão para gerenciar ou publicar eventos diretamente no Centro Acadêmico informado.
    #[error("operação recusada por falta de permissão no Centro Acadêmico")]
    PermissionDenied,

    /// Falha técnica na camada de persistência ou autorização.
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
            .filter(|event| event.academic_center_id == academic_center_id && event.is_published())
            .collect())
    }

    /// Atualiza os dados de um evento existente no armazenamento.
    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError>;
}

/// Repositório de eventos em memória para testes unitários e de contrato HTTP.
#[derive(Debug, Clone, Default)]
pub struct InMemoryEventRepository {
    events: Arc<Mutex<Vec<Event>>>,
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
            .find(|event| event.id == id)
            .cloned())
    }

    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError> {
        let mut guard = self.events.lock().unwrap();
        if let Some(existing) = guard.iter_mut().find(|item| item.id == event.id) {
            *existing = event.clone();
            Ok(event.clone())
        } else {
            Err(EventApplicationError::NotFound(event.id))
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
                event.academic_center_id == academic_center_id
                    && (!only_published || event.is_published())
            })
            .cloned()
            .collect();
        results.sort_by(|a, b| {
            a.starts_at
                .cmp(&b.starts_at)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        Ok(results)
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
                .can_manage_and_publish_directly(actor_id, event.academic_center_id)
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
            .can_manage_and_publish_directly(actor_id, event.academic_center_id)
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
            && event.academic_center_id != active_ca_id
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
                event.academic_center_id == query.academic_center_id
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
        let published_at = if status == EventPublicationStatus::Published {
            Some(now)
        } else {
            None
        };
        Event::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            title,
            description,
            "Auditório 2",
            starts_at,
            ends_at,
            status,
            EventRegistrationInfo::unrestricted(),
            published_at,
            now,
            now,
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
        assert_eq!(draft.status, EventPublicationStatus::Draft);

        // Publicação direta por usuário sem permissão no CA deve ser negada.
        let denied = service
            .publish_event_directly(PublishEventCommand {
                event_id: draft.id,
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
                event_id: draft.id,
                academic_center_id: Some(ca_id),
                actor_id: Some(authorized_user),
            })
            .await
            .expect("autorizado deve publicar diretamente");
        assert_eq!(published.status, EventPublicationStatus::Published);
        assert!(published.published_at.is_some());
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
            EventPublicationStatus::PendingApproval,
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

        let titles: Vec<&str> = events_computacao.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(titles, vec!["Semana Acadêmica", "Palestra de IA"]);

        // Alternar o CA ativo para CA de Matemática deve retornar apenas os eventos desse CA.
        let events_matematica = use_case
            .execute(ListPublishedEventsByCaQuery::new(ca_matematica))
            .await
            .expect("deve listar eventos publicados do CA de Matemática");

        let titles_mat: Vec<&str> = events_matematica.iter().map(|e| e.title.as_str()).collect();
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
            EventPublicationStatus::PendingApproval,
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
            EventPublicationStatus::PendingApproval,
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
                publicado.id,
                ca_computacao,
            ))
            .await
            .expect("deve retornar detalhes do evento publicado");
        assert_eq!(detail.title, "Mudança de sala");
        assert_eq!(detail.description, "A palestra acontecerá no auditório 2.");

        // Detalhes do mesmo evento quando o CA ativo é outro devem ser recusados (isolamento de CA).
        let wrong_ca = detail_use_case
            .execute(GetPublishedEventDetailQuery::for_academic_center(
                publicado.id,
                ca_matematica,
            ))
            .await;
        assert_eq!(wrong_ca, Err(EventApplicationError::NotFound(publicado.id)));

        // Eventos não publicados não devem ter seus detalhes disponibilizados.
        for unpublished in [&pendente, &rascunho, &cancelado] {
            let res = detail_use_case
                .execute(GetPublishedEventDetailQuery::for_academic_center(
                    unpublished.id,
                    ca_computacao,
                ))
                .await;
            assert_eq!(res, Err(EventApplicationError::NotFound(unpublished.id)));
        }
    }
}
