//! Serviço de aplicação para gestão do contexto ativo de navegação em sessão.
//!
//! Coordena a inicialização do contexto na abertura do aplicativo, a alternância
//! entre Centros Acadêmicos disponíveis e a garantia de isolamento de privilégios.

use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::{
    ca::{
        domain::{
            entities::{
                academic_center::CAER_DIDACTIC_ID,
                active_context::{ActiveCaContext, ActiveContextError, ManagementRoleAssignment},
            },
            rules::{
                assert_favorite_preference_unaltered_by_context_switch,
                has_administrative_permission_in_active_context, is_visitor_in_active_context,
                validate_active_context_switch,
            },
        },
        infrastructure::repository::{
            AcademicCenterRepository, ActiveContextSessionStore, InMemoryAcademicCenterRepository,
            InMemoryActiveContextSessionStore,
        },
    },
    identity::{
        domain::entities::user_preference::UserPreference,
        infrastructure::repository::{InMemoryUserPreferenceRepository, UserPreferenceRepository},
    },
};

/// Serviço responsável por coordenar a seleção, alternância e regras de contexto ativo de CA em sessão.
#[derive(Clone)]
pub struct ActiveContextService {
    session_store: Arc<dyn ActiveContextSessionStore>,
    preference_repo: Arc<dyn UserPreferenceRepository>,
    ca_repo: Arc<dyn AcademicCenterRepository>,
    role_assignments: Arc<RwLock<Vec<ManagementRoleAssignment>>>,
}

