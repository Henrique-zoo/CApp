//! Repositórios de identidade implementados sobre o adaptador PostgreSQL comum.
//!
//! [`PgDatabase`] resolve o UID autenticado para a conta interna e guarda a última
//! evidência por `user_id`. Provisionamento usa UPSERT com unicidade do UID e não
//! sobrescreve perfil ou status. Salvar evidência respeita `checked_at`, inclusive
//! negativas, sem manter transação aberta durante chamadas ao Graph.
//!
//! A leitura exige prazo vigente e configuração institucional compatível. Campos
//! de perfil externo são uma fotografia da consulta, nunca credenciais. Falhas
//! SQL são registradas antes da conversão para os contratos de aplicação.

use crate::infrastructure::database::PgDatabase;
use crate::modules::identity::application::error::InstitutionRepositoryError;
use crate::modules::identity::application::error::InstitutionalVerificationRepositoryError;
use crate::modules::identity::application::error::UserRepositoryError;
use crate::modules::identity::application::ports::InstitutionRepository;
use crate::modules::identity::application::ports::InstitutionalVerificationRepository;
use crate::modules::identity::application::ports::UserRepository;
use crate::modules::identity::domain::CatalogStatus;
use crate::modules::identity::domain::UserStatus;
use crate::modules::identity::domain::entities::institution::Institution;
use crate::modules::identity::domain::entities::institutional_verification::InstitutionalVerification;
use crate::modules::identity::domain::entities::user::User;
use crate::modules::identity::domain::entities::user_preference::UserPreference;
use crate::modules::identity::domain::entities::user_preference::UserPreferenceError;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
impl From<sqlx::Error> for InstitutionRepositoryError {
    /// Converte erro SQLx em falha técnica; a consulta registra o diagnóstico antes da conversão.
    fn from(_error: sqlx::Error) -> Self {
        Self::InternalServer
    }
}
#[async_trait::async_trait]
impl InstitutionRepository for PgDatabase {
    /// Consulta `institution` pelo tenant único, sem filtrar o status do catálogo.
    ///
    /// # Retorno
    ///
    /// `Ok(None)` se não houver linha; caso contrário, entidade com configuração do
    /// grupo e enum PostgreSQL desserializado. A ausência não gera `RowNotFound`.
    ///
    /// # Erros
    ///
    /// Falhas de aquisição de conexão, execução ou mapeamento são registradas com
    /// `tracing::error` e convertidas para `InstitutionRepositoryError`.
    async fn find_by_microsoft_tenant_id(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<Institution>, InstitutionRepositoryError> {
        let row = sqlx::query_as::<_, InstitutionRow>(
            "
            SELECT
                id, name, acronym,
                microsoft_tenant_id,
                microsoft_student_group_id,
                status
            FROM institution
            WHERE institution.microsoft_tenant_id = $1::uuid
            ",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|error| {
            tracing::error!(%error, "Falha ao consultar instituição pelo tenant");
        })?;

        row.map(InstitutionRow::into_entity).transpose()
    }
}
impl From<sqlx::Error> for InstitutionalVerificationRepositoryError {
    /// Converte erro SQLx no marcador de falha de persistência da evidência.
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}
#[async_trait::async_trait]
impl InstitutionalVerificationRepository for PgDatabase {
    /// Seleciona evidência vigente e compatível com o cadastro, incluindo não estudantes.
    ///
    /// O filtro por UID usa o índice único de app_user e o JOIN usa o ID da instituição. O prazo
    /// é comparado com `CURRENT_TIMESTAMP` do PostgreSQL. Tenant e grupo armazenados
    /// na evidência devem coincidir com a configuração atual. `institution.status` não
    /// é filtro; comparação do objeto Microsoft e checagem final do prazo cabem ao serviço.
    ///
    /// # Retorno
    ///
    /// Uma evidência ou `None` para ausência, expiração ou configuração
    /// alterada. A consulta não altera registros nem remove evidências expiradas.
    ///
    /// # Erros
    ///
    /// Falhas de consulta ou mapeamento são registradas e convertidas em erro do contrato.
    async fn find_valid_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<InstitutionalVerification>, InstitutionalVerificationRepositoryError> {
        let row = sqlx::query_as::<_, InstitutionalVerificationRow>(
            r#"
            SELECT v.*
            FROM institutional_verification AS v
            JOIN institution AS i ON i.id = v.institution_id
            JOIN app_user AS u ON u.id = v.user_id
            WHERE u.firebase_uid = $1
              AND u.status <> 'inactive'
              AND v.expires_at > CURRENT_TIMESTAMP
              AND v.microsoft_tenant_id = i.microsoft_tenant_id
              AND v.microsoft_student_group_id = i.microsoft_student_group_id
            "#,
        )
        .bind(firebase_uid)
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|error| {
            tracing::error!(
                %error,
                "Falha ao consultar verificação institucional"
            );
        })?;

