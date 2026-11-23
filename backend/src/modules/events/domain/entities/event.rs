//! Entidades, objetos de valor e enums do domínio de Eventos.
//!
//! Modela os eventos promovidos pelos Centros Acadêmicos no CApp conforme a seção 5
//! do Modelo Entidade-Relacionamento (`event`, `event_review` e `event_registration`),
//! contemplando:
//! - informações básicas obrigatórias (`title`, `description`, `location`);
//! - Centro Acadêmico responsável (`ca_id`) e autor (`author_id`);
//! - período do evento (`starts_at` e `ends_at`, exigindo `ends_at > starts_at`);
//! - capacidade opcional (`capacity`), revisão do conteúdo (`revision`) e estado ([`EventPublicationStatus`]);
//! - homologação de versões específicas ([`EventReview`] e [`PublicationReviewDecision`]);
//! - política e controle de inscrições ([`EventRegistrationPolicy`], [`EventRegistration`] e [`RegistrationStatus`]).
//!
//! Todos os campos das entidades e objetos de valor são privados para garantir que
//! nenhuma construção ou mutação contorne as invariantes de domínio.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
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

    /// O identificador do evento não pode ser o UUID nulo.
    #[error("o identificador do evento não pode ser nulo")]
    InvalidEventId,

    /// O identificador do Centro Acadêmico responsável não pode ser o UUID nulo.
    #[error("o identificador do Centro Acadêmico responsável não pode ser nulo")]
    InvalidAcademicCenterId,

    /// O identificador do autor responsável não pode ser o UUID nulo.
    #[error("o identificador do autor responsável não pode ser nulo")]
    InvalidAuthorId,

    /// O identificador do revisor responsável pela homologação não pode ser o UUID nulo.
    #[error("o identificador do revisor não pode ser nulo")]
    InvalidReviewerId,

    /// O identificador da revisão/homologação não pode ser o UUID nulo.
    #[error("o identificador da avaliação de homologação não pode ser nulo")]
    InvalidReviewId,

    /// O identificador da inscrição não pode ser o UUID nulo.
    #[error("o identificador da inscrição não pode ser nulo")]
    InvalidRegistrationId,

    /// O identificador do participante da inscrição não pode ser o UUID nulo.
    #[error("o identificador do participante não pode ser nulo")]
    InvalidParticipantId,

    /// O número de revisão do evento deve ser um inteiro positivo (>= 1).
    #[error("o número de revisão do evento deve ser maior ou igual a 1 (informado: {0})")]
    InvalidRevision(i32),

    /// A revisão avaliada na homologação não corresponde à revisão atual do evento.
    #[error(
        "a revisão avaliada ({reviewed_revision}) não corresponde à revisão atual do evento ({current_revision})"
    )]
    ReviewRevisionMismatch {
        /// Revisão atual do evento.
        current_revision: i32,
        /// Revisão informada na avaliação.
        reviewed_revision: i32,
    },

    /// O motivo informado na decisão de homologação não pode ser vazio quando presente.
    #[error("o motivo da decisão de homologação não pode ser vazio quando informado")]
    EmptyReviewReason,

    /// A decisão textual de homologação informada não é reconhecida pelo domínio.
    #[error("decisão de homologação inválida: '{0}'")]
    InvalidReviewDecision(String),

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

    /// O conteúdo ou a configuração do evento não pode ser editado no estado atual.
    #[error("o evento não pode ser editado no estado '{status}'")]
    EventNotEditable {
        /// Estado atual do evento que impede a edição.
        status: EventPublicationStatus,
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

    /// A janela de inscrição é inválida (`opens_at` deve ser estritamente anterior a `closes_at`).
    #[error(
        "a abertura das inscrições ({opens_at}) deve ser anterior ao encerramento ({closes_at})"
    )]
    InvalidRegistrationWindow {
        /// Instante de abertura das inscrições.
        opens_at: DateTime<Utc>,
        /// Instante de encerramento das inscrições.
        closes_at: DateTime<Utc>,
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

    /// O período de inscrição ainda não começou.
    #[error("as inscrições abrem em {opens_at} (tentativa realizada em {attempted_at})")]
    RegistrationNotYetOpen {
        /// Instante de abertura das inscrições.
        opens_at: DateTime<Utc>,
        /// Instante em que a tentativa de inscrição ocorreu.
        attempted_at: DateTime<Utc>,
    },

    /// O prazo de inscrição ou o horário de encerramento do evento já foi ultrapassado.
    #[error(
        "o prazo para inscrições encerrou em {deadline} (tentativa realizada em {attempted_at})"
    )]
    RegistrationDeadlinePassed {
        /// Instante limite para inscrição (prazo configurado ou término do evento).
        deadline: DateTime<Utc>,
        /// Instante em que a tentativa de inscrição ocorreu.
        attempted_at: DateTime<Utc>,
    },

    /// O evento não aceita inscrições (inscrições desabilitadas ou encerradas).
    #[error("este evento não aceita inscrições")]
    RegistrationsClosed,

    /// A operação exige que o evento esteja publicado na agenda do CA.
    #[error("o evento precisa estar publicado para receber inscrições")]
    EventNotPublished,

    /// A posição na fila de espera deve ser um inteiro positivo (>= 1).
    #[error("a posição na fila de espera deve ser maior ou igual a 1 (informado: {0})")]
    InvalidQueueSequence(i64),

    /// O prazo de expiração do convite da fila de espera deve ser posterior ao registro.
    #[error(
        "o prazo de expiração do convite ({expires_at}) deve ser posterior ao registro ({registered_at})"
    )]
    InvalidInvitationExpiration {
        /// Instante de registro da inscrição.
        registered_at: DateTime<Utc>,
        /// Instante de expiração do convite.
        expires_at: DateTime<Utc>,
    },

    /// O estado de cancelamento da inscrição é inconsistente com a data `cancelled_at`.
    #[error("inconsistência entre o status da inscrição e o instante de cancelamento")]
    InconsistentCancellationTimestamp,

    /// A reinscrição só é permitida quando a inscrição estiver cancelada.
    #[error("só é possível reinscrever uma inscrição que esteja cancelada")]
    RegistrationNotCancelled,
}

/// Estado de publicação de um evento no ciclo de vida do Centro Acadêmico.
///
/// Mapeado ao enum PostgreSQL `event_status` do Modelo Entidade-Relacionamento:
/// `draft`, `pending_review`, `published`, `rejected` e `cancelled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(type_name = "event_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventPublicationStatus {
    /// Evento em preparação, ainda indisponível ao público.
    #[default]
    Draft,
    /// Proposta de evento submetida e aguardando homologação.
    PendingReview,
    /// Evento publicado e visível na agenda oficial de eventos do CA.
    Published,
    /// Versão submetida recusada na análise de homologação.
    Rejected,
    /// Evento cancelado; novas inscrições e convocações ficam impedidas.
    Cancelled,
}

/// Alias semântico alinhado ao nome do tipo `event_status` no Modelo Entidade-Relacionamento.
pub type EventStatus = EventPublicationStatus;

impl EventPublicationStatus {
    /// Retorna a representação textual canônica do estado de publicação.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingReview => "pending_review",
            Self::Published => "published",
            Self::Rejected => "rejected",
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
            "pending_review" | "pending_approval" | "pendente" => Ok(Self::PendingReview),
            "published" | "publicado" => Ok(Self::Published),
            "rejected" | "rejeitado" => Ok(Self::Rejected),
            "cancelled" | "canceled" | "cancelado" => Ok(Self::Cancelled),
            other => Err(EventDomainError::InvalidPublicationStatus(
                other.to_string(),
            )),
        }
    }
}

