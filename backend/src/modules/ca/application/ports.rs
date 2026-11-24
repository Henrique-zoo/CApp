//! Contratos de persistência e integração do módulo de Centros Acadêmicos.
//!
//! Os casos de uso dependem exclusivamente destas traits, sem acoplamento a SQLx,
//! drivers de banco de dados ou detalhes de infraestrutura HTTP.

use crate::modules::ca::application::error::MandateApplicationError;
use crate::modules::ca::application::error::PollApplicationError;
use crate::modules::ca::domain::entities::AcademicCenter;
use crate::modules::ca::domain::entities::Mandate;
use crate::modules::ca::domain::entities::PollVote;
use crate::modules::ca::domain::entities::StudentPoll;
use crate::modules::ca::domain::entities::permission::Permission;
use crate::modules::ca::domain::entities::role::Role;
use crate::modules::ca::domain::entities::role_assignment::RoleAssignment;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;
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
/// Erros originados nas operações de persistência de mandato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateRepositoryError {
    /// Falha técnica interna no adaptador de persistência.
    Internal(String),
    /// Conflito de integridade relacional ou unicidade de mandato ativo.
    Conflict(String),
}
impl fmt::Display for MandateRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "falha no repositório de mandatos: {msg}"),
            Self::Conflict(msg) => write!(f, "conflito no repositório de mandatos: {msg}"),
        }
    }
}
impl Error for MandateRepositoryError {}
/// Contrato de persistência para recuperação e consulta de Centros Acadêmicos.
#[async_trait]
pub trait AcademicCenterRepository: Send + Sync {
    /// Busca um Centro Acadêmico pelo seu identificador único.
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<AcademicCenter>, AcademicCenterRepositoryError>;
}
/// Contrato de persistência para armazenamento e consulta de mandatos de gestão.
#[async_trait]
pub trait MandateRepository: Send + Sync {
    /// Persiste um novo mandato de gestão ou atualiza suas informações.
    async fn save(&self, mandate: &Mandate) -> Result<(), MandateRepositoryError>;

    /// Recupera um mandato pelo seu identificador único.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, MandateRepositoryError>;

    /// Recupera o mandato atualmente ativo de um Centro Acadêmico, se houver.
    async fn find_active_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Option<Mandate>, MandateRepositoryError>;

    /// Lista todos os mandatos (histórico) associados a um Centro Acadêmico.
    async fn find_all_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<Mandate>, MandateRepositoryError>;
}
/// Contrato de verificação de autorização de gestão no Centro Acadêmico.
#[async_trait]
pub trait AcademicCenterAuthorizationPort: Send + Sync {
    /// Determina se o usuário possui permissão administrativa no Centro Acadêmico indicado.
    async fn is_user_permitted(
        &self,
        user_id: Uuid,
        ca: &AcademicCenter,
    ) -> Result<bool, MandateApplicationError>;
}
/// Política padrão de autorização: o criador / responsável inicial do CA possui autorização.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultAcademicCenterAuthorization;
#[async_trait]
impl AcademicCenterAuthorizationPort for DefaultAcademicCenterAuthorization {
    async fn is_user_permitted(
        &self,
        user_id: Uuid,
        ca: &AcademicCenter,
    ) -> Result<bool, MandateApplicationError> {
        Ok(ca.created_by_id() == Some(user_id))
    }
}
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
pub trait PollAuthorizationPort: Send + Sync {
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
pub struct DefaultPollAuthorization;
#[async_trait]
impl PollAuthorizationPort for DefaultPollAuthorization {
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
/// Erros emitidos pelas portas e casos de uso de autorização.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorizationServiceError {
    /// O usuário não possui permissão para executar a operação no CA requisitado.
    #[error("Operação recusada por falta de permissão: {0}")]
    Forbidden(String),

