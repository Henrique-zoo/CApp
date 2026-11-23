//! Quadro PostgreSQL: uma transação confirma participação, registro e acumulador.
//!
//! A linha de `election_board` é bloqueada antes de verificar o relógio do banco.
//! Aceites e encerramento adquirem esse mesmo lock. Leituras públicas usam uma
//! fotografia REPEATABLE READ, sem expor o acumulador durante a votação.

use std::sync::Arc;

use chrono::{DateTime, SecondsFormat, Utc};
use crypto_bigint::U2048;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::infrastructure::database::PgDatabase;

use super::super::{
    application::{
        board::{BoardError, BoardSnapshot, ElectionBoard, FrozenBoard, RecordedBallot},
        ports::{BoardSnapshotStream, ElectionBoardReader},
        registration::{VoterIdentity, validate_registered_key},
    },
    crypto_wire::{
        BallotContext, CryptoWireError, ServerSigner, object_digest, parse_strict_json,
        verify_ballot, verify_calendar_origin, verify_manifest, verify_signed_object,
    },
    domain::{
        cryptography::elgamal::{
            ElGamalCiphertext, EncryptedBallot, GROUP_ID, accumulate_ballot, empty_accumulator,
            validate_public_key,
        },
        profile::MAX_VOTER_COUNT,
    },
};
use super::{
    codec::decode_base64_url,
    registration::{database_now, eligibility_deadline},
};

impl From<sqlx::Error> for BoardError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(Box::new(value))
    }
}

impl From<CryptoWireError> for BoardError {
    fn from(value: CryptoWireError) -> Self {
        match value {
            CryptoWireError::Signing => Self::SigningUnavailable,
            invalid => Self::InvalidEvidence(Box::new(invalid)),
        }
    }
}

/// Adaptador de escrita; sua chave assina somente evidências confirmáveis.
///
/// O servidor conserva sua própria chave. Nenhum segredo externo, escolha em
/// claro ou nonce de cifragem é recebido ou persistido por este adaptador.
#[derive(Clone)]
pub struct PgElectionBoard {
    pool: PgPool,
    signer: Arc<ServerSigner>,
}

/// Quantidade máxima de registros carregados por página da exportação HTTP.
pub const SNAPSHOT_PAGE_SIZE: i64 = 32;

#[derive(sqlx::FromRow)]
struct BoardRow {
    election_id: Uuid,
    manifest: Value,
    calendar_origin: Value,
    manifest_hash: String,
    schedule_hash: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: String,
    requires_voter_registration: bool,
    head_hash: String,
    voter_count: i32,
    accumulator: Value,
    closure: Option<Value>,
    aggregate: Option<Value>,
}

