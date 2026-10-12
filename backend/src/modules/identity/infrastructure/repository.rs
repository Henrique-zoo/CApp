//! Repositórios e contratos de persistência do módulo de identidade institucional.
//!
//! Fornece traits desacopladas e implementações em memória e PostgreSQL para:
//! - [`AppUserRepository`]: Usuários internos da aplicação (`app_user`).
//! - [`ExternalIdentityRepository`]: Identidades externas autenticadas (`external_identity`).
//! - [`UserPreferenceRepository`]: Preferências de navegação do usuário (`user_preference`).

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::identity::domain::{
    CALOURO_DIDACTIC_ID, VETERANO_1_DIDACTIC_ID, VETERANO_2_DIDACTIC_ID,
    entities::{
        AppUser, AppUserError, AppUserStatus, ExternalIdentity, ExternalIdentityError,
        UserPreference, UserPreferenceError,
    },
};

/// Erros de persistência e repositório de identidade institucional.
#[derive(Debug, Error)]
pub enum IdentityRepositoryError {
    /// Violação de invariante da entidade de usuário.
    #[error("erro de domínio de usuário: {0}")]
    DomainUser(#[from] AppUserError),

    /// Violação de invariante da entidade de identidade externa.
    #[error("erro de domínio de identidade externa: {0}")]
    DomainExternalIdentity(#[from] ExternalIdentityError),

    /// Violação de invariante da entidade de preferência de usuário.
    #[error("erro de domínio de preferência: {0}")]
    DomainPreference(#[from] UserPreferenceError),

    /// Falha na execução da consulta ou conexão com o banco de dados.
    #[error("erro no banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    /// Tentativa de cadastrar e-mail institucional já existente.
    #[error("o e-mail institucional '{0}' já está em uso")]
    DuplicateEmail(String),

    /// Tentativa de cadastrar Firebase UID já existente para outro usuário.
    #[error("o Firebase UID '{0}' já está associado a outro registro")]
    DuplicateFirebaseUid(String),

    /// Tentativa de cadastrar Microsoft Subject já existente para outro usuário.
    #[error("o Microsoft Subject '{0}' já está associado a outro registro")]
    DuplicateMicrosoftSubject(String),
}

// ============================================================================
// 1. AppUserRepository
// ============================================================================

/// Contrato de persistência para a entidade [`AppUser`].
#[async_trait]
pub trait AppUserRepository: Send + Sync {
    /// Localiza o usuário pelo seu identificador estável (`id`).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AppUser>, IdentityRepositoryError>;

    /// Localiza o usuário pelo seu endereço de e-mail institucional.
    async fn find_by_email(&self, email: &str) -> Result<Option<AppUser>, IdentityRepositoryError>;

    /// Salva ou atualiza os dados do usuário.
    async fn save(&self, user: &AppUser) -> Result<(), IdentityRepositoryError>;

    /// Retorna a quantidade total de usuários cadastrados no repositório.
    async fn count(&self) -> Result<usize, IdentityRepositoryError>;
}

/// Implementação em memória de [`AppUserRepository`] para testes e isolamento.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAppUserRepository {
    storage: Arc<RwLock<HashMap<Uuid, AppUser>>>,
    email_index: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl InMemoryAppUserRepository {
    /// Constrói um novo repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Constrói um repositório pré-populado com os usuários didáticos da UnB.
    pub fn with_didactic_data() -> Self {
        let veterano1 = AppUser::veterano_didactic();
        let veterano2 = {
            let ref_time = DateTime::from_timestamp(1_767_225_600, 0).unwrap_or_default();
            AppUser {
                id: VETERANO_2_DIDACTIC_ID,
                display_name: "Estudante Veterano 2 UnB".to_string(),
                institutional_email: "241098765@aluno.unb.br".to_string(),
                status: AppUserStatus::Active,
                created_at: ref_time,
                updated_at: ref_time,
            }
        };

        let mut store = HashMap::new();
        let mut idx = HashMap::new();

        idx.insert(veterano1.institutional_email.clone(), veterano1.id);
        store.insert(veterano1.id, veterano1);

        idx.insert(veterano2.institutional_email.clone(), veterano2.id);
        store.insert(veterano2.id, veterano2);

        Self {
            storage: Arc::new(RwLock::new(store)),
            email_index: Arc::new(RwLock::new(idx)),
        }
    }
}

#[async_trait]
impl AppUserRepository for InMemoryAppUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AppUser>, IdentityRepositoryError> {
        let lock = self.storage.read().await;
        Ok(lock.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<AppUser>, IdentityRepositoryError> {
        let normalized = email.trim().to_lowercase();
        let idx = self.email_index.read().await;
        if let Some(user_id) = idx.get(&normalized) {
            let lock = self.storage.read().await;
            Ok(lock.get(user_id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn save(&self, user: &AppUser) -> Result<(), IdentityRepositoryError> {
        let normalized_email = user.institutional_email.trim().to_lowercase();
        let mut idx = self.email_index.write().await;
        let mut store = self.storage.write().await;

        if let Some(existing_id) = idx.get(&normalized_email)
            && *existing_id != user.id
        {
            return Err(IdentityRepositoryError::DuplicateEmail(normalized_email));
        }

        idx.insert(normalized_email, user.id);
        store.insert(user.id, user.clone());
        Ok(())
    }

    async fn count(&self) -> Result<usize, IdentityRepositoryError> {
        let lock = self.storage.read().await;
        Ok(lock.len())
    }
}

/// Implementação PostgreSQL de [`AppUserRepository`] utilizando SQLx.
#[derive(Debug, Clone)]
pub struct PgAppUserRepository {
    pool: sqlx::PgPool,
}

impl PgAppUserRepository {
    /// Constrói uma nova instância vinculada ao pool PostgreSQL.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AppUserDbRow {
    id: Uuid,
    display_name: String,
    institutional_email: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[async_trait]
impl AppUserRepository for PgAppUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AppUser>, IdentityRepositoryError> {
        let row = sqlx::query_as::<_, AppUserDbRow>(
            "SELECT id, display_name, institutional_email, status, created_at, updated_at FROM app_user WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            let status: AppUserStatus = r.status.parse()?;
            AppUser::new(
                r.id,
                r.display_name,
                r.institutional_email,
                status,
                r.created_at,
                r.updated_at,
            )
            .map_err(IdentityRepositoryError::DomainUser)
        })
        .transpose()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<AppUser>, IdentityRepositoryError> {
        let normalized = email.trim().to_lowercase();
        let row = sqlx::query_as::<_, AppUserDbRow>(
            "SELECT id, display_name, institutional_email, status, created_at, updated_at FROM app_user WHERE LOWER(institutional_email) = $1",
        )
        .bind(normalized)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            let status: AppUserStatus = r.status.parse()?;
            AppUser::new(
                r.id,
                r.display_name,
                r.institutional_email,
                status,
                r.created_at,
                r.updated_at,
            )
            .map_err(IdentityRepositoryError::DomainUser)
        })
        .transpose()
    }

    async fn save(&self, user: &AppUser) -> Result<(), IdentityRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO app_user (id, display_name, institutional_email, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET display_name = EXCLUDED.display_name,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(user.id)
        .bind(&user.display_name)
        .bind(&user.institutional_email)
        .bind(user.status.as_str())
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn count(&self) -> Result<usize, IdentityRepositoryError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_user")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 as usize)
    }
}

// ============================================================================
// 2. ExternalIdentityRepository
// ============================================================================

/// Contrato de persistência para a entidade [`ExternalIdentity`].
#[async_trait]
pub trait ExternalIdentityRepository: Send + Sync {
    /// Localiza a identidade externa pelo seu identificador estável (`id`).
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError>;

    /// Localiza a identidade externa pelo seu Firebase UID estável.
    async fn find_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError>;

    /// Localiza a identidade externa pelo Microsoft Subject e instituição opcional.
    async fn find_by_microsoft_subject(
        &self,
        subject: &str,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError>;

    /// Localiza a identidade externa associada a um usuário interno (`user_id`).
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError>;

    /// Salva ou atualiza a identidade externa unívoca.
    async fn save(&self, identity: &ExternalIdentity) -> Result<(), IdentityRepositoryError>;

    /// Atualiza o instante do último login registrado para a identidade.
    async fn update_last_login(
        &self,
        id: Uuid,
        last_login_at: DateTime<Utc>,
    ) -> Result<(), IdentityRepositoryError>;

    /// Retorna a contagem total de identidades externas registradas.
    async fn count(&self) -> Result<usize, IdentityRepositoryError>;
}

/// Implementação em memória de [`ExternalIdentityRepository`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryExternalIdentityRepository {
    storage: Arc<RwLock<HashMap<Uuid, ExternalIdentity>>>,
    firebase_index: Arc<RwLock<HashMap<String, Uuid>>>,
    subject_index: Arc<RwLock<HashMap<String, Uuid>>>,
    user_index: Arc<RwLock<HashMap<Uuid, Uuid>>>,
}

impl InMemoryExternalIdentityRepository {
    /// Constrói um novo repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            firebase_index: Arc::new(RwLock::new(HashMap::new())),
            subject_index: Arc::new(RwLock::new(HashMap::new())),
            user_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Constrói um repositório pré-populado com os dados didáticos da UnB.
    pub fn with_didactic_data() -> Self {
        let veterano1 = ExternalIdentity::veterano_didactic(VETERANO_1_DIDACTIC_ID);
        let veterano2 = {
            let ref_time = DateTime::from_timestamp(1_767_225_600, 0);
            ExternalIdentity {
                id: Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0022),
                user_id: VETERANO_2_DIDACTIC_ID,
                institution_id: None,
                firebase_uid: "firebase-uid-veterano-2".to_string(),
                microsoft_subject: "sub-unb-241098765@aluno.unb.br".to_string(),
                last_login_at: ref_time,
            }
        };

        let mut store = HashMap::new();
        let mut f_idx = HashMap::new();
        let mut s_idx = HashMap::new();
        let mut u_idx = HashMap::new();

        f_idx.insert(veterano1.firebase_uid.clone(), veterano1.id);
        s_idx.insert(veterano1.microsoft_subject.clone(), veterano1.id);
        u_idx.insert(veterano1.user_id, veterano1.id);
        store.insert(veterano1.id, veterano1);

        f_idx.insert(veterano2.firebase_uid.clone(), veterano2.id);
        s_idx.insert(veterano2.microsoft_subject.clone(), veterano2.id);
        u_idx.insert(veterano2.user_id, veterano2.id);
        store.insert(veterano2.id, veterano2);

        Self {
            storage: Arc::new(RwLock::new(store)),
            firebase_index: Arc::new(RwLock::new(f_idx)),
            subject_index: Arc::new(RwLock::new(s_idx)),
            user_index: Arc::new(RwLock::new(u_idx)),
        }
    }
}

#[async_trait]
impl ExternalIdentityRepository for InMemoryExternalIdentityRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let lock = self.storage.read().await;
        Ok(lock.get(&id).cloned())
    }

    async fn find_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let f_idx = self.firebase_index.read().await;
        if let Some(id) = f_idx.get(firebase_uid.trim()) {
            let store = self.storage.read().await;
            Ok(store.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_microsoft_subject(
        &self,
        subject: &str,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let s_idx = self.subject_index.read().await;
        if let Some(id) = s_idx.get(subject.trim()) {
            let store = self.storage.read().await;
            Ok(store.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let u_idx = self.user_index.read().await;
        if let Some(id) = u_idx.get(&user_id) {
            let store = self.storage.read().await;
            Ok(store.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn save(&self, identity: &ExternalIdentity) -> Result<(), IdentityRepositoryError> {
        let f_uid = identity.firebase_uid.trim().to_string();
        let sub = identity.microsoft_subject.trim().to_string();

        let mut f_idx = self.firebase_index.write().await;
        let mut s_idx = self.subject_index.write().await;
        let mut u_idx = self.user_index.write().await;
        let mut store = self.storage.write().await;

        if let Some(existing_id) = f_idx.get(&f_uid)
            && *existing_id != identity.id
        {
            return Err(IdentityRepositoryError::DuplicateFirebaseUid(f_uid));
        }
        if let Some(existing_id) = s_idx.get(&sub)
            && *existing_id != identity.id
        {
            return Err(IdentityRepositoryError::DuplicateMicrosoftSubject(sub));
        }

        f_idx.insert(f_uid, identity.id);
        s_idx.insert(sub, identity.id);
        u_idx.insert(identity.user_id, identity.id);
        store.insert(identity.id, identity.clone());

        Ok(())
    }

    async fn update_last_login(
        &self,
        id: Uuid,
        last_login_at: DateTime<Utc>,
    ) -> Result<(), IdentityRepositoryError> {
        let mut store = self.storage.write().await;
        if let Some(identity) = store.get_mut(&id) {
            identity.last_login_at = Some(last_login_at);
        }
        Ok(())
    }

    async fn count(&self) -> Result<usize, IdentityRepositoryError> {
        let lock = self.storage.read().await;
        Ok(lock.len())
    }
}

/// Implementação PostgreSQL de [`ExternalIdentityRepository`].
#[derive(Debug, Clone)]
pub struct PgExternalIdentityRepository {
    pool: sqlx::PgPool,
}

impl PgExternalIdentityRepository {
    /// Constrói uma nova instância vinculada ao pool PostgreSQL.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ExternalIdentityDbRow {
    id: Uuid,
    user_id: Uuid,
    institution_id: Option<Uuid>,
    firebase_uid: String,
    microsoft_subject: String,
    last_login_at: Option<DateTime<Utc>>,
}

#[async_trait]
impl ExternalIdentityRepository for PgExternalIdentityRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let row = sqlx::query_as::<_, ExternalIdentityDbRow>(
            "SELECT id, user_id, institution_id, firebase_uid, microsoft_subject, last_login_at FROM external_identity WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            ExternalIdentity::new(
                r.id,
                r.user_id,
                r.institution_id,
                r.firebase_uid,
                r.microsoft_subject,
                r.last_login_at,
            )
            .map_err(IdentityRepositoryError::DomainExternalIdentity)
        })
        .transpose()
    }

    async fn find_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let row = sqlx::query_as::<_, ExternalIdentityDbRow>(
            "SELECT id, user_id, institution_id, firebase_uid, microsoft_subject, last_login_at FROM external_identity WHERE firebase_uid = $1",
        )
        .bind(firebase_uid.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            ExternalIdentity::new(
                r.id,
                r.user_id,
                r.institution_id,
                r.firebase_uid,
                r.microsoft_subject,
                r.last_login_at,
            )
            .map_err(IdentityRepositoryError::DomainExternalIdentity)
        })
        .transpose()
    }

    async fn find_by_microsoft_subject(
        &self,
        subject: &str,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let row = sqlx::query_as::<_, ExternalIdentityDbRow>(
            "SELECT id, user_id, institution_id, firebase_uid, microsoft_subject, last_login_at FROM external_identity WHERE microsoft_subject = $1",
        )
        .bind(subject.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            ExternalIdentity::new(
                r.id,
                r.user_id,
                r.institution_id,
                r.firebase_uid,
                r.microsoft_subject,
                r.last_login_at,
            )
            .map_err(IdentityRepositoryError::DomainExternalIdentity)
        })
        .transpose()
    }

    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ExternalIdentity>, IdentityRepositoryError> {
        let row = sqlx::query_as::<_, ExternalIdentityDbRow>(
            "SELECT id, user_id, institution_id, firebase_uid, microsoft_subject, last_login_at FROM external_identity WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            ExternalIdentity::new(
                r.id,
                r.user_id,
                r.institution_id,
                r.firebase_uid,
                r.microsoft_subject,
                r.last_login_at,
            )
            .map_err(IdentityRepositoryError::DomainExternalIdentity)
        })
        .transpose()
    }

    async fn save(&self, identity: &ExternalIdentity) -> Result<(), IdentityRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO external_identity (id, user_id, institution_id, firebase_uid, microsoft_subject, last_login_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET last_login_at = EXCLUDED.last_login_at
            "#,
        )
        .bind(identity.id)
        .bind(identity.user_id)
        .bind(identity.institution_id)
        .bind(&identity.firebase_uid)
        .bind(&identity.microsoft_subject)
        .bind(identity.last_login_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_last_login(
        &self,
        id: Uuid,
        last_login_at: DateTime<Utc>,
    ) -> Result<(), IdentityRepositoryError> {
        sqlx::query("UPDATE external_identity SET last_login_at = $1 WHERE id = $2")
            .bind(last_login_at)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn count(&self) -> Result<usize, IdentityRepositoryError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM external_identity")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 as usize)
    }
}

// ============================================================================
// 3. UserPreferenceRepository
// ============================================================================

/// Contrato de persistência para a entidade [`UserPreference`].
#[async_trait]
pub trait UserPreferenceRepository: Send + Sync {
    /// Localiza a preferência persistida para o usuário informado.
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserPreference>, IdentityRepositoryError>;

    /// Salva ou atualiza a preferência do usuário (upsert idempotente).
    async fn save(&self, preference: &UserPreference) -> Result<(), IdentityRepositoryError>;

    /// Remove a preferência associada ao usuário, retornando se havia registro anterior.
    async fn delete(&self, user_id: Uuid) -> Result<bool, IdentityRepositoryError>;
}

/// Implementação em memória de [`UserPreferenceRepository`] para testes e isolamento.
#[derive(Debug, Clone, Default)]
pub struct InMemoryUserPreferenceRepository {
    storage: Arc<RwLock<HashMap<Uuid, UserPreference>>>,
}

impl InMemoryUserPreferenceRepository {
    /// Constrói um novo repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Constrói um repositório pré-populado com os dados didáticos da UnB (CAER / CACC).
    pub fn with_didactic_data() -> Self {
        let caer_id = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
        let cacc_id = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);

        let mut map = HashMap::new();
        if let Ok(p1) = UserPreference::new(VETERANO_1_DIDACTIC_ID, Some(caer_id)) {
            map.insert(VETERANO_1_DIDACTIC_ID, p1);
        }
        if let Ok(p2) = UserPreference::new(CALOURO_DIDACTIC_ID, Some(cacc_id)) {
            map.insert(CALOURO_DIDACTIC_ID, p2);
        }

        Self {
            storage: Arc::new(RwLock::new(map)),
        }
    }
}

#[async_trait]
impl UserPreferenceRepository for InMemoryUserPreferenceRepository {
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserPreference>, IdentityRepositoryError> {
        let lock = self.storage.read().await;
        Ok(lock.get(&user_id).cloned())
    }

