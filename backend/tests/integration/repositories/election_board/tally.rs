//! Apuração de três eleitores e quatro autoridades, através do HTTP real.
//!
//! O teste gera a configuração local do pleito e usa a mesma autorização de
//! entrega de contribuição que as contas designadas usam na aplicação.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::modules::elections::{
    application::{
        ports::ElectionBoardRepository,
        services::{board, tally::submit_pending_server_shares},
    },
    crypto_wire::tally::{VerifiedTally, authority_key},
    domain::{
        cryptography::elgamal::generate_authority_key_pair,
        entities::authority::AuthorityRole,
        identifiers::{AuthorityId, SlateId},
    },
    infrastructure::PgElectionBoardRepository,
};
use chrono::{Duration, Timelike, Utc};
use rsa::BigUint;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{
    fixtures::{Origin, Voter, election_id, server, submit_vote},
    public_evidence,
    session::SessionApi,
};
use crate::support::TestDatabase;

fn contribution_request(uri: &str, envelope: &Value) -> Request<Body> {
    Request::put(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(envelope).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn three_votes_are_counted_after_all_designated_authorities_contribute() {
    let db = TestDatabase::new().await;
    let signer = server();
    let boards = PgElectionBoardRepository::new(db.pool.clone());
    let api = SessionApi::with_signer(&db.pool, Some(signer.clone())).await;

    let mut custodians = Vec::new();
    for _ in 0..3 {
        custodians.push(Voter::new(&db.pool, "valid/ballot-0").await);
    }
    let mut voters = Vec::new();
    for template in ["valid/ballot-0", "valid/ballot-0", "valid/ballot-1"] {
        voters.push(Voter::new(&db.pool, template).await);
    }

    let start = (Utc::now() + Duration::seconds(10))
        .with_nanosecond(0)
        .unwrap();
    let end = start + Duration::seconds(8);
    let mut origin = Origin::scheduled(&signer, start, end);
    let id = origin.election;
    let template = &origin.manifest["payload"];
    let center: Uuid = serde_json::from_value(template["academicCenterId"].clone()).unwrap();
    let options = template["options"].as_array().unwrap().clone();
    let slate = |position: usize| {
        SlateId::try_from_uuid(serde_json::from_value(options[position]["id"].clone()).unwrap())
            .unwrap()
    };
    let roles = [
        AuthorityRole::Commission,
        AuthorityRole::Slate(slate(0)),
        AuthorityRole::Slate(slate(1)),
    ];
    let prime = BigUint::parse_bytes(
        serde_json::from_str::<Value>(include_str!(
            "../../../../../packages/api-contracts/elections/profile.json"
        ))
        .unwrap()["group"]["primeHex"]
            .as_str()
            .unwrap()
            .as_bytes(),
        16,
    )
    .unwrap();
    sqlx::query(
        "INSERT INTO election
          (election_id, academic_center_id, starts_at, ends_at)
          VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(center)
    .bind(start)
    .bind(end)
    .execute(&db.pool)
    .await
    .unwrap();
    let mut participants = Vec::new();
    for (index, (custodian, role)) in custodians.iter().zip(roles).enumerate() {
        let authority = AuthorityId::try_from_uuid(Uuid::new_v4()).unwrap();
        let (name, slate_id, position, slate_name) = match role {
            AuthorityRole::Commission => ("commission", None, None, None),
            AuthorityRole::Slate(slate) => (
                "slate",
                Some(slate.into_uuid()),
                Some(index as i32),
                Some(options[index - 1]["name"].as_str().unwrap()),
            ),
            AuthorityRole::Server => unreachable!(),
        };
        sqlx::query(
            "INSERT INTO election_authority
              (election_id, authority_id, account_id, role, slate_id, position, slate_name)
              VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(id)
        .bind(authority.into_uuid())
        .bind(custodian.account)
        .bind(name)
        .bind(slate_id)
        .bind(position)
        .bind(slate_name)
        .execute(&db.pool)
        .await
        .unwrap();
        participants.push((
            custodian.account,
            authority,
            &custodian.key,
            role,
            generate_authority_key_pair().unwrap(),
        ));
    }

    let manifest_uri = format!("/elections/{id}/manifest");
    let publish_request = || Request::put(&manifest_uri).body(Body::empty()).unwrap();
    let (status, _) = api.send(participants[0].0, publish_request()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = api.send(participants[1].0, publish_request()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let authority_uri = format!("/elections/{id}/authority/me");
    for (account, authority, rsa, role, pair) in &participants {
        let key = authority_key(
            election_id(id),
            *authority,
            *role,
            &rsa.public_key_base64().unwrap(),
            pair,
        )
        .unwrap();
        let envelope = rsa
            .sign_payload(&json!({
                "version": 1, "kind": "authority-registration", "electionId": id, "authority": key,
            }))
            .unwrap();
        let malformed = serde_json::to_string(&envelope).unwrap().replace(
            "\"kind\":\"authority-registration\"",
            "\"kind\":\"authority-registration\",\"kind\":\"authority-registration\"",
        );
        let request = Request::put(&authority_uri)
            .header("content-type", "application/json")
            .body(Body::from(malformed))
            .unwrap();
        let (status, error) = api.send(*account, request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["error"]["code"], "invalid_tally_evidence");
        let (status, response) = api
            .send(*account, contribution_request(&authority_uri, &envelope))
            .await;
        assert_eq!(status, StatusCode::OK, "{response}");
        assert_eq!(response["registered"], true);
    }
    let (status, publication) = api.send(participants[0].0, publish_request()).await;
    assert_eq!(status, StatusCode::OK, "{publication}");
    let (status, replay) = api.send(participants[0].0, publish_request()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay, publication);

    let option_ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT option_ids FROM election_board WHERE election_id = $1")
            .bind(id)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(
        option_ids,
        [slate(0).into_uuid(), slate(1).into_uuid(), Uuid::nil()]
    );

    for invalid in [
        "UPDATE election_authority SET position = 50 WHERE election_id = $1 AND position = 1",
        "UPDATE election_board SET voter_count = voter_count + 1 WHERE election_id = $1",
        "UPDATE election SET status = 'closed' WHERE election_id = $1",
    ] {
        let error = sqlx::query(invalid)
            .bind(id)
            .execute(&db.pool)
            .await
            .unwrap_err();
        assert!(
            matches!(error, sqlx::Error::Database(error) if error.is_check_violation()),
            "{invalid}"
        );
    }

    let open = boards.snapshot(election_id(id)).await.unwrap();
    let combined = BigUint::parse_bytes(
        open.manifest["payload"]["combinedPublicKey"]
            .as_str()
            .unwrap()
            .as_bytes(),
        10,
    )
    .unwrap();
    origin.manifest = open.manifest;
    let uri = format!("/elections/{id}/tally");
    public_evidence(&db.pool, &uri, StatusCode::CONFLICT).await;
    wait_until(&db.pool, start).await;
    for (index, voter) in voters.iter().enumerate() {
        let mut payload = voter.payload.clone();
        let rows = payload["ciphertexts"].as_array_mut().unwrap();
        for (position, row) in rows.iter_mut().enumerate() {
            // O teste cifra valores pequenos conhecidos para conferir a contagem
            // no backend. O app usa nonces seguros e cifra no dispositivo.
            let nonce = BigUint::from(7_usize + 3 * index + position);
            let g = BigUint::from(2_u8);
            let bit = BigUint::from(u8::from(position == usize::from(index == 2)));
            row["a"] = json!(g.modpow(&nonce, &prime).to_str_radix(10));
            row["b"] = json!(
                (g.modpow(&bit, &prime) * combined.modpow(&nonce, &prime) % &prime)
                    .to_str_radix(10)
            );
        }
        let envelope = voter.sign(&origin, payload);
        submit_vote(&boards, id, voter.admitted(), &envelope)
            .await
            .unwrap();
    }

    wait_until(&db.pool, end).await;
    board::close_due(&boards, &*signer).await.unwrap();
    submit_pending_server_shares(&boards, &boards, &*signer)
        .await
        .unwrap();

    let closed = boards.snapshot(election_id(id)).await.unwrap();
    let tally = VerifiedTally::from_board(&closed, &signer.public_key_base64().unwrap()).unwrap();
    let pending = public_evidence(&db.pool, &uri, StatusCode::OK).await;
    assert_eq!(pending["shares"].as_array().unwrap().len(), 1);
    assert_eq!(pending["result"]["payload"]["status"], "incomplete");

    for (index, (account, authority, rsa, _, pair)) in participants.iter().enumerate() {
        let share = tally.sign_share(*authority, pair, rsa).unwrap();
        let path = format!("{uri}/shares/{}", authority.as_uuid());
        let (status, _) = api
            .send(voters[0].account, contribution_request(&path, &share))
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        let mut malformed = share.clone();
        malformed["payload"]["shares"][0]["proof"]["extra"] = json!(true);
        let (status, error) = api
            .send(*account, contribution_request(&path, &malformed))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["error"]["code"], "invalid_tally_evidence");
        let (status, accepted) = api
            .send(*account, contribution_request(&path, &share))
            .await;
        assert_eq!(status, StatusCode::OK, "{accepted}");
        let result = &accepted["result"]["payload"];
        if index + 1 < participants.len() {
            assert_eq!(result["status"], "incomplete");
            assert_eq!(result["counts"], Value::Null);
        } else {
            assert_eq!(result["status"], "valid");
            let totals: Vec<_> = result["counts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["count"].as_u64().unwrap())
                .collect();
            assert_eq!(totals, [2, 1, 0]);
        }
        let (status, retry) = api
            .send(*account, contribution_request(&path, &share))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(retry["result"], accepted["result"]);
    }
    let final_result = public_evidence(&db.pool, &uri, StatusCode::OK).await;
    assert_eq!(final_result["shares"].as_array().unwrap().len(), 4);
    let error =
        sqlx::query("UPDATE election_tally_result SET result = result WHERE election_id = $1")
            .bind(id)
            .execute(&db.pool)
            .await
            .unwrap_err();
    assert!(matches!(error, sqlx::Error::Database(error) if error.is_check_violation()));
    db.close().await;
}

async fn wait_until(pool: &sqlx::PgPool, deadline: chrono::DateTime<Utc>) {
    sqlx::query("SELECT pg_sleep(GREATEST(0,EXTRACT(EPOCH FROM ($1::timestamptz-clock_timestamp())))::float8)")
        .bind(deadline).execute(pool).await.unwrap();
}

#[tokio::test]
async fn commission_can_extend_and_cancel_while_accepted_votes_remain_public() {
    use backend::modules::elections::application::board::BoardError;

    let db = TestDatabase::new().await;
    let signer = server();
    let boards = PgElectionBoardRepository::new(db.pool.clone());
    let api = SessionApi::with_signer(&db.pool, Some(signer.clone())).await;
    let commission = Voter::new(&db.pool, "valid/ballot-0").await;
    let voter = Voter::new(&db.pool, "valid/ballot-1").await;

    let now = Utc::now().with_nanosecond(0).unwrap();
    let start = now - Duration::minutes(1);
    let end = now + Duration::hours(2);
    let origin = Origin::scheduled(&signer, start, end);
    let id = origin.election;
    let center: Uuid =
        serde_json::from_value(origin.manifest["payload"]["academicCenterId"].clone()).unwrap();

    sqlx::query(
        "INSERT INTO election
          (election_id, academic_center_id, starts_at, ends_at)
          VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(center)
    .bind(start)
    .bind(end)
    .execute(&db.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO election_authority (election_id, authority_id, account_id, role)
         VALUES ($1, $2, $3, 'commission')",
    )
    .bind(id)
    .bind(Uuid::new_v4())
    .bind(commission.account)
    .execute(&db.pool)
    .await
    .unwrap();

    let period_uri = format!("/elections/{id}/period");
    let propose = |begin, deadline| {
        contribution_request(
            &period_uri,
            &json!({"startsAt": begin, "endsAt": deadline, "reason": "Alteração de horário"}),
        )
    };
    let extension = end + Duration::hours(1);
    let (status, _) = api.send(voter.account, propose(start, extension)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = api.send(commission.account, propose(start, end)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, response) = api
        .send(commission.account, propose(start, extension))
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert_eq!(response["endsAt"], json!(extension));

    // Coloca o quadro de teste em votação, preservando a atribuição da comissão.
    let opened_at = start;
    let origin = Origin::scheduled(&signer, opened_at, extension);
    board::publish(&boards, &*signer, origin.frozen(false))
        .await
        .unwrap();
    let (status, _) = api
        .send(
            commission.account,
            propose(now, extension + Duration::hours(1)),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let new_end = extension + Duration::hours(1);
    let (status, response) = api
        .send(commission.account, propose(opened_at, new_end))
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");

    let envelope = voter.sign(&origin, voter.payload.clone());
    submit_vote(&boards, id, voter.admitted(), &envelope)
        .await
        .unwrap();

    let cancel_uri = format!("/elections/{id}/cancellation");
    let cancellation = |reason| contribution_request(&cancel_uri, &json!({"reason": reason}));
    let (status, _) = api
        .send(voter.account, cancellation("Tentativa externa"))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, response) = api
        .send(commission.account, cancellation("Pleito criado por engano"))
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert_eq!(response["cancelledBy"], json!(commission.account));
    let (status, replay) = api
        .send(commission.account, cancellation("Outro motivo"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay, response);

    let rejected = commission.sign(&origin, commission.payload.clone());
    assert!(matches!(
        submit_vote(&boards, id, commission.admitted(), &rejected).await,
        Err(BoardError::Cancelled)
    ));
    let public = public_evidence(&db.pool, &format!("/elections/{id}/board"), StatusCode::OK).await;
    assert_eq!(public["cancellation"], response);
    assert_eq!(public["entries"].as_array().unwrap().len(), 1);
    assert_eq!(public["endsAt"], json!(new_end));
    assert_eq!(public["closure"], Value::Null);
    assert_eq!(public["aggregate"], Value::Null);
    db.close().await;
}
