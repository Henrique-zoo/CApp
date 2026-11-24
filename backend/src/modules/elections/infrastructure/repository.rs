//! Quadro PostgreSQL: uma transação confirma a cédula, o recibo e o acumulador.
//!
//! A linha de `election` é bloqueada antes de verificar o relógio do banco.
//! Aceites e encerramento adquirem esse mesmo lock. Leituras públicas usam uma
//! fotografia REPEATABLE READ, sem expor o acumulador durante a votação.
//! O acumulador é armazenado em binário na ordem do manifesto. No encerramento,
//! seus valores são convertidos em strings decimais para assinar o agregado público.

use chrono::{DateTime, Utc};
use crypto_bigint::U2048;
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::infrastructure::database::PgDatabase;

use super::super::{
    application::{
        administration::BoardCancellation,
        board::{
            AcceptedVote, BoardClosing, BoardClosure, BoardError, BoardPublication, BoardSnapshot,
            NewVote, RecordedBallot, VoteState, VoteWindow,
        },
        ports::{
            BoardClosureTransaction, BoardSnapshotStream, ElectionBoardReader,
            ElectionBoardRepository, VoteRepository, VoteTransaction,
        },
        registration::VoterIdentity,
        verification::BallotVerificationContext,
    },
    domain::{
        cryptography::elgamal::{
            ElGamalCiphertext, GROUP_ID, empty_accumulator, validate_public_key,
        },
        entities::vote::VoteKey,
        identifiers::ElectionId,
    },
};
use super::{
    codec::decode_fingerprint,
    registration::{database_now, eligibility_deadline},
};

impl From<sqlx::Error> for BoardError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(Box::new(value))
    }
}

/// Quantidade máxima de registros carregados por página da exportação HTTP.
pub const SNAPSHOT_PAGE_SIZE: i64 = 32;

const COMPONENT_BYTES: usize = U2048::BYTES;
const CIPHERTEXT_BYTES: usize = 2 * COMPONENT_BYTES;

#[derive(sqlx::FromRow)]
struct BoardRow {
    election_id: Uuid,
    manifest: Value,
    option_ids: Vec<Uuid>,
    manifest_hash: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    server_signing_key: String,
    status: String,
    head_hash: String,
    voter_count: i32,
    accumulator: Vec<u8>,
    closure: Option<Value>,
    aggregate: Option<Value>,
    cancelled_at: Option<DateTime<Utc>>,
    cancelled_by: Option<Uuid>,
    cancellation_reason: Option<String>,
}

impl BoardRow {
    fn cancellation(&self) -> Option<BoardCancellation> {
        Some(BoardCancellation {
            cancelled_at: self.cancelled_at?,
            cancelled_by: self.cancelled_by?,
            reason: self.cancellation_reason.as_ref()?.clone(),
        })
    }
}

#[derive(sqlx::FromRow)]
struct StoredVote {
    ballot_hash: Vec<u8>,
    signed_entry: Value,
    signed_receipt: Value,
}

#[derive(sqlx::FromRow)]
struct JournalEntry {
    entry_index: i32,
    previous_hash: String,
    entry_hash: String,
    signed_entry: Value,
}

impl StoredVote {
    fn into_record(self, recovered: bool) -> RecordedBallot {
        RecordedBallot {
            entry: self.signed_entry,
            receipt: self.signed_receipt,
            recovered,
        }
    }
}

#[async_trait::async_trait]
impl ElectionBoardReader for PgDatabase {
    /// Exporta o mesmo JSON público em páginas, preservando uma única fotografia.
    ///
    /// A origem é lida antes de devolver o canal, permitindo tratar `NotFound`
    /// antes dos cabeçalhos HTTP. O produtor usa keyset e no máximo duas páginas
    /// aguardando o consumidor. Desconectar o consumidor descarta a transação.
    /// Em falha posterior, envia erro e não termina o objeto JSON: um prefixo
    /// parcial nunca é apresentado como uma exportação completa bem-sucedida.
    async fn stream_snapshot(
        &self,
        election_id: ElectionId,
    ) -> Result<BoardSnapshotStream, BoardError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;
        let board = load_board(&mut tx, election_id.into_uuid(), false).await?;
        let (sender, receiver) = mpsc::channel(2);
        tokio::spawn(async move {
            if let Err(error) = stream_snapshot_transaction(tx, board, &sender).await {
                let _ = sender.send(Err(error)).await;
            }
        });

