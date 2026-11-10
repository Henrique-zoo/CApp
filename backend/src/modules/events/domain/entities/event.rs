//! Entidades, objetos de valor e enums do domínio de Eventos.
//!
//! Modela os eventos promovidos pelos Centros Acadêmicos no CApp, contemplando:
//! - informações básicas obrigatórias (`title`, `description`, `location`);
//! - Centro Acadêmico responsável (`academic_center_id`) e autor da gestão (`author_id`);
//! - período do evento (`starts_at` e `ends_at`, exigindo `ends_at > starts_at`);
//! - ciclo de vida e estado de publicação ([`EventPublicationStatus`]);
//! - configuração e controle de inscrições ([`EventRegistrationInfo`], [`EventRegistration`] e [`RegistrationStatus`]).
//!
//! Referência: `relatorio_features_app_ca.pdf`, § 4.1, e especificação BDD em
//! `features/eventos_ca/criacao_edicao_eventos.feature`.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Comprimento máximo permitido (em caracteres) para o título de um evento.
pub const MAX_EVENT_TITLE_LENGTH: usize = 200;

/// Comprimento máximo permitido (em caracteres) para a descrição de um evento.
pub const MAX_EVENT_DESCRIPTION_LENGTH: usize = 5000;

/// Comprimento máximo permitido (em caracteres) para o local de um evento.
pub const MAX_EVENT_LOCATION_LENGTH: usize = 300;

/// Erros de validação e regras de negócio do domínio de eventos.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EventDomainError {
    /// O título do evento é obrigatório e não pode ser vazio ou apenas espaços.
    #[error("o título do evento é obrigatório e não pode ser vazio")]
    EmptyTitle,

    /// O título do evento ultrapassou o limite máximo de caracteres.
    #[error("o título do evento excede o limite de {max} caracteres")]
    TitleTooLong {
        /// Limite máximo de caracteres permitido.
        max: usize,
    },

    /// A descrição do evento é obrigatória e não pode ser vazia ou apenas espaços.
    #[error("a descrição do evento é obrigatória e não pode ser vazia")]
    EmptyDescription,

    /// A descrição do evento ultrapassou o limite máximo de caracteres.
    #[error("a descrição do evento excede o limite de {max} caracteres")]
    DescriptionTooLong {
        /// Limite máximo de caracteres permitido.
        max: usize,
    },

    /// O local do evento é obrigatório e não pode ser vazio ou apenas espaços.
    #[error("o local do evento é obrigatório e não pode ser vazio")]
    EmptyLocation,

    /// O local do evento ultrapassou o limite máximo de caracteres.
    #[error("o local do evento excede o limite de {max} caracteres")]
    LocationTooLong {
        /// Limite máximo de caracteres permitido.
        max: usize,
    },

    /// O identificador do Centro Acadêmico responsável não pode ser o UUID nulo.
    #[error("o identificador do Centro Acadêmico responsável não pode ser nulo")]
    InvalidAcademicCenterId,

    /// O identificador do autor responsável não pode ser o UUID nulo.
    #[error("o identificador do autor responsável não pode ser nulo")]
    InvalidAuthorId,

    /// O identificador do participante da inscrição não pode ser o UUID nulo.
    #[error("o identificador do participante não pode ser nulo")]
    InvalidParticipantId,

    /// O horário de término deve ser estritamente posterior ao horário de início.
    #[error("o horário de término ({ends_at}) deve ser posterior ao início ({starts_at})")]
    InvalidEventPeriod {
        /// Data e horário de início informados.
        starts_at: DateTime<Utc>,
        /// Data e horário de término informados.
        ends_at: DateTime<Utc>,
    },

    /// O estado de publicação textual informado não é reconhecido pelo domínio.
    #[error("estado de publicação de evento inválido: '{0}'")]
    InvalidPublicationStatus(String),

    /// A transição solicitada entre estados de publicação não é permitida.
    #[error("transição de estado de publicação inválida de '{from}' para '{to}'")]
    InvalidStatusTransition {
        /// Estado atual do evento.
        from: EventPublicationStatus,
        /// Estado pretendido na transição.
        to: EventPublicationStatus,
    },

    /// O estado de inscrição textual informado não é reconhecido pelo domínio.
    #[error("estado de inscrição em evento inválido: '{0}'")]
    InvalidRegistrationStatus(String),

    /// O limite máximo de vagas configurado deve ser um inteiro estritamente positivo.
    #[error("o limite máximo de vagas deve ser maior que zero (informado: {0})")]
    InvalidMaxCapacity(u32),

    /// O número de inscrições confirmadas ultrapassa a capacidade máxima configurada.
    #[error(
        "o total de inscrições confirmadas ({confirmed}) excede o limite de vagas ({max_capacity})"
    )]
    CapacityExceeded {
        /// Quantidade de inscrições confirmadas.
        confirmed: u32,
        /// Capacidade máxima do evento.
        max_capacity: u32,
    },

    /// O prazo limite para inscrição não pode ser posterior ao término do evento.
    #[error(
        "o prazo limite de inscrição ({deadline}) não pode ser posterior ao término do evento ({ends_at})"
    )]
    RegistrationDeadlineAfterEventEnd {
        /// Prazo limite de inscrição informado.
        deadline: DateTime<Utc>,
        /// Data/hora de término do evento.
        ends_at: DateTime<Utc>,
    },

    /// O evento não aceita inscrições (inscrições desabilitadas ou encerradas).
    #[error("este evento não aceita inscrições")]
    RegistrationsClosed,

    /// A operação exige que o evento esteja publicado na agenda do CA.
    #[error("o evento precisa estar publicado para receber inscrições")]
    EventNotPublished,

    /// Um evento em estado publicado exige que o instante de publicação esteja registrado.
    #[error("um evento publicado deve possuir data de publicação registrada")]
    MissingPublishedTimestamp,

    /// Os instantes de criação e atualização são temporalmente inconsistentes.
    #[error(
        "a data de atualização ({updated_at}) não pode ser anterior à data de criação ({created_at})"
    )]
    InconsistentTimestamps {
        /// Instante de criação do registro.
        created_at: DateTime<Utc>,
        /// Instante da última atualização.
        updated_at: DateTime<Utc>,
    },
}

