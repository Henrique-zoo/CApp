//! Aceite atravÃ©s da porta da aplicaÃ§Ã£o, com commits reais em PostgreSQL.

mod fixtures;
#[cfg(feature = "test-support")]
mod receipts;
mod registration;

use crate::support::TestDatabase;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::modules::elections::{
    application::board::{BoardError, ElectionBoard},
    crypto_wire::object_digest,
    infrastructure::PgElectionBoard,
};
use backend::modules::identity::infrastructure::{
    firebase::{AppCheckVerifier, FirebaseTokenVerifier},
    microsoft::MicrosoftGraphClient,
};
use backend::{
    api::{
        router::create_router,
        state::{AppState, Clients},
    },
    infrastructure::database::PgDatabase,
};
use chrono::{Duration, Utc};
use fixtures::{Origin, Voter, identity, server, vector};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn paged_export_keeps_its_original_snapshot_while_another_vote_commits() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgElectionBoard::new(db.pool.clone(), Arc::clone(&signer));
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let id = origin.election;
    store.open(origin.frozen()).await.unwrap();
    // Material sintÃ©tico compartilhado para testar pÃ¡ginas e isolamento da leitura.
    let public_key = signer.public_key_base64().unwrap();
    let manifest_hash = object_digest(&origin.manifest["payload"]).unwrap();
    let cast = async || {
        let account_id = Uuid::new_v4();
        let voter_id = Uuid::new_v4();
        sqlx::query("INSERT INTO app_user (id,firebase_uid) VALUES ($1,$2)")
            .bind(account_id)
            .bind(account_id.to_string())
            .execute(&db.pool)
            .await
            .unwrap();
        let mut ballot = vector("valid/ballot-0").clone();
        ballot["manifestHash"] = json!(&manifest_hash);
        ballot["voterId"] = json!(voter_id);
        ballot["submissionId"] = json!(Uuid::new_v4());
        ballot["publicKey"] = json!(&public_key);
        fixtures::seed_registration(&db.pool, id, account_id, voter_id, &public_key).await;
        let raw = serde_json::to_vec(&signer.sign_payload(&ballot).unwrap()).unwrap();
        store.record(id, identity(account_id), &raw).await.unwrap();
    };
    for _ in 0..33 {
        cast().await;
    }
    let mut receiver = PgElectionBoard::stream_snapshot(&db.pool, id)
        .await
        .unwrap();
    cast().await;
    let mut bytes = Vec::new();
    while let Some(part) = receiver.recv().await {
        bytes.extend(part.unwrap());
    }
    let snapshot: Value = serde_json::from_slice(&bytes).unwrap();
    let entries = snapshot["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 33);
    assert_eq!(
        snapshot["checkpoint"]["payload"]["entryHash"],
        object_digest(&entries[32]["payload"]).unwrap()
    );
    let latest = PgElectionBoard::read_checkpoint(&db.pool, id)
        .await
        .unwrap();
    assert_eq!(latest["payload"]["index"], 34);
    db.close().await;
}

async fn public_evidence(pool: &sqlx::PgPool, uri: &str, expected: StatusCode) -> Value {
    let http = reqwest::Client::new();
    let clients = Clients::new(
        FirebaseTokenVerifier::new("capp-test".into(), http.clone()).unwrap(),
        MicrosoftGraphClient::new(http.clone()),
        AppCheckVerifier::new(
            "1234567890".into(),
            vec!["1:1234567890:android:capp".into()],
            http,
        )
        .unwrap(),
    );
    let router = create_router(AppState::new(PgDatabase::new(pool.clone()), clients));
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), expected);
    serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
}

