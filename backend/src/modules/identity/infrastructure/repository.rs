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

use uuid::Uuid;

use crate::{
    infrastructure::database::PgDatabase,
    modules::identity::{
        application::{
            error::{
                InstitutionRepositoryError, InstitutionalVerificationRepositoryError,
                UserRepositoryError,
            },
            ports::{InstitutionRepository, InstitutionalVerificationRepository, UserRepository},
        },
        domain::entities::{
            institution::Institution, institutional_verification::InstitutionalVerification,
            user::User,
        },
    },
};

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
        Ok(sqlx::query_as(
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
        })?)
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
        Ok(sqlx::query_as::<_, InstitutionalVerification>(
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
        })?)
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
                expires_at, profile_name, profile_email, course_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
                profile_email = EXCLUDED.profile_email,
                course_name = EXCLUDED.course_name
            WHERE EXCLUDED.checked_at > institutional_verification.checked_at
            "#,
        )
        .bind(verification.user_id)
        .bind(verification.microsoft_object_id)
        .bind(verification.institution_id)
        .bind(verification.microsoft_tenant_id)
        .bind(verification.microsoft_student_group_id)
        .bind(verification.student_membership_confirmed)
        .bind(verification.checked_at)
        .bind(verification.expires_at)
        .bind(&verification.profile_name)
        .bind(&verification.profile_email)
        .bind(&verification.course_name)
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
    async fn update_profile(&self, user: &User) -> Result<bool, UserRepositoryError> {
        sqlx::query(
            r#"
            UPDATE app_user SET
                nickname = $2, email = $3, current_semester = $4,
                institution_entry_semester = $5, status = 'active',
                profile_completed_at = COALESCE(profile_completed_at, $6),
                updated_at = GREATEST(updated_at, $7),
                name = $9, institutional_email = $10, course_name = $11
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
        .bind(user.institutional_email())
        .bind(user.course_name())
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(|error| {
            tracing::error!(%error, "Falha na conclusão do perfil");
            UserRepositoryError::InternalServer
        })
    }

    async fn provision(&self, firebase_uid: &str) -> Result<User, UserRepositoryError> {
        sqlx::query_as(
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
        })
    }

    async fn get_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<User>, UserRepositoryError> {
        sqlx::query_as("SELECT * FROM app_user WHERE firebase_uid = $1")
            .bind(firebase_uid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                tracing::error!(%error, "Falha na consulta do usuário");
                UserRepositoryError::InternalServer
            })
    }
}
