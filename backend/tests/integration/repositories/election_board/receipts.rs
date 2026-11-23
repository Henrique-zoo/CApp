//! Cadastro e recibo na API real, com sessões locais de duas contas distintas.

use super::session::SessionApi;
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use backend::modules::elections::{
    application::registration::RegistrationStatus,
    crypto_wire::{MAX_SIGNED_MESSAGE_BYTES, ServerSigner},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn assert_own_receipts(pool: &PgPool, id: Uuid, expected: [(Uuid, &Value); 2]) {
    let api = SessionApi::new(pool).await;
    for (account, receipt) in expected {
        let request = Request::get(format!("/elections/{id}/receipts/me"))
            .body(Body::empty())
            .unwrap();
        let (status, json) = api.send(account, request).await;
        assert_eq!(status, StatusCode::OK, "{json}");
        assert_eq!(&json, receipt);
        let request = Request::get(format!("/elections/{id}/registration/me"))
            .body(Body::empty())
            .unwrap();
        let (status, registration) = api.send(account, request).await;
        assert_eq!(status, StatusCode::OK, "{registration}");
        assert_eq!(registration["participated"], true);
        assert!(registration["voterId"].is_string() && registration["publicKey"].is_string());
    }
}

pub(super) async fn assert_legacy_http(
    pool: &PgPool,
    id: Uuid,
    owner: Uuid,
    other: Uuid,
    recovered: &RegistrationStatus,
    receipt: &Value,
) {
    let api = SessionApi::new(pool).await;
    let get = |path: &str| {
        Request::get(format!("/elections/{id}/{path}/me"))
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(
        api.send(owner, get("registration")).await,
        (StatusCode::OK, json!(recovered))
    );
    assert_eq!(
        api.send(owner, get("receipts")).await,
        (StatusCode::OK, receipt.clone())
    );
    assert_eq!(
        api.send(other, get("registration")).await,
        (
            StatusCode::OK,
            json!({"voterId":null,"publicKey":null,"participated":false})
        )
    );
    let (status, error) = api.send(other, get("receipts")).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{error}");
    assert_eq!(error["error"]["code"], "receipt_not_found");
}

pub(super) async fn assert_registration_http(
    pool: &PgPool,
    election_id: Uuid,
    account: Uuid,
    other_account: Uuid,
    signer: &ServerSigner,
) {
    let api = SessionApi::new(pool).await;
    let uri = format!("/elections/{election_id}/registration/me");
    let get = || Request::get(&uri).body(Body::empty()).unwrap();
    let put = |media_type: &str, body: String| {
        Request::builder()
            .method(Method::PUT)
            .uri(&uri)
            .header("content-type", media_type)
            .body(Body::from(body))
            .unwrap()
    };
    let empty = json!({"publicKey":null, "voterId":null, "participated":false});
    assert_eq!(
        api.send(account, get()).await,
        (StatusCode::OK, empty.clone())
    );
    let key = signer.public_key_base64().unwrap();
    let payload = json!({"version": 1, "kind":"voter-registration", "electionId":election_id,
                        "accountId":account, "publicKey":key});
    let signed = signer.sign_payload(&payload).unwrap().to_string();
    let (status, response) = api
        .send_with_microsoft_identity(
            account,
            other_account,
            put("application/json", signed.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{response}");
    assert_eq!(response["error"]["code"], "voter_not_eligible");
    let (status, response) = api
        .send(other_account, put("application/json", signed.clone()))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{response}");
    assert_eq!(response["error"]["code"], "invalid_registration_proof");
    let duplicate = signed.replace("\"version\":1", "\"version\":1,\"version\":1");
    assert_ne!(duplicate, signed);
    let (status, response) = api.send(account, put("application/json", duplicate)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{response}");
    assert_eq!(response["error"]["code"], "invalid_registration_proof");
    let (status, _) = api.send(account, put("text/plain", signed.clone())).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let oversized = " ".repeat(MAX_SIGNED_MESSAGE_BYTES + 1);
    let (status, response) = api.send(account, put("application/json", oversized)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{response}");
    assert_eq!(response["error"]["code"], "payload_too_large");

    let (status, saved) = api
        .send(
            account,
            put("application/json; charset=utf-8", signed.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let voter_id: Uuid = serde_json::from_value(saved["voterId"].clone()).unwrap();
    assert_ne!(voter_id, account);
    assert_eq!(
        saved,
        json!({"voterId":voter_id,"publicKey":key,"participated":false})
    );
    assert_eq!(
        api.send(account, put("application/json", signed)).await,
        (StatusCode::OK, saved.clone())
    );
    assert_eq!(api.send(account, get()).await, (StatusCode::OK, saved));
    assert_eq!(
        api.send(other_account, get()).await,
        (StatusCode::OK, empty)
    );
}
