//! Serializa alterações da comissão com publicação, voto e encerramento.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, Postgres, Transaction};
use uuid::Uuid;

use crate::infrastructure::database::PgDatabase;
use crate::modules::elections::{
    application::{
        administration::{AdministrationState, BoardCancellation},
        board::BoardError,
        ports::{ElectionAdministrationRepository, ElectionAdministrationTransaction},
    },
    domain::{
        entities::election::ElectionPeriod,
        identifiers::{AccountId, ElectionId},
    },
};

use super::registration::database_now;

pub(super) async fn read_cancellation(
    connection: &mut PgConnection,
    election: Uuid,
) -> Result<Option<BoardCancellation>, sqlx::Error> {
    let row: Option<(DateTime<Utc>, Uuid, String)> = sqlx::query_as(
        "SELECT cancelled_at, cancelled_by, cancellation_reason FROM election
         WHERE election_id = $1 AND status = 'cancelled'",
    )
    .bind(election)
    .fetch_optional(connection)
    .await?;
    Ok(
        row.map(|(cancelled_at, cancelled_by, reason)| BoardCancellation {
            cancelled_at,
            cancelled_by,
            reason,
        }),
    )
}

/// Transação do comando administrativo autorizado.
pub struct PgAdministrationTransaction {
    transaction: Transaction<'static, Postgres>,
    election: ElectionId,
    actor: AccountId,
}

#[async_trait::async_trait]
impl ElectionAdministrationRepository for PgDatabase {
    type Transaction = PgAdministrationTransaction;

    async fn lock_administration(
        &self,
        election: ElectionId,
        actor: AccountId,
    ) -> Result<(AdministrationState, Self::Transaction), BoardError> {
        let id = election.into_uuid();
        let mut transaction = self.pool.begin().await?;
        let (start, end, status): (DateTime<Utc>, DateTime<Utc>, String) = sqlx::query_as(
            "SELECT e.starts_at, e.ends_at, e.status
             FROM election e
             WHERE e.election_id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(BoardError::AuthorityNotAssigned)?;

        let is_commission: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM election_authority
               WHERE election_id = $1 AND account_id = $2 AND role = 'commission')",
        )
        .bind(id)
        .bind(actor.into_uuid())
        .fetch_one(&mut *transaction)
        .await?;
        if !is_commission {
            return Err(BoardError::AuthorityNotAssigned);
        }
        let state = AdministrationState {
            election,
            period: ElectionPeriod::try_new(start, end)
                .map_err(|_| BoardError::InvalidArtifact("período da eleição"))?,
            now: database_now(&mut transaction).await?,
            published: status == "published",
            closed: status == "closed",
            cancellation: read_cancellation(&mut transaction, id).await?,
        };
        Ok((
            state,
            PgAdministrationTransaction {
                transaction,
                election,
                actor,
            },
        ))
    }
}

#[async_trait::async_trait]
impl ElectionAdministrationTransaction for PgAdministrationTransaction {
    async fn reschedule(
        mut self,
        state: &AdministrationState,
        period: &ElectionPeriod,
        reason: &str,
    ) -> Result<(), BoardError> {
        let id = self.election.into_uuid();
        sqlx::query("UPDATE election SET starts_at = $2, ends_at = $3 WHERE election_id = $1")
            .bind(id)
            .bind(period.starts_at())
            .bind(period.ends_at())
            .execute(&mut *self.transaction)
            .await?;
        sqlx::query("INSERT INTO election_schedule_revision
                (election_id, changed_by, changed_at, previous_starts_at, previous_ends_at, starts_at, ends_at, reason)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(id).bind(self.actor.into_uuid()).bind(state.now)
            .bind(state.period.starts_at()).bind(state.period.ends_at())
            .bind(period.starts_at()).bind(period.ends_at()).bind(reason)
            .execute(&mut *self.transaction).await?;
        self.transaction.commit().await?;
        Ok(())
    }

    async fn cancel(mut self, cancellation: &BoardCancellation) -> Result<(), BoardError> {
        sqlx::query(
            "UPDATE election SET status = 'cancelled', cancelled_at = $2, cancelled_by = $3,
                    cancellation_reason = $4 WHERE election_id = $1",
        )
        .bind(self.election.into_uuid())
        .bind(cancellation.cancelled_at)
        .bind(cancellation.cancelled_by)
        .bind(&cancellation.reason)
        .execute(&mut *self.transaction)
        .await?;
        self.transaction.commit().await?;
        Ok(())
    }
}
