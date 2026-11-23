//! Três contas votam sem cadastro prévio e conferem os recibos no quadro.

use super::{
    fixtures::{Origin, Voter, server},
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
    infrastructure::PgElectionBoard,
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::test]
async fn three_distinct_voters_can_verify_their_receipts_in_submission_order() {
    let db = TestDatabase::new().await;
    let signer = server();
    let api = SessionApi::with_signer(&db.pool, Some(signer.clone())).await;
    let mut voters = Vec::new();
    for template in ["valid/ballot-0", "valid/ballot-1", "valid/ballot-0"] {
        let voter = Voter::new(&db.pool, template).await;
        voters.push(voter);
    }
    let origin = Origin::during(&signer, Utc::now() + Duration::hours(1));
    let id = origin.election;
    let manifest = &origin.manifest["payload"];
    let manifest_hash = object_digest(manifest).unwrap();
    let options: Vec<Uuid> = manifest["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| serde_json::from_value(o["id"].clone()).unwrap())
        .collect();
    let store = PgElectionBoard::new(db.pool.clone(), signer.clone());
    store.open(origin.frozen(false)).await.unwrap();

    let mut keys = HashSet::new();
    let mut voter_ids = HashSet::new();
    let mut contexts = Vec::new();
    let mut ballots = Vec::new();
    for voter in &voters {
        let request = Request::get(format!("/elections/{id}/registration/me"))
            .body(Body::empty())
            .unwrap();
        let (status, registration) = api.send(voter.account, request).await;
        assert_eq!(status, StatusCode::OK, "{registration}");
        assert_eq!(
            registration,
            json!({"requiresRegistration":false,"voterId":null,"publicKey":null,"participated":false})
        );
        let voter_id = voter.account;
        let key = voter.payload["publicKey"].as_str().unwrap().to_owned();
        assert!(keys.insert(key.clone()));
        assert!(voter_ids.insert(voter_id));
        let mut payload = voter.payload.clone();
        payload["voterId"] = json!(voter_id);
        payload["submissionId"] = json!(Uuid::new_v4());
        ballots.push(voter.sign(&origin, payload));
        contexts.push(BallotContext {
            election_id: id,
            manifest_hash: manifest_hash.clone(),
            voter_id,
            expected_public_key: Some(key),
            option_ids: options.clone(),
        });
    }

    // A ordem pública segue o aceite das cédulas.
    let order = [2, 0, 1];
    let mut records = Vec::new();
    for i in order {
        let request = Request::post(format!("/elections/{id}/ballots"))
            .header("content-type", "application/json")
            .body(Body::from(ballots[i].clone()))
            .unwrap();
        let (status, record) = api.send(voters[i].account, request).await;
        assert_eq!(status, StatusCode::CREATED, "{record}");
        records.push(record);
    }
    let board = public_evidence(&db.pool, &format!("/elections/{id}/board"), StatusCode::OK).await;
    let entries = board["entries"].as_array().unwrap();
    assert_eq!(entries.len(), order.len());
    let accepted_accounts = sqlx::query_scalar::<_, Uuid>(
        "SELECT account_id FROM election_participation WHERE election_id=$1 ORDER BY accepted_at",
    )
    .bind(id)
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(accepted_accounts, order.map(|i| voters[i].account));
    let mut previous = manifest_hash;
    for (index, i) in order.into_iter().enumerate() {
        let request = Request::get(format!("/elections/{id}/receipts/me"))
            .body(Body::empty())
            .unwrap();
        let (status, receipt) = api.send(voters[i].account, request).await;
        assert_eq!(status, StatusCode::OK, "{receipt}");
        assert_eq!(receipt, records[index]["receipt"]);
        assert_eq!(entries[index], records[index]["entry"]);
        let ballot: Value = serde_json::from_slice(&ballots[i]).unwrap();
        assert_eq!(entries[index]["payload"]["ballot"], ballot);
        assert_eq!(ballot["payload"]["voterId"], json!(voters[i].account));
        // O verificador exige exatamente os campos públicos de cédula, entrada e
        // recibo. Incluir a escolha ou o horário individual invalida o registro.
        let record = verify_record(
            &serde_json::to_vec(&entries[index]).unwrap(),
            &serde_json::to_vec(&receipt).unwrap(),
            &contexts[i],
            (index + 1) as u32,
            &previous,
            &signer.public_key_base64().unwrap(),
        )
        .unwrap();
        previous = record.entry_hash;
    }
    assert_eq!(board["checkpoint"], records.last().unwrap()["receipt"]);
    db.close().await;
}
