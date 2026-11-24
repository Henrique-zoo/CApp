//! Guarda as contribuições públicas e publica o resultado no mesmo commit.
//!
//! A eleição já encerrada serve de bloqueio para contribuições concorrentes.
//! A transação consulta a atribuição da autoridade, lê o último estado e
//! aplica a decisão tomada pelo serviço de apuração.

use crypto_bigint::U2048;
use serde_json::Value;
use sqlx::{PgConnection, Postgres, Transaction};
use uuid::Uuid;

use crate::infrastructure::database::PgDatabase;
use crate::modules::elections::{
    application::{
        board::{BoardError, BoardSnapshot},
        ports::{TallyRepository, TallyTransaction, TallyVerifier},
        tally::{PendingServerShare, ShareSubmission, StoredShare, TallyEvidence, TallyState},
    },
    crypto_wire::{
        ServerSigner, object_digest,
        tally::{VerifiedShare, VerifiedTally},
    },
    domain::{
        cryptography::elgamal::ElGamalAuthorityKeyPair,
        identifiers::{AccountId, AuthorityId, ElectionId},
    },
};

impl TallyVerifier for ServerSigner {
    type Context = VerifiedTally;
    type Share = VerifiedShare;

    fn verify_tally(&self, board: &BoardSnapshot) -> Result<Self::Context, BoardError> {
        Ok(VerifiedTally::from_board(
            board,
            &self.public_key_base64()?,
        )?)
    }

    fn verify_submission(
        &self,
        context: &Self::Context,
        submission: &ShareSubmission,
    ) -> Result<(StoredShare, Self::Share), BoardError> {
        let envelope = super::cryptography::authority_codec::share_envelope(submission);
        let (authority, digest, share) = self.verify_share(context, &envelope)?;
        Ok((
            StoredShare {
                authority,
                digest,
                envelope,
            },
            share,
        ))
    }

    fn verify_share(
        &self,
        context: &Self::Context,
        envelope: &Value,
    ) -> Result<(AuthorityId, String, Self::Share), BoardError> {
        let share = context.verify_share(envelope)?;
        let authority = AuthorityId::try_from_uuid(share.authority_id())
            .map_err(|_| BoardError::InvalidArtifact("identificador de autoridade"))?;
        let digest = object_digest(&envelope["payload"])?;
        Ok((authority, digest, share))
    }

    fn sign_server_share(
        &self,
        context: &Self::Context,
        pending: &PendingServerShare,
    ) -> Result<Value, BoardError> {
        if !context.is_server_authority(pending.authority) {
            return Err(BoardError::InvalidArtifact(
                "autoridade privada do servidor",
            ));
        }
        let pair = ElGamalAuthorityKeyPair::from_secret(pending.secret)
            .map_err(|_| BoardError::InvalidArtifact("chave privada do servidor"))?;
        Ok(context.sign_share(pending.authority, &pair, self)?)
    }

    fn sign_result(
        &self,
        context: &Self::Context,
        shares: &[Self::Share],
    ) -> Result<Value, BoardError> {
        Ok(self.sign_payload(&context.result_payload(shares)?)?)
    }
}

/// Transação de uma eleição encerrada com atribuição de autoridade verificada.
pub struct PgTallyTransaction {
    transaction: Transaction<'static, Postgres>,
    election: ElectionId,
}

#[async_trait::async_trait]
impl TallyRepository for PgDatabase {
    type Transaction = PgTallyTransaction;

