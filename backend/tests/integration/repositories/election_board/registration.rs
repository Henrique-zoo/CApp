//! Cadastro, mudança de vínculo institucional e reenvio exercem o banco real.

use crate::support::TestDatabase;
use backend::modules::elections::{
    application::{
        board::{BoardError, ElectionBoard},
        registration::{RegistrationError, VoterIdentity},
    },
    infrastructure::{PgElectionBoard, PgVoterRegistry},
};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::fixtures::{
    COURSE, Origin, Voter, identity, institutional_evidence, registration_proof, server,
};

async fn account(pool: &PgPool) -> Uuid {
    let account = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user(id,firebase_uid) VALUES($1,$2)")
        .bind(account)
        .bind(account.to_string())
        .execute(pool)
        .await
        .unwrap();
    account
}

#[tokio::test]
async fn registration_checks_proof_course_period_and_authenticated_owner() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgElectionBoard::new(db.pool.clone(), signer.clone());
    let origin = Origin::scheduled(
        &signer,
        Utc::now() + Duration::hours(1),
        Utc::now() + Duration::hours(3),
    );
    let id = origin.election;
    let pool = &db.pool;
    let a = Voter::new(pool, "valid/ballot-0").await;
    let b = account(pool).await;
    let proof = a.proof(id);
    store.open(origin.frozen()).await.unwrap();
    let empty = PgVoterRegistry::status(pool, id, a.account).await.unwrap();
    assert!(empty.voter_id.is_none() && empty.public_key.is_none() && !empty.participated);
    assert!(matches!(
        PgVoterRegistry::status(pool, Uuid::new_v4(), a.account).await,
        Err(RegistrationError::NotFound)
    ));
    assert!(matches!(
        PgVoterRegistry::register(pool, id, identity(a.account), &proof).await,
        Err(RegistrationError::NotEligible)
    ));
    institutional_evidence(pool, a.account).await;
    PgVoterRegistry::configure_course(pool, id, COURSE)
        .await
        .unwrap();
    PgVoterRegistry::configure_course(pool, id, COURSE)
        .await
        .unwrap();
    assert!(matches!(
        PgVoterRegistry::configure_course(pool, id, "Outro curso").await,
        Err(BoardError::AlreadyExists)
    ));
    assert!(matches!(
        PgVoterRegistry::register(pool, id, identity(b), &proof).await,
        Err(RegistrationError::InvalidProof)
    ));
    assert!(matches!(
        PgVoterRegistry::register(pool, Uuid::new_v4(), identity(a.account), &proof).await,
        Err(RegistrationError::InvalidProof)
    ));
    let other_proof = registration_proof(&signer, id, a.account);
    let mut forged: serde_json::Value = serde_json::from_slice(&proof).unwrap();
    forged["signature"] =
        serde_json::from_slice::<serde_json::Value>(&other_proof).unwrap()["signature"].clone();
    assert!(matches!(
        PgVoterRegistry::register(
            pool,
            id,
            identity(a.account),
            &serde_json::to_vec(&forged).unwrap()
        )
        .await,
        Err(RegistrationError::InvalidProof)
    ));
    let b_proof = registration_proof(&signer, id, b);
    assert!(matches!(
        PgVoterRegistry::register(pool, id, identity(b), &b_proof).await,
        Err(RegistrationError::NotEligible)
    ));
    institutional_evidence(pool, b).await;
    for invalid in [
        "course_name='Outro curso'",
        "expires_at=now()-interval '1 minute'",
        "microsoft_student_group_id=gen_random_uuid()",
        "student_membership_confirmed=false,course_name=NULL",
    ] {
        sqlx::query(&format!(
            "UPDATE institutional_verification SET {invalid} WHERE user_id=$1"
        ))
        .bind(b)
        .execute(pool)
        .await
        .unwrap();
        assert!(matches!(
            PgVoterRegistry::register(pool, id, identity(b), &b_proof).await,
            Err(RegistrationError::NotEligible)
        ));
        sqlx::query("DELETE FROM institutional_verification WHERE user_id=$1")
            .bind(b)
            .execute(pool)
            .await
            .unwrap();
        institutional_evidence(pool, b).await;
    }
    let (left, right) = tokio::join!(
        PgVoterRegistry::register(pool, id, identity(a.account), &proof),
        PgVoterRegistry::register(pool, id, identity(a.account), &other_proof)
    );
    let confirmed = match (left, right) {
        (Ok(s), Err(RegistrationError::KeyConflict))
        | (Err(RegistrationError::KeyConflict), Ok(s)) => s,
        pair => panic!("Esperados um cadastro e um conflito: {pair:?}"),
    };
    assert!(confirmed.voter_id.is_some() && !confirmed.participated);
    assert_ne!(confirmed.voter_id, Some(a.account));
    assert_eq!(
        PgVoterRegistry::status(pool, id, a.account).await.unwrap(),
        confirmed
    );
    assert_eq!(PgVoterRegistry::status(pool, id, b).await.unwrap(), empty);
    #[cfg(feature = "test-support")]
    {
        let fresh = account(pool).await;
        institutional_evidence(pool, fresh).await;
        super::receipts::assert_registration_http(pool, id, fresh, b, &signer).await;
    }
    db.close().await;
}

