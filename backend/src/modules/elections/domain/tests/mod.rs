//! Cenários do domínio isolado; evidências sintéticas simulam somente as portas.

use super::{
    entities::{
        authority::{Authority, AuthorityRole},
        ballot::{BallotContent, Ciphertext, SignedBallot, VerifiedBallot},
        board_entry::{BoardEntry, InvalidBoardEntry},
        election::{Election, ElectionStatus, ElectionTransitionError, TallyStatus},
        election_snapshot::ElectionReconstitutionError,
        manifest::{Manifest, VerifiedManifest},
        participation::{Participation, ParticipationError, ParticipationKey},
        result::{ElectionResult, InvalidElectionResult, ResultReason, ResultStatus},
        tally::{Aggregate, VerifiedTally},
    },
    value_objects::{
        base64_url::Base64Url,
        big_uint::BigUInt,
        calendar_origin::CalendarOrigin,
        election_period::ElectionPeriod,
        fingerprint::Fingerprint,
        identifiers::*,
        key_proof::KeyProof,
        modality::ElectionModality,
        option::{ElectionOption, OptionKind},
        option_count::OptionCount,
        options::ElectionOptions,
        schedule_change::ScheduleChange,
        voter_count::VoterCount,
    },
};
use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

mod board_entries;
mod contracts;
mod lifecycle;
mod participation;
mod preparation;
mod reconstitution;

/// Relógio controlado em UTC, sem infraestrutura externa.
fn at(day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, day, hour, 0, 0).unwrap()
}
fn fp(c: char) -> Fingerprint {
    Fingerprint::try_from_hex(c.to_string().repeat(64)).unwrap()
}
fn eid() -> ElectionId {
    ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap()
}
fn aid() -> AcademicCenterId {
    AcademicCenterId::try_from_uuid(Uuid::from_u128(10)).unwrap()
}
fn actor() -> ActorId {
    ActorId::try_from_uuid(Uuid::from_u128(20)).unwrap()
}
fn account(n: u128) -> AccountId {
    AccountId::try_from_uuid(Uuid::from_u128(n)).unwrap()
}
fn voter(n: u128) -> VoterId {
    VoterId::try_from_uuid(Uuid::from_u128(n)).unwrap()
}
fn option(n: u128) -> OptionId {
    OptionId::from_uuid(Uuid::from_u128(n))
}
fn number(n: u32) -> BigUInt {
    BigUInt::try_from_decimal(n.to_string()).unwrap()
}
fn encoded(s: &str) -> Base64Url {
    Base64Url::try_new(s).unwrap()
}
fn options() -> Vec<ElectionOption> {
    vec![
        ElectionOption::try_new(option(30), OptionKind::Slate, "Aurora").unwrap(),
        ElectionOption::try_new(option(0), OptionKind::Null, "Nulo").unwrap(),
    ]
}
fn preparing(mode: ElectionModality) -> Election {
    Election::try_new(
        eid(),
        aid(),
        mode,
        CalendarOrigin::new(
            eid(),
            ElectionPeriod::try_new(at(20, 10), at(20, 18)).unwrap(),
        ),
        fp('a'),
    )
    .unwrap()
}

