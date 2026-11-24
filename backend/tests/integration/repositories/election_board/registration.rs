//! Exercita a política de cadastro e a associação do curso no CA com PostgreSQL.

use backend::infrastructure::database::PgDatabase;
use backend::modules::elections::{
    application::{
        board::BoardError, error::RegistrationError, ports::VoterKeyProofVerifier,
        services::board as board_service, services::voter_registration,
    },
    infrastructure::RsaPssVoterKeyProofVerifier,
};
use chrono::{Duration, Utc};
use serde_json::json;

use super::fixtures::{Origin, Voter, account_id, election_id, server, submit_vote};
use crate::support::TestDatabase;

#[tokio::test]
async fn registration_uses_the_authenticated_account_and_the_courses_of_the_center() {
    let db = TestDatabase::new().await;
    let signer = server();
    let database = PgDatabase::new(db.pool.clone());
    let verifier = RsaPssVoterKeyProofVerifier;
    let voter = Voter::new(&db.pool, "valid/ballot-0").await;
    let other = Voter::new(&db.pool, "valid/ballot-1").await;
    let now = Utc::now();
    let origin = Origin::scheduled(&signer, now + Duration::hours(1), now + Duration::hours(2));
    board_service::publish(&database, &*signer, origin.frozen(true))
        .await
        .unwrap();
    let election = origin.election;
    let proof = voter.registration_proof(election);

    let status =
        voter_registration::status(&database, election_id(election), account_id(voter.account))
            .await
            .unwrap();
    assert!(status.requires_registration);
    assert!(status.public_key.is_none());
    assert!(matches!(
        voter_registration::register(
            &database,
            &verifier,
            election_id(election),
            other.admitted(),
            voter.registration_proof_typed(election)
        )
        .await,
        Err(RegistrationError::InvalidProof)
    ));

    sqlx::query(
        "UPDATE institutional_verification SET course_name = 'Outro curso' WHERE user_id = $1",
    )
    .bind(voter.account)
    .execute(&db.pool)
    .await
    .unwrap();
    assert!(matches!(
        voter_registration::register(
            &database,
            &verifier,
            election_id(election),
            voter.admitted(),
            voter.registration_proof_typed(election)
        )
        .await,
        Err(RegistrationError::NotEligible)
    ));
    sqlx::query(
        "INSERT INTO academic_center_course (academic_center_id, course_name)
         SELECT academic_center_id, 'Outro curso'
         FROM election WHERE election_id = $1",
    )
    .bind(election)
    .execute(&db.pool)
    .await
    .unwrap();

    #[cfg(feature = "test-support")]
    super::private_routes::assert_own_registration(
        &db.pool,
        election,
        voter.account,
        &proof,
        &voter.payload["publicKey"],
    )
    .await;

    let accepted = voter_registration::register(
        &database,
        &verifier,
        election_id(election),
        voter.admitted(),
        voter.registration_proof_typed(election),
    )
    .await
    .unwrap();
    assert_eq!(accepted.account_id, Some(account_id(voter.account)));
    assert!(accepted.public_key.is_some());
    assert!(!accepted.participated);
    let mut renewed = voter.admitted();
    renewed.microsoft_object_id = None;
    assert_eq!(
        voter_registration::register(
            &database,
            &verifier,
            election_id(election),
            renewed,
            voter.registration_proof_typed(election)
        )
        .await
        .unwrap(),
        accepted
    );
    db.close().await;
}

#[tokio::test]
async fn board_requires_the_registered_key_only_when_that_policy_is_selected() {
    let db = TestDatabase::new().await;
    let signer = server();
    let database = PgDatabase::new(db.pool.clone());
    let verifier = RsaPssVoterKeyProofVerifier;
    let voter = Voter::new(&db.pool, "valid/ballot-0").await;
    let other = Voter::new(&db.pool, "valid/ballot-1").await;
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let election = origin.election;
    board_service::publish(&database, &*signer, origin.frozen(true))
        .await
        .unwrap();
    let ballot = voter.sign(&origin, voter.payload.clone());

    assert!(matches!(
        board_service::publish(&database, &*signer, origin.frozen(false)).await,
        Err(BoardError::AlreadyExists)
    ));
    assert!(matches!(
        submit_vote(&database, election, voter.admitted(), &ballot).await,
        Err(BoardError::RegistrationRequired)
    ));
    assert!(matches!(
        voter_registration::register(
            &database,
            &verifier,
            election_id(election),
            voter.admitted(),
            voter.registration_proof_typed(election)
        )
        .await,
        Err(RegistrationError::Closed)
    ));
    let key = verifier
        .verify(
            voter.registration_proof_typed(election),
            election_id(election),
            account_id(voter.account),
        )
        .unwrap();
    // Representa um cadastro confirmado antes do início usando a chave do teste.
    sqlx::query(
        "INSERT INTO election_voter_registration (election_id, account_id, public_key, registered_at)
         SELECT election_id, $2, $3, starts_at - interval '1 second'
         FROM election WHERE election_id = $1",
    )
    .bind(election)
    .bind(voter.account)
    .bind(key.as_bytes())
    .execute(&db.pool)
    .await
    .unwrap();

    assert!(matches!(
        submit_vote(
            &database,
            election,
            voter.admitted(),
            &other.sign(&origin, voter.payload.clone())
        )
        .await,
        Err(BoardError::RegisteredKeyMismatch)
    ));
    sqlx::query(
        "UPDATE institutional_verification SET course_name = 'Outro curso' WHERE user_id = $1",
    )
    .bind(voter.account)
    .execute(&db.pool)
    .await
    .unwrap();
    assert!(matches!(
        submit_vote(&database, election, voter.admitted(), &ballot).await,
        Err(BoardError::NotEligible)
    ));
    sqlx::query("UPDATE institutional_verification SET course_name = $2 WHERE user_id = $1")
        .bind(voter.account)
        .bind(super::fixtures::COURSE)
        .execute(&db.pool)
        .await
        .unwrap();
    let accepted = submit_vote(&database, election, voter.admitted(), &ballot)
        .await
        .unwrap();
    assert_eq!(
        accepted.entry["payload"]["ballot"]["payload"]["accountId"],
        json!(voter.account)
    );
    db.close().await;
}