#[tokio::test]
async fn new_votes_require_live_evidence_but_confirmed_votes_keep_their_receipt() {
    let db = TestDatabase::new().await;
    let pool = &db.pool;
    let signer = server();
    let store = PgElectionBoard::new(pool.clone(), signer.clone());
    let mut a = Voter::new(pool, "valid/ballot-0").await;
    let b = account(pool).await;
    institutional_evidence(pool, a.account).await;
    institutional_evidence(pool, b).await;
    let start = Utc::now() + Duration::seconds(12);
    let origin = Origin::scheduled(&signer, start, start + Duration::hours(1));
    let id = origin.election;
    store.open(origin.frozen()).await.unwrap();
    PgVoterRegistry::configure_course(pool, id, COURSE)
        .await
        .unwrap();
    let proof = a.proof(id);
    let registered = PgVoterRegistry::register(pool, id, identity(a.account), &proof)
        .await
        .unwrap();
    a.payload["voterId"] = json!(registered.voter_id.unwrap());
    let ballot = a.sign(&origin, a.payload.clone());
    assert!(matches!(
        store.record(id, identity(a.account), &ballot).await,
        Err(BoardError::VotingNotStarted)
    ));
    sqlx::query("SELECT pg_sleep(GREATEST(0,EXTRACT(EPOCH FROM ($1::timestamptz-clock_timestamp())))::float8)")
        .bind(start).execute(pool).await.unwrap();
    let b_proof = registration_proof(&signer, id, b);
    assert!(matches!(
        PgVoterRegistry::register(pool, id, identity(b), &b_proof).await,
        Err(RegistrationError::RegistrationClosed)
    ));
    assert_eq!(
        PgVoterRegistry::register(pool, id, identity(a.account), &proof)
            .await
            .unwrap(),
        registered
    );
    assert!(matches!(
        store.record(id, identity(b), &ballot).await,
        Err(BoardError::NotEligible)
    ));
    sqlx::query("UPDATE institutional_verification SET course_name='Outro curso' WHERE user_id=$1")
        .bind(a.account)
        .execute(pool)
        .await
        .unwrap();
    assert!(matches!(
        store.record(id, identity(a.account), &ballot).await,
        Err(BoardError::NotEligible)
    ));
    assert!(
        !PgVoterRegistry::status(pool, id, a.account)
            .await
            .unwrap()
            .participated
    );
    sqlx::query("UPDATE institutional_verification SET course_name=$2 WHERE user_id=$1")
        .bind(a.account)
        .bind(COURSE)
        .execute(pool)
        .await
        .unwrap();
    let relinked = VoterIdentity {
        account_id: a.account,
        microsoft_object_id: Some(Uuid::new_v4()),
    };
    assert!(matches!(
        store.record(id, relinked, &ballot).await,
        Err(BoardError::NotEligible)
    ));
    let accepted = store
        .record(id, identity(a.account), &ballot)
        .await
        .unwrap();
    sqlx::query("UPDATE institutional_verification SET expires_at=now()-interval '1 minute' WHERE user_id=$1")
        .bind(a.account).execute(pool).await.unwrap();
    let retry = store.record(id, relinked, &ballot).await.unwrap();
    assert!(retry.recovered);
    assert_eq!(retry.receipt, accepted.receipt);
    assert!(
        PgVoterRegistry::status(pool, id, a.account)
            .await
            .unwrap()
            .participated
    );
    db.close().await;
}