/// Estado de publicação de um evento no ciclo de vida do Centro Acadêmico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventPublicationStatus {
    /// Evento salvo como rascunho pela gestão antes da divulgação oficial.
    #[default]
    Draft,
    /// Proposta de evento submetida e aguardando análise de homologação.
    PendingApproval,
    /// Proposta aprovada pela homologação do CA.
    Approved,
    /// Proposta rejeitada na etapa de homologação.
    Rejected,
    /// Evento publicado e visível na agenda oficial de eventos do CA.
    Published,
    /// Evento cancelado após criação ou publicação.
    Cancelled,
}

impl EventPublicationStatus {
    /// Retorna a representação textual canônica do estado de publicação.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingApproval => "pending_approval",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Published => "published",
            Self::Cancelled => "cancelled",
        }
    }

    /// Indica se o evento deve figurar na agenda pública de eventos publicados do CA.
    pub const fn is_visible_in_schedule(&self) -> bool {
        matches!(self, Self::Published)
    }

    /// Indica se o estado atual permite novas inscrições de participantes.
    pub const fn can_accept_registrations(&self) -> bool {
        matches!(self, Self::Published)
    }
}

impl fmt::Display for EventPublicationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for EventPublicationStatus {
    type Err = EventDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "draft" | "rascunho" => Ok(Self::Draft),
            "pending_approval" | "pendente" => Ok(Self::PendingApproval),
            "approved" | "aprovado" => Ok(Self::Approved),
            "rejected" | "rejeitado" => Ok(Self::Rejected),
            "published" | "publicado" => Ok(Self::Published),
            "cancelled" | "canceled" | "cancelado" => Ok(Self::Cancelled),
            other => Err(EventDomainError::InvalidPublicationStatus(
                other.to_string(),
            )),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for EventPublicationStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for EventPublicationStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let text = <&'r str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        text.parse()
            .map_err(|err: EventDomainError| Box::new(err) as _)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for EventPublicationStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + 'static + Send + Sync>> {
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

/// Período de realização de um evento (início e término).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPeriod {
    /// Data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Data e horário de término do evento (deve ser posterior a `starts_at`).
    pub ends_at: DateTime<Utc>,
}