        Ok(Box::pin(ReceiverStream::new(receiver)))
    }

    /// Distribui uma âncora assinada: último recibo, ou manifesto no quadro vazio.
    ///
    /// A resposta obtida do servidor ainda deve ser conservada em outro lugar
    /// para detectar truncamento/reescrita em exportações posteriores.
    async fn read_checkpoint(&self, election_id: ElectionId) -> Result<Value, BoardError> {
        let row: Option<(i32, Value, Option<Value>)> = sqlx::query_as(
            "SELECT b.voter_count, b.manifest, e.signed_receipt
             FROM election_board b
             LEFT JOIN election_vote e ON e.election_id = b.election_id
                 AND e.entry_index = b.voter_count
             WHERE b.election_id = $1",
        )
        .bind(election_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        let (count, manifest, receipt) = row.ok_or(BoardError::NotFound)?;
        match (count, receipt) {
            (0, None) => Ok(manifest),
            (count, Some(receipt)) if count > 0 => Ok(receipt),
            _ => Err(BoardError::InvalidArtifact("checkpoint sem registro")),
        }
    }

    async fn find_receipt(&self, key: VoteKey) -> Result<Option<Value>, BoardError> {
        Ok(sqlx::query_scalar(
            "SELECT signed_receipt FROM election_vote
             WHERE election_id = $1 AND account_id = $2",
        )
        .bind(key.election_id().into_uuid())
        .bind(key.account_id().into_uuid())
        .fetch_optional(&self.pool)
        .await?)
    }
}

async fn read_accepted_vote<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    key: VoteKey,
) -> Result<Option<StoredVote>, BoardError> {
    let stored = sqlx::query_as(
        "SELECT ballot_hash, signed_entry, signed_receipt
         FROM election_vote WHERE election_id = $1 AND account_id = $2",
    )
    .bind(key.election_id().into_uuid())
    .bind(key.account_id().into_uuid())
    .fetch_optional(executor)
    .await?;

    Ok(stored)
}

#[async_trait::async_trait]
impl ElectionBoardRepository for PgDatabase {
    type Transaction = PgBoardClosureTransaction;

    async fn publish(&self, board: BoardPublication) -> Result<BoardSnapshot, BoardError> {
        let election_id = board.election_id.into_uuid();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO election
                 (election_id, academic_center_id, starts_at, ends_at, requires_voter_registration)
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
        )
        .bind(election_id)
        .bind(board.academic_center_id.into_uuid())
        .bind(board.starts_at)
        .bind(board.ends_at)
        .bind(board.requires_registration)
        .execute(&mut *tx)
        .await?;
        lock_election(&mut tx, election_id).await?;
        let config: (Uuid, DateTime<Utc>, DateTime<Utc>, bool) = sqlx::query_as(
            "SELECT academic_center_id, starts_at, ends_at, requires_voter_registration
             FROM election WHERE election_id = $1",
        )
        .bind(election_id)
        .fetch_one(&mut *tx)
        .await?;
        if config
            != (
                board.academic_center_id.into_uuid(),
                board.starts_at,
                board.ends_at,
                board.requires_registration,
            )
        {
            return Err(BoardError::AlreadyExists);
        }
        if insert_publication(&mut tx, &board).await? {
            mark_published(&mut tx, election_id).await?;
        }
        let existing = load_board(&mut tx, election_id, true).await?;
        if existing.manifest_hash != board.manifest_hash {
            return Err(BoardError::AlreadyExists);
        }
        let snapshot = snapshot_in_transaction(&mut tx, existing).await?;
        tx.commit().await?;
        Ok(snapshot)
    }

    async fn snapshot(&self, election_id: ElectionId) -> Result<BoardSnapshot, BoardError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;
        let board = load_board(&mut tx, election_id.into_uuid(), false).await?;
        let snapshot = snapshot_in_transaction(&mut tx, board).await?;
        tx.commit().await?;
        Ok(snapshot)
    }

    async fn due_for_closure(&self) -> Result<Vec<ElectionId>, BoardError> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT election_id FROM election
             WHERE status = 'published' AND ends_at <= statement_timestamp()
             ORDER BY ends_at, election_id",
        )
        .fetch_all(&self.pool)
        .await?;
        ids.into_iter()
            .map(|id| {
                ElectionId::try_from_uuid(id)
                    .map_err(|_| BoardError::InvalidArtifact("identificador da eleição"))
            })
            .collect()
    }

    async fn lock_closure(
        &self,
        election_id: ElectionId,
    ) -> Result<(Option<BoardClosing>, Self::Transaction), BoardError> {
        let mut transaction = self.pool.begin().await?;
        let board = load_board(&mut transaction, election_id.into_uuid(), true).await?;
        if board.status == "cancelled" {
            return Err(BoardError::Cancelled);
        }
        let closing = if board.status == "closed" {
            None
        } else {
            Some(BoardClosing {
                election_id,
                ends_at: board.ends_at,
                now: database_now(&mut transaction).await?,
                server_key: board.server_signing_key.clone(),
                manifest_hash: board.manifest_hash.clone(),
                head_hash: board.head_hash.clone(),
                voter_count: board.voter_count,
                option_ids: board.option_ids.clone(),
                accumulator: decode_accumulator(&board.accumulator, board.option_ids.len())?,
            })
        };

        Ok((closing, PgBoardClosureTransaction { transaction, board }))
    }
}

