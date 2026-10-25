//! Adaptador de autorização em memória para testes e desenvolvimento.

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::news::application::ports::NewsAuthorizationPort;
use crate::modules::news::domain::NewsError;

/// Implementação em memória de [`NewsAuthorizationPort`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryNewsAuthorizationService {
    manage_permissions: Arc<RwLock<HashSet<(Uuid, Uuid)>>>,
    publish_permissions: Arc<RwLock<HashSet<(Uuid, Uuid)>>>,
    allow_all: bool,
}

impl InMemoryNewsAuthorizationService {
    /// Constrói um serviço com permissões vazias (nega por padrão).
    pub fn new() -> Self {
        Self {
            manage_permissions: Arc::new(RwLock::new(HashSet::new())),
            publish_permissions: Arc::new(RwLock::new(HashSet::new())),
            allow_all: false,
        }
    }

    /// Constrói um serviço que autoriza qualquer operação (útil para testes não focados em permissão).
    pub fn permissive() -> Self {
        Self {
            manage_permissions: Arc::new(RwLock::new(HashSet::new())),
            publish_permissions: Arc::new(RwLock::new(HashSet::new())),
            allow_all: true,
        }
    }

    /// Concede permissão de gestão no Centro Acadêmico para o usuário.
    pub fn grant_manage_permission(&self, user_id: Uuid, ca_id: Uuid) {
        if let Ok(mut set) = self.manage_permissions.write() {
            set.insert((user_id, ca_id));
        }
    }

    /// Concede permissão de publicação direta no Centro Acadêmico para o usuário.
    pub fn grant_publish_permission(&self, user_id: Uuid, ca_id: Uuid) {
        if let Ok(mut set) = self.publish_permissions.write() {
            set.insert((user_id, ca_id));
        }
    }

    /// Revoga permissão de gestão no Centro Acadêmico para o usuário.
    pub fn revoke_manage_permission(&self, user_id: Uuid, ca_id: Uuid) {
        if let Ok(mut set) = self.manage_permissions.write() {
            set.remove(&(user_id, ca_id));
        }
    }

    /// Revoga permissão de publicação direta no Centro Acadêmico para o usuário.
    pub fn revoke_publish_permission(&self, user_id: Uuid, ca_id: Uuid) {
        if let Ok(mut set) = self.publish_permissions.write() {
            set.remove(&(user_id, ca_id));
        }
    }
}

#[async_trait]
impl NewsAuthorizationPort for InMemoryNewsAuthorizationService {
    async fn can_manage_news(&self, user_id: Uuid, ca_id: Uuid) -> Result<bool, NewsError> {
        if self.allow_all {
            return Ok(true);
        }
        let set = self
            .manage_permissions
            .read()
            .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
        Ok(set.contains(&(user_id, ca_id)))
    }

    async fn can_publish_directly(&self, user_id: Uuid, ca_id: Uuid) -> Result<bool, NewsError> {
        if self.allow_all {
            return Ok(true);
        }
        let set = self
            .publish_permissions
            .read()
            .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
        Ok(set.contains(&(user_id, ca_id)))
    }
}