/// Chaves e provas fictícias: não são vetores de verificação criptográfica.
fn manifest() -> Manifest {
    let authorities = [
        (31, AuthorityRole::Server, None, "YQ"),
        (32, AuthorityRole::Commission, None, "Yg"),
        (
            33,
            AuthorityRole::Slate,
            Some(SlateId::try_from_uuid(Uuid::from_u128(30)).unwrap()),
            "Yw",
        ),
    ]
    .into_iter()
    .map(|(id, role, slate, key)| {
        Authority::try_new(
            AuthorityId::try_from_uuid(Uuid::from_u128(id)).unwrap(),
            role,
            slate,
            encoded(key),
            number(id as u32),
            KeyProof::new(number(2), number(3)),
        )
        .unwrap()
    })
    .collect();
    Manifest::try_new(
        eid(),
        aid(),
        options(),
        authorities,
        number(8),
        encoded("ZA"),
        fp('a'),
    )
    .unwrap()
}
fn configured(mode: ElectionModality) -> Election {
    let mut election = preparing(mode);
    match mode {
        ElectionModality::Identified => election
            .configure_identified(ElectionOptions::try_new(options()).unwrap())
            .unwrap(),
        ElectionModality::Confidential => election
            .publish_manifest(VerifiedManifest::from_verified_parts(manifest(), fp('b')))
            .unwrap(),
    }
    election
}
fn opened(mode: ElectionModality) -> Election {
    let mut election = configured(mode);
    election.open(at(20, 10)).unwrap();
    election
}
fn closed(mode: ElectionModality, n: u32) -> Election {
    let mut election = opened(mode);
    match mode {
        ElectionModality::Identified => {
            election
                .close_identified(at(20, 18), VoterCount::try_new(n).unwrap(), fp('d'))
                .unwrap();
        }
        ElectionModality::Confidential => {
            election
                .close(
                    at(20, 18),
                    if n == 0 { fp('b') } else { fp('c') },
                    VoterCount::try_new(n).unwrap(),
                    fp('d'),
                )
                .unwrap();
        }
    }
    election
}
fn counts(slate: u32, null: u32) -> Vec<OptionCount> {
    vec![
        OptionCount::try_new(option(30), slate).unwrap(),
        OptionCount::try_new(option(0), null).unwrap(),
    ]
}
fn content(mode: ElectionModality) -> BallotContent {
    match mode {
        ElectionModality::Identified => BallotContent::Identified(option(30)),
        ElectionModality::Confidential => BallotContent::Confidential {
            manifest_hash: fp('b'),
            ciphertexts: vec![
                Ciphertext::new(option(30), number(2), number(4)),
                Ciphertext::new(option(0), number(8), number(16)),
            ],
        },
    }
}
fn ballot(mode: ElectionModality, submission: u128, hash: char, key: char) -> VerifiedBallot {
    VerifiedBallot::from_verified_parts(
        SignedBallot::try_new(
            eid(),
            voter(40),
            SubmissionId::try_from_uuid(Uuid::from_u128(submission)).unwrap(),
            encoded("YQ"),
            encoded("Yg"),
            content(mode),
        )
        .unwrap(),
        fp(hash),
        fp(key),
    )
}
fn change(
    election: &Election,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    changed: DateTime<Utc>,
) -> ScheduleChange {
    ScheduleChange::try_new(
        election.id(),
        election.manifest_hash().cloned(),
        election.schedule_revision() + 1,
        election.current_schedule_hash().clone(),
        ElectionPeriod::try_new(start, end).unwrap(),
        actor(),
        changed,
        "Prorrogação",
    )
    .unwrap()
}
fn confidential_tally(
    election: &Election,
    status: ResultStatus,
    reason: Option<ResultReason>,
    values: Option<Vec<OptionCount>>,
    missing: Vec<AuthorityId>,
) -> VerifiedTally {
    let result = ElectionResult::try_new(
        eid(),
        Some(fp('b')),
        Some(fp('d')),
        Some(fp('e')),
        None,
        election.closure().unwrap().voter_count(),
        status,
        reason,
        values,
        missing.clone(),
    )
    .unwrap();
    let aggregate = Aggregate::try_new(
        eid(),
        fp('b'),
        fp('d'),
        [30, 0]
            .into_iter()
            .map(|id| Ciphertext::new(option(id), number(1), number(1)))
            .collect(),
    )
    .unwrap();
    let authorities = election
        .manifest()
        .unwrap()
        .authorities()
        .iter()
        .map(Authority::id)
        .filter(|id| !missing.contains(id))
        .collect();
    VerifiedTally::from_verified_parts(result, aggregate, fp('e'), authorities)
}