impl ActiveContextService {
    /// Constrói uma nova instância de [`ActiveContextService`] com dependências explícitas.
    pub fn new(
        session_store: Arc<dyn ActiveContextSessionStore>,
        preference_repo: Arc<dyn UserPreferenceRepository>,
        ca_repo: Arc<dyn AcademicCenterRepository>,
    ) -> Self {
        Self {
            session_store,
            preference_repo,
            ca_repo,
            role_assignments: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Cria uma configuração pronta para uso com repositórios em memória e dados didáticos da UnB (CAER / CACC).
    pub fn with_didactic_setup() -> Self {
        let session_store = Arc::new(InMemoryActiveContextSessionStore::new());
        let preference_repo = Arc::new(InMemoryUserPreferenceRepository::with_didactic_data());
        let ca_repo = Arc::new(InMemoryAcademicCenterRepository::with_didactic_cas());

        Self::new(session_store, preference_repo, ca_repo)
    }

    /// Simula a abertura inicial do aplicativo pelo estudante, estabelecendo o contexto ativo.
    ///
    /// Se o estudante possuir um CA favorito configurado em sua preferência persistida,
    /// o contexto ativo inicial em sessão adotará esse CA. Caso contrário, utiliza o CAER
    /// como contexto padrão de navegação institucional.
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError`] caso os dados ou validações falhem.
    pub async fn open_app(&self, user_id: Uuid) -> Result<ActiveCaContext, ActiveContextError> {
        let preference = self
            .preference_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| ActiveContextError::NilUserId)?
            .unwrap_or_else(|| {
                UserPreference::default_for_user(user_id).unwrap_or(UserPreference {
                    user_id,
                    favorite_ca_id: None,
                    updated_at: chrono::Utc::now(),
                })
            });

        let context = ActiveCaContext::from_preference(user_id, &preference, CAER_DIDACTIC_ID)?;
        self.session_store.set_active_context(&context).await?;
        Ok(context)
    }

    /// Alterna o contexto ativo de navegação do estudante para outro Centro Acadêmico cadastrado.
    ///
    /// # Regra Estrita de Negócio
    /// 1. Valida que o `target_ca_id` pertence à lista de CAs cadastrados e ativos.
    /// 2. Atualiza o contexto ativo da sessão em memória.
    /// 3. Confirma e garante que a preferência persistida de CA favorito do estudante
    ///    **NÃO FOI MODIFICADA** no repositório de preferências.
    ///
    /// # Erros
    ///
    /// Retorna erro se o CA de destino não existir, não estiver ativo ou se o identificador for nulo.
    pub async fn switch_active_ca(
        &self,
        user_id: Uuid,
        target_ca_id: Uuid,
    ) -> Result<ActiveCaContext, ActiveContextError> {
        let active_cas = self
            .ca_repo
            .list_active()
            .await
            .map_err(|_| ActiveContextError::AcademicCenterNotFound(target_ca_id))?;

        let current_context = self
            .session_store
            .get_active_context(user_id)
            .await?
            .unwrap_or_else(|| {
                ActiveCaContext::new(user_id, CAER_DIDACTIC_ID).expect("valid fallback")
            });

        validate_active_context_switch(&current_context, target_ca_id, &active_cas)?;

        let mut updated_context = current_context;
        updated_context.switch_to_registered(target_ca_id, &active_cas)?;

        self.session_store
            .set_active_context(&updated_context)
            .await?;

        // Validação estrita da regra: confirma que a preferência persistida permaneceu inalterada
        if let Ok(Some(saved_preference)) = self.preference_repo.find_by_user_id(user_id).await {
            assert_favorite_preference_unaltered_by_context_switch(
                &saved_preference,
                &updated_context,
            )?;
        }

        Ok(updated_context)
    }

    /// Retorna o contexto ativo atual em sessão para o estudante informado, se existir.
    pub async fn current_active_context(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ActiveCaContext>, ActiveContextError> {
        self.session_store.get_active_context(user_id).await
    }

    /// Atribui um cargo de gestão a um usuário para fins de avaliação de privilégios e isolamento.
    pub async fn assign_management_role(&self, assignment: ManagementRoleAssignment) {
        let mut lock = self.role_assignments.write().await;
        lock.push(assignment);
    }

    /// Verifica se o estudante possui a permissão administrativa solicitada no Centro Acadêmico ativo.
    pub async fn has_active_permission(&self, user_id: Uuid, permission: &str) -> bool {
        let Ok(Some(context)) = self.session_store.get_active_context(user_id).await else {
            return false;
        };
        let lock = self.role_assignments.read().await;
        has_administrative_permission_in_active_context(&context, &lock, permission)
    }

    /// Confere se o estudante atual atua estritamente como visitante no contexto ativo (sem permissões de gestão).
    pub async fn is_visitor_in_active_context(&self, user_id: Uuid) -> bool {
        let Ok(Some(context)) = self.session_store.get_active_context(user_id).await else {
            return true;
        };
        let lock = self.role_assignments.read().await;
        is_visitor_in_active_context(&context, &lock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ca::domain::entities::academic_center::CACC_DIDACTIC_ID;

    const VETERANO_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[tokio::test]
    async fn should_open_app_and_initialize_context_from_favorite() {
        let service = ActiveContextService::with_didactic_setup();

        // No setup didático, VETERANO_ID tem CAER como favorito
        let context = service.open_app(VETERANO_ID).await.expect("open app");
        assert_eq!(context.active_ca_id(), CAER_DIDACTIC_ID);
    }

    #[tokio::test]
    async fn should_switch_active_ca_without_mutating_persisted_favorite() {
        let service = ActiveContextService::with_didactic_setup();

        // 1. Abre o app -> contexto ativo inicial em CAER (favorito prévio)
        service.open_app(VETERANO_ID).await.expect("open app");

        // 2. Alterna o contexto ativo em sessão para CACC
        let switched = service
            .switch_active_ca(VETERANO_ID, CACC_DIDACTIC_ID)
            .await
            .expect("switch to cacc");
        assert_eq!(switched.active_ca_id(), CACC_DIDACTIC_ID);

        // 3. Verifica que a preferência salva no repositório PERMANECE CAER
        let saved_pref = service
            .preference_repo
            .find_by_user_id(VETERANO_ID)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved_pref.favorite_ca_id(), Some(CAER_DIDACTIC_ID));
    }

    #[tokio::test]
    async fn should_isolate_permissions_on_active_context_switch() {
        let service = ActiveContextService::with_didactic_setup();

        // VETERANO_ID tem cargo em CAER de Diretor de Comunicação
        service
            .assign_management_role(
                ManagementRoleAssignment::new(
                    VETERANO_ID,
                    CAER_DIDACTIC_ID,
                    "Diretor de Comunicação",
                    vec!["Publicar notícias diretamente".to_string()],
                )
                .unwrap(),
            )
            .await;

        // Contexto ativo em CAER: possui a permissão
        service.open_app(VETERANO_ID).await.unwrap();
        assert!(
            service
                .has_active_permission(VETERANO_ID, "Publicar notícias diretamente")
                .await
        );
        assert!(!service.is_visitor_in_active_context(VETERANO_ID).await);

        // Alterna contexto ativo para CACC: NÃO possui a permissão e é visitante
        service
            .switch_active_ca(VETERANO_ID, CACC_DIDACTIC_ID)
            .await
            .unwrap();
        assert!(
            !service
                .has_active_permission(VETERANO_ID, "Publicar notícias diretamente")
                .await
        );
        assert!(service.is_visitor_in_active_context(VETERANO_ID).await);
    }
}
