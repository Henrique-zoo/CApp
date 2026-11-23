//! Dados de teste em memória para os cenários eleitorais e os testes unitários.
//!
//! As cédulas e os manifestos simulam o retorno das portas de verificação.
//! Isso permite testar o aceite, o calendário e os recibos com horários fixos.

use super::infrastructure::codec::{decode_base64_url, decode_fingerprint};

use super::domain::{
    Fingerprint,
    choices::{ElectionOption, ElectionOptions},
    counts::VoterCount,
    cryptography::elgamal::ElGamalCiphertext,
    entities::{
        authority::{Authority, AuthorityRole, KeyProof},
        ballot::{BallotCiphertexts, BallotContent, Ciphertext, SignedBallot, VerifiedBallot},
        configuration::ElectionConfiguration,
        election::{Election, ElectionModality, ElectionPeriod, ElectionTransition},
        manifest::{Manifest, VerifiedManifest},
    },
    identifiers::{AcademicCenterId, AccountId, AuthorityId, ElectionId, SlateId, SubmissionId},
};
use chrono::{DateTime, TimeZone, Utc};
use crypto_bigint::U2048;
use uuid::Uuid;

/// Relógio controlado em UTC, sem infraestrutura externa.
pub(super) fn at(day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, day, hour, 0, 0).unwrap()
}

pub(super) fn fp(c: char) -> Fingerprint {
    decode_fingerprint(&c.to_string().repeat(64)).unwrap()
}

pub(super) fn eid() -> ElectionId {
    ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap()
}

pub(super) fn aid() -> AcademicCenterId {
    AcademicCenterId::try_from_uuid(Uuid::from_u128(10)).unwrap()
}

pub(super) fn actor() -> Uuid {
    Uuid::from_u128(20)
}

pub(super) fn account(n: u128) -> AccountId {
    AccountId::try_from_uuid(Uuid::from_u128(n)).unwrap()
}

pub(super) fn option(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

pub(super) fn decoded(s: &str) -> Vec<u8> {
    decode_base64_url(s).unwrap()
}

fn options() -> Vec<ElectionOption> {
    vec![
        ElectionOption::try_slate(SlateId::try_from_uuid(option(30)).unwrap(), "Aurora").unwrap(),
        ElectionOption::Null,
    ]
}

pub(super) fn preparing(mode: ElectionModality) -> Election {
    let period = ElectionPeriod::try_new(at(20, 10), at(20, 18)).unwrap();

    Election::new(eid(), aid(), period, mode, at(19, 10))
}

/// Chaves e provas fictícias: não são vetores de verificação criptográfica.
pub(super) fn manifest() -> Manifest {
    let authorities = [
        (31, AuthorityRole::Server, "YQ"),
        (32, AuthorityRole::Commission, "Yg"),
        (
            33,
            AuthorityRole::Slate(SlateId::try_from_uuid(Uuid::from_u128(30)).unwrap()),
            "Yw",
        ),
    ]
    .into_iter()
    .map(|(id, role, key)| {
        Authority::try_new(
            AuthorityId::try_from_uuid(Uuid::from_u128(id)).unwrap(),
            role,
            decoded(key),
            U2048::from_u128(id),
            KeyProof::new(U2048::from_u32(2), U2048::from_u32(3)),
        )
        .unwrap()
    })
    .collect();
    Manifest::try_new(
        eid(),
        aid(),
        options(),
        authorities,
        U2048::from_u32(8),
        decoded("ZA"),
    )
    .unwrap()
}

pub(super) fn configured(mode: ElectionModality) -> Election {
    let mut election = preparing(mode);
    let configuration = match mode {
        ElectionModality::Identified => {
            ElectionConfiguration::Identified(ElectionOptions::try_new(options()).unwrap())
        }
        ElectionModality::Confidential => ElectionConfiguration::Confidential(Box::new(
            VerifiedManifest::from_verified_parts(manifest(), fp('b')),
        )),
    };

    election
        .advance(ElectionTransition::CompletePreparation { configuration })
        .unwrap();
    election
}

pub(super) fn opened(mode: ElectionModality) -> Election {
    let mut election = configured(mode);
    election
        .advance(ElectionTransition::Open {
            opened_at: at(20, 10),
        })
        .unwrap();
    election
}

pub(super) fn closed(mode: ElectionModality) -> Election {
    let mut election = opened(mode);
    election
        .advance(ElectionTransition::Close {
            closed_at: at(20, 18),
            voter_count: VoterCount::ZERO,
        })
        .unwrap();
    election
}

pub(super) fn ciphertext(id: Uuid, a: u32, b: u32) -> Ciphertext {
    Ciphertext::new(
        id,
        ElGamalCiphertext {
            a: U2048::from_u32(a),
            b: U2048::from_u32(b),
        },
    )
}

fn content(mode: ElectionModality) -> BallotContent {
    match mode {
        ElectionModality::Identified => BallotContent::Identified(option(30)),
        ElectionModality::Confidential => BallotContent::Confidential {
            manifest_hash: fp('b'),
            ciphertexts: BallotCiphertexts::try_new(vec![
                ciphertext(option(30), 2, 4),
                ciphertext(option(0), 8, 16),
            ])
            .unwrap(),
        },
    }
}

pub(super) fn ballot(mode: ElectionModality, submission: u128, hash: char) -> VerifiedBallot {
    VerifiedBallot::from_verified_parts(
        SignedBallot::new(
            eid(),
            account(50),
            SubmissionId::try_from_uuid(Uuid::from_u128(submission)).unwrap(),
            decoded("YQ"),
            decoded("Yg"),
            content(mode),
        ),
        fp(hash),
    )
}
