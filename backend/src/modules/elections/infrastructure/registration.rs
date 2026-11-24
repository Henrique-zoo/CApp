//! Persistência do cadastro prévio e leitura transacional da elegibilidade.
//!
//! O cadastro e o voto usam o mesmo bloqueio da eleição. A evidência vigente é
//! lida junto da associação de cursos do CA e permanece estável até o commit.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, Postgres, Transaction};

use crate::infrastructure::database::PgDatabase;
use crate::modules::elections::{
    application::{
        error::RegistrationError,
        ports::{RegistrationTransaction, VoterRegistrationRepository},
        registration::{RegistrationStatus, RegistrationWindow, VoterIdentity},
    },
    domain::identifiers::{AccountId, ElectionId},
};

impl From<sqlx::Error> for RegistrationError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(Box::new(error))
    }
}

/// Transação de gravação do cadastro, vinculada à eleição e à conta.
pub struct PgRegistrationTransaction {
    transaction: Transaction<'static, Postgres>,
    election_id: ElectionId,
    account_id: AccountId,
}

#[async_trait::async_trait]
impl VoterRegistrationRepository for PgDatabase {
    type Transaction = PgRegistrationTransaction;

    async fn status(
        &self,
        election_id: ElectionId,
        account_id: AccountId,
    ) -> Result<RegistrationStatus, RegistrationError> {
        let mut conn = self.pool.acquire().await?;
        registration_status(&mut conn, election_id, account_id).await
    }

    async fn lock_registration(
        &self,
        election_id: ElectionId,
        identity: VoterIdentity,
    ) -> Result<(RegistrationStatus, RegistrationWindow, Self::Transaction), RegistrationError>
    {
        let mut tx = self.pool.begin().await?;
        let (starts_at, closed): (DateTime<Utc>, bool) = sqlx::query_as(
            "SELECT e.starts_at, e.status <> 'published'
             FROM election e JOIN election_board b USING (election_id)
             WHERE e.election_id = $1 FOR UPDATE OF e",
        )
        .bind(election_id.into_uuid())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(RegistrationError::NotFound)?;

        let status = registration_status(&mut tx, election_id, identity.account_id).await?;
        let eligible_until = if status.requires_registration && status.public_key.is_none() {
            eligibility_deadline(&mut tx, election_id, identity).await?
        } else {
            None
        };
        let now = database_now(&mut tx).await?;
        Ok((
            status,
            RegistrationWindow {
                starts_at,
                closed,
                now,
                eligible_until,
            },
            PgRegistrationTransaction {
                transaction: tx,
                election_id,
                account_id: identity.account_id,
            },
        ))
    }
}

#[async_trait::async_trait]
impl RegistrationTransaction for PgRegistrationTransaction {
    async fn save(
        mut self,
        public_key: &[u8],
        registered_at: DateTime<Utc>,
    ) -> Result<(), RegistrationError> {
        sqlx::query(
            "INSERT INTO election_voter_registration
                 (election_id, account_id, public_key, registered_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(self.election_id.into_uuid())
        .bind(self.account_id.into_uuid())
        .bind(public_key)
        .bind(registered_at)
        .execute(&mut *self.transaction)
        .await?;
        self.transaction.commit().await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RegistrationRow {
    requires_registration: bool,
    participated: bool,
    public_key: Option<Vec<u8>>,
}

async fn registration_status(
    conn: &mut PgConnection,
    election_id: ElectionId,
    account_id: AccountId,
) -> Result<RegistrationStatus, RegistrationError> {
    let row: RegistrationRow = sqlx::query_as(
        "SELECT e.requires_voter_registration AS requires_registration,
                v.account_id IS NOT NULL AS participated,
                COALESCE(v.public_key, r.public_key) AS public_key
         FROM election_board b JOIN election e USING (election_id)
         LEFT JOIN election_voter_registration r ON r.election_id = b.election_id AND r.account_id = $2
         LEFT JOIN election_vote v ON v.election_id = b.election_id AND v.account_id = $2
         WHERE b.election_id = $1",
    )
    .bind(election_id.into_uuid())
    .bind(account_id.into_uuid())
    .fetch_optional(conn)
    .await?
    .ok_or(RegistrationError::NotFound)?;

    Ok(RegistrationStatus {
        requires_registration: row.requires_registration,
        account_id: (row.participated || row.public_key.is_some()).then_some(account_id),
        public_key: row.public_key,
        participated: row.participated,
    })
}

/// Obtém o prazo da comprovação do aluno em um curso representado pelo CA
/// da eleição. As linhas usadas pelo cálculo ficam estáveis até o commit.
pub(super) async fn eligibility_deadline(
    tx: &mut Transaction<'_, Postgres>,
    election_id: ElectionId,
    identity: VoterIdentity,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT v.expires_at FROM election e
         JOIN academic_center c ON c.id = e.academic_center_id
         JOIN institution i ON i.id = c.institution_id
         JOIN institutional_verification v ON v.institution_id = i.id AND v.user_id = $2
         JOIN academic_center_course cc ON cc.academic_center_id = c.id AND cc.course_name = btrim(v.course_name)
         JOIN app_user u ON u.id = v.user_id
         WHERE e.election_id = $1 AND c.status = 'active' AND i.status = 'active'
           AND u.status <> 'inactive' AND v.student_membership_confirmed
           AND v.microsoft_object_id = $3 AND v.microsoft_tenant_id = i.microsoft_tenant_id
           AND v.microsoft_student_group_id = i.microsoft_student_group_id
           AND v.checked_at <= clock_timestamp()
         FOR SHARE OF c, i, v, cc, u",
    )
    .bind(election_id.into_uuid())
    .bind(identity.account_id.into_uuid())
    .bind(identity.microsoft_object_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Usa o instante real do banco após adquirir os bloqueios da operação.
pub(super) async fn database_now(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await
}
