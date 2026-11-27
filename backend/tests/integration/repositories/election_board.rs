//! Aceite através da porta da aplicação, com commits reais em PostgreSQL.

#[cfg(feature = "test-support")]
mod ballots;
mod fixtures;
#[cfg(feature = "test-support")]
mod private_routes;
mod registration;
#[cfg(feature = "test-support")]
mod session;
#[cfg(feature = "test-support")]
mod tally;
#[cfg(feature = "test-support")]
mod three_voters;

use crate::support::TestDatabase;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::modules::elections::{
    application::{
        board::BoardError, ports::ElectionBoardRepository, services::board as board_service,
    },
    crypto_wire::object_digest,
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
use crypto_bigint::U2048;
use fixtures::{
    Origin, Voter, election_id, seed_academic_center, server, submit_vote, vector, vote_key,
};
use serde_json::{Value, json};
use tokio_stream::StreamExt;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn paged_export_keeps_its_original_snapshot_while_another_vote_commits() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgDatabase::new(db.pool.clone());
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let id = origin.election;

    assert!(matches!(
        board_service::publish(&store, &*signer, origin.frozen(false)).await,
        Err(BoardError::Storage(error))
            if error.downcast_ref::<sqlx::Error>()
                .and_then(sqlx::Error::as_database_error)
                .is_some_and(|error| error.is_foreign_key_violation())
    ));
    seed_academic_center(&db.pool).await;
    board_service::publish(&store, &*signer, origin.frozen(false))
        .await
        .unwrap();

    // Material sintético compartilhado para testar páginas e isolamento da leitura.
    let cast = async || {
        let voter = Voter::with_pem(
            &db.pool,
            "valid/ballot-0",
            include_str!("../../fixtures/jwt/test-only-rsa-private.pem"),
        )
        .await;
        let raw = voter.sign(&origin, voter.payload.clone());
        submit_vote(&store, id, voter.admitted(), &raw)
            .await
            .unwrap();
    };
    for _ in 0..33 {
        cast().await;
    }
    let database = PgDatabase::new(db.pool.clone());
    let mut stream = board_service::stream_snapshot(&database, election_id(id))
        .await
        .unwrap();
    cast().await;
    let mut bytes = Vec::new();
    while let Some(part) = stream.next().await {
        bytes.extend(part.unwrap());
    }
    let snapshot: Value = serde_json::from_slice(&bytes).unwrap();
    let entries = snapshot["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 33);
    assert_eq!(
        snapshot["checkpoint"]["payload"]["entryHash"],
        object_digest(&entries[32]["payload"]).unwrap()
    );
    let latest = board_service::read_checkpoint(&database, election_id(id))
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
    );
    let app_check = AppCheckVerifier::new(
        "1234567890".into(),
        vec!["1:1234567890:android:capp".into()],
        http,
    )
    .unwrap();
    let router = create_router(
        AppState::new(PgDatabase::new(pool.clone()), clients),
        app_check,
    );
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
    let store = PgDatabase::new(db.pool.clone());
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let id = origin.election;
    seed_academic_center(&db.pool).await;
    let empty = board_service::publish(&store, &*signer, origin.frozen(false))
        .await
        .unwrap();
    assert!(empty.entries.is_empty() && empty.checkpoint.is_none());
    assert_eq!(
        board_service::read_checkpoint(&PgDatabase::new(db.pool.clone()), election_id(id))
            .await
            .unwrap(),
        origin.manifest
    );

    let a = Voter::new(&db.pool, "valid/ballot-0").await;
    let b = Voter::new(&db.pool, "valid/ballot-1").await;
    let first = a.sign(&origin, a.payload.clone());
    let second = b.sign(&origin, b.payload.clone());

    // Falha diferida até COMMIT, depois de todas as escritas da transação.
    sqlx::raw_sql(
        "CREATE FUNCTION fail_board_commit() RETURNS trigger LANGUAGE plpgsql AS
        $$ BEGIN RAISE EXCEPTION 'injected commit failure'; END $$;
        CREATE CONSTRAINT TRIGGER fail_test_commit AFTER INSERT ON election_vote
        DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION fail_board_commit()",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    assert!(matches!(
        submit_vote(&store, id, a.admitted(), &first).await,
        Err(BoardError::Storage(_))
    ));
    assert!(
        board_service::read_own_receipt(&PgDatabase::new(db.pool.clone()), vote_key(id, a.account))
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        serde_json::to_value(store.snapshot(election_id(id)).await.unwrap()).unwrap(),
        serde_json::to_value(&empty).unwrap()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM election_vote")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    sqlx::query("DROP TRIGGER fail_test_commit ON election_vote")
        .execute(&db.pool)
        .await
        .unwrap();

    let (one, retry, other, observed) = tokio::join!(
        submit_vote(&store, id, a.admitted(), &first),
        submit_vote(&store, id, a.admitted(), &first),
        submit_vote(&store, id, b.admitted(), &second),
        store.snapshot(election_id(id))
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

    let board = store.snapshot(election_id(id)).await.unwrap();
    assert_eq!(board.entries.len(), 2);
    let exported =
        public_evidence(&db.pool, &format!("/elections/{id}/board"), StatusCode::OK).await;
    assert_eq!(exported, serde_json::to_value(&board).unwrap());
    assert_eq!(
        one.entry["payload"]["ballot"]["payload"]["accountId"],
        json!(a.account)
    );
    assert_eq!(
        other.entry["payload"]["ballot"]["payload"]["accountId"],
        json!(b.account)
    );
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
    let invalid = public_evidence(
        &db.pool,
        &format!("/elections/{}/board", Uuid::nil()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(invalid["error"]["code"], "bad_request");

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
    // O corpus contém o produto esperado, calculado independentemente do store Rust.
    let accumulated: Vec<u8> =
        sqlx::query_scalar("SELECT accumulator FROM election_board WHERE election_id=$1")
            .bind(id)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    let expected_components = vector("valid/aggregate")["ciphertexts"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|cipher| [&cipher["a"], &cipher["b"]]);
    let mut expected = Vec::new();
    for component in expected_components {
        let number = U2048::from_str_radix_vartime(component.as_str().unwrap(), 10).unwrap();
        expected.extend_from_slice(&number.to_be_bytes());
    }
    assert_eq!(accumulated, expected);

    // Novo adaptador e novas conexões reconstroem a resposta perdida após o commit.
    let fresh_pool = sqlx::postgres::PgPoolOptions::new()
        .connect_with((*db.pool.connect_options()).clone())
        .await
        .unwrap();
    let restarted = PgDatabase::new(fresh_pool.clone());
    let saved = board_service::read_own_receipt(&restarted, vote_key(id, a.account))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved, one.receipt);
    #[cfg(feature = "test-support")]
    private_routes::assert_own_receipts(
        &fresh_pool,
        id,
        [(a.account, &one.receipt), (b.account, &other.receipt)],
    )
    .await;
    assert!(
        board_service::read_own_receipt(&restarted, vote_key(id, Uuid::new_v4()))
            .await
            .unwrap()
            .is_none()
    );
    let resigned = a.sign(&origin, a.payload.clone());
    assert_ne!(resigned, first);
    assert_eq!(
        submit_vote(&restarted, id, a.admitted(), &resigned)
            .await
            .unwrap()
            .receipt,
        one.receipt
    );
    let mut different = a.payload.clone();
    different["ciphertexts"] = b.payload["ciphertexts"].clone();
    assert!(matches!(
        submit_vote(&restarted, id, a.admitted(), &a.sign(&origin, different)).await,
        Err(BoardError::AlreadyParticipated)
    ));
    fresh_pool.close().await;
    db.close().await;
}

#[tokio::test]
async fn close_freezes_the_accepted_vote_and_preserves_its_receipt() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgDatabase::new(db.pool.clone());
    let voter = Voter::new(&db.pool, "valid/ballot-0").await;
    let late_voter = Voter::new(&db.pool, "valid/ballot-1").await;
    let end = Utc::now() + Duration::seconds(15);
    let origin = Origin::during(&signer, end);
    let id = origin.election;
    board_service::publish(&store, &*signer, origin.frozen(false))
        .await
        .unwrap();
    assert!(matches!(
        board_service::close(&store, &*signer, election_id(id)).await,
        Err(BoardError::NotYetEnded)
    ));
    let ballot = voter.sign(&origin, voter.payload.clone());
    assert!(matches!(
        submit_vote(&store, id, late_voter.admitted(), &ballot).await,
        Err(BoardError::VoterMismatch)
    ));
    let mut competing = voter.payload.clone();
    competing["ciphertexts"] = late_voter.payload["ciphertexts"].clone();
    let competing = voter.sign(&origin, competing);
    let (left, right) = tokio::join!(
        submit_vote(&store, id, voter.admitted(), &ballot),
        submit_vote(&store, id, voter.admitted(), &competing)
    );
    let (accepted, committed_ballot) = match (left, right) {
        (Ok(record), Err(BoardError::AlreadyParticipated)) => (record, ballot),
        (Err(BoardError::AlreadyParticipated), Ok(record)) => (record, competing),
        result => {
            panic!("Duas operações da mesma conta precisam disputar um único aceite: {result:?}")
        }
    };
    assert!(matches!(
        submit_vote(
            &store,
            id,
            voter.admitted(),
            &late_voter.sign(&origin, voter.payload.clone())
        )
        .await,
        Err(BoardError::AlreadyParticipated)
    ));
    sqlx::query("SELECT pg_sleep(GREATEST(0, EXTRACT(EPOCH FROM ($1::timestamptz - clock_timestamp())))::float8)")
        .bind(end).execute(&db.pool).await.unwrap();
    let closed = board_service::close(&store, &*signer, election_id(id))
        .await
        .unwrap();
    let closure = &closed.closure.as_ref().unwrap()["payload"];
    assert_eq!(closure["voterCount"], 1);
    assert_eq!(
        closure["headHash"],
        accepted.receipt["payload"]["entryHash"]
    );
    let aggregate = &closed.aggregate.as_ref().unwrap()["payload"];
    assert_eq!(aggregate["closureHash"], object_digest(closure).unwrap());
    let committed: Value = serde_json::from_slice(&committed_ballot).unwrap();
    assert_eq!(
        aggregate["ciphertexts"],
        committed["payload"]["ciphertexts"]
    );
    assert_eq!(
        board_service::close(&store, &*signer, election_id(id))
            .await
            .unwrap()
            .closure,
        closed.closure
    );
    sqlx::query("UPDATE institutional_verification SET expires_at = now() - interval '1 hour' WHERE user_id = $1")
        .bind(voter.account)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        submit_vote(&store, id, voter.admitted(), &committed_ballot)
            .await
            .unwrap()
            .receipt,
        accepted.receipt
    );
    let late = late_voter.sign(&origin, late_voter.payload.clone());
    assert!(matches!(
        submit_vote(&store, id, late_voter.admitted(), &late).await,
        Err(BoardError::Closed)
    ));
    db.close().await;
}
