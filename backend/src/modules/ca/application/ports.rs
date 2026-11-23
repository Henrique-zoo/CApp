//! Contratos de persistência e integração do módulo de Centros Acadêmicos.
//!
//! Os casos de uso dependem exclusivamente destas traits, sem acoplamento a SQLx,
//! drivers de banco de dados ou detalhes de infraestrutura HTTP.

use std::{collections::HashMap, error::Error, fmt};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::ca::{
    application::error::PollApplicationError,
    domain::entities::{AcademicCenter, PollVote, StudentPoll},
};

/// Erros originados nas operações de persistência de Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademicCenterRepositoryError {
    /// Falha técnica interna no adaptador de persistência.
    Internal(String),
}

impl fmt::Display for AcademicCenterRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "falha no repositório de Centro Acadêmico: {msg}"),
        }
    }
}

impl Error for AcademicCenterRepositoryError {}

/// Erros originados nas operações de persistência de consultas e enquetes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollRepositoryError {
    /// Falha técnica interna no adaptador de persistência.
    Internal(String),
    /// Conflito de integridade relacional ou voto duplicado.
    Conflict(String),
    /// Recurso não encontrado no repositório.
    NotFound(String),
}

impl fmt::Display for PollRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "falha no repositório de enquetes: {msg}"),
            Self::Conflict(msg) => write!(f, "conflito no repositório de enquetes: {msg}"),
            Self::NotFound(msg) => write!(f, "não encontrado no repositório de enquetes: {msg}"),
        }
    }
}

impl Error for PollRepositoryError {}

/// Contrato de persistência para recuperação e consulta de Centros Acadêmicos.
#[async_trait]
pub trait AcademicCenterRepository: Send + Sync {
    /// Busca um Centro Acadêmico pelo seu identificador único.
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<AcademicCenter>, AcademicCenterRepositoryError>;
}

/// Contrato de persistência para consultas, enquetes e votos estudantis.
#[async_trait]
pub trait PollRepository: Send + Sync {
    /// Persiste uma nova consulta estudantil com suas opções de voto.
    async fn save(&self, poll: &StudentPoll) -> Result<(), PollRepositoryError>;

    /// Recupera uma consulta estudantil pelo seu identificador único.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StudentPoll>, PollRepositoryError>;

    /// Lista todas as consultas associadas a um Centro Acadêmico.
    async fn find_all_by_ca_id(&self, ca_id: Uuid)
    -> Result<Vec<StudentPoll>, PollRepositoryError>;

    /// Persiste um novo registro de voto na enquete.
    async fn save_vote(&self, vote: &PollVote) -> Result<(), PollRepositoryError>;

    /// Verifica se um estudante já votou na consulta informada.
    async fn has_user_voted(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, PollRepositoryError>;

    /// Apura a contagem de votos agrupada por opção para a consulta indicada.
    async fn count_votes_by_option(
        &self,
        poll_id: Uuid,
    ) -> Result<HashMap<Uuid, i64>, PollRepositoryError>;

    /// Recupera todos os votos registrados para a consulta indicada.
    async fn find_votes_by_poll_id(
        &self,
        poll_id: Uuid,
    ) -> Result<Vec<PollVote>, PollRepositoryError>;

    /// Encerra a consulta e atualiza o seu registro de encerramento.
    async fn close_poll(
        &self,
        poll_id: Uuid,
        closed_by_id: Uuid,
        closed_at: DateTime<Utc>,
    ) -> Result<StudentPoll, PollRepositoryError>;
}

/// Contrato de verificação de permissões e elegibilidade nas consultas do Centro Acadêmico.
#[async_trait]
pub trait AcademicCenterAuthorizationPort: Send + Sync {
    /// Determina se o usuário possui permissão administrativa no Centro Acadêmico para gerenciar consultas.
    async fn is_user_permitted_to_manage(
        &self,
        user_id: Uuid,
        ca: &AcademicCenter,
    ) -> Result<bool, PollApplicationError>;

    /// Determina se o estudante é elegível para participar/votar na consulta.
    async fn is_student_eligible_to_vote(
        &self,
        user_id: Uuid,
        poll: &StudentPoll,
    ) -> Result<bool, PollApplicationError>;
}

/// Política padrão de autorização: o criador / responsável inicial do CA possui permissão de gestão;
/// estudantes autenticados são elegíveis para enquetes direcionadas a todos os estudantes.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultAcademicCenterAuthorization;

#[async_trait]
impl AcademicCenterAuthorizationPort for DefaultAcademicCenterAuthorization {
    async fn is_user_permitted_to_manage(
        &self,
        user_id: Uuid,
        ca: &AcademicCenter,
    ) -> Result<bool, PollApplicationError> {
        Ok(ca.created_by_id() == Some(user_id))
    }

    async fn is_student_eligible_to_vote(
        &self,
        _user_id: Uuid,
        _poll: &StudentPoll,
    ) -> Result<bool, PollApplicationError> {
        // No escopo atual de consultas estudantis, o público padrão "all_students"
        // abrange qualquer estudante autenticado do Centro Acadêmico.
        Ok(true)
    }
}