    async fn read_tally(&self, election: ElectionId) -> Result<TallyEvidence, BoardError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;
        let status: String = sqlx::query_scalar(
            "SELECT e.status FROM election e JOIN election_board b USING (election_id)
             WHERE e.election_id = $1",
        )
        .bind(election.into_uuid())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(BoardError::NotFound)?;
        if status == "cancelled" {
            return Err(BoardError::Cancelled);
        }
        if status != "closed" {
            return Err(BoardError::NotYetEnded);
        }
        let state = read_state(&mut tx, election).await?;
        tx.commit().await?;
        Ok(state.into())
    }

    async fn lock_tally(
        &self,
        election: ElectionId,
        authority: AuthorityId,
        account: AccountId,
    ) -> Result<(TallyState, Self::Transaction), BoardError> {
        let mut transaction = self.pool.begin().await?;
        let closed: bool = sqlx::query_scalar(
            "SELECT status = 'closed' FROM election WHERE election_id = $1 FOR UPDATE",
        )
        .bind(election.into_uuid())
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(BoardError::NotFound)?;
        if !closed {
            return Err(BoardError::NotYetEnded);
        }
        let assigned = sqlx::query_scalar::<_, Uuid>(
            "SELECT account_id FROM election_authority
             WHERE election_id = $1 AND authority_id = $2 AND account_id = $3 FOR SHARE",
        )
        .bind(election.into_uuid())
        .bind(authority.into_uuid())
        .bind(account.into_uuid())
        .fetch_optional(&mut *transaction)
        .await?;
        if assigned.is_none() {
            return Err(BoardError::AuthorityNotAssigned);
        }

        let state = read_state(&mut transaction, election).await?;
        Ok((
            state,
            PgTallyTransaction {
                transaction,
                election,
            },
        ))
    }

    async fn pending_server_shares(&self) -> Result<Vec<PendingServerShare>, BoardError> {
        let rows: Vec<(Uuid, Uuid, Vec<u8>)> = sqlx::query_as(
            "SELECT s.election_id, s.authority_id, s.server_secret
             FROM election_authority s JOIN election e USING (election_id)
             WHERE e.status = 'closed' AND s.role = 'server' AND NOT EXISTS (
                 SELECT 1 FROM election_decryption_share h
                 WHERE h.election_id = s.election_id AND h.authority_id = s.authority_id
             )",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(election, authority, bytes)| {
                if bytes.len() != U2048::BYTES {
                    return Err(BoardError::InvalidArtifact(
                        "tamanho do segredo do servidor",
                    ));
                }
                Ok(PendingServerShare {
                    election: ElectionId::try_from_uuid(election)
                        .map_err(|_| BoardError::InvalidArtifact("eleição do servidor"))?,
                    authority: AuthorityId::try_from_uuid(authority)
                        .map_err(|_| BoardError::InvalidArtifact("autoridade do servidor"))?,
                    secret: U2048::from_be_slice(&bytes),
                })
            })
            .collect()
    }

    async fn lock_server_share(
        &self,
        pending: &PendingServerShare,
    ) -> Result<(TallyState, Self::Transaction), BoardError> {
        let election = pending.election;
        let mut transaction = self.pool.begin().await?;
        let exists: Option<Uuid> = sqlx::query_scalar(
            "SELECT e.election_id FROM election e JOIN election_authority s USING (election_id)
             WHERE e.election_id = $1 AND e.status = 'closed'
                 AND s.role = 'server' AND s.authority_id = $2
             FOR UPDATE OF e",
        )
        .bind(election.into_uuid())
        .bind(pending.authority.into_uuid())
        .fetch_optional(&mut *transaction)
        .await?;
        if exists.is_none() {
            return Err(BoardError::NotYetEnded);
        }
        let state = read_state(&mut transaction, election).await?;
        Ok((
            state,
            PgTallyTransaction {
                transaction,
                election,
            },
        ))
    }

    async fn check_assignment(
        &self,
        election: ElectionId,
        authority: AuthorityId,
        account: AccountId,
    ) -> Result<(), BoardError> {
        let assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM election_authority a JOIN election_board b USING (election_id)
              WHERE a.election_id = $1 AND a.authority_id = $2 AND a.account_id = $3)",
        )
        .bind(election.into_uuid())
        .bind(authority.into_uuid())
        .bind(account.into_uuid())
        .fetch_one(&self.pool)
        .await?;
        if !assigned {
            return Err(BoardError::AuthorityNotAssigned);
        }
        Ok(())
    }
}

async fn read_state(
    conn: &mut PgConnection,
    election: ElectionId,
) -> Result<TallyState, BoardError> {
    let rows: Vec<(Uuid, String, Value)> = sqlx::query_as(
        "SELECT authority_id, digest, evidence FROM election_decryption_share
         WHERE election_id = $1 ORDER BY authority_id",
    )
    .bind(election.into_uuid())
    .fetch_all(&mut *conn)
    .await?;
    let shares = rows
        .into_iter()
        .map(|(id, digest, envelope)| {
            Ok(StoredShare {
                authority: AuthorityId::try_from_uuid(id)
                    .map_err(|_| BoardError::InvalidArtifact("autoridade de contribuição"))?,
                digest,
                envelope,
            })
        })
        .collect::<Result<_, BoardError>>()?;
    let result =
        sqlx::query_scalar("SELECT result FROM election_tally_result WHERE election_id = $1")
            .bind(election.into_uuid())
            .fetch_optional(conn)
            .await?;
    Ok(TallyState { shares, result })
}

#[async_trait::async_trait]
impl TallyTransaction for PgTallyTransaction {
    async fn save(mut self, share: &StoredShare, result: &Value) -> Result<(), BoardError> {
        sqlx::query(
            "WITH accepted AS (
                 INSERT INTO election_decryption_share (election_id, authority_id, digest, evidence)
                 VALUES ($1, $2, $3, $4) RETURNING election_id
             )
             INSERT INTO election_tally_result (election_id, result)
             SELECT election_id, $5 FROM accepted
             ON CONFLICT (election_id) DO UPDATE SET result = EXCLUDED.result",
        )
        .bind(self.election.into_uuid())
        .bind(share.authority.into_uuid())
        .bind(&share.digest)
        .bind(&share.envelope)
        .bind(result)
        .execute(&mut *self.transaction)
        .await?;

        self.transaction.commit().await?;
        Ok(())
    }
}