#[tokio::test]
async fn rollback_concurrency_and_recovery_preserve_one_chain() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgElectionBoard::new(db.pool.clone(), Arc::clone(&signer));
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let id = origin.election;
    let empty = store.open(origin.frozen()).await.unwrap();
    assert!(empty.entries.is_empty() && empty.checkpoint.is_none());
    assert_eq!(
        PgElectionBoard::read_checkpoint(&db.pool, id)
            .await
            .unwrap(),
        origin.manifest
    );

    let a = Voter::new(&db.pool, "valid/ballot-0").await;
    let b = Voter::new(&db.pool, "valid/ballot-1").await;
    a.seed_registration(&db.pool, id).await;
    b.seed_registration(&db.pool, id).await;
    let first = a.sign(&origin, a.payload.clone());
    let second = b.sign(&origin, b.payload.clone());

    // Falha diferida atÃ© COMMIT, depois de todas as escritas da transaÃ§Ã£o.
    sqlx::raw_sql(
        "CREATE FUNCTION fail_board_commit() RETURNS trigger LANGUAGE plpgsql AS
        $$ BEGIN RAISE EXCEPTION 'injected commit failure'; END $$;
        CREATE CONSTRAINT TRIGGER fail_test_commit AFTER INSERT ON election_board_entry
        DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION fail_board_commit()",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    assert!(matches!(
        store.record(id, identity(a.account), &first).await,
        Err(BoardError::Storage(_))
    ));
    assert!(store.recover(id, a.account).await.unwrap().is_none());
    assert_eq!(
        serde_json::to_value(store.snapshot(id).await.unwrap()).unwrap(),
        serde_json::to_value(&empty).unwrap()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM election_participation")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    sqlx::query("DROP TRIGGER fail_test_commit ON election_board_entry")
        .execute(&db.pool)
        .await
        .unwrap();

    let (one, retry, other, observed) = tokio::join!(
        store.record(id, identity(a.account), &first),
        store.record(id, identity(a.account), &first),
        store.record(id, identity(b.account), &second),
        store.snapshot(id)
    );
    let (one, retry, other, observed) = (
        one.unwrap(),
        retry.unwrap(),
        other.unwrap(),
        observed.unwrap(),
    );
    assert_ne!(one.recovered, retry.recovered);
    assert!(!other.recovered);
    assert_eq!(one.entry, retry.entry);
    assert_eq!(one.receipt, retry.receipt);
    assert_eq!(
        observed
            .checkpoint
            .as_ref()
            .map(|c| c["payload"]["index"].as_u64().unwrap()),
        observed
            .entries
            .last()
            .map(|e| e["payload"]["index"].as_u64().unwrap())
    );

    let board = store.snapshot(id).await.unwrap();
    assert_eq!(board.entries.len(), 2);
    let exported =
        public_evidence(&db.pool, &format!("/elections/{id}/board"), StatusCode::OK).await;
    assert_eq!(exported, serde_json::to_value(&board).unwrap());
    assert!(!exported.to_string().contains(&a.account.to_string()));
    let remote_anchor = public_evidence(
        &db.pool,
        &format!("/elections/{id}/board/checkpoint"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(Some(&remote_anchor), board.checkpoint.as_ref());
    let missing = public_evidence(
        &db.pool,
        &format!("/elections/{}/board", Uuid::new_v4()),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing["error"]["code"], "board_not_found");
    for (index, entry) in board.entries.iter().enumerate() {
        let previous = if index == 0 {
            &origin.manifest
        } else {
            &board.entries[index - 1]
        };
        assert_eq!(entry["payload"]["index"], index + 1);
        assert_eq!(
            entry["payload"]["previousHash"],
            object_digest(&previous["payload"]).unwrap()
        );
    }
    // O corpus contÃ©m o produto esperado, calculado independentemente do store Rust.
    let accumulated: Value =
        sqlx::query_scalar("SELECT accumulator FROM election_board WHERE election_id=$1")
            .bind(id)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(accumulated, vector("valid/aggregate")["ciphertexts"]);

    // Novo adaptador e novas conexÃµes reconstroem a resposta perdida apÃ³s o commit.
    let fresh_pool = sqlx::postgres::PgPoolOptions::new()
        .connect_with((*db.pool.connect_options()).clone())
        .await
        .unwrap();
    let restarted = PgElectionBoard::new(fresh_pool.clone(), signer);
    let saved = restarted.recover(id, a.account).await.unwrap().unwrap();
    assert_eq!(saved.receipt, one.receipt);
    #[cfg(feature = "test-support")]
    receipts::assert_own_receipts(
        &fresh_pool,
        id,
        [(a.account, &one.receipt), (b.account, &other.receipt)],
    )
    .await;
    assert!(
        restarted
            .recover(id, Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
    let resigned = a.sign(&origin, a.payload.clone());
    assert_ne!(resigned, first);
    assert_eq!(
        restarted
            .record(id, identity(a.account), &resigned)
            .await
            .unwrap()
            .receipt,
        one.receipt
    );
    let mut different = a.payload.clone();
    different["ciphertexts"] = b.payload["ciphertexts"].clone();
    assert!(matches!(
        restarted
            .record(id, identity(a.account), &a.sign(&origin, different))
            .await,
        Err(BoardError::SubmissionConflict)
    ));
    let mut another = a.payload.clone();
    another["submissionId"] = json!(Uuid::new_v4());
    assert!(matches!(
        restarted
            .record(id, identity(a.account), &a.sign(&origin, another))
            .await,
        Err(BoardError::AlreadyParticipated)
    ));

    fresh_pool.close().await;
    db.close().await;
}

#[tokio::test]
async fn close_freezes_the_accepted_vote_and_preserves_its_receipt() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgElectionBoard::new(db.pool.clone(), Arc::clone(&signer));
    let voter = Voter::new(&db.pool, "valid/ballot-0").await;
    let late_voter = Voter::new(&db.pool, "valid/ballot-1").await;
    let end = Utc::now() + Duration::seconds(15);
    let origin = Origin::during(&signer, end);
    let id = origin.election;
    store.open(origin.frozen()).await.unwrap();
    assert!(matches!(
        store.close(id).await,
        Err(BoardError::NotYetEnded)
    ));
    let ballot = voter.sign(&origin, voter.payload.clone());
    voter.seed_registration(&db.pool, id).await;
    late_voter.seed_registration(&db.pool, id).await;
    assert!(
        store
            .record(id, identity(late_voter.account), &ballot)
            .await
            .is_err()
    );
    let mut competing = voter.payload.clone();
    competing["submissionId"] = json!(Uuid::new_v4());
    let competing = voter.sign(&origin, competing);
    let (left, right) = tokio::join!(
        store.record(id, identity(voter.account), &ballot),
        store.record(id, identity(voter.account), &competing)
    );
    let (accepted, committed_ballot) = match (left, right) {
        (Ok(record), Err(BoardError::AlreadyParticipated)) => (record, ballot),
        (Err(BoardError::AlreadyParticipated), Ok(record)) => (record, competing),
        result => {
            panic!("Duas operaÃ§Ãµes da mesma conta precisam disputar um Ãºnico aceite: {result:?}")
        }
    };
    sqlx::query("SELECT pg_sleep(GREATEST(0, EXTRACT(EPOCH FROM ($1::timestamptz - clock_timestamp())))::float8)")
        .bind(end).execute(&db.pool).await.unwrap();
    let closed = store.close(id).await.unwrap();
    let closure = &closed.closure.as_ref().unwrap()["payload"];
    assert_eq!(closure["voterCount"], 1);
    assert_eq!(
        closure["headHash"],
        accepted.receipt["payload"]["entryHash"]
    );
    let aggregate = &closed.aggregate.as_ref().unwrap()["payload"];
    assert_eq!(aggregate["closureHash"], object_digest(closure).unwrap());
    assert_eq!(aggregate["ciphertexts"], voter.payload["ciphertexts"]);
    assert_eq!(store.close(id).await.unwrap().closure, closed.closure);
    sqlx::query("UPDATE institutional_verification SET expires_at=now()-interval '1 second' WHERE user_id=$1")
        .bind(voter.account).execute(&db.pool).await.unwrap();
    assert_eq!(
        store
            .record(id, identity(voter.account), &committed_ballot)
            .await
            .unwrap()
            .receipt,
        accepted.receipt
    );
    let late = late_voter.sign(&origin, late_voter.payload.clone());
    assert!(matches!(
        store.record(id, identity(late_voter.account), &late).await,
        Err(BoardError::Closed)
    ));
    db.close().await;
}