/// Transação que conserva o quadro estável até publicar o encerramento.
pub struct PgBoardClosureTransaction {
    transaction: Transaction<'static, Postgres>,
    board: BoardRow,
}

#[async_trait::async_trait]
impl BoardClosureTransaction for PgBoardClosureTransaction {
    async fn finish(mut self, evidence: Option<BoardClosure>) -> Result<BoardSnapshot, BoardError> {
        if let Some(evidence) = evidence {
            sqlx::query(
                "UPDATE election_board SET closure = $2, aggregate = $3
                 WHERE election_id = $1",
            )
            .bind(self.board.election_id)
            .bind(&evidence.closure)
            .bind(&evidence.aggregate)
            .execute(&mut *self.transaction)
            .await?;

            sqlx::query("UPDATE election SET status = 'closed' WHERE election_id = $1")
                .bind(self.board.election_id)
                .execute(&mut *self.transaction)
                .await?;
            self.board.closure = Some(evidence.closure);
            self.board.aggregate = Some(evidence.aggregate);
        }

        let snapshot = snapshot_in_transaction(&mut self.transaction, self.board).await?;
        self.transaction.commit().await?;
        Ok(snapshot)
    }
}

/// Estado mutável necessário ao aceite, consultado sob o bloqueio da eleição.
#[derive(sqlx::FromRow)]
struct VoteBoardRow {
    closed: bool,
    cancelled: bool,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    server_signing_key: String,
    requires_voter_registration: bool,
    head_hash: String,
    voter_count: i32,
    choice_count: i32,
    accumulator: Vec<u8>,
}

/// Transação PostgreSQL que conserva o bloqueio até a gravação do aceite.
pub struct PgVoteTransaction {
    transaction: Transaction<'static, Postgres>,
    key: VoteKey,
}

#[async_trait::async_trait]
impl VoteRepository for PgDatabase {
    type Transaction = PgVoteTransaction;