impl EventPeriod {
    /// Constrói um período validando que o término é estritamente posterior ao início.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::InvalidEventPeriod`] se `ends_at <= starts_at`.
    pub fn new(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Result<Self, EventDomainError> {
        if ends_at <= starts_at {
            return Err(EventDomainError::InvalidEventPeriod { starts_at, ends_at });
        }
        Ok(Self { starts_at, ends_at })
    }
}

/// Informações e regras de inscrição configuradas para um evento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRegistrationInfo {
    /// Indica se o evento aceita inscrições de participantes na plataforma.
    pub requires_registration: bool,
    /// Limite máximo opcional de participantes (`None` significa vagas ilimitadas).
    pub max_capacity: Option<u32>,
    /// Quantidade atual de inscrições confirmadas no evento.
    pub confirmed_registrations: u32,
    /// Data/hora limite opcional para realização de inscrições.
    pub registration_deadline: Option<DateTime<Utc>>,
}

impl Default for EventRegistrationInfo {
    fn default() -> Self {
        Self::unrestricted()
    }
}

impl EventRegistrationInfo {
    /// Cria uma configuração de inscrição aberta e sem limite máximo de vagas.
    pub const fn unrestricted() -> Self {
        Self {
            requires_registration: true,
            max_capacity: None,
            confirmed_registrations: 0,
            registration_deadline: None,
        }
    }

    /// Cria uma configuração para evento puramente informativo (sem necessidade de inscrição).
    pub const fn disabled() -> Self {
        Self {
            requires_registration: false,
            max_capacity: None,
            confirmed_registrations: 0,
            registration_deadline: None,
        }
    }

    /// Cria uma configuração de inscrição com limite máximo de vagas definido.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::InvalidMaxCapacity`] se `max_capacity == 0`.
    pub fn with_capacity(max_capacity: u32) -> Result<Self, EventDomainError> {
        if max_capacity == 0 {
            return Err(EventDomainError::InvalidMaxCapacity(0));
        }
        Ok(Self {
            requires_registration: true,
            max_capacity: Some(max_capacity),
            confirmed_registrations: 0,
            registration_deadline: None,
        })
    }

    /// Valida a consistência das informações de inscrição em relação ao horário de término do evento.
    pub fn validate(&self, event_ends_at: DateTime<Utc>) -> Result<(), EventDomainError> {
        if let Some(limit) = self.max_capacity {
            if limit == 0 {
                return Err(EventDomainError::InvalidMaxCapacity(0));
            }
            if self.confirmed_registrations > limit {
                return Err(EventDomainError::CapacityExceeded {
                    confirmed: self.confirmed_registrations,
                    max_capacity: limit,
                });
            }
        }

        if let Some(deadline) = self.registration_deadline
            && deadline > event_ends_at
        {
            return Err(EventDomainError::RegistrationDeadlineAfterEventEnd {
                deadline,
                ends_at: event_ends_at,
            });
        }

        Ok(())
    }

    /// Indica se ainda há vagas disponíveis para inscrição.
    pub const fn has_available_spots(&self) -> bool {
        if !self.requires_registration {
            return false;
        }
        match self.max_capacity {
            Some(limit) => self.confirmed_registrations < limit,
            None => true,
        }
    }

    /// Retorna o número de vagas restantes quando o evento possui capacidade máxima configurada.
    pub const fn remaining_spots(&self) -> Option<u32> {
        match self.max_capacity {
            Some(limit) => Some(limit.saturating_sub(self.confirmed_registrations)),
            None => None,
        }
    }
}

/// Estado da inscrição individual de um participante em um evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    /// Inscrição ativa e confirmada dentro das vagas do evento.
    #[default]
    Confirmed,
    /// Inscrição cancelada pelo participante.
    Cancelled,
    /// Participante aguardando na fila de espera de um evento lotado.
    Waitlisted,
}

