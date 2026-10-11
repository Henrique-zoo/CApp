//! Repositórios e adaptadores de persistência para preferências do usuário.
//!
//! Fornece o contrato abstrato [`UserPreferenceRepository`], a implementação
//! em memória [`InMemoryUserPreferenceRepository`] para testes rápidos, e
//! a implementação PostgreSQL [`PgUserPreferenceRepository`] baseada em SQLx.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::identity::domain::entities::user_preference::{
    UserPreference, UserPreferenceError,
};

/// Estrutura auxiliar de mapeamento de tupla SQLx para `user_preference`.
#[derive(Debug, sqlx::FromRow)]
struct UserPreferenceDbRow {
    user_id: Uuid,
    favorite_ca_id: Option<Uuid>,
    updated_at: DateTime<Utc>,
}

/// Erros decorrentes de operações no repositório de preferências de usuário.
#[derive(Debug, thiserror::Error)]
pub enum UserPreferenceRepositoryError {
    /// Falha na comunicação com o banco de dados ou execução de comando SQLx.
    #[error("falha na camada de persistência: {0}")]
    Database(#[from] sqlx::Error),

    /// Falha de integridade ou validação da entidade de domínio.
    #[error("violação de integridade da entidade: {0}")]
    Domain(#[from] UserPreferenceError),
}

/// Contrato assíncrono para operações de persistência de preferências do usuário.
#[async_trait]
pub trait UserPreferenceRepository: Send + Sync {
    /// Localiza a preferência persistida para o usuário informado.
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserPreference>, UserPreferenceRepositoryError>;

    /// Salva ou atualiza a preferência do usuário (upsert idempotente).
    async fn save(&self, preference: &UserPreference) -> Result<(), UserPreferenceRepositoryError>;

    /// Remove a preferência associada ao usuário, retornando se havia registro anterior.
    async fn delete(&self, user_id: Uuid) -> Result<bool, UserPreferenceRepositoryError>;
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
        let veterano_id = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
        let calouro_id = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0013);

        let mut map = HashMap::new();
        if let Ok(p1) = UserPreference::new(veterano_id, Some(caer_id)) {
            map.insert(veterano_id, p1);
        }
        if let Ok(p2) = UserPreference::new(calouro_id, Some(cacc_id)) {
            map.insert(calouro_id, p2);
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
    ) -> Result<Option<UserPreference>, UserPreferenceRepositoryError> {
        let lock = self.storage.read().await;
        Ok(lock.get(&user_id).cloned())
    }

    async fn save(&self, preference: &UserPreference) -> Result<(), UserPreferenceRepositoryError> {
        let mut lock = self.storage.write().await;
        lock.insert(preference.user_id(), preference.clone());
        Ok(())
    }

    async fn delete(&self, user_id: Uuid) -> Result<bool, UserPreferenceRepositoryError> {
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

#[async_trait]
impl UserPreferenceRepository for PgUserPreferenceRepository {
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserPreference>, UserPreferenceRepositoryError> {
        let row = sqlx::query_as::<_, UserPreferenceDbRow>(
            "SELECT user_id, favorite_ca_id, updated_at FROM user_preference WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| UserPreference::with_timestamp(r.user_id, r.favorite_ca_id, r.updated_at))
            .transpose()
            .map_err(UserPreferenceRepositoryError::Domain)
    }

    async fn save(&self, preference: &UserPreference) -> Result<(), UserPreferenceRepositoryError> {
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

    async fn delete(&self, user_id: Uuid) -> Result<bool, UserPreferenceRepositoryError> {
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
    const CACC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);
    const VETERANO_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
    const CALOURO_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0013);

    #[tokio::test]
    async fn should_persist_and_retrieve_preference_in_memory() {
        let repo = InMemoryUserPreferenceRepository::new();

        assert_eq!(repo.find_by_user_id(VETERANO_ID).await.unwrap(), None);

        let preference = UserPreference::new(VETERANO_ID, Some(CAER_ID)).expect("valid preference");
        repo.save(&preference)
            .await
            .expect("should save preference");

        let found = repo
            .find_by_user_id(VETERANO_ID)
            .await
            .expect("should find preference")
            .expect("must be Some");

        assert_eq!(found.user_id(), VETERANO_ID);
        assert_eq!(found.favorite_ca_id(), Some(CAER_ID));
        assert!(found.is_favorite(&CAER_ID));
    }

    #[tokio::test]
    async fn should_update_existing_preference_in_memory() {
        let repo = InMemoryUserPreferenceRepository::new();

        let mut preference =
            UserPreference::new(VETERANO_ID, Some(CAER_ID)).expect("valid preference");
        repo.save(&preference).await.expect("save initial");

        preference.set_favorite_ca(CACC_ID).expect("update to CACC");
        repo.save(&preference).await.expect("save update");

        let updated = repo
            .find_by_user_id(VETERANO_ID)
            .await
            .unwrap()
            .expect("must exist");
        assert_eq!(updated.favorite_ca_id(), Some(CACC_ID));
        assert!(updated.is_favorite(&CACC_ID));
    }

    #[tokio::test]
    async fn should_delete_preference_in_memory() {
        let repo = InMemoryUserPreferenceRepository::new();
        let preference = UserPreference::new(VETERANO_ID, Some(CAER_ID)).expect("valid preference");
        repo.save(&preference).await.expect("save");

        let deleted = repo.delete(VETERANO_ID).await.expect("delete");
        assert!(deleted);

        let not_found = repo.find_by_user_id(VETERANO_ID).await.expect("search");
        assert_eq!(not_found, None);

        let delete_again = repo.delete(VETERANO_ID).await.expect("delete again");
        assert!(!delete_again);
    }

    #[tokio::test]
    async fn should_support_didactic_data_repository() {
        let repo = InMemoryUserPreferenceRepository::with_didactic_data();

        let veterano_pref = repo
            .find_by_user_id(VETERANO_ID)
            .await
            .unwrap()
            .expect("veterano preference");
        assert_eq!(veterano_pref.favorite_ca_id(), Some(CAER_ID));
        assert!(veterano_pref.is_favorite(&CAER_ID));

        let calouro_pref = repo
            .find_by_user_id(CALOURO_ID)
            .await
            .unwrap()
            .expect("calouro preference");
        assert_eq!(calouro_pref.favorite_ca_id(), Some(CACC_ID));
        assert!(calouro_pref.is_favorite(&CACC_ID));
    }
}