/// Decisão registrada na avaliação de homologação de um evento.
///
/// Mapeada ao enum PostgreSQL `publication_review_decision`: `approved` ou `rejected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "publication_review_decision", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PublicationReviewDecision {
    /// Revisor autorizou a publicação da revisão analisada.
    Approved,
    /// Revisor recusou a publicação da revisão analisada.
    Rejected,
}

impl PublicationReviewDecision {
    /// Retorna a representação textual canônica da decisão.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

impl fmt::Display for PublicationReviewDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PublicationReviewDecision {
    type Err = EventDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "approved" | "aprovado" => Ok(Self::Approved),
            "rejected" | "rejeitado" => Ok(Self::Rejected),
            other => Err(EventDomainError::InvalidReviewDecision(other.to_string())),
        }
    }
}

/// Entidade que representa um registro de homologação (`event_review`) de uma versão do evento.
///
/// Campos privados impedem adulteração de `reviewed_revision` ou `decision` fora dos
/// construtores validados.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventReview {
    /// Identificador interno e estável deste registro de homologação.
    id: Uuid,
    /// Identificador do evento ao qual a análise pertence.
    event_id: Uuid,
    /// Identificador do usuário revisor que realizou a homologação.
    reviewer_id: Uuid,
    /// Número exato da revisão do evento analisada pelo revisor.
    reviewed_revision: i32,
    /// Decisão tomada na homologação (`approved` ou `rejected`).
    decision: PublicationReviewDecision,
    /// Motivo opcional informado para a decisão.
    reason: Option<String>,
    /// Instante em que a decisão foi registrada.
    decided_at: DateTime<Utc>,
}

impl EventReview {
    /// Constrói um novo registro de homologação validando todas as invariantes.
    ///
    /// # Erros
    ///
    /// - [`EventDomainError::InvalidReviewId`] se `id` for nulo.
    /// - [`EventDomainError::InvalidEventId`] se `event_id` for nulo.
    /// - [`EventDomainError::InvalidReviewerId`] se `reviewer_id` for nulo.
    /// - [`EventDomainError::InvalidRevision`] se `reviewed_revision < 1`.
    /// - [`EventDomainError::EmptyReviewReason`] se `reason` for `Some` com texto vazio.
    pub fn new(
        id: Uuid,
        event_id: Uuid,
        reviewer_id: Uuid,
        reviewed_revision: i32,
        decision: PublicationReviewDecision,
        reason: Option<String>,
        decided_at: DateTime<Utc>,
    ) -> Result<Self, EventDomainError> {
        let normalized_reason = match reason {
            Some(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return Err(EventDomainError::EmptyReviewReason);
                }
                Some(trimmed.to_string())
            }
            None => None,
        };

        let review = Self {
            id,
            event_id,
            reviewer_id,
            reviewed_revision,
            decision,
            reason: normalized_reason,
            decided_at,
        };
        review.validate()?;
        Ok(review)
    }

    /// Cria uma decisão de aprovação para uma revisão específica de um evento.
    pub fn approve(
        event_id: Uuid,
        reviewer_id: Uuid,
        reviewed_revision: i32,
        reason: Option<String>,
        decided_at: DateTime<Utc>,
    ) -> Result<Self, EventDomainError> {
        Self::new(
            Uuid::new_v4(),
            event_id,
            reviewer_id,
            reviewed_revision,
            PublicationReviewDecision::Approved,
            reason,
            decided_at,
        )
    }

    /// Cria uma decisão de rejeição para uma revisão específica de um evento.
    pub fn reject(
        event_id: Uuid,
        reviewer_id: Uuid,
        reviewed_revision: i32,
        reason: Option<String>,
        decided_at: DateTime<Utc>,
    ) -> Result<Self, EventDomainError> {
        Self::new(
            Uuid::new_v4(),
            event_id,
            reviewer_id,
            reviewed_revision,
            PublicationReviewDecision::Rejected,
            reason,
            decided_at,
        )
    }

    /// Valida as invariantes estruturais do registro de homologação.
    pub fn validate(&self) -> Result<(), EventDomainError> {
        if self.id.is_nil() {
            return Err(EventDomainError::InvalidReviewId);
        }
        if self.event_id.is_nil() {
            return Err(EventDomainError::InvalidEventId);
        }
        if self.reviewer_id.is_nil() {
            return Err(EventDomainError::InvalidReviewerId);
        }
        if self.reviewed_revision < 1 {
            return Err(EventDomainError::InvalidRevision(self.reviewed_revision));
        }
        if let Some(ref reason) = self.reason
            && reason.trim().is_empty()
        {
            return Err(EventDomainError::EmptyReviewReason);
        }
        Ok(())
    }

    /// Retorna o identificador único do registro de homologação.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador do evento avaliado.
    pub const fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Retorna o identificador do usuário revisor.
    pub const fn reviewer_id(&self) -> Uuid {
        self.reviewer_id
    }

    /// Retorna o número da revisão do evento avaliada.
    pub const fn reviewed_revision(&self) -> i32 {
        self.reviewed_revision
    }

    /// Retorna a decisão registrada (`Approved` ou `Rejected`).
    pub const fn decision(&self) -> PublicationReviewDecision {
        self.decision
    }

    /// Retorna o motivo opcional informado pelo revisor.
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// Retorna o instante em que a decisão foi registrada.
    pub const fn decided_at(&self) -> DateTime<Utc> {
        self.decided_at
    }
}

/// Período de realização de um evento (início e término).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPeriod {
    /// Data e horário de início do evento.
    starts_at: DateTime<Utc>,
    /// Data e horário de término do evento (deve ser posterior a `starts_at`).
    ends_at: DateTime<Utc>,
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

    /// Retorna o instante de início do evento.
    pub const fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    /// Retorna o instante de término do evento.
    pub const fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }
}

/// Política mínima de inscrição persistida em `registration_policy` (JSONB).
///
/// Capacidade e ocupação **não** pertencem a esta política: vagas ficam em
/// [`Event::capacity`] e a ocupação é a contagem de `event_registration` com
/// status `registered`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRegistrationPolicy {
    /// Indica se o evento aceita inscrições de participantes na plataforma.
    #[serde(default = "default_registration_enabled")]
    enabled: bool,
    /// Instante opcional de abertura das inscrições.
    #[serde(default)]
    opens_at: Option<DateTime<Utc>>,
    /// Instante opcional de encerramento das inscrições.
    #[serde(default)]
    closes_at: Option<DateTime<Utc>>,
}

const fn default_registration_enabled() -> bool {
    true
}

/// Alias alinhado ao contrato MVP do plano de domínio (`enabled` / `opens_at` / `closes_at`).
pub type RegistrationPolicy = EventRegistrationPolicy;

/// Alias legado mantido para compatibilidade com reexportações existentes.
pub type EventRegistrationInfo = EventRegistrationPolicy;

impl Default for EventRegistrationPolicy {
    fn default() -> Self {
        Self::unrestricted()
    }
}

impl EventRegistrationPolicy {
    /// Cria uma política com inscrições habilitadas e sem janela explícita.
    pub const fn unrestricted() -> Self {
        Self {
            enabled: true,
            opens_at: None,
            closes_at: None,
        }
    }