#[derive(sqlx::FromRow)]
struct StoredVote {
    voter_id: Uuid,
    submission_id: Uuid,
    ballot_hash: String,
    signed_ballot: Value,
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

impl PgElectionBoard {
    /// Usa o pool da aplicação e a chave de assinatura própria do servidor.
    pub fn new(pool: PgPool, signer: Arc<ServerSigner>) -> Self {
        Self { pool, signer }
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
    async fn stream_snapshot(&self, election_id: Uuid) -> Result<BoardSnapshotStream, BoardError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;
        let board = load_board(&mut tx, election_id, false).await?;
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
    async fn read_checkpoint(&self, election_id: Uuid) -> Result<Value, BoardError> {
        let row: Option<(i32, Value, Option<Value>)> = sqlx::query_as(
            "SELECT b.voter_count, b.manifest, e.signed_receipt
             FROM election_board b
             LEFT JOIN election_board_entry e ON e.election_id = b.election_id
                 AND e.entry_index = b.voter_count
             WHERE b.election_id = $1",
        )
        .bind(election_id)
        .fetch_optional(&self.pool)
        .await?;
        let (count, manifest, receipt) = row.ok_or(BoardError::NotFound)?;
        match (count, receipt) {
            (0, None) => Ok(manifest),
            (count, Some(receipt)) if count > 0 => Ok(receipt),
            _ => Err(BoardError::InvalidArtifact("checkpoint sem registro")),
        }
    }

    async fn find_receipt(
        &self,
        election_id: Uuid,
        authenticated_account_id: Uuid,
    ) -> Result<Option<Value>, BoardError> {
        let accepted =
            read_accepted_vote(&self.pool, election_id, authenticated_account_id).await?;

        Ok(accepted.map(|vote| vote.signed_receipt))
    }
}

async fn read_accepted_vote(
    pool: &PgPool,
    election_id: Uuid,
    authenticated_account_id: Uuid,
) -> Result<Option<StoredVote>, BoardError> {
    let stored = sqlx::query_as(
        "SELECT p.voter_id, p.submission_id, p.ballot_hash,
                e.signed_ballot, e.signed_entry, e.signed_receipt
         FROM election_participation p
         JOIN election_board_entry e USING (election_id, voter_id)
         WHERE p.election_id = $1 AND p.account_id = $2",
    )
    .bind(election_id)
    .bind(authenticated_account_id)
    .fetch_optional(pool)
    .await?;

    Ok(stored)
}

#[async_trait::async_trait]
impl ElectionBoard for PgElectionBoard {
    async fn open(&self, input: FrozenBoard) -> Result<BoardSnapshot, BoardError> {
        let server_key = self.signer.public_key_base64()?;
        let manifest = verify_manifest(&input.manifest, &server_key)?;
        let calendar = verify_calendar_origin(&input.calendar_origin, &manifest, &server_key)?;
        let starts_at = timestamp(&calendar["payload"]["startsAt"])?;
        let ends_at = timestamp(&calendar["payload"]["endsAt"])?;
        let schedule_hash = object_digest(&calendar["payload"])?;
        let accumulator = empty_accumulator(manifest.option_ids.len())
            .map_err(|_| BoardError::InvalidArtifact("opções do manifesto"))?;
        let accumulator = encode_ciphertexts(&manifest.option_ids, &accumulator);
        let mut tx = self.pool.begin().await?;
        // A chave primária também serializa duas publicações simultâneas.
        sqlx::query(
            "INSERT INTO election_board
             (election_id, manifest, calendar_origin, manifest_hash, schedule_hash,
              starts_at, ends_at, head_hash, accumulator, requires_voter_registration)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$4,$8,$9)
             ON CONFLICT (election_id) DO NOTHING",
        )
        .bind(manifest.election_id)
        .bind(&manifest.envelope)
        .bind(&calendar)
        .bind(&manifest.manifest_hash)
        .bind(&schedule_hash)
        .bind(starts_at)
        .bind(ends_at)
        .bind(accumulator)
        .bind(input.requires_voter_registration)
        .execute(&mut *tx)
        .await?;
        let board = load_board(&mut tx, manifest.election_id, true).await?;
        // A assinatura PSS pode variar numa repetição; os payloads são a origem.
        if board.manifest["payload"] != manifest.envelope["payload"]
            || board.calendar_origin["payload"] != calendar["payload"]
            || board.requires_voter_registration != input.requires_voter_registration
        {
            return Err(BoardError::AlreadyExists);
        }
        let snapshot = snapshot_in_transaction(&mut tx, board).await?;
        tx.commit().await?;
        Ok(snapshot)
    }