    async fn ballot_context(
        &self,
        election_id: ElectionId,
    ) -> Result<BallotVerificationContext, BoardError> {
        let (manifest_hash, option_ids): (String, Vec<Uuid>) = sqlx::query_as(
            "SELECT manifest_hash, option_ids FROM election_board WHERE election_id = $1",
        )
        .bind(election_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(BoardError::NotFound)?;

        Ok(BallotVerificationContext {
            manifest_hash: decode_fingerprint(&manifest_hash)
                .map_err(|_| BoardError::InvalidArtifact("hash do manifesto"))?,
            option_ids,
        })
    }

    async fn lock_vote(
        &self,
        election_id: ElectionId,
        voter: VoterIdentity,
    ) -> Result<(VoteState, Self::Transaction), BoardError> {
        let mut transaction = self.pool.begin().await?;
        let key = VoteKey::new(election_id, voter.account_id);

        // A leitura do quadro começa depois do bloqueio para ver o último aceite confirmado.
        lock_election(&mut transaction, election_id.into_uuid()).await?;
        let board: VoteBoardRow = sqlx::query_as(
            "SELECT e.status = 'closed' AS closed, e.status = 'cancelled' AS cancelled,
                    e.starts_at, e.ends_at, b.server_signing_key, e.requires_voter_registration,
                    b.head_hash, b.voter_count, b.accumulator, cardinality(b.option_ids) AS choice_count
             FROM election e JOIN election_board b USING (election_id) WHERE e.election_id = $1",
        )
        .bind(election_id.into_uuid())
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(BoardError::NotFound)?;

        // A leitura ocorre depois de adquirir o lock, inclusive após esperar por
        // um aceite concorrente da mesma conta.
        let state = if let Some(stored) = read_accepted_vote(&mut *transaction, key).await? {
            let ballot_hash = stored
                .ballot_hash
                .as_slice()
                .try_into()
                .map_err(|_| BoardError::InvalidArtifact("hash do voto aceito"))?;
            VoteState::Accepted(AcceptedVote {
                ballot_hash,
                record: stored.into_record(true),
            })
        } else {
            let eligible_until = eligibility_deadline(&mut transaction, election_id, voter).await?;
            let (now, registered_key): (DateTime<Utc>, Option<Vec<u8>>) = sqlx::query_as(
                "SELECT clock_timestamp(), (
                     SELECT public_key FROM election_voter_registration
                     WHERE election_id = $1 AND account_id = $2 AND $3
                 )",
            )
            .bind(election_id.into_uuid())
            .bind(key.account_id().into_uuid())
            .bind(board.requires_voter_registration)
            .fetch_one(&mut *transaction)
            .await?;

            VoteState::Pending(VoteWindow {
                closed: board.closed,
                cancelled: board.cancelled,
                starts_at: board.starts_at,
                ends_at: board.ends_at,
                eligible_until,
                now,
                server_key: board.server_signing_key,
                requires_registration: board.requires_voter_registration,
                registered_key,
                head_hash: board.head_hash,
                voter_count: board.voter_count as u32,
                accumulator: decode_accumulator(&board.accumulator, board.choice_count as usize)?,
            })
        };

        Ok((state, PgVoteTransaction { transaction, key }))
    }
}

#[async_trait::async_trait]
impl VoteTransaction for PgVoteTransaction {
    async fn save(mut self, vote: NewVote) -> Result<RecordedBallot, BoardError> {
        sqlx::query(
            "WITH accepted AS (
                 INSERT INTO election_vote
                     (election_id, account_id, ballot_hash, public_key, accepted_at,
                      entry_index, entry_hash, signed_entry, signed_receipt)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING election_id, entry_index, entry_hash
             )
             UPDATE election_board b SET head_hash = v.entry_hash, voter_count = v.entry_index, accumulator = $10
             FROM accepted v WHERE b.election_id = v.election_id",
        )
        .bind(self.key.election_id().into_uuid())
        .bind(self.key.account_id().into_uuid())
        .bind(vote.ballot_hash.as_slice())
        .bind(&vote.public_key)
        .bind(vote.accepted_at)
        .bind(vote.index as i32)
        .bind(&vote.entry_hash)
        .bind(&vote.record.entry)
        .bind(&vote.record.receipt)
        .bind(encode_accumulator(&vote.accumulator))
        .execute(&mut *self.transaction)
        .await?;

        self.transaction.commit().await?;
        Ok(vote.record)
    }
}

pub(super) async fn insert_publication(
    tx: &mut Transaction<'_, Postgres>,
    board: &BoardPublication,
) -> Result<bool, BoardError> {
    let election_id = board.election_id.into_uuid();
    let accumulator = empty_accumulator(board.option_ids.len())
        .map_err(|_| BoardError::InvalidArtifact("opções do manifesto"))?;

    // A eleição já está bloqueada. A chave primária garante um único quadro.
    let result = sqlx::query(
        "INSERT INTO election_board (election_id, manifest, manifest_hash, head_hash, accumulator)
         SELECT $1, $2, $3, $3, $4
         WHERE NOT EXISTS (SELECT 1 FROM election_board WHERE election_id = $1)",
    )
    .bind(election_id)
    .bind(&board.manifest)
    .bind(&board.manifest_hash)
    .bind(encode_accumulator(&accumulator))
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected() == 1)
}

