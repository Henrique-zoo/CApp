//! Do cadastro ao recibo pela API pública do CApp, com PostgreSQL real.

use super::{
    fixtures::{COURSE, Origin, Voter, institutional_evidence, server, vector},
    public_evidence,
    session::SessionApi,
};
use crate::support::TestDatabase;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::modules::elections::{
    application::board::ElectionBoard,
    crypto_wire::{BallotContext, object_digest, verify_record},
    infrastructure::{PgElectionBoard, PgVoterRegistry},
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use uuid::Uuid;

fn ballot_request(uri: &str, envelope: &[u8]) -> Request<Body> {
    Request::post(uri)
        .header("content-type", "application/json")
        .body(Body::from(envelope.to_vec()))
        .unwrap()
}

#[tokio::test]
async fn authenticated_vote_is_recorded_once_and_included_in_the_public_board() {
    let db = TestDatabase::new().await;
    let signer = server();
    let store = PgElectionBoard::new(db.pool.clone(), signer.clone());
    let api = SessionApi::with_signer(&db.pool, Some(signer.clone())).await;
    let mut voter = Voter::new(&db.pool, "valid/ballot-0").await;
    institutional_evidence(&db.pool, voter.account).await;
    let starts_at = Utc::now() + Duration::seconds(15);
    let origin = Origin::scheduled(&signer, starts_at, starts_at + Duration::hours(1));
    let id = origin.election;
    store.open(origin.frozen()).await.unwrap();
    PgVoterRegistry::configure_course(&db.pool, id, COURSE)
        .await
        .unwrap();

    // O vínculo nasce na rota do eleitor antes da abertura.
    let uri = format!("/elections/{id}/registration/me");
    let register = Request::put(&uri)
        .header("content-type", "application/json")
        .body(Body::from(voter.proof(id)))
        .unwrap();
    let (status, registration) = api.send(voter.account, register).await;
    assert_eq!(status, StatusCode::OK, "{registration}");
    assert_eq!(registration["participated"], false);
    let voter_id: Uuid = serde_json::from_value(registration["voterId"].clone()).unwrap();
    voter.payload["voterId"] = json!(voter_id);
    let raw = voter.sign(&origin, voter.payload.clone());
    let uri = format!("/elections/{id}/ballots");
    let (status, error) = api.send(voter.account, ballot_request(&uri, &raw)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["code"], "voting_not_started");
    sqlx::query("SELECT pg_sleep(GREATEST(0,EXTRACT(EPOCH FROM ($1::timestamptz-clock_timestamp())))::float8)")
        .bind(starts_at).execute(&db.pool).await.unwrap();

    // A rota recebe uma cédula íntegra somente em nome da sessão elegível.
    let (status, error) = api.send_attested(ballot_request(&uri, &raw)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{error}");
    let no_signer = SessionApi::new(&db.pool).await;
    let (status, error) = no_signer
        .send(voter.account, ballot_request(&uri, &raw))
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{error}");
    let (status, error) = api
        .send_with_microsoft_identity(voter.account, Uuid::new_v4(), ballot_request(&uri, &raw))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{error}");
    assert_eq!(error["error"]["code"], "voter_not_eligible");
    let signed: Value = serde_json::from_slice(&raw).unwrap();
    let mut invalid = signed.clone();
    invalid["payload"]["submissionId"] = json!(Uuid::new_v4());
    let (status, error) = api
        .send(
            voter.account,
            ballot_request(&uri, &serde_json::to_vec(&invalid).unwrap()),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{error}");
    assert_eq!(error["error"]["code"], "invalid_ballot");

    let (left, right) = tokio::join!(
        api.send(voter.account, ballot_request(&uri, &raw)),
        api.send(voter.account, ballot_request(&uri, &raw)),
    );
    let ((first_status, first), (retry_status, retry)) = if left.0 == StatusCode::CREATED {
        (left, right)
    } else {
        (right, left)
    };
    assert_eq!(first_status, StatusCode::CREATED, "{first}");
    assert_eq!(retry_status, StatusCode::OK, "{retry}");
    assert_eq!(first["recovered"], false);
    assert_eq!(retry["recovered"], true);
    assert_eq!(retry["receipt"], first["receipt"]);
    assert_eq!(retry["entry"], first["entry"]);

    // Um outro voto assinado com a mesma chave não substitui o original.
    let mut changed = voter.payload.clone();
    changed["submissionId"] = json!(Uuid::new_v4());
    changed["ciphertexts"] = vector("valid/ballot-1")["ciphertexts"].clone();
    let (status, error) = api
        .send(
            voter.account,
            ballot_request(&uri, &voter.sign(&origin, changed)),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["code"], "already_voted");

    let (status, receipt) = api
        .send(
            voter.account,
            Request::get(format!("/elections/{id}/receipts/me"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{receipt}");
    assert_eq!(receipt, first["receipt"]);
    let board = public_evidence(&db.pool, &format!("/elections/{id}/board"), StatusCode::OK).await;
    assert_eq!(board["entries"], json!([first["entry"]]));
    assert_eq!(board["entries"][0]["payload"]["ballot"], signed);
    assert_eq!(board["checkpoint"], receipt);
    assert!(!board.to_string().contains(&voter.account.to_string()));
    let manifest = &origin.manifest["payload"];
    let manifest_hash = object_digest(manifest).unwrap();
    let context = BallotContext {
        election_id: id,
        manifest_hash: manifest_hash.clone(),
        voter_id,
        registered_public_key: Some(registration["publicKey"].as_str().unwrap().to_owned()),
        option_ids: manifest["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| serde_json::from_value(v["id"].clone()).unwrap())
            .collect(),
    };
    verify_record(
        &serde_json::to_vec(&board["entries"][0]).unwrap(),
        &serde_json::to_vec(&receipt).unwrap(),
        &context,
        1,
        &manifest_hash,
        &signer.public_key_base64().unwrap(),
    )
    .unwrap();
    db.close().await;
}