    /// Cria uma política para evento puramente informativo (inscrições desabilitadas).
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            opens_at: None,
            closes_at: None,
        }
    }

    /// Cria uma política personalizada validando a janela `opens_at`/`closes_at`.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::InvalidRegistrationWindow`] se ambos os limites
    /// existirem e `opens_at >= closes_at`.
    pub fn new(
        enabled: bool,
        opens_at: Option<DateTime<Utc>>,
        closes_at: Option<DateTime<Utc>>,
    ) -> Result<Self, EventDomainError> {
        let policy = Self {
            enabled,
            opens_at,
            closes_at,
        };
        policy.validate_window()?;
        Ok(policy)
    }

    /// Define o instante de abertura das inscrições.
    pub const fn with_opens_at(mut self, opens_at: DateTime<Utc>) -> Self {
        self.opens_at = Some(opens_at);
        self
    }

    /// Define o instante de encerramento das inscrições.
    pub const fn with_closes_at(mut self, closes_at: DateTime<Utc>) -> Self {
        self.closes_at = Some(closes_at);
        self
    }

    /// Valida apenas a consistência interna da janela `opens_at`/`closes_at`.
    fn validate_window(&self) -> Result<(), EventDomainError> {
        if let (Some(opens_at), Some(closes_at)) = (self.opens_at, self.closes_at)
            && opens_at >= closes_at
        {
            return Err(EventDomainError::InvalidRegistrationWindow {
                opens_at,
                closes_at,
            });
        }
        Ok(())
    }

    /// Valida a política em relação ao horário de término do evento.
    pub fn validate(&self, event_ends_at: DateTime<Utc>) -> Result<(), EventDomainError> {
        self.validate_window()?;

        if let Some(closes_at) = self.closes_at
            && closes_at > event_ends_at
        {
            return Err(EventDomainError::RegistrationDeadlineAfterEventEnd {
                deadline: closes_at,
                ends_at: event_ends_at,
            });
        }

        if let Some(opens_at) = self.opens_at
            && opens_at > event_ends_at
        {
            return Err(EventDomainError::RegistrationDeadlineAfterEventEnd {
                deadline: opens_at,
                ends_at: event_ends_at,
            });
        }

        Ok(())
    }

    /// Indica se as inscrições estão habilitadas na política.
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Alias semântico de [`Self::enabled`].
    pub const fn requires_registration(&self) -> bool {
        self.enabled
    }

    /// Retorna o instante de abertura das inscrições, quando configurado.
    pub const fn opens_at(&self) -> Option<DateTime<Utc>> {
        self.opens_at
    }

    /// Retorna o instante de encerramento das inscrições, quando configurado.
    pub const fn closes_at(&self) -> Option<DateTime<Utc>> {
        self.closes_at
    }

    /// Indica se, no instante `now`, a janela de inscrição está aberta.
    pub fn is_open_at(&self, now: DateTime<Utc>) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(opens_at) = self.opens_at
            && now < opens_at
        {
            return false;
        }
        if let Some(closes_at) = self.closes_at
            && now > closes_at
        {
            return false;
        }
        true
    }
}

/// Estado da inscrição individual de um participante em um evento.
///
/// Mapeado ao enum PostgreSQL `event_registration_status` do Modelo Entidade-Relacionamento:
/// `registered`, `waitlisted`, `invited` e `cancelled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(type_name = "event_registration_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    /// Participação confirmada, ocupando uma vaga quando houver capacidade limitada.
    #[default]
    Registered,
    /// Usuário na fila de espera, ainda sem inscrição confirmada.
    Waitlisted,
    /// Usuário convocado a confirmar uma vaga quando a política de espera usar convites.
    Invited,
    /// Inscrição ou participação na fila cancelada.
    Cancelled,
}

/// Alias semântico alinhado ao tipo `event_registration_status` do Modelo Entidade-Relacionamento.
pub type EventRegistrationStatus = RegistrationStatus;

impl RegistrationStatus {
    /// Variante associada para compatibilidade semântica com inscrição confirmada.
    pub const CONFIRMED: Self = Self::Registered;

    /// Retorna a representação textual canônica do status da inscrição.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::Waitlisted => "waitlisted",
            Self::Invited => "invited",
            Self::Cancelled => "cancelled",
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
            "registered" | "confirmed" | "confirmada" => Ok(Self::Registered),
            "waitlisted" | "fila_espera" => Ok(Self::Waitlisted),
            "invited" | "convidado" | "convocado" => Ok(Self::Invited),
            "cancelled" | "canceled" | "cancelada" => Ok(Self::Cancelled),
            other => Err(EventDomainError::InvalidRegistrationStatus(
                other.to_string(),
            )),
        }
    }
}

/// Entidade que representa a inscrição de um usuário em um evento (`event_registration`).
///
/// Todos os campos são privados para impedir mutações de estado ou de timestamps
/// que contornem as invariantes de validação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRegistration {
    /// Identificador interno e estável da inscrição.
    id: Uuid,
    /// Identificador do evento ao qual a inscrição pertence.
    event_id: Uuid,
    /// Identificador do usuário inscrito no evento (`user_id` no ERD).
    user_id: Uuid,
    /// Situação atual da inscrição (`registered`, `waitlisted`, `invited` ou `cancelled`).
    status: RegistrationStatus,
    /// Posição ou chave de ordenação atribuída na fila de espera, quando aplicável.
    queue_sequence: Option<i64>,
    /// Data e horário em que a inscrição foi registrada.
    registered_at: DateTime<Utc>,
    /// Limite para aceitar uma convocação da fila quando a política usar convites.
    invitation_expires_at: Option<DateTime<Utc>>,
    /// Data e horário de cancelamento, quando a inscrição estiver cancelada.
    cancelled_at: Option<DateTime<Utc>>,
}

