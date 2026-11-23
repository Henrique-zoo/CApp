//! Calendário dinâmico e cifras dos vetores públicos, com chaves exclusivas dos testes.

use backend::modules::elections::{
    application::board::{EligibleVoter, FrozenBoard},
    crypto_wire::{ServerSigner, object_digest},
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, LineEnding},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

pub(super) fn vector(name: &str) -> &'static Value {
    static CASES: OnceLock<Value> = OnceLock::new();
    let cases = CASES.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../../packages/api-contracts/elections/vectors.json"
        ))
        .unwrap()
    });
    &cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()["payload"]
}

pub(super) fn server() -> Arc<ServerSigner> {
    Arc::new(
        ServerSigner::from_pkcs8_pem(include_str!(
            "../../../fixtures/jwt/test-only-rsa-private.pem"
        ))
        .unwrap(),
    )
}

pub(super) struct Origin {
    pub(super) election: Uuid,
    pub(super) manifest: Value,
    calendar: Value,
}

impl Origin {
    pub(super) fn during(server: &ServerSigner, end: DateTime<Utc>) -> Self {
        let mut calendar = vector("valid/calendar-origin").clone();
        calendar["startsAt"] =
            json!((Utc::now() - Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Secs, true));
        calendar["endsAt"] = json!(end.to_rfc3339_opts(SecondsFormat::Secs, true));
        let mut manifest = vector("valid/manifest").clone();
        manifest["calendarOriginHash"] = json!(object_digest(&calendar).unwrap());
        manifest["serverSigningKey"] = json!(server.public_key_base64().unwrap());
        Self {
            election: serde_json::from_value(manifest["electionId"].clone()).unwrap(),
            calendar: server.sign_payload(&calendar).unwrap(),
            manifest: server.sign_payload(&manifest).unwrap(),
        }
    }

    pub(super) fn frozen(&self) -> FrozenBoard {
        FrozenBoard {
            manifest: serde_json::to_vec(&self.manifest).unwrap(),
            calendar_origin: serde_json::to_vec(&self.calendar).unwrap(),
        }
    }
}

pub(super) struct Voter {
    pub(super) account: Uuid,
    key: ServerSigner,
    pub(super) payload: Value,
}

impl Voter {
    pub(super) async fn new(pool: &PgPool, ballot_case: &str) -> Self {
        let pem = RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048)
            .unwrap()
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap();
        let key = ServerSigner::from_pkcs8_pem(&pem).unwrap();
        let account = Uuid::new_v4();
        sqlx::query("INSERT INTO app_user (id,firebase_uid) VALUES ($1,$2)")
            .bind(account)
            .bind(account.to_string())
            .execute(pool)
            .await
            .unwrap();
        let mut payload = vector(ballot_case).clone();
        payload["voterId"] = json!(account);
        payload["publicKey"] = json!(key.public_key_base64().unwrap());
        Self {
            account,
            key,
            payload,
        }
    }

    pub(super) fn admitted(&self) -> EligibleVoter {
        EligibleVoter {
            account_id: self.account,
            eligible_until: Utc::now() + Duration::hours(1),
        }
    }

    pub(super) fn sign(&self, origin: &Origin, mut payload: Value) -> Vec<u8> {
        payload["manifestHash"] = json!(object_digest(&origin.manifest["payload"]).unwrap());
        serde_json::to_vec(&self.key.sign_payload(&payload).unwrap()).unwrap()
    }
}