impl RegistrationStatus {
    /// Retorna a representação textual canônica do status da inscrição.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Cancelled => "cancelled",
            Self::Waitlisted => "waitlisted",
        }
    }
}

impl fmt::Display for RegistrationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RegistrationStatus {
    type Err = EventDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "confirmed" | "confirmada" => Ok(Self::Confirmed),
            "cancelled" | "canceled" | "cancelada" => Ok(Self::Cancelled),
            "waitlisted" | "fila_espera" => Ok(Self::Waitlisted),
            other => Err(EventDomainError::InvalidRegistrationStatus(
                other.to_string(),
            )),
        }
    }
}

/// Entidade que representa a inscrição de um usuário participante em um evento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRegistration {
    /// Identificador único da inscrição.
    pub id: Uuid,
    /// Identificador do evento ao qual a inscrição pertence.
    pub event_id: Uuid,
    /// Identificador do usuário inscrito no evento.
    pub participant_id: Uuid,
    /// Situação atual da inscrição.
    pub status: RegistrationStatus,
    /// Data e horário em que a inscrição foi realizada.
    pub registered_at: DateTime<Utc>,
    /// Data e horário de cancelamento, caso a inscrição tenha sido cancelada.
    pub cancelled_at: Option<DateTime<Utc>>,
}

impl EventRegistration {
    /// Cria uma nova inscrição confirmada para um participante em um evento.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::InvalidParticipantId`] caso `participant_id` seja nulo.
    pub fn new_confirmed(event_id: Uuid, participant_id: Uuid) -> Result<Self, EventDomainError> {
        if participant_id.is_nil() {
            return Err(EventDomainError::InvalidParticipantId);
        }
        Ok(Self {
            id: Uuid::new_v4(),
            event_id,
            participant_id,
            status: RegistrationStatus::Confirmed,
            registered_at: Utc::now(),
            cancelled_at: None,
        })
    }

    /// Cancela a inscrição registrando o instante do cancelamento.
    pub fn cancel(&mut self) {
        if self.status != RegistrationStatus::Cancelled {
            self.status = RegistrationStatus::Cancelled;
            self.cancelled_at = Some(Utc::now());
        }
    }
}

/// Entidade principal de domínio que representa um Evento promovido por um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Identificador único do evento.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico responsável pelo evento.
    pub academic_center_id: Uuid,
    /// Identificador do integrante da gestão / autor responsável pelo evento.
    pub author_id: Uuid,
    /// Título oficial do evento.
    pub title: String,
    /// Descrição detalhada sobre o objetivo e programação do evento.
    pub description: String,
    /// Local de realização do evento.
    pub location: String,
    /// Data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Data e horário de término do evento.
    pub ends_at: DateTime<Utc>,
    /// Estado atual de publicação do evento no CA.
    pub status: EventPublicationStatus,
    /// Informações de inscrição e controle opcional de vagas.
    pub registration: EventRegistrationInfo,
    /// Instante em que o evento foi publicado na agenda do CA.
    pub published_at: Option<DateTime<Utc>>,
    /// Instante de criação do registro do evento.
    pub created_at: DateTime<Utc>,
    /// Instante da última atualização do evento.
    pub updated_at: DateTime<Utc>,
}

impl Event {
    /// Constrói uma instância de [`Event`] validando todas as invariantes de negócio.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError`] caso campos obrigatórios estejam vazios, excedam os
    /// limites de tamanho, o término não seja posterior ao início ou a configuração de
    /// inscrição seja inconsistente.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        academic_center_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        status: EventPublicationStatus,
        registration: EventRegistrationInfo,
        published_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, EventDomainError> {
        let event = Self {
            id,
            academic_center_id,
            author_id,
            title: title.into().trim().to_string(),
            description: description.into().trim().to_string(),
            location: location.into().trim().to_string(),
            starts_at,
            ends_at,
            status,
            registration,
            published_at,
            created_at,
            updated_at,
        };
        event.validate()?;
        Ok(event)
    }

