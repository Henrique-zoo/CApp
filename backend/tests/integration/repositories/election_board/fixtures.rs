//! Calendário dinâmico e cifras dos vetores públicos, com chaves exclusivas dos testes.

use backend::modules::elections::{
    application::registration::RegistrationProof,
    application::{
        board::{BoardError, FrozenBoard, RecordedBallot},
        ports::VoteRepository,
        registration::VoterIdentity,
        services::board::record_ballot,
    },
    crypto_wire::{ServerSigner, object_digest, parse_strict_json},
    domain::{
        cryptography::elgamal::ElGamalCiphertext,
        entities::{
            ballot::{BallotCiphertexts, BallotContent, Ciphertext, SignedBallot},
            election::ElectionPeriod,
            vote::VoteKey,
        },
        identifiers::{AccountId, ElectionId},
    },
    infrastructure::{
        RsaPssBallotVerifier,
        codec::{decode_base64_url, decode_fingerprint},
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Timelike, Utc};
use crypto_bigint::U2048;
use rsa::{
    Pss, RsaPrivateKey,
    pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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
    period: ElectionPeriod,
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
        let period = ElectionPeriod::try_new(
            start.with_nanosecond(0).unwrap(),
            end.with_nanosecond(0).unwrap(),
        )
        .unwrap();
        let mut manifest = vector("valid/manifest").clone();
        manifest["serverSigningKey"] = json!(server.public_key_base64().unwrap());
        Self {
            election: serde_json::from_value(manifest["electionId"].clone()).unwrap(),
            period,
            manifest: server.sign_payload(&manifest).unwrap(),
        }
    }

    pub(super) fn frozen(&self, requires_voter_registration: bool) -> FrozenBoard {
        FrozenBoard {
            election_id: election_id(self.election),
            manifest: serde_json::to_vec(&self.manifest).unwrap(),
            period: self.period.clone(),
            requires_voter_registration,
        }
    }
}

pub(super) struct Voter {
    pub(super) account: Uuid,
    pub(super) key: ServerSigner,
    private_key: RsaPrivateKey,
    pub(super) payload: Value,
}

impl Voter {
    pub(super) async fn new(pool: &PgPool, ballot_case: &str) -> Self {
        let pem = RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048)
            .unwrap()
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap();
        Self::with_pem(pool, ballot_case, &pem).await
    }

    // O teste de paginação usa a mesma chave sintética em muitas contas.
    pub(super) async fn with_pem(pool: &PgPool, ballot_case: &str, pem: &str) -> Self {
        let key = ServerSigner::from_pkcs8_pem(pem).unwrap();
        let account = Uuid::new_v4();
        institutional_evidence(pool, account).await;
        let mut payload = vector(ballot_case).clone();
        payload["accountId"] = json!(account);
        payload["publicKey"] = json!(key.public_key_base64().unwrap());
        Self {
            account,
            key,
            private_key: RsaPrivateKey::from_pkcs8_pem(pem).unwrap(),
            payload,
        }
    }

    pub(super) fn admitted(&self) -> VoterIdentity {
        identity(self.account)
    }

    /// Testa os mesmos bytes usados pelo dispositivo Android, sem depender
    /// das assinaturas genéricas de artefatos que carregam `kind` e `version`.
    fn sign_with_domain(&self, payload: Value, domain: &[u8]) -> Vec<u8> {
        let canonical = serde_jcs::to_vec(&payload).unwrap();
        let mut message = Vec::with_capacity(domain.len() + canonical.len());
        message.extend_from_slice(domain);
        message.extend_from_slice(&canonical);
        let digest = Sha256::digest(&message);
        let signature = self
            .private_key
            .sign_with_rng(
                &mut rand::rngs::OsRng,
                Pss::new_blinded_with_salt::<Sha256>(32),
                &digest,
            )
            .unwrap();
        serde_json::to_vec(&json!({
            "payload": payload,
            "signature": URL_SAFE_NO_PAD.encode(signature),
        }))
        .unwrap()
    }

    pub(super) fn registration_proof(&self, election_id: Uuid) -> Vec<u8> {
        let payload = json!({
            "electionId": election_id,
            "accountId": self.account,
            "publicKey": self.payload["publicKey"],
        });
        let signed = self.sign_with_domain(payload, b"CAPP-ELECTIONS/REGISTRATION/1\0");
        let envelope: Value = serde_json::from_slice(&signed).unwrap();
        serde_json::to_vec(&json!({
            "payload": {"publicKey": envelope["payload"]["publicKey"]},
            "signature": envelope["signature"],
        }))
        .unwrap()
    }

    pub(super) fn registration_proof_typed(&self, election_id: Uuid) -> RegistrationProof {
        let wire: Value = serde_json::from_slice(&self.registration_proof(election_id)).unwrap();
        RegistrationProof::new(
            URL_SAFE_NO_PAD
                .decode(wire["payload"]["publicKey"].as_str().unwrap())
                .unwrap(),
            URL_SAFE_NO_PAD
                .decode(wire["signature"].as_str().unwrap())
                .unwrap(),
        )
    }

    pub(super) fn sign(&self, origin: &Origin, mut payload: Value) -> Vec<u8> {
        payload["electionId"] = json!(origin.election);
        // A conta da cédula pode ser a mesma mesmo quando o teste troca a chave.
        payload["publicKey"] = json!(self.key.public_key_base64().unwrap());
        payload["manifestHash"] = json!(object_digest(&origin.manifest["payload"]).unwrap());
        self.sign_with_domain(payload, b"CAPP-ELECTIONS/BALLOT/1\0")
    }
}

