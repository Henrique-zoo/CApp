//! Reabrir o banco da #381 preserva votos aceitos antes de existir cadastro prévio.

use super::fixtures::{Origin, Voter, identity, server};
use crate::support::TestDatabase;
use backend::{
    infrastructure::database::MIGRATOR,
    modules::elections::{
        application::board::{BoardError, ElectionBoard},
        crypto_wire::{BallotContext, object_digest, verify_ballot},
        infrastructure::{PgElectionBoard, PgVoterRegistry},
    },
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::migrate::Migrate;
use uuid::Uuid;

#[tokio::test]
async fn upgrading_the_381_schema_preserves_existing_vote_without_inventing_registration() {
    let db = TestDatabase::unmigrated().await;
    let pool = &db.pool;
    let mut conn = pool.acquire().await.unwrap();
    conn.ensure_migrations_table().await.unwrap();
    for migration in MIGRATOR.iter().filter(|m| m.version <= 20261122130000) {
        conn.apply(migration).await.unwrap();
    }
    drop(conn);
    let signer = server();
    let board = PgElectionBoard::new(pool.clone(), signer.clone());
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let id = origin.election;
    board.open(origin.frozen()).await.unwrap();
    let a = Voter::new(pool, "valid/ballot-0").await;
    let b = Voter::new(pool, "valid/ballot-1").await;
    let raw = a.sign(&origin, a.payload.clone());
    let manifest = &origin.manifest["payload"];
    let manifest_hash = object_digest(manifest).unwrap();
    let voter_id: Uuid = serde_json::from_value(a.payload["voterId"].clone()).unwrap();
    let key = a.payload["publicKey"].as_str().unwrap();
    let ctx = BallotContext {
        election_id: id,
        manifest_hash: manifest_hash.clone(),
        voter_id,
        registered_public_key: Some(key.to_owned()),
        option_ids: manifest["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| serde_json::from_value(v["id"].clone()).unwrap())
            .collect(),
    };
    let verified = verify_ballot(&raw, &ctx).unwrap();
    let confirmed = signer.issue_record(&verified, 1, &manifest_hash).unwrap();
    let submission: Uuid = serde_json::from_value(a.payload["submissionId"].clone()).unwrap();
    // Mesmas três escritas da #381, com a primeira cédula como agregado de um único voto.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO election_participation(election_id,account_id,voter_id,submission_id,ballot_hash,accepted_at)
        VALUES($1,$2,$3,$4,$5,clock_timestamp())")
        .bind(id).bind(a.account).bind(voter_id).bind(submission).bind(&verified.ballot_hash).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO election_board_entry(election_id,entry_index,voter_id,previous_hash,entry_hash,signed_ballot,signed_entry,signed_receipt)
        VALUES($1,1,$2,$3,$4,$5,$6,$7)")
        .bind(id).bind(voter_id).bind(&manifest_hash).bind(&confirmed.entry_hash)
        .bind(&verified.envelope).bind(&confirmed.board_entry).bind(&confirmed.receipt).execute(&mut *tx).await.unwrap();
    sqlx::query(
        "UPDATE election_board SET head_hash=$2,voter_count=1,accumulator=$3 WHERE election_id=$1",
    )
    .bind(id)
    .bind(&confirmed.entry_hash)
    .bind(&a.payload["ciphertexts"])
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    MIGRATOR.run(pool).await.unwrap();
    let state = PgVoterRegistry::status(pool, id, a.account).await.unwrap();
    assert_eq!(state.voter_id, Some(voter_id));
    assert_eq!(state.public_key.as_deref(), Some(key));
    assert!(state.participated);
    #[cfg(feature = "test-support")]
    super::receipts::assert_legacy_http(pool, id, a.account, b.account, &state, &confirmed.receipt)
        .await;
    assert_eq!(
        board.recover(id, a.account).await.unwrap().unwrap().receipt,
        confirmed.receipt
    );
    let restored = board.record(id, identity(a.account), &raw).await.unwrap();
    assert!(restored.recovered);
    assert_eq!(restored.receipt, confirmed.receipt);
    let mut other = a.payload.clone();
    other["ciphertexts"] = b.payload["ciphertexts"].clone();
    assert!(matches!(
        board
            .record(id, identity(a.account), &a.sign(&origin, other))
            .await,
        Err(BoardError::SubmissionConflict)
    ));
    assert!(matches!(
        board
            .record(id, identity(b.account), &b.sign(&origin, b.payload.clone()))
            .await,
        Err(BoardError::NotEligible)
    ));
    let missing = PgVoterRegistry::status(pool, id, b.account).await.unwrap();
    assert_eq!(
        serde_json::to_value(missing).unwrap(),
        json!({"voterId":Value::Null,"publicKey":Value::Null,"participated":false})
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM election_voter_registration")
            .fetch_one(pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        board.snapshot(id).await.unwrap().entries,
        vec![confirmed.board_entry]
    );
    db.close().await;
}