pub(super) async fn mark_published(
    tx: &mut Transaction<'_, Postgres>,
    election_id: Uuid,
) -> Result<(), BoardError> {
    sqlx::query("UPDATE election SET status = 'published' WHERE election_id = $1")
        .bind(election_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn lock_election(
    tx: &mut Transaction<'_, Postgres>,
    election_id: Uuid,
) -> Result<(), BoardError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT election_id FROM election WHERE election_id = $1 FOR UPDATE",
    )
    .bind(election_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(BoardError::NotFound)?;
    Ok(())
}

async fn load_board(
    tx: &mut Transaction<'_, Postgres>,
    election_id: Uuid,
    locked: bool,
) -> Result<BoardRow, BoardError> {
    if locked {
        lock_election(tx, election_id).await?;
    }
    sqlx::query_as(
        "SELECT b.*, e.starts_at, e.ends_at, e.status, e.cancelled_at, e.cancelled_by, e.cancellation_reason
         FROM election_board b JOIN election e USING (election_id) WHERE b.election_id = $1",
    )
    .bind(election_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(BoardError::NotFound)
}

async fn snapshot_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
    board: BoardRow,
) -> Result<BoardSnapshot, BoardError> {
    let rows: Vec<JournalEntry> = sqlx::query_as(
        "SELECT entry_index,previous_hash,entry_hash,signed_entry
         FROM election_vote WHERE election_id=$1 ORDER BY entry_index",
    )
    .bind(board.election_id)
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() != board.voter_count as usize {
        return Err(BoardError::InvalidArtifact("contagem do quadro"));
    }
    let mut expected_head = board.manifest_hash.clone();
    for (position, row) in rows.iter().enumerate() {
        validate_entry(&board, position as i32 + 1, &expected_head, row)?;
        expected_head.clone_from(&row.entry_hash);
    }
    validate_final_head(&board, rows.len() as i32, &expected_head)?;
    let checkpoint = load_checkpoint(tx, &board).await?;
    let cancellation = board.cancellation();
    Ok(BoardSnapshot {
        manifest: board.manifest,
        starts_at: board.starts_at,
        ends_at: board.ends_at,
        cancellation,
        entries: rows.into_iter().map(|row| row.signed_entry).collect(),
        checkpoint,
        closure: board.closure,
        aggregate: board.aggregate,
    })
}

fn validate_entry(
    board: &BoardRow,
    expected_index: i32,
    expected_head: &str,
    row: &JournalEntry,
) -> Result<(), BoardError> {
    let payload = &row.signed_entry["payload"];
    if row.entry_index != expected_index
        || row.previous_hash != expected_head
        || payload["index"].as_u64() != Some(row.entry_index as u64)
        || uuid(&payload["electionId"])? != board.election_id
        || payload["manifestHash"].as_str() != Some(board.manifest_hash.as_str())
        || payload["previousHash"].as_str() != Some(row.previous_hash.as_str())
    {
        return Err(BoardError::InvalidArtifact("sequência do quadro"));
    }
    Ok(())
}

fn validate_final_head(board: &BoardRow, count: i32, head: &str) -> Result<(), BoardError> {
    if count != board.voter_count || head != board.head_hash {
        return Err(BoardError::InvalidArtifact("cabeça ou contagem do quadro"));
    }
    Ok(())
}

async fn load_checkpoint(
    tx: &mut Transaction<'_, Postgres>,
    board: &BoardRow,
) -> Result<Option<Value>, BoardError> {
    // Só o último recibo integra a fotografia: não carregar os outros até
    // 100000 recibos para descartá-los em seguida.
    let checkpoint: Option<Value> = if board.voter_count == 0 {
        None
    } else {
        Some(
            sqlx::query_scalar(
                "SELECT signed_receipt FROM election_vote
             WHERE election_id=$1 AND entry_index=$2",
            )
            .bind(board.election_id)
            .bind(board.voter_count)
            .fetch_optional(&mut **tx)
            .await?
            .ok_or(BoardError::InvalidArtifact("checkpoint sem registro"))?,
        )
    };
    if let Some(receipt) = checkpoint.as_ref() {
        let payload = &receipt["payload"];
        if payload["index"].as_u64() != Some(board.voter_count as u64)
            || uuid(&payload["electionId"])? != board.election_id
            || payload["manifestHash"].as_str() != Some(board.manifest_hash.as_str())
            || payload["entryHash"].as_str() != Some(board.head_hash.as_str())
        {
            return Err(BoardError::InvalidArtifact("checkpoint divergente"));
        }
    }
    Ok(checkpoint)
}

async fn stream_snapshot_transaction(
    mut tx: Transaction<'static, Postgres>,
    board: BoardRow,
    sender: &mpsc::Sender<Result<Vec<u8>, BoardError>>,
) -> Result<(), BoardError> {
    let mut prefix = b"{\"manifest\":".to_vec();
    append_json(&mut prefix, &board.manifest)?;
    prefix.extend_from_slice(b",\"startsAt\":");
    append_json(&mut prefix, &board.starts_at)?;
    prefix.extend_from_slice(b",\"endsAt\":");
    append_json(&mut prefix, &board.ends_at)?;
    if let Some(cancellation) = board.cancellation() {
        prefix.extend_from_slice(b",\"cancellation\":");
        append_json(&mut prefix, &cancellation)?;
    }
    prefix.extend_from_slice(b",\"entries\":[");
    if sender.send(Ok(prefix)).await.is_err() {
        return Ok(());
    }
    let mut last_index = 0;
    let mut head = board.manifest_hash.clone();
    loop {
        if sender.is_closed() {
            return Ok(());
        }
        let page: Vec<JournalEntry> = sqlx::query_as(
            "SELECT entry_index,previous_hash,entry_hash,signed_entry
             FROM election_vote WHERE election_id=$1 AND entry_index>$2
             ORDER BY entry_index LIMIT $3",
        )
        .bind(board.election_id)
        .bind(last_index)
        .bind(SNAPSHOT_PAGE_SIZE)
        .fetch_all(&mut *tx)
        .await?;
        if page.is_empty() {
            break;
        }
        let mut chunk = Vec::new();
        for entry in page {
            validate_entry(&board, last_index + 1, &head, &entry)?;
            if last_index != 0 {
                chunk.push(b',');
            }
            append_json(&mut chunk, &entry.signed_entry)?;
            last_index = entry.entry_index;
            head = entry.entry_hash;
        }
        if sender.send(Ok(chunk)).await.is_err() {
            return Ok(());
        }
    }
    validate_final_head(&board, last_index, &head)?;
    let checkpoint = load_checkpoint(&mut tx, &board).await?;
    let mut suffix = b"],\"checkpoint\":".to_vec();
    append_json(&mut suffix, &checkpoint)?;
    suffix.extend_from_slice(b",\"closure\":");
    append_json(&mut suffix, &board.closure)?;
    suffix.extend_from_slice(b",\"aggregate\":");
    append_json(&mut suffix, &board.aggregate)?;
    suffix.push(b'}');
    tx.commit().await?;
    let _ = sender.send(Ok(suffix)).await;
    Ok(())
}

fn append_json(output: &mut Vec<u8>, value: &impl serde::Serialize) -> Result<(), BoardError> {
    serde_json::to_writer(output, value)
        .map_err(|_| BoardError::InvalidArtifact("serialização do quadro"))
}

fn text(value: &Value) -> Result<&str, BoardError> {
    value
        .as_str()
        .ok_or(BoardError::InvalidArtifact("campo textual"))
}

fn uuid(value: &Value) -> Result<Uuid, BoardError> {
    Uuid::parse_str(text(value)?).map_err(|_| BoardError::InvalidArtifact("UUID"))
}

/// Concatena os pares `(a, b)` em big-endian, seguindo a ordem do manifesto.
fn encode_accumulator(ciphertexts: &[ElGamalCiphertext]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(ciphertexts.len() * CIPHERTEXT_BYTES);

    for cipher in ciphertexts {
        bytes.extend_from_slice(&cipher.a.to_be_bytes());
        bytes.extend_from_slice(&cipher.b.to_be_bytes());
    }

    bytes
}

/// Reconstitui uma cifra por posição do manifesto e valida os elementos do grupo.
/// O produto de cifras pode conter a identidade `1`, inclusive no quadro vazio.
fn decode_accumulator(
    bytes: &[u8],
    choice_count: usize,
) -> Result<Vec<ElGamalCiphertext>, BoardError> {
    let (pairs, remainder) = bytes.as_chunks::<CIPHERTEXT_BYTES>();
    if !remainder.is_empty() || pairs.len() != choice_count {
        return Err(BoardError::InvalidArtifact("dimensão do acumulador"));
    }

    pairs
        .iter()
        .map(|pair| {
            let (a, b) = pair.split_at(COMPONENT_BYTES);
            let a = U2048::from_be_slice(a);
            let b = U2048::from_be_slice(b);

            for component in [&a, &b] {
                if *component != U2048::ONE {
                    validate_public_key(GROUP_ID, component)
                        .map_err(|_| BoardError::InvalidArtifact("elemento do acumulador"))?;
                }
            }

            Ok(ElGamalCiphertext { a, b })
        })
        .collect()
}