        row.map(InstitutionalVerificationRow::into_entity)
            .transpose()
    }

    /// Seleciona evidência vigente e compatível com o cadastro por `user_id`.
    ///
    /// O filtro por `user_id` usa a PK de `institutional_verification` e `app_user`.
    /// O prazo é comparado com `CURRENT_TIMESTAMP` do PostgreSQL. Tenant e grupo
    /// devem coincidir com a configuração atual da instituição.
    async fn find_valid_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<InstitutionalVerification>, InstitutionalVerificationRepositoryError> {
        let row = sqlx::query_as::<_, InstitutionalVerificationRow>(
            r#"
            SELECT v.*
            FROM institutional_verification AS v
            JOIN institution AS i ON i.id = v.institution_id
            JOIN app_user AS u ON u.id = v.user_id
            WHERE u.id = $1
              AND u.status <> 'inactive'
              AND v.expires_at > CURRENT_TIMESTAMP
              AND v.microsoft_tenant_id = i.microsoft_tenant_id
              AND v.microsoft_student_group_id = i.microsoft_student_group_id
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|error| {
            tracing::error!(
                %error,
                "Falha ao consultar verificação institucional por user_id"
            );
        })?;

        row.map(InstitutionalVerificationRow::into_entity)
            .transpose()
    }

    /// Insere ou substitui atomicamente a evidência mais recente por usuário interno.
    ///
    /// Usa `ON CONFLICT (user_id) DO UPDATE` condicionado a `checked_at` estritamente
    /// maior que o valor existente. Todos os campos são substituídos, inclusive uma
    /// resposta negativa do grupo. Exige FK para usuário interno e não armazena tokens.
    /// O PostgreSQL aplica FK de instituição, usuário existente e validade posterior à consulta.
    ///
    /// # Retorno
    ///
    /// `true` se uma linha foi inserida/atualizada; `false` se o registro existente tem
    /// instante igual ou posterior. Não há exceção especial para negativa: ela também
    /// respeita a ordenação. A precisão temporal efetiva é a armazenada pelo PostgreSQL.
    ///
    /// # Erros
    ///
    /// Falhas SQL, de conexão ou de restrição são registradas antes da conversão.
    /// O resultado só confirma a escrita no banco, não uma transação com o provedor externo.
    async fn save(
        &self,
        verification: &InstitutionalVerification,
    ) -> Result<bool, InstitutionalVerificationRepositoryError> {
        let result = sqlx::query(
            r#"
            INSERT INTO institutional_verification (
                user_id,
                microsoft_object_id,
                institution_id,
                microsoft_tenant_id,
                microsoft_student_group_id,
                student_membership_confirmed,
                checked_at,
                expires_at, profile_name, course_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id) DO UPDATE SET
                microsoft_object_id = EXCLUDED.microsoft_object_id,
                institution_id = EXCLUDED.institution_id,
                microsoft_tenant_id = EXCLUDED.microsoft_tenant_id,
                microsoft_student_group_id = EXCLUDED.microsoft_student_group_id,
                student_membership_confirmed =
                    EXCLUDED.student_membership_confirmed,
                checked_at = EXCLUDED.checked_at,
                expires_at = EXCLUDED.expires_at,
                profile_name = EXCLUDED.profile_name,
                course_name = EXCLUDED.course_name
            WHERE EXCLUDED.checked_at > institutional_verification.checked_at
            "#,
        )
        .bind(verification.user_id())
        .bind(verification.microsoft_object_id())
        .bind(verification.institution_id())
        .bind(verification.microsoft_tenant_id())
        .bind(verification.microsoft_student_group_id())
        .bind(verification.student_membership_confirmed())
        .bind(verification.checked_at())
        .bind(verification.expires_at())
        .bind(verification.profile_name())
        .bind(verification.course_name())
        .execute(&self.pool)
        .await
        .inspect_err(|error| {
            tracing::error!(
                %error,
                "Falha ao registrar verificação institucional"
            );
        })?;

        Ok(result.rows_affected() == 1)
    }
}
/// O UPSERT preserva o ID e o perfil existente; nunca reativa conta desativada.
#[async_trait::async_trait]
impl UserRepository for PgDatabase {
    /// Grava a fotografia validada do perfil usando ID e UID como condições.
    ///
    /// O serviço deve fornecer um [`User`] concluído. O UPDATE grava seus dados
    /// internos e o estado ativo, mantém a primeira conclusão com `COALESCE` e
    /// impede que `updated_at` retroceda. Identidade e criação não são alteradas.
    /// A condição de estado recusa uma conta desativada depois da leitura do serviço.
    ///
    /// Retorna `true` para uma linha atualizada e `false` se ID/UID não correspondem
    /// a uma conta não desativada. Falhas SQL, de restrição ou conexão são registradas
    /// e convertidas em [`UserRepositoryError::InternalServer`]. Não autentica,
    /// consulta o Graph ou altera a evidência institucional.
    async fn update_profile(&self, user: &User) -> Result<bool, UserRepositoryError> {
        sqlx::query(
            r#"
            UPDATE app_user SET
                nickname = $2, email = $3, current_semester = $4,
                institution_entry_semester = $5, status = 'active',
                profile_completed_at = COALESCE(profile_completed_at, $6),
                updated_at = GREATEST(updated_at, $7),
                name = $9
            WHERE id = $1 AND status <> 'inactive' AND firebase_uid = $8
            "#,
        )
        .bind(user.id())
        .bind(user.nickname())
        .bind(user.email())
        .bind(user.current_semester())
        .bind(user.institution_entry_semester())
        .bind(user.profile_completed_at())
        .bind(user.updated_at())
        .bind(user.firebase_uid())
        .bind(user.name())
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(|error| {
            tracing::error!(%error, "Falha na conclusão do perfil");
            UserRepositoryError::InternalServer
        })
    }