    async fn save(&self, preference: &UserPreference) -> Result<(), IdentityRepositoryError> {
        let mut lock = self.storage.write().await;
        lock.insert(preference.user_id(), preference.clone());
        Ok(())
    }

    async fn delete(&self, user_id: Uuid) -> Result<bool, IdentityRepositoryError> {
        let mut lock = self.storage.write().await;
        Ok(lock.remove(&user_id).is_some())
    }
}

/// Implementação PostgreSQL de [`UserPreferenceRepository`] utilizando SQLx.
#[derive(Debug, Clone)]
pub struct PgUserPreferenceRepository {
    pool: sqlx::PgPool,
}

impl PgUserPreferenceRepository {
    /// Cria uma nova instância vinculada ao pool de conexões PostgreSQL.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Retorna uma referência ao pool configurado.
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[derive(sqlx::FromRow)]
struct UserPreferenceDbRow {
    user_id: Uuid,
    favorite_ca_id: Option<Uuid>,
    updated_at: DateTime<Utc>,
}

#[async_trait]
impl UserPreferenceRepository for PgUserPreferenceRepository {
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserPreference>, IdentityRepositoryError> {
        let row = sqlx::query_as::<_, UserPreferenceDbRow>(
            "SELECT user_id, favorite_ca_id, updated_at FROM user_preference WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| UserPreference::with_timestamp(r.user_id, r.favorite_ca_id, r.updated_at))
            .transpose()
            .map_err(IdentityRepositoryError::DomainPreference)
    }

    async fn save(&self, preference: &UserPreference) -> Result<(), IdentityRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO user_preference (user_id, favorite_ca_id, updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) DO UPDATE
            SET favorite_ca_id = EXCLUDED.favorite_ca_id,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(preference.user_id())
        .bind(preference.favorite_ca_id())
        .bind(preference.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, user_id: Uuid) -> Result<bool, IdentityRepositoryError> {
        let result = sqlx::query("DELETE FROM user_preference WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[tokio::test]
    async fn should_manage_users_in_memory() {
        let repo = InMemoryAppUserRepository::new();
        assert_eq!(repo.count().await.unwrap(), 0);

        let user = AppUser::calouro_didactic();
        repo.save(&user).await.expect("save calouro");

        assert_eq!(repo.count().await.unwrap(), 1);
        let by_id = repo.find_by_id(user.id).await.unwrap().expect("found");
        assert_eq!(by_id.institutional_email, user.institutional_email);

        let by_email = repo
            .find_by_email(&user.institutional_email)
            .await
            .unwrap()
            .expect("found by email");
        assert_eq!(by_email.id, user.id);
    }

    #[tokio::test]
    async fn should_manage_external_identities_in_memory() {
        let repo = InMemoryExternalIdentityRepository::new();
        assert_eq!(repo.count().await.unwrap(), 0);

        let identity = ExternalIdentity::calouro_didactic(USER_ID);
        repo.save(&identity).await.expect("save calouro identity");

        assert_eq!(repo.count().await.unwrap(), 1);
        let by_f_uid = repo
            .find_by_firebase_uid(identity.firebase_uid())
            .await
            .unwrap()
            .expect("found by uid");
        assert_eq!(by_f_uid.user_id(), USER_ID);

        let by_sub = repo
            .find_by_microsoft_subject(identity.microsoft_subject())
            .await
            .unwrap()
            .expect("found by sub");
        assert_eq!(by_sub.id(), identity.id());

        let new_time = Utc::now();
        repo.update_last_login(identity.id(), new_time)
            .await
            .unwrap();
        let updated = repo.find_by_id(identity.id()).await.unwrap().unwrap();
        assert_eq!(updated.last_login_at(), Some(new_time));
    }

    #[tokio::test]
    async fn should_manage_user_preferences_in_memory() {
        let repo = InMemoryUserPreferenceRepository::new();
        assert_eq!(repo.find_by_user_id(USER_ID).await.unwrap(), None);

        let pref = UserPreference::default_for_user(USER_ID).unwrap();
        repo.save(&pref).await.expect("save default pref");

        let found = repo.find_by_user_id(USER_ID).await.unwrap().expect("found");
        assert_eq!(found.favorite_ca_id(), None);

        let mut updated_pref = found;
        updated_pref.set_favorite_ca(CAER_ID).unwrap();
        repo.save(&updated_pref).await.expect("save updated pref");

        let found_updated = repo.find_by_user_id(USER_ID).await.unwrap().unwrap();
        assert_eq!(found_updated.favorite_ca_id(), Some(CAER_ID));
    }
}
