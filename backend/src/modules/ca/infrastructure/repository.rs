//! Repositórios e adaptadores de armazenamento em memória e sessão do módulo de Centros Acadêmicos.
//!
//! Fornece o contrato de sessão [`ActiveContextSessionStore`] e sua implementação
//! em memória [`InMemoryActiveContextSessionStore`], além do contrato de consulta
//! [`AcademicCenterRepository`] e implementação com dados didáticos da UnB (CAER / CACC).

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::ca::domain::entities::{
    academic_center::{AcademicCenter, AcademicCenterError, AcademicCenterStatus},
    active_context::{ActiveCaContext, ActiveContextError},
};

/// Contrato assíncrono para manipulação do contexto ativo de navegação em sessão/memória.
#[async_trait]
pub trait ActiveContextSessionStore: Send + Sync {
    /// Obtém o contexto ativo atualmente em sessão para o usuário informado.
    async fn get_active_context(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ActiveCaContext>, ActiveContextError>;

    /// Armazena ou atualiza o contexto ativo na sessão em memória do usuário.
    async fn set_active_context(&self, context: &ActiveCaContext)
    -> Result<(), ActiveContextError>;

    /// Remove o contexto ativo da sessão do usuário, retornando se havia contexto anterior.
    async fn clear_active_context(&self, user_id: Uuid) -> Result<bool, ActiveContextError>;
}

/// Implementação em memória de [`ActiveContextSessionStore`] para sessões de navegação.
#[derive(Debug, Clone, Default)]
pub struct InMemoryActiveContextSessionStore {
    storage: Arc<RwLock<HashMap<Uuid, ActiveCaContext>>>,
}

impl InMemoryActiveContextSessionStore {
    /// Constrói um armazenamento de sessão em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Constrói o armazenamento com uma sessão inicial ativa pré-configurada.
    pub fn with_didactic_session(user_id: Uuid, ca_id: Uuid) -> Self {
        let store = Self::new();
        if let Ok(context) = ActiveCaContext::new(user_id, ca_id) {
            let mut map = HashMap::new();
            map.insert(user_id, context);
            return Self {
                storage: Arc::new(RwLock::new(map)),
            };
        }
        store
    }
}

#[async_trait]
impl ActiveContextSessionStore for InMemoryActiveContextSessionStore {
    async fn get_active_context(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ActiveCaContext>, ActiveContextError> {
        let lock = self.storage.read().await;
        Ok(lock.get(&user_id).cloned())
    }

    async fn set_active_context(
        &self,
        context: &ActiveCaContext,
    ) -> Result<(), ActiveContextError> {
        let mut lock = self.storage.write().await;
        lock.insert(context.user_id(), context.clone());
        Ok(())
    }

    async fn clear_active_context(&self, user_id: Uuid) -> Result<bool, ActiveContextError> {
        let mut lock = self.storage.write().await;
        Ok(lock.remove(&user_id).is_some())
    }
}

/// Contrato assíncrono para consulta e cadastro de Centros Acadêmicos.
#[async_trait]
pub trait AcademicCenterRepository: Send + Sync {
    /// Localiza um Centro Acadêmico pelo seu identificador único estável.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AcademicCenter>, AcademicCenterError>;

    /// Lista todos os Centros Acadêmicos com status ativo no sistema.
    async fn list_active(&self) -> Result<Vec<AcademicCenter>, AcademicCenterError>;
}

/// Implementação em memória de [`AcademicCenterRepository`] pré-populada com CAs didáticos.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAcademicCenterRepository {
    storage: Arc<RwLock<HashMap<Uuid, AcademicCenter>>>,
}

impl InMemoryAcademicCenterRepository {
    /// Constrói um repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Constrói um repositório pré-populado com os CAs didáticos da UnB (CAER e CACC).
    pub fn with_didactic_cas() -> Self {
        let mut map = HashMap::new();
        let caer = AcademicCenter::caer_didactic();
        let cacc = AcademicCenter::cacc_didactic();
        map.insert(caer.id, caer);
        map.insert(cacc.id, cacc);

        Self {
            storage: Arc::new(RwLock::new(map)),
        }
    }

    /// Cadastra ou atualiza um Centro Acadêmico no repositório em memória.
    pub async fn save(&self, ca: AcademicCenter) {
        let mut lock = self.storage.write().await;
        lock.insert(ca.id, ca);
    }
}

#[async_trait]
impl AcademicCenterRepository for InMemoryAcademicCenterRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AcademicCenter>, AcademicCenterError> {
        let lock = self.storage.read().await;
        Ok(lock.get(&id).cloned())
    }

    async fn list_active(&self) -> Result<Vec<AcademicCenter>, AcademicCenterError> {
        let lock = self.storage.read().await;
        Ok(lock
            .values()
            .filter(|ca| ca.status == AcademicCenterStatus::Active)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ca::domain::entities::academic_center::{
        CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    };

    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[tokio::test]
    async fn should_store_and_retrieve_active_context_in_session() {
        let store = InMemoryActiveContextSessionStore::new();

        assert!(store.get_active_context(USER_ID).await.unwrap().is_none());

        let context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("valid context");
        store.set_active_context(&context).await.unwrap();

        let retrieved = store.get_active_context(USER_ID).await.unwrap().unwrap();
        assert_eq!(retrieved.user_id(), USER_ID);
        assert_eq!(retrieved.active_ca_id(), CAER_DIDACTIC_ID);

        assert!(store.clear_active_context(USER_ID).await.unwrap());
        assert!(store.get_active_context(USER_ID).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn should_query_didactic_academic_centers() {
        let repo = InMemoryAcademicCenterRepository::with_didactic_cas();

        let caer = repo.find_by_id(CAER_DIDACTIC_ID).await.unwrap().unwrap();
        assert_eq!(caer.acronym, "CAER");

        let cacc = repo.find_by_id(CACC_DIDACTIC_ID).await.unwrap().unwrap();
        assert_eq!(cacc.acronym, "CACC");

        let active = repo.list_active().await.unwrap();
        assert_eq!(active.len(), 2);
    }
}