    async fn record(
        &self,
        election_id: Uuid,
        voter: VoterIdentity,
        raw: &[u8],
    ) -> Result<RecordedBallot, BoardError> {
        let envelope = parse_strict_json(raw)?;
        if voter.account_id.is_nil() {
            return Err(BoardError::NotEligible);
        }
        // A verificação RSA e as exponenciações da cédula não dependem da
        // cabeça mutável. Fazê-las antes do lock evita serializar esse custo
        // para todos os eleitores. Origem e participação são relidas abaixo.
        let origin: BoardRow = sqlx::query_as("SELECT * FROM election_board WHERE election_id=$1")
            .bind(election_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(BoardError::NotFound)?;
        let option_ids = option_ids(&origin.manifest)?;
        let previous: Option<(Uuid, Value)> = sqlx::query_as(
            "SELECT p.voter_id,e.signed_ballot FROM election_participation p
             JOIN election_board_entry e USING (election_id,voter_id)
             WHERE p.election_id=$1 AND p.account_id=$2",
        )
        .bind(election_id)
        .bind(voter.account_id)
        .fetch_optional(&self.pool)
        .await?;
        // A repetição preserva a identidade pública e a chave do voto já aceito.
        let (verified_voter, verified_key) = match previous.as_ref() {
            Some((id, ballot)) => (*id, Some(text(&ballot["payload"]["publicKey"])?.to_owned())),
            None => (voter.account_id, None),
        };
        let verified = verify_ballot(
            raw,
            &BallotContext {
                election_id,
                manifest_hash: origin.manifest_hash.clone(),
                voter_id: verified_voter,
                expected_public_key: verified_key,
                option_ids: option_ids.clone(),
            },
        )?;
        let ciphertexts = EncryptedBallot::new(verified.ciphertexts.clone())
            .map_err(|_| BoardError::InvalidArtifact("cifras da cédula"))?;
        let mut tx = self.pool.begin().await?;
        let board = load_board(&mut tx, election_id, true).await?;
        if board.manifest != origin.manifest
            || board.manifest_hash != origin.manifest_hash
            || board.calendar_origin != origin.calendar_origin
            || board.schedule_hash != origin.schedule_hash
            || board.starts_at != origin.starts_at
            || board.ends_at != origin.ends_at
            || board.requires_voter_registration != origin.requires_voter_registration
        {
            return Err(BoardError::InvalidArtifact(
                "origem modificada durante o aceite",
            ));
        }
        let stored = sqlx::query_as::<_, StoredVote>(
            "SELECT p.voter_id, p.submission_id, p.ballot_hash,
                    e.signed_ballot, e.signed_entry, e.signed_receipt
             FROM election_participation p
             JOIN election_board_entry e USING (election_id, voter_id)
             WHERE p.election_id = $1 AND p.account_id = $2",
        )
        .bind(election_id)
        .bind(voter.account_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(stored) = stored {
            // Uma nova assinatura PSS válida do mesmo payload recupera o
            // registro original. Assinatura adulterada nunca recupera aceite.
            if uuid(&envelope["payload"]["voterId"])? != stored.voter_id {
                return Err(BoardError::VoterMismatch);
            }
            if uuid(&envelope["payload"]["submissionId"])? != stored.submission_id {
                return Err(BoardError::AlreadyParticipated);
            }
            if verified.ballot_hash != stored.ballot_hash
                || envelope["payload"] != stored.signed_ballot["payload"]
            {
                return Err(BoardError::SubmissionConflict);
            }
            tx.commit().await?;
            return Ok(stored.into_record(true));
        }
        if previous.is_some() {
            return Err(BoardError::InvalidArtifact(
                "participação removida durante o aceite",
            ));
        }

        let eligible_until = eligibility_deadline(&mut tx, election_id, voter).await?;
        let now = database_now(&mut tx).await?;
        if board.status != "open" {
            return Err(BoardError::Closed);
        }
        if now < board.starts_at {
            return Err(BoardError::VotingNotStarted);
        }
        if now >= board.ends_at {
            return Err(BoardError::VotingEnded);
        }
        if eligible_until.is_none_or(|end| now >= end) {
            return Err(BoardError::NotEligible);
        }
        if board.requires_voter_registration {
            let registered_key: Option<Vec<u8>> = sqlx::query_scalar(
                "SELECT public_key FROM election_voter_registration
                 WHERE election_id = $1 AND account_id = $2",
            )
            .bind(election_id)
            .bind(voter.account_id)
            .fetch_optional(&mut *tx)
            .await?;
            let ballot_key = decode_base64_url(text(&verified.envelope["payload"]["publicKey"])?)
                .map_err(|error| BoardError::InvalidEvidence(Box::new(error)))?;
            validate_registered_key(registered_key.as_deref(), &ballot_key)?;
        }
        validate_signing_origin(&board, &self.signer)?;
        if board.voter_count >= MAX_VOTER_COUNT as i32 {
            return Err(BoardError::LimitExceeded);
        }
        let used_voter: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM election_participation
                           WHERE election_id=$1 AND voter_id=$2)",
        )
        .bind(election_id)
        .bind(verified_voter)
        .fetch_one(&mut *tx)
        .await?;
        if used_voter {
            return Err(BoardError::VoterMismatch);
        }
        let submission_id = uuid(&verified.envelope["payload"]["submissionId"])?;
        let accumulator = decode_ciphertexts(&board.accumulator, &option_ids)?;
        let accumulated = accumulate_ballot(accumulator, &ciphertexts)
            .map_err(|_| BoardError::InvalidArtifact("acumulador"))?;
        let next_index = board.voter_count + 1;
        let signed = self
            .signer
            .issue_record(&verified, next_index as u32, &board.head_hash)?;
        sqlx::query(
            "INSERT INTO election_participation
             (election_id,account_id,voter_id,submission_id,ballot_hash,accepted_at)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(election_id)
        .bind(voter.account_id)
        .bind(verified_voter)
        .bind(submission_id)
        .bind(&verified.ballot_hash)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO election_board_entry
             (election_id,entry_index,voter_id,previous_hash,entry_hash,
              signed_ballot,signed_entry,signed_receipt)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(election_id)
        .bind(next_index)
        .bind(verified_voter)
        .bind(&board.head_hash)
        .bind(&signed.entry_hash)
        .bind(&verified.envelope)
        .bind(&signed.board_entry)
        .bind(&signed.receipt)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE election_board SET head_hash=$2,voter_count=$3,accumulator=$4
             WHERE election_id=$1",
        )
        .bind(election_id)
        .bind(&signed.entry_hash)
        .bind(next_index)
        .bind(encode_ciphertexts(&option_ids, &accumulated))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(RecordedBallot {
            entry: signed.board_entry,
            receipt: signed.receipt,
            recovered: false,
        })
    }

