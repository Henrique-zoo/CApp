//! Repositórios de identidade implementados sobre o adaptador PostgreSQL comum.
//!
//! [`PgDatabase`] implementa busca por tenant e armazenamento de evidência por UID.
//! Consultas usam parâmetros SQLx e `fetch_optional` para ausência esperada; o
//! mapeamento é feito pelos derives das entidades. Falhas SQL são registradas e
//! convertidas para erros técnicos dos contratos, sem expor SQLx à aplicação.
//!
//! A gravação é um único UPSERT, sem manter transação aberta durante chamadas ao
//! Graph. A confirmação não constitui sessão nem provisionamento de usuário.

use uuid::Uuid;

use crate::{
    infrastructure::database::PgDatabase,
    modules::identity::{
        application::{
            error::{InstitutionRepositoryError, InstitutionalVerificationRepositoryError},
            ports::{InstitutionRepository, InstitutionalVerificationRepository},
        },
        domain::entities::{
            institution::Institution, institutional_verification::InstitutionalVerification,
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
        Ok(sqlx::query_as(&format!(
            "
            SELECT
                id, name, acronym,
                microsoft_tenant_id,
                microsoft_student_group_id,
                status
            FROM institution
            WHERE institution.microsoft_tenant_id = $1::uuid
            "
        ))
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
    /// O filtro por UID usa a chave primária e o JOIN usa o ID da instituição. O prazo
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
            WHERE v.firebase_uid = $1
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

    /// Insere ou substitui atomicamente a evidência mais recente por UID Firebase.
    ///
    /// Usa `ON CONFLICT (firebase_uid) DO UPDATE` condicionado a `checked_at` estritamente
    /// maior que o valor existente. Todos os campos são substituídos, inclusive uma
    /// resposta negativa do grupo. Não exige usuário interno e não armazena tokens.
    /// O PostgreSQL aplica FK de instituição, UID não vazio e validade posterior à consulta.
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
                firebase_uid,
                microsoft_object_id,
                institution_id,
                microsoft_tenant_id,
                microsoft_student_group_id,
                student_membership_confirmed,
                checked_at,
                expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (firebase_uid) DO UPDATE SET
                microsoft_object_id = EXCLUDED.microsoft_object_id,
                institution_id = EXCLUDED.institution_id,
                microsoft_tenant_id = EXCLUDED.microsoft_tenant_id,
                microsoft_student_group_id = EXCLUDED.microsoft_student_group_id,
                student_membership_confirmed =
                    EXCLUDED.student_membership_confirmed,
                checked_at = EXCLUDED.checked_at,
                expires_at = EXCLUDED.expires_at
            WHERE EXCLUDED.checked_at > institutional_verification.checked_at
            "#,
        )
        .bind(&verification.firebase_uid)
        .bind(verification.microsoft_object_id)
        .bind(verification.institution_id)
        .bind(verification.microsoft_tenant_id)
        .bind(verification.microsoft_student_group_id)
        .bind(verification.student_membership_confirmed)
        .bind(verification.checked_at)
        .bind(verification.expires_at)
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
