//! Repositórios e adaptadores de persistência de cargos e autorizações do CA.
//!
//! Fornece implementações em memória concorrentes e seguras para testes de integração
//! e execução sem dependência obrigatória de banco de dados externo.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::ca::{
    application::ports::{
        RoleAssignmentRepositoryError, RoleAssignmentRepositoryPort, RoleRepositoryError,
        RoleRepositoryPort,
    },
    domain::entities::{role::Role, role_assignment::RoleAssignment},
};

/// Repositório de cargos em memória, seguro para concorrência entre threads.
#[derive(Debug, Default, Clone)]
pub struct InMemoryRoleRepository {
    roles: Arc<RwLock<HashMap<Uuid, Role>>>,
}

impl InMemoryRoleRepository {
    /// Cria uma nova instância vazia do repositório de cargos em memória.
    pub fn new() -> Self {
        Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Cria uma instância preenchida com os cargos padrão didáticos da UnB (CAER e CACC).
    pub fn with_didactic_roles(caer_id: Uuid, cacc_id: Uuid) -> Self {
        let mut repo = Self::new();
        let mut map = HashMap::new();

        // Cargos de CAER
        let pres_caer = Role::presidente(caer_id);
        let vice_caer = Role::vice_presidente(caer_id);
        let tes_caer = Role::tesoureiro(caer_id);
        let dir_com_caer = Role::diretor_comunicacao(caer_id);
        let dir_eve_caer = Role::diretor_eventos(caer_id);

        map.insert(pres_caer.id, pres_caer);
        map.insert(vice_caer.id, vice_caer);
        map.insert(tes_caer.id, tes_caer);
        map.insert(dir_com_caer.id, dir_com_caer);
        map.insert(dir_eve_caer.id, dir_eve_caer);

        // Cargos de CACC
        let pres_cacc = Role::presidente(cacc_id);
        let vice_cacc = Role::vice_presidente(cacc_id);
        let tes_cacc = Role::tesoureiro(cacc_id);

        map.insert(pres_cacc.id, pres_cacc);
        map.insert(vice_cacc.id, vice_cacc);
        map.insert(tes_cacc.id, tes_cacc);

        repo.roles = Arc::new(RwLock::new(map));
        repo
    }
}

#[async_trait]
impl RoleRepositoryPort for InMemoryRoleRepository {
    async fn save_role(&self, role: &Role) -> Result<(), RoleRepositoryError> {
        let mut lock = self.roles.write().await;
        lock.insert(role.id, role.clone());
        Ok(())
    }

    async fn find_role_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleRepositoryError> {
        let lock = self.roles.read().await;
        Ok(lock.get(&id).cloned())
    }

    async fn find_roles_by_ca_id(&self, ca_id: Uuid) -> Result<Vec<Role>, RoleRepositoryError> {
        let lock = self.roles.read().await;
        Ok(lock
            .values()
            .filter(|r| r.academic_center_id == ca_id)
            .cloned()
            .collect())
    }

    async fn find_standard_role_by_name(
        &self,
        ca_id: Uuid,
        name: &str,
    ) -> Result<Option<Role>, RoleRepositoryError> {
        let lock = self.roles.read().await;
        let normalized = name.trim().to_lowercase();
        Ok(lock
            .values()
            .find(|r| r.academic_center_id == ca_id && r.name.to_lowercase() == normalized)
            .cloned())
    }

    async fn delete_role(&self, id: Uuid) -> Result<(), RoleRepositoryError> {
        let mut lock = self.roles.write().await;
        lock.remove(&id);
        Ok(())
    }
}

/// Repositório de vinculações e histórico administrativo em memória.
#[derive(Debug, Default, Clone)]
pub struct InMemoryRoleAssignmentRepository {
    assignments: Arc<RwLock<Vec<RoleAssignment>>>,
}

impl InMemoryRoleAssignmentRepository {
    /// Constrói um repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            assignments: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl RoleAssignmentRepositoryPort for InMemoryRoleAssignmentRepository {
    async fn save_assignment(
        &self,
        assignment: &RoleAssignment,
    ) -> Result<(), RoleAssignmentRepositoryError> {
        let mut lock = self.assignments.write().await;
        if let Some(pos) = lock.iter().position(|a| a.id == assignment.id) {
            lock[pos] = assignment.clone();
        } else {
            lock.push(assignment.clone());
        }
        Ok(())
    }

    async fn find_assignment_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<RoleAssignment>, RoleAssignmentRepositoryError> {
        let lock = self.assignments.read().await;
        Ok(lock.iter().find(|a| a.id == id).cloned())
    }

    async fn find_active_by_user_and_ca(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError> {
        let lock = self.assignments.read().await;
        Ok(lock
            .iter()
            .filter(|a| a.user_id == user_id && a.academic_center_id == ca_id && a.is_active())
            .cloned()
            .collect())
    }

    async fn find_active_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError> {
        let lock = self.assignments.read().await;
        Ok(lock
            .iter()
            .filter(|a| a.user_id == user_id && a.is_active())
            .cloned()
            .collect())
    }

    async fn find_history_by_user_and_ca(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError> {
        let lock = self.assignments.read().await;
        Ok(lock
            .iter()
            .filter(|a| a.user_id == user_id && a.academic_center_id == ca_id)
            .cloned()
            .collect())
    }

    async fn find_active_by_role_id(
        &self,
        role_id: Uuid,
    ) -> Result<Vec<RoleAssignment>, RoleAssignmentRepositoryError> {
        let lock = self.assignments.read().await;
        Ok(lock
            .iter()
            .filter(|a| a.role_id == role_id && a.is_active())
            .cloned()
            .collect())
    }

    async fn revoke_assignment(
        &self,
        id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), RoleAssignmentRepositoryError> {
        let mut lock = self.assignments.write().await;
        if let Some(assignment) = lock.iter_mut().find(|a| a.id == id) {
            let _ = assignment.revoke(revoked_at);
            Ok(())
        } else {
            Err(RoleAssignmentRepositoryError::NotFound(id))
        }
    }
}
