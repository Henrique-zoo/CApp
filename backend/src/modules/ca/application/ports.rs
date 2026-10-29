//! Portas de entrada e saída para autorização e gerenciamento de cargos.
//!
//! Define contratos abstratos desacoplados de bibliotecas externas e bancos de dados,
//! em estrita conformidade com a Clean Architecture.

use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::modules::ca::domain::entities::{
    permission::Permission, role::Role, role_assignment::RoleAssignment,
};

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