impl EventRegistration {
    /// Constrói uma instância de [`EventRegistration`] validando todas as invariantes.
    ///
    /// # Erros
    ///
    /// - [`EventDomainError::InvalidRegistrationId`] se `id` for nulo.
    /// - [`EventDomainError::InvalidEventId`] se `event_id` for nulo.
    /// - [`EventDomainError::InvalidParticipantId`] se `user_id` for nulo.
    /// - [`EventDomainError::InvalidQueueSequence`] se `queue_sequence < 1`.
    /// - [`EventDomainError::InvalidInvitationExpiration`] se `invitation_expires_at <= registered_at`.
    /// - [`EventDomainError::InconsistentCancellationTimestamp`] se `status` e `cancelled_at` divergirem.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        event_id: Uuid,
        user_id: Uuid,
        status: RegistrationStatus,
        queue_sequence: Option<i64>,
        registered_at: DateTime<Utc>,
        invitation_expires_at: Option<DateTime<Utc>>,
        cancelled_at: Option<DateTime<Utc>>,
    ) -> Result<Self, EventDomainError> {
        let registration = Self {
            id,
            event_id,
            user_id,
            status,
            queue_sequence,
            registered_at,
            invitation_expires_at,
            cancelled_at,
        };
        registration.validate()?;
        Ok(registration)
    }

    /// Cria uma nova inscrição confirmada (`registered`) para um participante em um evento.
    ///
    /// # Erros
    ///
    /// - [`EventDomainError::InvalidEventId`] caso `event_id` seja nulo.
    /// - [`EventDomainError::InvalidParticipantId`] caso `user_id` seja nulo.
    pub fn new_confirmed(event_id: Uuid, user_id: Uuid) -> Result<Self, EventDomainError> {
        Self::new(
            Uuid::new_v4(),
            event_id,
            user_id,
            RegistrationStatus::Registered,
            None,
            Utc::now(),
            None,
            None,
        )
    }

    /// Cria uma nova inscrição na fila de espera (`waitlisted`) com posição sequencial validada.
    pub fn new_waitlisted(
        event_id: Uuid,
        user_id: Uuid,
        queue_sequence: i64,
        registered_at: DateTime<Utc>,
    ) -> Result<Self, EventDomainError> {
        Self::new(
            Uuid::new_v4(),
            event_id,
            user_id,
            RegistrationStatus::Waitlisted,
            Some(queue_sequence),
            registered_at,
            None,
            None,
        )
    }

    /// Valida todas as invariantes estruturais da inscrição.
    pub fn validate(&self) -> Result<(), EventDomainError> {
        if self.id.is_nil() {
            return Err(EventDomainError::InvalidRegistrationId);
        }
        if self.event_id.is_nil() {
            return Err(EventDomainError::InvalidEventId);
        }
        if self.user_id.is_nil() {
            return Err(EventDomainError::InvalidParticipantId);
        }
        if let Some(seq) = self.queue_sequence
            && seq < 1
        {
            return Err(EventDomainError::InvalidQueueSequence(seq));
        }
        if let Some(expires_at) = self.invitation_expires_at
            && expires_at <= self.registered_at
        {
            return Err(EventDomainError::InvalidInvitationExpiration {
                registered_at: self.registered_at,
                expires_at,
            });
        }
        match (self.status, self.cancelled_at) {
            (RegistrationStatus::Cancelled, Some(cancelled_at))
                if cancelled_at >= self.registered_at =>
            {
                Ok(())
            }
            (RegistrationStatus::Cancelled, _) | (_, Some(_)) => {
                Err(EventDomainError::InconsistentCancellationTimestamp)
            }
            (_, None) => Ok(()),
        }
    }

    /// Cancela a inscrição registrando o instante do cancelamento.
    pub fn cancel(&mut self) {
        if self.status != RegistrationStatus::Cancelled {
            let now = Utc::now();
            let cancelled_at = if now < self.registered_at {
                self.registered_at
            } else {
                now
            };
            self.status = RegistrationStatus::Cancelled;
            self.cancelled_at = Some(cancelled_at);
        }
    }

    /// Reabre uma inscrição cancelada como confirmada (`registered`).
    ///
    /// Atualiza sempre `registered_at` com o instante informado, limpa
    /// `cancelled_at` e `invitation_expires_at`, e remove `queue_sequence`.
    ///
    /// # Erros
    ///
    /// - [`EventDomainError::RegistrationNotCancelled`] se a inscrição não estiver cancelada.
    /// - Demais erros de [`Self::validate`] após a transição.
    pub fn reactivate_registered(
        &mut self,
        registered_at: DateTime<Utc>,
    ) -> Result<(), EventDomainError> {
        if self.status != RegistrationStatus::Cancelled {
            return Err(EventDomainError::RegistrationNotCancelled);
        }

        self.status = RegistrationStatus::Registered;
        self.registered_at = registered_at;
        self.cancelled_at = None;
        self.invitation_expires_at = None;
        self.queue_sequence = None;
        self.validate()
    }

    /// Reabre uma inscrição cancelada na fila de espera (`waitlisted`).
    ///
    /// Atualiza sempre `registered_at` com o instante informado, limpa
    /// `cancelled_at` e `invitation_expires_at`, e define a nova posição na fila.
    ///
    /// # Erros
    ///
    /// - [`EventDomainError::RegistrationNotCancelled`] se a inscrição não estiver cancelada.
    /// - Demais erros de [`Self::validate`] após a transição (ex.: sequência inválida).
    pub fn reactivate_waitlisted(
        &mut self,
        registered_at: DateTime<Utc>,
        queue_sequence: i64,
    ) -> Result<(), EventDomainError> {
        if self.status != RegistrationStatus::Cancelled {
            return Err(EventDomainError::RegistrationNotCancelled);
        }

        self.status = RegistrationStatus::Waitlisted;
        self.registered_at = registered_at;
        self.queue_sequence = Some(queue_sequence);
        self.cancelled_at = None;
        self.invitation_expires_at = None;
        self.validate()
    }

    /// Retorna o identificador único da inscrição.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador do evento ao qual a inscrição pertence.
    pub const fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Retorna o identificador do usuário inscrito (`user_id` no ERD).
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o identificador do usuário participante (alias para [`Self::user_id`]).
    pub const fn participant_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o estado atual da inscrição.
    pub const fn status(&self) -> RegistrationStatus {
        self.status
    }

    /// Retorna a posição na fila de espera, quando aplicável.
    pub const fn queue_sequence(&self) -> Option<i64> {
        self.queue_sequence
    }

    /// Retorna a data/hora em que a inscrição foi registrada.
    pub const fn registered_at(&self) -> DateTime<Utc> {
        self.registered_at
    }

    /// Retorna a data/hora limite do convite da fila de espera, quando aplicável.
    pub const fn invitation_expires_at(&self) -> Option<DateTime<Utc>> {
        self.invitation_expires_at
    }

    /// Retorna a data/hora de cancelamento da inscrição, quando aplicável.
    pub const fn cancelled_at(&self) -> Option<DateTime<Utc>> {
        self.cancelled_at
    }
}

/// Entidade principal de domínio que representa um Evento promovido por um Centro Acadêmico (`event`).
///
/// A estrutura reflete fielmente a tabela `event` do Modelo Entidade-Relacionamento:
/// `id`, `ca_id`, `author_id`, `title`, `description`, `location`, `starts_at`, `ends_at`,
/// `capacity`, `status`, `revision` e `registration_policy`.
///
/// Todos os campos são privados para garantir que alterações passem pelas validações de domínio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    /// Identificador interno e estável do evento.
    id: Uuid,
    /// Identificador do Centro Acadêmico responsável pelo evento (`ca_id` no ERD).
    ca_id: Uuid,
    /// Identificador do autor responsável pelo evento.
    author_id: Uuid,
    /// Título oficial do evento.
    title: String,
    /// Descrição detalhada sobre o objetivo e programação do evento.
    description: String,
    /// Local ou informação de acesso ao evento.
    location: String,
    /// Data e horário previstos para o início do evento.
    starts_at: DateTime<Utc>,
    /// Data e horário previstos para o término do evento.
    ends_at: DateTime<Utc>,
    /// Quantidade máxima de vagas; `None` representa evento sem limite definido.
    capacity: Option<i32>,
    /// Estado atual do evento no ciclo de publicação (`event_status`).
    status: EventPublicationStatus,
    /// Número da revisão deste registro, incrementado a cada edição de conteúdo.
    revision: i32,
    /// Configuração validada das inscrições (`registration_policy` em JSONB).
    registration_policy: Json<EventRegistrationPolicy>,
}

