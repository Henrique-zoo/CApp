//! Portas de persistência e integração do módulo de perguntas frequentes (FAQ).

use uuid::Uuid;

use crate::modules::faq::{
    application::error::FaqRepositoryError, domain::entities::faq_entry::FaqEntry,
};

/// Estrutura que resume o status do Centro Acadêmico e o nível de acesso do usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaManagementAccess {
    /// Indica se o Centro Acadêmico existe no sistema.
    pub ca_exists: bool,

    /// Indica se o Centro Acadêmico está ativo no ciclo de vida.
    pub ca_active: bool,

    /// Indica se o usuário possui cargo ou atribuição ativa de gestão no respectivo CA.
    pub is_ca_manager: bool,
}

impl CaManagementAccess {
    /// Cria uma nova instância de verificação de acesso de gestão ao CA.
    pub fn new(ca_exists: bool, ca_active: bool, is_ca_manager: bool) -> Self {
        Self {
            ca_exists,
            ca_active,
            is_ca_manager,
        }
    }
}

/// Contrato de persistência para recuperação e cadastro de perguntas e respostas de FAQ.
#[async_trait::async_trait]
pub trait FaqRepository: Send + Sync {
    /// Persiste uma nova entrada de FAQ.
    async fn save(&self, entry: &FaqEntry) -> Result<(), FaqRepositoryError>;

    /// Localiza uma entrada de FAQ por seu identificador único.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FaqEntry>, FaqRepositoryError>;

    /// Lista entradas de FAQ cadastradas em determinado Centro Acadêmico.
    async fn find_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<FaqEntry>, FaqRepositoryError>;

    /// Consulta a existência, situação ativa e permissão de gestão do usuário no CA especificado.
    async fn check_ca_management_access(
        &self,
        academic_center_id: Uuid,
        user_id: Uuid,
    ) -> Result<CaManagementAccess, FaqRepositoryError>;
}

#[async_trait::async_trait]
impl<T: FaqRepository + ?Sized> FaqRepository for &T {
    async fn save(&self, entry: &FaqEntry) -> Result<(), FaqRepositoryError> {
        (**self).save(entry).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FaqEntry>, FaqRepositoryError> {
        (**self).find_by_id(id).await
    }

    async fn find_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<FaqEntry>, FaqRepositoryError> {
        (**self)
            .find_by_academic_center_id(academic_center_id)
            .await
    }

    async fn check_ca_management_access(
        &self,
        academic_center_id: Uuid,
        user_id: Uuid,
    ) -> Result<CaManagementAccess, FaqRepositoryError> {
        (**self)
            .check_ca_management_access(academic_center_id, user_id)
            .await
    }
}