    /// Tentativa de utilizar credenciais de um Centro Acadêmico em outro distinto.
    #[error(
        "Isolamento violado: o cargo pertence ao CA '{role_ca_id}', mas o recurso alvo é do CA '{target_ca_id}'."
    )]
    AcademicCenterMismatch {
        /// CA ao qual o cargo pertence.
        role_ca_id: Uuid,
        /// CA do recurso acessado.
        target_ca_id: Uuid,
    },

    /// O usuário não possui nenhum cargo ativo no Centro Acadêmico informado.
    #[error("O usuário '{user_id}' não possui nenhum cargo ativo no CA '{ca_id}'.")]
    NoActiveRoleInCa {
        /// Identificador do usuário.
        user_id: Uuid,
        /// Identificador do CA.
        ca_id: Uuid,
    },

    /// Falha técnica no repositório subjacente.
    #[error("Falha técnica no serviço de autorização: {0}")]
    RepositoryFailure(String),
}
/// Erros decorrentes de persistência ou recuperação de cargos.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleRepositoryError {
    /// Cargo não localizado.
    #[error("Cargo '{0}' não encontrado.")]
    NotFound(Uuid),

    /// Falha técnica no armazenamento de cargos.
    #[error("Falha técnica no repositório de cargos: {0}")]
    DatabaseFailure(String),
}
/// Erros decorrentes de persistência ou recuperação de vinculações de integrantes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleAssignmentRepositoryError {
    /// Atribuição não localizada.
    #[error("Vinculação de cargo '{0}' não encontrada.")]
    NotFound(Uuid),

    /// Falha técnica no armazenamento de vinculações.
    #[error("Falha técnica no repositório de vinculações: {0}")]
    DatabaseFailure(String),
}
/// Porta do serviço de autorização institucional nos Centros Acadêmicos.
#[async_trait]
pub trait AuthorizationServicePort: Send + Sync {
    /// Valida se o usuário tem a permissão requerida no CA, retornando erro detalhado em caso de recusa.
    async fn check_permission(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        permission: &Permission,
    ) -> Result<(), AuthorizationServiceError>;

    /// Retorna verdadeiro se o usuário possuir a permissão requerida no CA informado.
    async fn has_permission(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        permission: &Permission,
    ) -> Result<bool, AuthorizationServiceError>;

    /// Consolida o conjunto completo de todas as permissões ativas e derivadas do usuário no CA informado.
    async fn get_effective_permissions(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<HashSet<Permission>, AuthorizationServiceError>;

    /// Lista os cargos atualmente exercidos pelo usuário no CA informado.
    async fn list_active_roles(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<Vec<Role>, AuthorizationServiceError>;
}
/// Porta para repositório de cargos administrativos de Centros Acadêmicos.
#[async_trait]
pub trait RoleRepositoryPort: Send + Sync {
    /// Salva ou atualiza um cargo administrativo.
    async fn save_role(&self, role: &Role) -> Result<(), RoleRepositoryError>;

    /// Busca um cargo pelo seu identificador único.
    async fn find_role_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleRepositoryError>;

    /// Lista todos os cargos vinculados a um Centro Acadêmico específico.
    async fn find_roles_by_ca_id(&self, ca_id: Uuid) -> Result<Vec<Role>, RoleRepositoryError>;

    /// Busca um cargo padrão por nome no contexto de um CA.
    async fn find_standard_role_by_name(
        &self,
        ca_id: Uuid,
        name: &str,
    ) -> Result<Option<Role>, RoleRepositoryError>;

    /// Remove um cargo administrativo do CA.
    async fn delete_role(&self, id: Uuid) -> Result<(), RoleRepositoryError>;
}
/// Porta para repositório de vinculação de integrantes e histórico administrativo.
#[async_trait]
pub trait RoleAssignmentRepositoryPort: Send + Sync {
    /// Persiste uma nova vinculação de integrante.
    async fn save_assignment(
        &self,
        assignment: &RoleAssignment,
    ) -> Result<(), RoleAssignmentRepositoryError>;

    /// Localiza um registro de vinculação pelo ID.
    async fn find_assignment_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<RoleAssignment>, RoleAssignmentRepositoryError>;

    /// Lista as vinculações ativas de um usuário em determinado CA.
    async fn find_active_by_user_and_ca(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError>;

    /// Lista todas as vinculações ativas de um usuário em qualquer Centro Acadêmico.
    async fn find_active_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError>;

    /// Lista o histórico integral de vinculações (ativas e revogadas) de um usuário no CA.
    async fn find_history_by_user_and_ca(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError>;

    /// Lista as vinculações ativas para um cargo específico.
    async fn find_active_by_role_id(
        &self,
        role_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError>;

    /// Revoga uma vinculação pelo seu identificador, marcando a data de término sem remover o registro.
    async fn revoke_assignment(
        &self,
        id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), RoleAssignmentRepositoryError>;
}