impl Event {
    /// Constrói uma instância de [`Event`] validando todas as invariantes de negócio.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError`] caso identificadores sejam nulos, campos obrigatórios
    /// estejam vazios ou excedam os limites, o término não seja posterior ao início, a
    /// revisão/capacidade seja inválida ou a política de inscrição seja inconsistente.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        ca_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        capacity: Option<i32>,
        status: EventPublicationStatus,
        revision: i32,
        registration: EventRegistrationPolicy,
    ) -> Result<Self, EventDomainError> {
        let event = Self {
            id,
            ca_id,
            author_id,
            title: title.into().trim().to_string(),
            description: description.into().trim().to_string(),
            location: location.into().trim().to_string(),
            starts_at,
            ends_at,
            capacity,
            status,
            revision,
            registration_policy: Json(registration),
        };

        event.validate()?;
        Ok(event)
    }

    /// Cria um novo evento em estado de rascunho ([`EventPublicationStatus::Draft`]) na revisão `1`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_draft(
        ca_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        capacity: Option<i32>,
        registration: EventRegistrationPolicy,
    ) -> Result<Self, EventDomainError> {
        Self::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            title,
            description,
            location,
            starts_at,
            ends_at,
            capacity,
            EventPublicationStatus::Draft,
            1,
            registration,
        )
    }

    /// Cria um novo evento publicado diretamente ([`EventPublicationStatus::Published`]) na revisão `1`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_published(
        ca_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        capacity: Option<i32>,
        registration: EventRegistrationPolicy,
    ) -> Result<Self, EventDomainError> {
        Self::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            title,
            description,
            location,
            starts_at,
            ends_at,
            capacity,
            EventPublicationStatus::Published,
            1,
            registration,
        )
    }

    /// Valida todas as regras de negócio e invariantes estruturais do evento,
    /// espelhando exatamente as restrições da tabela `event` no banco de dados.
    pub fn validate(&self) -> Result<(), EventDomainError> {
        if self.id.is_nil() {
            return Err(EventDomainError::InvalidEventId);
        }
        if self.ca_id.is_nil() {
            return Err(EventDomainError::InvalidAcademicCenterId);
        }
        if self.author_id.is_nil() {
            return Err(EventDomainError::InvalidAuthorId);
        }
        if self.revision < 1 {
            return Err(EventDomainError::InvalidRevision(self.revision));
        }

        if let Some(cap) = self.capacity
            && cap <= 0
        {
            return Err(EventDomainError::InvalidMaxCapacity(
                u32::try_from(cap).unwrap_or(0),
            ));
        }

        Self::validate_basic_fields(&self.title, &self.description, &self.location)?;
        EventPeriod::new(self.starts_at, self.ends_at)?;
        self.registration_policy.validate(self.ends_at)?;

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

    /// Retorna o identificador interno e estável do evento.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador do Centro Acadêmico responsável pelo evento (`ca_id`).
    pub const fn ca_id(&self) -> Uuid {
        self.ca_id
    }

    /// Retorna o identificador do Centro Acadêmico responsável (alias para [`Self::ca_id`]).
    pub const fn academic_center_id(&self) -> Uuid {
        self.ca_id
    }

    /// Retorna o identificador do autor responsável pelo evento.
    pub const fn author_id(&self) -> Uuid {
        self.author_id
    }

    /// Retorna o título oficial do evento.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Retorna a descrição do evento.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Retorna o local do evento.
    pub fn location(&self) -> &str {
        &self.location
    }

    /// Retorna o instante de início do evento.
    pub const fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    /// Retorna o instante de término do evento.
    pub const fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }

    /// Retorna a capacidade máxima de vagas configurada na coluna `capacity`, se houver.
    pub const fn capacity(&self) -> Option<i32> {
        self.capacity
    }

    /// Retorna o estado atual de publicação do evento.
    pub const fn status(&self) -> EventPublicationStatus {
        self.status
    }

    /// Retorna o número da revisão atual do conteúdo do evento.
    pub const fn revision(&self) -> i32 {
        self.revision
    }

    /// Retorna a configuração de inscrição do evento (`registration_policy`).
    pub fn registration(&self) -> &EventRegistrationPolicy {
        &self.registration_policy.0
    }

    /// Retorna a política de inscrição do evento (alias para [`Self::registration`]).
    pub fn registration_policy(&self) -> &EventRegistrationPolicy {
        &self.registration_policy.0
    }

    /// Indica se ainda há vagas disponíveis considerando a capacidade do evento e a
    /// contagem externa de inscrições confirmadas (`registered`).
    pub fn has_available_spots(&self, registered_count: u32) -> bool {
        match self.capacity {
            Some(limit) => u32::try_from(limit)
                .map(|limit| registered_count < limit)
                .unwrap_or(false),
            None => true,
        }
    }

    /// Retorna o número de vagas restantes quando o evento possui capacidade máxima.
    pub fn remaining_spots(&self, registered_count: u32) -> Option<u32> {
        self.capacity.map(|limit| {
            u32::try_from(limit)
                .map(|limit| limit.saturating_sub(registered_count))
                .unwrap_or(0)
        })
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

    /// Indica se o conteúdo e a configuração do evento podem ser editados no status atual.
    ///
    /// Permitido em `Draft`, `PendingReview`, `Rejected` e `Published`.
    /// Bloqueado em `Cancelled`.
    pub const fn can_edit_content(&self) -> bool {
        matches!(
            self.status,
            EventPublicationStatus::Draft
                | EventPublicationStatus::PendingReview
                | EventPublicationStatus::Rejected
                | EventPublicationStatus::Published
        )
    }

    /// Garante que o evento está em um status editável.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::EventNotEditable`] se o status for `Cancelled`.
    fn ensure_content_editable(&self) -> Result<(), EventDomainError> {
        if self.can_edit_content() {
            Ok(())
        } else {
            Err(EventDomainError::EventNotEditable {
                status: self.status,
            })
        }
    }

    /// Atualiza de forma atômica as informações básicas e o período do evento,
    /// incrementando o contador de revisão (`revision`).
    ///
    /// Se o evento estiver [`EventPublicationStatus::Published`], a edição o retira
    /// da agenda pública transitando para [`EventPublicationStatus::PendingReview`].
    /// Demais status editáveis preservam o status atual.
    ///
    /// Caso qualquer validação falhe (como campo obrigatório vazio ou término não posterior ao início),
    /// o evento original permanece inalterado.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::EventNotEditable`] se o evento estiver cancelado.
    pub fn update_details(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<(), EventDomainError> {
        self.ensure_content_editable()?;

        let new_title = title.into().trim().to_string();
        let new_description = description.into().trim().to_string();
        let new_location = location.into().trim().to_string();

        Self::validate_basic_fields(&new_title, &new_description, &new_location)?;
        EventPeriod::new(starts_at, ends_at)?;
        self.registration_policy.validate(ends_at)?;

        self.title = new_title;
        self.description = new_description;
        self.location = new_location;
        self.starts_at = starts_at;
        self.ends_at = ends_at;
        self.revision += 1;
        self.unpublish_if_published();
        Ok(())
    }

    /// Submete um evento em rascunho ou rejeitado para análise de homologação (`pending_review`).
    pub fn submit_for_review(&mut self) -> Result<(), EventDomainError> {
        match self.status {
            EventPublicationStatus::PendingReview => Ok(()),
            EventPublicationStatus::Draft | EventPublicationStatus::Rejected => {
                self.status = EventPublicationStatus::PendingReview;
                Ok(())
            }
            from => Err(EventDomainError::InvalidStatusTransition {
                from,
                to: EventPublicationStatus::PendingReview,
            }),
        }
    }

    /// Aplica uma decisão de homologação ([`EventReview`]) ao evento, verificando se a revisão
    /// analisada corresponde à revisão atual do evento.
    pub fn apply_review(&mut self, review: &EventReview) -> Result<(), EventDomainError> {
        review.validate()?;
        if review.event_id != self.id {
            return Err(EventDomainError::InvalidEventId);
        }
        if review.reviewed_revision != self.revision {
            return Err(EventDomainError::ReviewRevisionMismatch {
                current_revision: self.revision,
                reviewed_revision: review.reviewed_revision,
            });
        }

        let target_status = match review.decision {
            PublicationReviewDecision::Approved => EventPublicationStatus::Published,
            PublicationReviewDecision::Rejected => EventPublicationStatus::Rejected,
        };

        if self.status != EventPublicationStatus::PendingReview {
            return Err(EventDomainError::InvalidStatusTransition {
                from: self.status,
                to: target_status,
            });
        }

        self.status = target_status;
        Ok(())
    }

    /// Publica diretamente um evento em rascunho ou pendente de revisão por gestor autorizado.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::InvalidStatusTransition`] se o evento estiver cancelado ou rejeitado.
    pub fn publish_directly(&mut self) -> Result<(), EventDomainError> {
        match self.status {
            EventPublicationStatus::Published => Ok(()),
            EventPublicationStatus::Draft | EventPublicationStatus::PendingReview => {
                self.status = EventPublicationStatus::Published;
                Ok(())
            }
            from => Err(EventDomainError::InvalidStatusTransition {
                from,
                to: EventPublicationStatus::Published,
            }),
        }
    }

    /// Cancela o evento, impedindo novas inscrições e convocações.
    ///
    /// Idempotente: se o evento já estiver cancelado, retorna `Ok(())` sem alterar o estado.
    pub fn cancel(&mut self) -> Result<(), EventDomainError> {
        if self.status == EventPublicationStatus::Cancelled {
            return Ok(());
        }
        self.status = EventPublicationStatus::Cancelled;
        Ok(())
    }

    /// Atualiza a capacidade e a política de inscrição do evento preservando a
    /// consistência com o período.
    ///
    /// Se o evento estiver [`EventPublicationStatus::Published`], a reconfiguração
    /// o retira da agenda pública transitando para [`EventPublicationStatus::PendingReview`].
    /// Demais status editáveis preservam o status atual.
    ///
    /// # Erros
    ///
    /// Retorna [`EventDomainError::EventNotEditable`] se o evento estiver cancelado.
    pub fn configure_registration(
        &mut self,
        capacity: Option<i32>,
        registration: EventRegistrationPolicy,
    ) -> Result<(), EventDomainError> {
        self.ensure_content_editable()?;

        if let Some(cap) = capacity
            && cap <= 0
        {
            return Err(EventDomainError::InvalidMaxCapacity(
                u32::try_from(cap).unwrap_or(0),
            ));
        }
        registration.validate(self.ends_at)?;
        self.capacity = capacity;
        self.registration_policy = Json(registration);
        self.revision += 1;
        self.unpublish_if_published();
        Ok(())
    }

    /// Se o status atual for `Published`, transiciona para `PendingReview`
    /// (edição/reconfiguração retira o evento da agenda até nova homologação).
    fn unpublish_if_published(&mut self) {
        if self.status == EventPublicationStatus::Published {
            self.status = EventPublicationStatus::PendingReview;
        }
    }

    /// Valida se uma nova inscrição confirmada pode ser aceita no instante `now`.
    ///
    /// A ocupação atual deve ser informada por `registered_count` (fonte de verdade:
    /// contagem de `event_registration` com status `registered`). Este método **não**
    /// muta a política JSONB.
    ///
    /// Valida:
    /// - se o evento está publicado ([`EventPublicationStatus::Published`]);
    /// - se a política está habilitada (`enabled`);
    /// - se a janela `opens_at`/`closes_at` e o término do evento permitem a inscrição;
    /// - se ainda há disponibilidade dentro de [`Event::capacity`].
    pub fn register_participant(
        &self,
        now: DateTime<Utc>,
        registered_count: u32,
    ) -> Result<(), EventDomainError> {
        if !self.status.can_accept_registrations() {
            return Err(EventDomainError::EventNotPublished);
        }
        if !self.registration_policy.enabled {
            return Err(EventDomainError::RegistrationsClosed);
        }
        if let Some(opens_at) = self.registration_policy.opens_at
            && now < opens_at
        {
            return Err(EventDomainError::RegistrationNotYetOpen {
                opens_at,
                attempted_at: now,
            });
        }
        if let Some(closes_at) = self.registration_policy.closes_at
            && now > closes_at
        {
            return Err(EventDomainError::RegistrationDeadlinePassed {
                deadline: closes_at,
                attempted_at: now,
            });
        }
        if now >= self.ends_at {
            return Err(EventDomainError::RegistrationDeadlinePassed {
                deadline: self.ends_at,
                attempted_at: now,
            });
        }
        if let Some(limit) = self.capacity {
            let limit =
                u32::try_from(limit).map_err(|_| EventDomainError::InvalidMaxCapacity(0))?;
            if registered_count >= limit {
                return Err(EventDomainError::CapacityExceeded {
                    confirmed: registered_count,
                    max_capacity: limit,
                });
            }
        }

        Ok(())
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
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento publicado válido");

        assert_eq!(event.ca_id(), ca_id);
        assert_eq!(event.academic_center_id(), ca_id);
        assert_eq!(event.author_id(), author_id);
        assert_eq!(event.title(), "Semana Acadêmica de Engenharia de Redes");
        assert_eq!(event.status(), EventPublicationStatus::Published);
        assert_eq!(event.revision(), 1);
        assert_eq!(event.capacity(), None);
        assert!(event.registration_policy().enabled());
        assert!(event.is_published());
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
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento em rascunho");

        assert_eq!(event.status(), EventPublicationStatus::Draft);
        assert!(!event.is_published());

        event
            .publish_directly()
            .expect("deve publicar diretamente o rascunho");

        assert_eq!(event.status(), EventPublicationStatus::Published);
        assert!(event.is_published());
    }

    #[test]
    fn should_enforce_review_revision_matching_in_homologation() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Simpósio de Segurança em Redes",
            "Apresentação de pesquisas e práticas de defesa cibernética",
            "Auditório BCE",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento em rascunho");

        event
            .submit_for_review()
            .expect("deve submeter para homologação");
        assert_eq!(event.status(), EventPublicationStatus::PendingReview);

        let stale_review = EventReview::approve(
            event.id(),
            Uuid::new_v4(),
            event.revision() + 1,
            Some("Revisão diferente".to_string()),
            Utc::now(),
        )
        .expect("deve criar review");

        assert!(matches!(
            event.apply_review(&stale_review),
            Err(EventDomainError::ReviewRevisionMismatch { .. })
        ));

        let valid_review = EventReview::approve(
            event.id(),
            Uuid::new_v4(),
            event.revision(),
            Some("Proposta alinhada ao calendário do CA".to_string()),
            Utc::now(),
        )
        .expect("deve criar review válido");

        event
            .apply_review(&valid_review)
            .expect("deve aprovar a revisão correspondente");
        assert_eq!(event.status(), EventPublicationStatus::Published);
    }

    #[test]
    fn should_reject_empty_mandatory_fields_on_creation() {
        let (starts_at, ends_at) = sample_dates();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let empty_title = Event::create_published(
            ca_id,
            author_id,
            "   ",
            "Descrição válida",
            "Local válido",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        );
        assert_eq!(empty_title, Err(EventDomainError::EmptyTitle));

        let empty_description = Event::create_published(
            ca_id,
            author_id,
            "Título válido",
            "   ",
            "Local válido",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        );
        assert_eq!(empty_description, Err(EventDomainError::EmptyDescription));

        let empty_location = Event::create_published(
            ca_id,
            author_id,
            "Título válido",
            "Descrição válida",
            "",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        );
        assert_eq!(empty_location, Err(EventDomainError::EmptyLocation));
    }

    #[test]
    fn should_reject_end_date_before_or_equal_to_start_date() {
        let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 16, 0, 0).unwrap();
        let ends_before = Utc.with_ymd_and_hms(2026, 11, 20, 14, 0, 0).unwrap();

        let result_before = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Minicurso de Segurança em Redes",
            "Fundamentos de análise de tráfego e firewall",
            "Sala AT-11",
            starts_at,
            ends_before,
            None,
            EventRegistrationPolicy::unrestricted(),
        );
        assert_eq!(
            result_before,
            Err(EventDomainError::InvalidEventPeriod {
                starts_at,
                ends_at: ends_before,
            })
        );

        let result_equal = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Minicurso de Segurança em Redes",
            "Fundamentos de análise de tráfego e firewall",
            "Sala AT-11",
            starts_at,
            starts_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        );
        assert_eq!(
            result_equal,
            Err(EventDomainError::InvalidEventPeriod {
                starts_at,
                ends_at: starts_at,
            })
        );
    }

    #[test]
    fn should_keep_event_unchanged_when_edit_validation_fails() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Semana Acadêmica de Engenharia de Redes",
            "Ciclo de palestras técnicas",
            "Auditório do Departamento de Engenharia Elétrica",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento válido");

        let invalid_edit =
            event.update_details("", "Nova descrição", "Novo local", starts_at, ends_at);
        assert_eq!(invalid_edit, Err(EventDomainError::EmptyTitle));
        assert_eq!(event.title(), "Semana Acadêmica de Engenharia de Redes");
        assert_eq!(event.description(), "Ciclo de palestras técnicas");
        assert_eq!(event.revision(), 1);

        let invalid_period_edit = event.update_details(
            "Semana Acadêmica de Engenharia de Redes",
            "Ciclo de palestras técnicas",
            "Auditório do Departamento de Engenharia Elétrica",
            starts_at,
            starts_at - chrono::Duration::hours(1),
        );
        assert!(matches!(
            invalid_period_edit,
            Err(EventDomainError::InvalidEventPeriod { .. })
        ));
        assert_eq!(event.ends_at(), ends_at);
        assert_eq!(event.revision(), 1);
    }

    #[test]
    fn should_enforce_registration_capacity_and_window_limits() {
        let (starts_at, ends_at) = sample_dates();
        let closes_at = starts_at - chrono::Duration::hours(1);
        let registration = EventRegistrationPolicy::unrestricted().with_closes_at(closes_at);

        let event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Workshop de Roteamento BGP",
            "Configuração prática de sessões BGP",
            "Laboratório de Redes - Bloco B",
            starts_at,
            ends_at,
            Some(2),
            registration,
        )
        .expect("deve criar evento com capacidade limitada e janela");

        // Capacidade vive só em Event; a policy carrega apenas enabled/opens_at/closes_at.
        assert_eq!(event.capacity(), Some(2));
        assert!(event.registration_policy().enabled());
        assert_eq!(event.registration_policy().opens_at(), None);
        assert_eq!(event.registration().closes_at(), Some(closes_at));
        assert_eq!(event.remaining_spots(0), Some(2));
        assert!(event.has_available_spots(0));

        // Tentativa após o encerramento das inscrições deve ser recusada.
        let after_closes = closes_at + chrono::Duration::minutes(1);
        assert_eq!(
            event.register_participant(after_closes, 0),
            Err(EventDomainError::RegistrationDeadlinePassed {
                deadline: closes_at,
                attempted_at: after_closes,
            })
        );

        let before_closes = closes_at - chrono::Duration::hours(1);
        let before_registration = event.clone();
        // Contagem vem de fora (tabela event_registration); a validação não muta o evento/policy.
        assert!(event.register_participant(before_closes, 0).is_ok());
        assert!(event.register_participant(before_closes, 1).is_ok());
        assert_eq!(event, before_registration);
        assert_eq!(event.remaining_spots(2), Some(0));
        assert!(!event.has_available_spots(2));

        assert_eq!(
            event.register_participant(before_closes, 2),
            Err(EventDomainError::CapacityExceeded {
                confirmed: 2,
                max_capacity: 2,
            })
        );
        assert_eq!(event, before_registration);

        assert_eq!(event.remaining_spots(1), Some(1));
        assert!(event.has_available_spots(1));
    }

    #[test]
    fn should_reject_registration_when_policy_disabled_or_not_yet_open() {
        let (starts_at, ends_at) = sample_dates();
        let opens_at = starts_at - chrono::Duration::hours(2);
        let before_opens = opens_at - chrono::Duration::minutes(5);
        let after_opens = opens_at + chrono::Duration::minutes(5);

        let disabled = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Aviso institucional sem inscrição",
            "Comunicado meramente informativo",
            "Canal interno do CA",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::disabled(),
        )
        .expect("deve criar evento informativo");

        assert_eq!(
            disabled.register_participant(after_opens, 0),
            Err(EventDomainError::RegistrationsClosed)
        );

        let gated = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Oficina com janela de inscrição",
            "Vagas liberadas somente após abertura oficial",
            "Laboratório de Redes - Bloco B",
            starts_at,
            ends_at,
            Some(10),
            EventRegistrationPolicy::unrestricted().with_opens_at(opens_at),
        )
        .expect("deve criar evento com abertura futura");

        assert_eq!(
            gated.register_participant(before_opens, 0),
            Err(EventDomainError::RegistrationNotYetOpen {
                opens_at,
                attempted_at: before_opens,
            })
        );
        assert!(gated.register_participant(after_opens, 0).is_ok());
        // Ocupação não é desnormalizada na policy: a mesma contagem externa continua válida.
        assert!(gated.register_participant(after_opens, 0).is_ok());
        assert_eq!(gated.capacity(), Some(10));
        assert_eq!(gated.registration_policy().opens_at(), Some(opens_at));
    }

    #[test]
    fn registration_policy_json_omits_legacy_capacity_counter_fields() {
        let policy = EventRegistrationPolicy::unrestricted()
            .with_opens_at(Utc.with_ymd_and_hms(2026, 11, 1, 12, 0, 0).unwrap())
            .with_closes_at(Utc.with_ymd_and_hms(2026, 11, 9, 12, 0, 0).unwrap());

        let json = serde_json::to_value(&policy).expect("serializa policy");
        let obj = json.as_object().expect("objeto JSON");
        assert!(obj.contains_key("enabled"));
        assert!(obj.contains_key("opens_at"));
        assert!(obj.contains_key("closes_at"));
        assert!(!obj.contains_key("requires_registration"));
        assert!(!obj.contains_key("max_capacity"));
        assert!(!obj.contains_key("confirmed_registrations"));
        assert!(!obj.contains_key("registration_deadline"));
    }

    #[test]
    fn should_validate_event_id_and_participant_id_on_event_registration() {
        assert_eq!(
            EventRegistration::new_confirmed(Uuid::nil(), Uuid::new_v4()),
            Err(EventDomainError::InvalidEventId)
        );
        assert_eq!(
            EventRegistration::new_confirmed(Uuid::new_v4(), Uuid::nil()),
            Err(EventDomainError::InvalidParticipantId)
        );

        let mut reg =
            EventRegistration::new_confirmed(Uuid::new_v4(), Uuid::new_v4()).expect("válida");
        assert_eq!(reg.status(), RegistrationStatus::Registered);
        assert!(reg.cancelled_at().is_none());

        reg.cancel();
        assert_eq!(reg.status(), RegistrationStatus::Cancelled);
        assert!(reg.cancelled_at().is_some());
    }

    #[test]
    fn should_reactivate_cancelled_registration_as_registered() {
        let mut reg =
            EventRegistration::new_confirmed(Uuid::new_v4(), Uuid::new_v4()).expect("válida");
        reg.cancel();
        assert_eq!(reg.status(), RegistrationStatus::Cancelled);
        assert!(reg.cancelled_at().is_some());

        let reactivated_at = Utc.with_ymd_and_hms(2026, 12, 1, 10, 0, 0).unwrap();
        reg.reactivate_registered(reactivated_at)
            .expect("reativação permitida");

        assert_eq!(reg.status(), RegistrationStatus::Registered);
        assert!(reg.cancelled_at().is_none());
        assert!(reg.invitation_expires_at().is_none());
        assert!(reg.queue_sequence().is_none());
        assert_eq!(reg.registered_at(), reactivated_at);
    }

    #[test]
    fn should_reactivate_cancelled_registration_as_waitlisted() {
        let mut reg = EventRegistration::new_waitlisted(
            Uuid::new_v4(),
            Uuid::new_v4(),
            3,
            Utc.with_ymd_and_hms(2026, 11, 1, 9, 0, 0).unwrap(),
        )
        .expect("válida");
        reg.cancel();

        let reactivated_at = Utc.with_ymd_and_hms(2026, 12, 2, 11, 0, 0).unwrap();
        reg.reactivate_waitlisted(reactivated_at, 1)
            .expect("reativação na fila permitida");

        assert_eq!(reg.status(), RegistrationStatus::Waitlisted);
        assert_eq!(reg.queue_sequence(), Some(1));
        assert!(reg.cancelled_at().is_none());
        assert_eq!(reg.registered_at(), reactivated_at);
    }

    #[test]
    fn should_reject_reactivate_when_registration_is_not_cancelled() {
        let mut reg =
            EventRegistration::new_confirmed(Uuid::new_v4(), Uuid::new_v4()).expect("válida");

        let result = reg.reactivate_registered(Utc::now());
        assert_eq!(result, Err(EventDomainError::RegistrationNotCancelled));

        let result = reg.reactivate_waitlisted(Utc::now(), 1);
        assert_eq!(result, Err(EventDomainError::RegistrationNotCancelled));
        assert_eq!(reg.status(), RegistrationStatus::Registered);
        assert!(reg.cancelled_at().is_none());
    }

    #[test]
    fn should_transition_published_to_pending_review_on_update_details() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Evento publicado editável",
            "Descrição original publicada",
            "Local original",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento publicado");

        assert_eq!(event.status(), EventPublicationStatus::Published);
        assert_eq!(event.revision(), 1);
        assert!(event.is_published());

        event
            .update_details(
                "Título revisado",
                "Descrição revisada após publicação",
                "Novo local",
                starts_at,
                ends_at,
            )
            .expect("edição de publicado deve ser permitida");

        assert_eq!(event.status(), EventPublicationStatus::PendingReview);
        assert_eq!(event.revision(), 2);
        assert!(!event.is_published());
        assert_eq!(event.title(), "Título revisado");
    }

    #[test]
    fn should_keep_draft_status_when_updating_details() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Rascunho editável",
            "Descrição do rascunho",
            "Local do rascunho",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar rascunho");

        event
            .update_details(
                "Rascunho atualizado",
                "Nova descrição do rascunho",
                "Novo local do rascunho",
                starts_at,
                ends_at,
            )
            .expect("edição de rascunho deve ser permitida");

        assert_eq!(event.status(), EventPublicationStatus::Draft);
        assert_eq!(event.revision(), 2);
        assert!(!event.is_published());
        assert_eq!(event.title(), "Rascunho atualizado");
    }

    #[test]
    fn should_transition_published_to_pending_review_on_configure_registration() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Evento com inscrição configurável",
            "Descrição do evento publicado",
            "Auditório principal",
            starts_at,
            ends_at,
            Some(50),
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento publicado");

        assert_eq!(event.status(), EventPublicationStatus::Published);
        assert_eq!(event.revision(), 1);
        assert!(event.is_published());

        event
            .configure_registration(Some(30), EventRegistrationPolicy::disabled())
            .expect("reconfigurar inscrição de publicado deve ser permitido");

        assert_eq!(event.status(), EventPublicationStatus::PendingReview);
        assert_eq!(event.revision(), 2);
        assert!(!event.is_published());
        assert_eq!(event.capacity(), Some(30));
        assert!(!event.registration_policy().enabled());
    }

    #[test]
    fn should_reject_update_details_when_event_is_cancelled() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Evento a cancelar",
            "Descrição original",
            "Local original",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento publicado");

        event.cancel().expect("deve cancelar o evento publicado");
        assert_eq!(event.status(), EventPublicationStatus::Cancelled);
        assert!(!event.can_edit_content());

        let result = event.update_details(
            "Novo título",
            "Nova descrição",
            "Novo local",
            starts_at,
            ends_at,
        );
        assert_eq!(
            result,
            Err(EventDomainError::EventNotEditable {
                status: EventPublicationStatus::Cancelled,
            })
        );
        assert_eq!(event.title(), "Evento a cancelar");
        assert_eq!(event.revision(), 1);
    }

    #[test]
    fn should_reject_configure_registration_when_event_is_cancelled() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Rascunho cancelável",
            "Descrição do rascunho",
            "Local do rascunho",
            starts_at,
            ends_at,
            Some(20),
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar rascunho");

        event.cancel().expect("deve cancelar o rascunho");

        let result = event.configure_registration(Some(10), EventRegistrationPolicy::disabled());
        assert_eq!(
            result,
            Err(EventDomainError::EventNotEditable {
                status: EventPublicationStatus::Cancelled,
            })
        );
        assert_eq!(event.capacity(), Some(20));
        assert!(event.registration_policy().enabled());
        assert_eq!(event.revision(), 1);
    }

    #[test]
    fn should_cancel_published_event_and_treat_second_cancel_as_noop() {
        let (starts_at, ends_at) = sample_dates();
        let mut event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Evento publicado cancelável",
            "Descrição do evento publicado",
            "Auditório principal",
            starts_at,
            ends_at,
            None,
            EventRegistrationPolicy::unrestricted(),
        )
        .expect("deve criar evento publicado");

        event
            .cancel()
            .expect("deve cancelar evento publicado");
        assert_eq!(event.status(), EventPublicationStatus::Cancelled);

        let snapshot = event.clone();
        event
            .cancel()
            .expect("segundo cancel deve ser no-op seguro");
        assert_eq!(event, snapshot);
        assert_eq!(event.status(), EventPublicationStatus::Cancelled);
    }
}