    async fn recover(
        &self,
        election_id: Uuid,
        authenticated_account_id: Uuid,
    ) -> Result<Option<RecordedBallot>, BoardError> {
        let accepted =
            read_accepted_vote(&self.pool, election_id, authenticated_account_id).await?;

        Ok(accepted.map(|vote| vote.into_record(true)))
    }

    async fn snapshot(&self, election_id: Uuid) -> Result<BoardSnapshot, BoardError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;
        let board = load_board(&mut tx, election_id, false).await?;
        let snapshot = snapshot_in_transaction(&mut tx, board).await?;
        tx.commit().await?;
        Ok(snapshot)
    }

    async fn close(&self, election_id: Uuid) -> Result<BoardSnapshot, BoardError> {
        let mut tx = self.pool.begin().await?;
        let mut board = load_board(&mut tx, election_id, true).await?;
        if board.status == "closed" {
            let snapshot = snapshot_in_transaction(&mut tx, board).await?;
            tx.commit().await?;
            return Ok(snapshot);
        }
        let now = database_now(&mut tx).await?;
        if now < board.ends_at {
            return Err(BoardError::NotYetEnded);
        }
        validate_signing_origin(&board, &self.signer)?;
        decode_ciphertexts(&board.accumulator, &option_ids(&board.manifest)?)?;
        let closure = self.signer.sign_payload(&json!({
            "version":1,"kind":"closure","electionId":election_id,
            "manifestHash":board.manifest_hash,"scheduleHash":board.schedule_hash,
            "headHash":board.head_hash,"voterCount":board.voter_count,
            "closedAt":now.to_rfc3339_opts(SecondsFormat::Secs,true),
        }))?;
        let aggregate = self.signer.sign_payload(&json!({
            "version":1,"kind":"aggregate","electionId":election_id,
            "manifestHash":board.manifest_hash,
            "closureHash":object_digest(&closure["payload"])?,
            "ciphertexts":board.accumulator,
        }))?;
        sqlx::query(
            "UPDATE election_board SET status='closed',closure=$2,aggregate=$3
             WHERE election_id=$1",
        )
        .bind(election_id)
        .bind(&closure)
        .bind(&aggregate)
        .execute(&mut *tx)
        .await?;
        board.closure = Some(closure);
        board.aggregate = Some(aggregate);
        let snapshot = snapshot_in_transaction(&mut tx, board).await?;
        tx.commit().await?;
        Ok(snapshot)
    }
}

