//! Calendário dinâmico e cifras dos vetores públicos, com chaves exclusivas dos testes.

use backend::modules::elections::{
    application::{board::FrozenBoard, registration::VoterIdentity},
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
        Self::scheduled(server, Utc::now() - Duration::hours(1), end)
    }

    pub(super) fn scheduled(
        server: &ServerSigner,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        let mut calendar = vector("valid/calendar-origin").clone();
        calendar["startsAt"] = json!(start.to_rfc3339_opts(SecondsFormat::Secs, true));
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
        payload["publicKey"] = json!(key.public_key_base64().unwrap());
        Self {
            account,
            key,
            payload,
        }
    }

    pub(super) async fn seed_registration(&self, pool: &PgPool, election: Uuid) {
        seed_registration(
            pool,
            election,
            self.account,
            serde_json::from_value(self.payload["voterId"].clone()).unwrap(),
            &self.key.public_key_base64().unwrap(),
        )
        .await;
    }

    pub(super) fn proof(&self, election: Uuid) -> Vec<u8> {
        registration_proof(&self.key, election, self.account)
    }

    pub(super) fn sign(&self, origin: &Origin, mut payload: Value) -> Vec<u8> {
        payload["manifestHash"] = json!(object_digest(&origin.manifest["payload"]).unwrap());
        serde_json::to_vec(&self.key.sign_payload(&payload).unwrap()).unwrap()
    }
}

pub(super) fn registration_proof(key: &ServerSigner, election: Uuid, account: Uuid) -> Vec<u8> {
    serde_json::to_vec(
        &key.sign_payload(&json!({
            "version":1, "kind":"voter-registration","electionId":election,
            "accountId":account, "publicKey":key.public_key_base64().unwrap()
        }))
        .unwrap(),
    )
    .unwrap()
}

pub(super) fn identity(account: Uuid) -> VoterIdentity {
    VoterIdentity {
        account_id: account,
        microsoft_object_id: Some(account),
    }
}

pub(super) const COURSE: &str = "Ciência da Computação";

pub(super) async fn institutional_evidence(pool: &PgPool, account: Uuid) {
    let center: Uuid =
        serde_json::from_value(vector("valid/manifest")["academicCenterId"].clone()).unwrap();
    sqlx::query(
        "INSERT INTO institution (id,name,microsoft_tenant_id,microsoft_student_group_id)
                 VALUES ($1,'Instituição de teste',$1,$1) ON CONFLICT DO NOTHING",
    )
    .bind(center)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO academic_center (id,institution_id,name) VALUES ($1,$1,'CA de teste') ON CONFLICT DO NOTHING")
        .bind(center).execute(pool).await.unwrap();
    sqlx::query(
        "INSERT INTO institutional_verification (user_id,microsoft_object_id,institution_id,microsoft_tenant_id,
             microsoft_student_group_id,student_membership_confirmed,course_name,checked_at,expires_at)
         VALUES ($1,$1,$2,$2,$2,true,$3,now()-interval '2 hours',now()+interval '1 hour')"
    ).bind(account).bind(center).bind(COURSE).execute(pool).await.unwrap();
}

// Os testes de concorrência do quadro começam com a eleição em andamento e
// eleitores previamente cadastrados. O fluxo real de inscrição é exercitado em registration.rs.
pub(super) async fn seed_registration(
    pool: &PgPool,
    election: Uuid,
    account: Uuid,
    voter: Uuid,
    key: &str,
) {
    institutional_evidence(pool, account).await;
    sqlx::query("INSERT INTO election_eligibility_course(election_id,academic_center_id,course_name)
                 SELECT election_id,(manifest->'payload'->>'academicCenterId')::uuid,$2 FROM election_board
                 WHERE election_id=$1 ON CONFLICT DO NOTHING")
        .bind(election).bind(COURSE).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO election_voter_registration(election_id,account_id,voter_id,public_key,registered_at)
                 SELECT election_id,$2,$3,$4,starts_at-interval '1 minute' FROM election_board WHERE election_id=$1")
        .bind(election).bind(account).bind(voter).bind(key).execute(pool).await.unwrap();
}