pub(super) fn election_id(id: Uuid) -> ElectionId {
    ElectionId::try_from_uuid(id).unwrap()
}

pub(super) fn account_id(id: Uuid) -> AccountId {
    AccountId::try_from_uuid(id).unwrap()
}

pub(super) fn vote_key(election: Uuid, account: Uuid) -> VoteKey {
    VoteKey::new(election_id(election), account_id(account))
}

pub(super) fn identity(account: Uuid) -> VoterIdentity {
    VoterIdentity {
        account_id: account_id(account),
        microsoft_object_id: Some(account),
    }
}

pub(super) const COURSE: &str = "Ciência da Computação";

pub(super) async fn seed_academic_center(pool: &PgPool) -> Uuid {
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

    sqlx::query(
        "INSERT INTO academic_center (id,institution_id,name) VALUES ($1,$1,'CA de teste')
         ON CONFLICT DO NOTHING",
    )
    .bind(center)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO academic_center_course (academic_center_id,course_name)
         VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(center)
    .bind(COURSE)
    .execute(pool)
    .await
    .unwrap();

    center
}

pub(super) async fn institutional_evidence(pool: &PgPool, account: Uuid) {
    let center = seed_academic_center(pool).await;

    sqlx::query("INSERT INTO app_user (id,firebase_uid) VALUES ($1,$2)")
        .bind(account)
        .bind(account.to_string())
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO institutional_verification (user_id,microsoft_object_id,institution_id,microsoft_tenant_id,
             microsoft_student_group_id,student_membership_confirmed,course_name,checked_at,expires_at)
         VALUES ($1,$1,$2,$2,$2,true,$3,now()-interval '2 hours',now()+interval '1 hour')",
    )
    .bind(account).bind(center).bind(COURSE).execute(pool).await.unwrap();
}

pub(super) async fn submit_vote(
    repository: &impl VoteRepository,
    election: Uuid,
    voter: VoterIdentity,
    raw: &[u8],
) -> Result<RecordedBallot, BoardError> {
    let envelope = parse_strict_json(raw)?;
    let payload = &envelope["payload"];
    let ciphertexts = payload["ciphertexts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|cipher| {
            Ciphertext::new(
                serde_json::from_value(cipher["optionId"].clone()).unwrap(),
                ElGamalCiphertext {
                    a: U2048::from_str_radix_vartime(cipher["a"].as_str().unwrap(), 10).unwrap(),
                    b: U2048::from_str_radix_vartime(cipher["b"].as_str().unwrap(), 10).unwrap(),
                },
            )
        })
        .collect();
    let signed = SignedBallot::new(
        election_id(election),
        account_id(serde_json::from_value(payload["accountId"].clone()).unwrap()),
        decode_base64_url(payload["publicKey"].as_str().unwrap()).unwrap(),
        decode_base64_url(envelope["signature"].as_str().unwrap()).unwrap(),
        BallotContent::Confidential {
            manifest_hash: decode_fingerprint(payload["manifestHash"].as_str().unwrap()).unwrap(),
            ciphertexts: BallotCiphertexts::try_new(ciphertexts).unwrap(),
        },
    );

    record_ballot(
        &RsaPssBallotVerifier,
        repository,
        server().as_ref(),
        voter,
        signed,
    )
    .await
}

#[cfg(feature = "test-support")]
pub(super) fn ballot_submission(envelope: &[u8]) -> Vec<u8> {
    let envelope: Value = serde_json::from_slice(envelope).unwrap();
    let payload = &envelope["payload"];
    serde_json::to_vec(&json!({
        "payload": {
            "manifestHash": payload["manifestHash"],
            "publicKey": payload["publicKey"],
            "ciphertexts": payload["ciphertexts"],
        },
        "signature": envelope["signature"],
    }))
    .unwrap()
}