async fn load_board(
    tx: &mut Transaction<'_, Postgres>,
    election_id: Uuid,
    locked: bool,
) -> Result<BoardRow, BoardError> {
    let sql = if locked {
        "SELECT * FROM election_board WHERE election_id=$1 FOR UPDATE"
    } else {
        "SELECT * FROM election_board WHERE election_id=$1"
    };
    sqlx::query_as(sql)
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
         FROM election_board_entry WHERE election_id=$1 ORDER BY entry_index",
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
    Ok(BoardSnapshot {
        manifest: board.manifest,
        calendar_origin: board.calendar_origin,
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
                "SELECT signed_receipt FROM election_board_entry
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
    prefix.extend_from_slice(b",\"calendarOrigin\":");
    append_json(&mut prefix, &board.calendar_origin)?;
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
             FROM election_board_entry WHERE election_id=$1 AND entry_index>$2
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

fn timestamp(value: &Value) -> Result<DateTime<Utc>, BoardError> {
    DateTime::parse_from_rfc3339(text(value)?)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| BoardError::InvalidArtifact("instante do calendário"))
}

fn option_ids(manifest: &Value) -> Result<Vec<Uuid>, BoardError> {
    manifest["payload"]["options"]
        .as_array()
        .ok_or(BoardError::InvalidArtifact("opções do manifesto"))?
        .iter()
        .map(|option| uuid(&option["id"]))
        .collect()
}

fn encode_ciphertexts(options: &[Uuid], ciphertexts: &[ElGamalCiphertext]) -> Value {
    Value::Array(options.iter().zip(ciphertexts).map(|(option, cipher)| {
        json!({
            "optionId":option,
            "a":rsa::BigUint::from_bytes_be(cipher.a.to_be_bytes().as_slice()).to_str_radix(10),
            "b":rsa::BigUint::from_bytes_be(cipher.b.to_be_bytes().as_slice()).to_str_radix(10),
        })
    }).collect())
}

fn decode_ciphertexts(
    value: &Value,
    options: &[Uuid],
) -> Result<Vec<ElGamalCiphertext>, BoardError> {
    let values = value
        .as_array()
        .ok_or(BoardError::InvalidArtifact("acumulador"))?;
    if values.len() != options.len() {
        return Err(BoardError::InvalidArtifact("dimensão do acumulador"));
    }
    values
        .iter()
        .zip(options)
        .map(|(value, option)| {
            if uuid(&value["optionId"])? != *option {
                return Err(BoardError::InvalidArtifact("ordem do acumulador"));
            }
            let parse = |value: &Value| -> Result<U2048, BoardError> {
                let decimal = text(value)?;
                if decimal.is_empty()
                    || (decimal.len() > 1 && decimal.starts_with('0'))
                    || !decimal.bytes().all(|byte| byte.is_ascii_digit())
                {
                    return Err(BoardError::InvalidArtifact("inteiro do acumulador"));
                }
                let number = U2048::from_str_radix_vartime(decimal, 10)
                    .map_err(|_| BoardError::InvalidArtifact("inteiro do acumulador"))?;
                if number != U2048::ONE {
                    validate_public_key(GROUP_ID, &number)
                        .map_err(|_| BoardError::InvalidArtifact("elemento do acumulador"))?;
                }
                Ok(number)
            };
            Ok(ElGamalCiphertext {
                a: parse(&value["a"])?,
                b: parse(&value["b"])?,
            })
        })
        .collect()
}

fn validate_signing_origin(board: &BoardRow, signer: &ServerSigner) -> Result<(), BoardError> {
    let key = signer.public_key_base64()?;
    let manifest = &board.manifest["payload"];
    let calendar = &board.calendar_origin["payload"];
    if text(&manifest["serverSigningKey"])? != key
        || uuid(&manifest["electionId"])? != board.election_id
        || uuid(&calendar["electionId"])? != board.election_id
        || object_digest(manifest)? != board.manifest_hash
        || object_digest(calendar)? != board.schedule_hash
        || text(&manifest["calendarOriginHash"])? != board.schedule_hash
        || timestamp(&calendar["startsAt"])? != board.starts_at
        || timestamp(&calendar["endsAt"])? != board.ends_at
    {
        return Err(BoardError::InvalidArtifact(
            "origem ou chave de assinatura divergente",
        ));
    }
    verify_signed_object(&board.manifest, &key)?;
    verify_signed_object(&board.calendar_origin, &key)?;
    Ok(())
}