    /// Cria um novo evento salvo inicialmente como rascunho ([`EventPublicationStatus::Draft`]).
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError`] se qualquer informação básica ou período for inválido.
    #[allow(clippy::too_many_arguments)]
    pub fn create_draft(
        academic_center_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        registration: EventRegistrationInfo,
    ) -> Result<Self, EventDomainError> {
        let now = Utc::now();
        Self::new(
            Uuid::new_v4(),
            academic_center_id,
            author_id,
            title,
            description,
            location,
            starts_at,
            ends_at,
            EventPublicationStatus::Draft,
            registration,
            None,
            now,
            now,
        )
    }

    /// Cria e publica diretamente um evento ([`EventPublicationStatus::Published`]) sem exigir homologação.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError`] se qualquer informação básica ou período for inválido.
    #[allow(clippy::too_many_arguments)]
    pub fn create_published(
        academic_center_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        registration: EventRegistrationInfo,
    ) -> Result<Self, EventDomainError> {
        let now = Utc::now();
        Self::new(
            Uuid::new_v4(),
            academic_center_id,
            author_id,
            title,
            description,
            location,
            starts_at,
            ends_at,
            EventPublicationStatus::Published,
            registration,
            Some(now),
            now,
            now,
        )
    }

    /// Valida todas as regras de negócio e invariantes estruturais do evento.
    ///
    /// # Regras verificadas
    ///
    /// - `academic_center_id` e `author_id` não podem ser nulos (`Uuid::nil()`).
    /// - `title`, `description` e `location` não podem ser vazios e devem respeitar os tamanhos máximos.
    /// - `ends_at` deve ser estritamente posterior a `starts_at`.
    /// - `registration` deve possuir capacidade válida e prazo compatível com `ends_at`.
    /// - Eventos com status [`EventPublicationStatus::Published`] devem possuir `published_at` preenchido.
    /// - `updated_at` não pode ser anterior a `created_at`.
    pub fn validate(&self) -> Result<(), EventDomainError> {
        if self.academic_center_id.is_nil() {
            return Err(EventDomainError::InvalidAcademicCenterId);
        }
        if self.author_id.is_nil() {
            return Err(EventDomainError::InvalidAuthorId);
        }

        Self::validate_basic_fields(&self.title, &self.description, &self.location)?;
        EventPeriod::new(self.starts_at, self.ends_at)?;
        self.registration.validate(self.ends_at)?;

        if self.status == EventPublicationStatus::Published && self.published_at.is_none() {
            return Err(EventDomainError::MissingPublishedTimestamp);
        }

        if self.updated_at < self.created_at {
            return Err(EventDomainError::InconsistentTimestamps {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }

        Ok(())
    }

    /// Valida os campos textuais obrigatórios do evento (`title`, `description` e `location`).
    pub fn validate_basic_fields(
        title: &str,
        description: &str,
        location: &str,
    ) -> Result<(), EventDomainError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(EventDomainError::EmptyTitle);
        }
        if trimmed_title.chars().count() > MAX_EVENT_TITLE_LENGTH {
            return Err(EventDomainError::TitleTooLong {
                max: MAX_EVENT_TITLE_LENGTH,
            });
        }

        let trimmed_description = description.trim();
        if trimmed_description.is_empty() {
            return Err(EventDomainError::EmptyDescription);
        }
        if trimmed_description.chars().count() > MAX_EVENT_DESCRIPTION_LENGTH {
            return Err(EventDomainError::DescriptionTooLong {
                max: MAX_EVENT_DESCRIPTION_LENGTH,
            });
        }

        let trimmed_location = location.trim();
        if trimmed_location.is_empty() {
            return Err(EventDomainError::EmptyLocation);
        }
        if trimmed_location.chars().count() > MAX_EVENT_LOCATION_LENGTH {
            return Err(EventDomainError::LocationTooLong {
                max: MAX_EVENT_LOCATION_LENGTH,
            });
        }

        Ok(())
    }

    /// Retorna o período estruturado do evento.
    pub const fn period(&self) -> EventPeriod {
        EventPeriod {
            starts_at: self.starts_at,
            ends_at: self.ends_at,
        }
    }

    /// Indica se o evento está visível na agenda pública de eventos publicados do CA.
    pub const fn is_published(&self) -> bool {
        self.status.is_visible_in_schedule()
    }

    /// Atualiza de forma atômica as informações básicas e o período do evento.
    ///
    /// Caso qualquer validação falhe (como campo obrigatório vazio ou término não posterior ao início),
    /// o evento original permanece inalterado.
    pub fn update_details(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<(), EventDomainError> {
        let new_title = title.into().trim().to_string();
        let new_description = description.into().trim().to_string();
        let new_location = location.into().trim().to_string();

        Self::validate_basic_fields(&new_title, &new_description, &new_location)?;
        EventPeriod::new(starts_at, ends_at)?;
        self.registration.validate(ends_at)?;

        self.title = new_title;
        self.description = new_description;
        self.location = new_location;
        self.starts_at = starts_at;
        self.ends_at = ends_at;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Publica diretamente um evento em rascunho ou aprovado, registrando a data de publicação.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::InvalidStatusTransition`] se o evento estiver cancelado ou rejeitado.
    pub fn publish_directly(&mut self) -> Result<(), EventDomainError> {
        match self.status {
            EventPublicationStatus::Published => Ok(()),
            EventPublicationStatus::Draft
            | EventPublicationStatus::Approved
            | EventPublicationStatus::PendingApproval => {
                let now = Utc::now();
                self.status = EventPublicationStatus::Published;
                self.published_at = Some(now);
                self.updated_at = now;
                Ok(())
            }
            from => Err(EventDomainError::InvalidStatusTransition {
                from,
                to: EventPublicationStatus::Published,
            }),
        }
    }

    /// Atualiza a configuração de inscrição do evento preservando sua consistência com o período.
    pub fn configure_registration(
        &mut self,
        registration: EventRegistrationInfo,
    ) -> Result<(), EventDomainError> {
        registration.validate(self.ends_at)?;
        self.registration = registration;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Registra uma nova inscrição confirmada no evento, validando publicação e disponibilidade de vagas.
    pub fn register_participant(&mut self) -> Result<(), EventDomainError> {
        if !self.status.can_accept_registrations() {
            return Err(EventDomainError::EventNotPublished);
        }
        if !self.registration.requires_registration {
            return Err(EventDomainError::RegistrationsClosed);
        }
        if let Some(limit) = self.registration.max_capacity
            && self.registration.confirmed_registrations >= limit
        {
            return Err(EventDomainError::CapacityExceeded {
                confirmed: self.registration.confirmed_registrations,
                max_capacity: limit,
            });
        }

        self.registration.confirmed_registrations += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Libera uma vaga previamente ocupada quando um participante cancela sua inscrição.
    pub fn cancel_participant_registration(&mut self) {
        if self.registration.confirmed_registrations > 0 {
            self.registration.confirmed_registrations -= 1;
            self.updated_at = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_dates() -> (DateTime<Utc>, DateTime<Utc>) {
        let starts_at = Utc.with_ymd_and_hms(2026, 11, 10, 14, 0, 0).unwrap();
        let ends_at = Utc.with_ymd_and_hms(2026, 11, 10, 18, 0, 0).unwrap();
        (starts_at, ends_at)
    }

    #[test]
    fn should_create_published_event_with_valid_data() {
        let (starts_at, ends_at) = sample_dates();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let event = Event::create_published(
            ca_id,
            author_id,
            "Semana Acadêmica de Engenharia de Redes",
            "Ciclo de palestras e oficinas técnicas abertas ao curso",
            "Auditório do Departamento de Engenharia Elétrica",
            starts_at,
            ends_at,
            EventRegistrationInfo::unrestricted(),
        )
        .expect("deve criar evento publicado válido");

        assert_eq!(event.academic_center_id, ca_id);
        assert_eq!(event.author_id, author_id);
        assert_eq!(event.title, "Semana Acadêmica de Engenharia de Redes");
        assert_eq!(event.status, EventPublicationStatus::Published);
        assert!(event.is_published());
        assert!(event.published_at.is_some());
    }

    #[test]
    fn should_create_draft_event_and_publish_directly() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Oficina de Redes Definidas por Software",
            "Prática em laboratório sobre controladores e comutadores",
            "Laboratório de Redes - Bloco B",
            starts_at,
            ends_at,
            EventRegistrationInfo::with_capacity(30).unwrap(),
        )
        .expect("deve criar rascunho válido");

        assert_eq!(event.status, EventPublicationStatus::Draft);
        assert!(!event.is_published());
        assert!(event.published_at.is_none());

        event
            .publish_directly()
            .expect("deve publicar rascunho diretamente");
        assert_eq!(event.status, EventPublicationStatus::Published);
        assert!(event.is_published());
        assert!(event.published_at.is_some());
    }

    #[test]
    fn should_reject_empty_mandatory_fields_on_creation() {
        let (starts_at, ends_at) = sample_dates();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        assert_eq!(
            Event::create_published(
                ca_id,
                author_id,
                "   ",
                "Descrição válida",
                "Auditório FT",
                starts_at,
                ends_at,
                EventRegistrationInfo::unrestricted(),
            )
            .unwrap_err(),
            EventDomainError::EmptyTitle
        );

        assert_eq!(
            Event::create_published(
                ca_id,
                author_id,
                "Palestra de Cibersegurança",
                "",
                "Auditório FT",
                starts_at,
                ends_at,
                EventRegistrationInfo::unrestricted(),
            )
            .unwrap_err(),
            EventDomainError::EmptyDescription
        );

        assert_eq!(
            Event::create_published(
                ca_id,
                author_id,
                "Palestra de Cibersegurança",
                "Descrição válida",
                "  ",
                starts_at,
                ends_at,
                EventRegistrationInfo::unrestricted(),
            )
            .unwrap_err(),
            EventDomainError::EmptyLocation
        );
    }

    #[test]
    fn should_reject_end_date_before_or_equal_to_start_date() {
        let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 16, 0, 0).unwrap();
        let ends_at = Utc.with_ymd_and_hms(2026, 11, 20, 14, 0, 0).unwrap();

        let err = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Palestra de Cibersegurança",
            "Boas práticas de defesa em redes acadêmicas",
            "Auditório FT",
            starts_at,
            ends_at,
            EventRegistrationInfo::unrestricted(),
        )
        .unwrap_err();

        assert_eq!(
            err,
            EventDomainError::InvalidEventPeriod { starts_at, ends_at }
        );
    }

    #[test]
    fn should_keep_event_unchanged_when_edit_validation_fails() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Palestra de Cibersegurança",
            "Palestra sobre segurança em redes",
            "Auditório FT",
            starts_at,
            ends_at,
            EventRegistrationInfo::unrestricted(),
        )
        .unwrap();

        let snapshot = event.clone();

        let err = event
            .update_details(
                "",
                "Palestra sobre segurança em redes",
                "Auditório FT",
                starts_at,
                ends_at,
            )
            .unwrap_err();
        assert_eq!(err, EventDomainError::EmptyTitle);
        assert_eq!(event, snapshot);

        let invalid_end = starts_at - chrono::Duration::hours(1);
        let err_period = event
            .update_details(
                "Palestra de Cibersegurança",
                "Palestra sobre segurança em redes",
                "Auditório FT",
                starts_at,
                invalid_end,
            )
            .unwrap_err();
        assert_eq!(
            err_period,
            EventDomainError::InvalidEventPeriod {
                starts_at,
                ends_at: invalid_end,
            }
        );
        assert_eq!(event, snapshot);
    }

    #[test]
    fn should_enforce_registration_capacity_limits() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Minicurso de Rust",
            "Introdução prática ao desenvolvimento backend",
            "Laboratório B1",
            starts_at,
            ends_at,
            EventRegistrationInfo::with_capacity(2).unwrap(),
        )
        .unwrap();

        assert_eq!(event.registration.remaining_spots(), Some(2));
        event.register_participant().unwrap();
        event.register_participant().unwrap();
        assert!(!event.registration.has_available_spots());

        assert_eq!(
            event.register_participant().unwrap_err(),
            EventDomainError::CapacityExceeded {
                confirmed: 2,
                max_capacity: 2,
            }
        );

        event.cancel_participant_registration();
        assert_eq!(event.registration.remaining_spots(), Some(1));
    }
}