    /// Obtém uma única conta por UID com INSERT/UPSERT atômico.
    ///
    /// `firebase_uid` deve ter sido autenticado pelo serviço. Uma conta nova recebe
    /// UUID aleatório e os valores iniciais definidos na migration. No conflito,
    /// somente `last_login_at` é atualizado pelo relógio PostgreSQL; ID, perfil e
    /// estado são preservados, inclusive para contas inativas. A restrição única
    /// evita duplicação quando primeiros acessos ocorrem concorrentemente.
    ///
    /// Retorna a linha persistida como [`User`]. Falhas SQL ou de mapeamento são
    /// registradas e convertidas em [`UserRepositoryError::InternalServer`].
    /// Não inicia uma sessão Firebase nem cria credenciais.
    async fn provision(&self, firebase_uid: &str) -> Result<User, UserRepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO app_user (id, firebase_uid, last_login_at)
            VALUES ($1, $2, CURRENT_TIMESTAMP)
            ON CONFLICT (firebase_uid) DO UPDATE SET last_login_at = CURRENT_TIMESTAMP
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(firebase_uid)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Falha no provisionamento");
            UserRepositoryError::InternalServer
        })?;

        row.into_entity()
    }

    /// Lê a conta de um UID autenticado, incluindo seu estado atual.
    ///
    /// Retorna `None` se o provisionamento ainda não aconteceu. Não filtra contas
    /// inativas, modifica último login ou consulta a confirmação institucional;
    /// a aplicação decide o acesso a partir da fotografia retornada. Falhas de
    /// consulta/mapeamento são registradas e convertidas em
    /// [`UserRepositoryError::InternalServer`], preservando a distinção de ausência.
    async fn get_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<User>, UserRepositoryError> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM app_user WHERE firebase_uid = $1")
            .bind(firebase_uid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                tracing::error!(%error, "Falha na consulta do usuário");
                UserRepositoryError::InternalServer
            })?;

        row.map(UserRow::into_entity).transpose()
    }
}
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "catalog_status", rename_all = "lowercase")]
enum PgCatalogStatus {
    Active,
    Inactive,
}
impl From<PgCatalogStatus> for CatalogStatus {
    fn from(status: PgCatalogStatus) -> Self {
        match status {
            PgCatalogStatus::Active => CatalogStatus::Active,
            PgCatalogStatus::Inactive => CatalogStatus::Inactive,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
enum PgUserStatus {
    Incomplete,
    Active,
    Inactive,
}
impl From<PgUserStatus> for UserStatus {
    fn from(status: PgUserStatus) -> Self {
        match status {
            PgUserStatus::Incomplete => UserStatus::Incomplete,
            PgUserStatus::Active => UserStatus::Active,
            PgUserStatus::Inactive => UserStatus::Inactive,
        }
    }
}
#[derive(Debug, sqlx::FromRow)]
struct InstitutionRow {
    id: Uuid,
    name: String,
    acronym: Option<String>,
    microsoft_tenant_id: Uuid,
    microsoft_student_group_id: Uuid,
    status: PgCatalogStatus,
}
impl InstitutionRow {
    fn into_entity(self) -> Result<Institution, InstitutionRepositoryError> {
        Institution::reconstitute(
            self.id,
            self.name,
            self.acronym,
            self.microsoft_tenant_id,
            self.microsoft_student_group_id,
            self.status.into(),
        )
        .map_err(|_| InstitutionRepositoryError::InternalServer)
    }
}
#[derive(Debug, sqlx::FromRow)]
struct InstitutionalVerificationRow {
    user_id: Uuid,
    microsoft_object_id: Uuid,
    institution_id: Uuid,
    microsoft_tenant_id: Uuid,
    microsoft_student_group_id: Uuid,
    student_membership_confirmed: bool,
    checked_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    profile_name: Option<String>,
    course_name: Option<String>,
}
impl InstitutionalVerificationRow {
    fn into_entity(
        self,
    ) -> Result<InstitutionalVerification, InstitutionalVerificationRepositoryError> {
        InstitutionalVerification::reconstitute(
            self.user_id,
            self.profile_name,
            self.course_name,
            self.institution_id,
            self.microsoft_object_id,
            self.microsoft_tenant_id,
            self.microsoft_student_group_id,
            self.student_membership_confirmed,
            self.checked_at,
            self.expires_at,
        )
        .map_err(|_| InstitutionalVerificationRepositoryError)
    }
}
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    firebase_uid: String,
    name: Option<String>,
    nickname: Option<String>,
    email: Option<String>,
    current_semester: Option<i32>,
    institution_entry_semester: Option<String>,
    status: PgUserStatus,
    profile_completed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl UserRow {
    fn into_entity(self) -> Result<User, UserRepositoryError> {
        Ok(User::reconstitute(
            self.id,
            self.firebase_uid,
            self.name,
            self.nickname,
            self.email,
            self.current_semester,
            self.institution_entry_semester,
            self.status.into(),
            self.profile_completed_at,
            self.created_at,
            self.updated_at,
            self.last_login_at,
        ))
    }
}
