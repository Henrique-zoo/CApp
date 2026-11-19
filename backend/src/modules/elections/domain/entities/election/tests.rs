//! Regressões do agregado com evidências criptográficas sintéticas e relógio controlado.

use super::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

fn encoded_key(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(value.as_bytes())
}
use crate::modules::elections::domain::{
    entities::authority::{Authority, AuthorityRole},
    value_objects::{
        base64_url::Base64Url,
        big_uint::BigUInt,
        identifiers::{AuthorityId, OptionId, SlateId},
        key_proof::KeyProof,
        option::{ElectionOption, OptionKind},
        option_count::OptionCount,
        voter_count::VoterCount,
    },
};
use chrono::TimeZone;
use uuid::Uuid;

fn election_id(value: u128) -> ElectionId {
    ElectionId::try_from_uuid(Uuid::from_u128(value)).unwrap()
}

fn academic_center_id() -> AcademicCenterId {
    AcademicCenterId::try_from_uuid(Uuid::from_u128(10)).unwrap()
}

fn fingerprint(value: char) -> Fingerprint {
    Fingerprint::try_from_hex(value.to_string().repeat(64)).unwrap()
}

fn period() -> ElectionPeriod {
    let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    let ends_at = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();

    ElectionPeriod::try_new(starts_at, ends_at).unwrap()
}

fn calendar_origin(id: ElectionId) -> CalendarOrigin {
    CalendarOrigin::new(id, period())
}

fn valid_manifest(
    id: ElectionId,
    center_id: AcademicCenterId,
    origin_hash: Fingerprint,
) -> Manifest {
    let slate_id = SlateId::try_from_uuid(Uuid::from_u128(30)).unwrap();

    let options = vec![
        ElectionOption::try_new(
            OptionId::from_uuid(slate_id.into_uuid()),
            OptionKind::Slate,
            "Chapa Horizonte",
        )
        .unwrap(),
        ElectionOption::try_new(OptionId::from_uuid(Uuid::nil()), OptionKind::Null, "Nulo")
            .unwrap(),
    ];

    let make_authority = |id_value: u128,
                          role: AuthorityRole,
                          authority_slate_id: Option<SlateId>,
                          signing_key: &str| {
        Authority::try_new(
            AuthorityId::try_from_uuid(Uuid::from_u128(id_value)).unwrap(),
            role,
            authority_slate_id,
            Base64Url::try_new(encoded_key(signing_key)).unwrap(),
            BigUInt::try_from_decimal((id_value + 100).to_string()).unwrap(),
            KeyProof::new(
                BigUInt::try_from_decimal("456").unwrap(),
                BigUInt::try_from_decimal("789").unwrap(),
            ),
        )
        .unwrap()
    };

    let authorities = vec![
        make_authority(31, AuthorityRole::Server, None, "server-key"),
        make_authority(32, AuthorityRole::Commission, None, "commission-key"),
        make_authority(33, AuthorityRole::Slate, Some(slate_id), "slate-key"),
    ];

    Manifest::try_new(
        id,
        center_id,
        options,
        authorities,
        BigUInt::try_from_decimal("987654").unwrap(),
        Base64Url::try_new(encoded_key("server-signing-key")).unwrap(),
        origin_hash,
    )
    .unwrap()
}

fn election_with_manifest() -> Election {
    let id = election_id(1);
    let center_id = academic_center_id();
    let origin_hash = fingerprint('a');

    let mut election = Election::try_new(
        id,
        center_id,
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        origin_hash.clone(),
    )
    .unwrap();

    let manifest = valid_manifest(id, center_id, origin_hash);
    election
        .publish_manifest(crate::modules::elections::domain::entities::manifest::VerifiedManifest::from_verified_parts(manifest, fingerprint('b')))
        .unwrap();

    election
}

fn period_with(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> ElectionPeriod {
    ElectionPeriod::try_new(starts_at, ends_at).unwrap()
}

fn schedule_change(
    id: ElectionId,
    revision: u32,
    manifest_hash: Option<Fingerprint>,
    previous_hash: Fingerprint,
    new_period: ElectionPeriod,
) -> ScheduleChange {
    let changed_at = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();
    let actor_id = ActorId::try_from_uuid(Uuid::from_u128(20)).unwrap();

    ScheduleChange::try_new(
        id,
        manifest_hash,
        revision,
        previous_hash,
        new_period,
        actor_id,
        changed_at,
        "Alteração aprovada pela comissão",
    )
    .unwrap()
}

fn schedule_change_at(
    id: ElectionId,
    revision: u32,
    manifest_hash: Option<Fingerprint>,
    previous_hash: Fingerprint,
    new_period: ElectionPeriod,
    changed_at: DateTime<Utc>,
) -> ScheduleChange {
    let actor_id = ActorId::try_from_uuid(Uuid::from_u128(20)).unwrap();

    ScheduleChange::try_new(
        id,
        manifest_hash,
        revision,
        previous_hash,
        new_period,
        actor_id,
        changed_at,
        "Prorrogação aprovada pela comissão",
    )
    .unwrap()
}

#[test]
fn cancels_election_during_preparation() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();

    let cancellation = election
        .cancel(
            actor,
            cancelled_at,
            None,
            VoterCount::try_new(0).unwrap(),
            "Cancelamento aprovado pela comissão",
        )
        .unwrap()
        .clone();

    assert_eq!(election.status(), ElectionStatus::Cancelled);
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert_eq!(cancellation.election_id(), id);
    assert_eq!(cancellation.manifest_hash(), None);
    assert_eq!(cancellation.schedule_hash(), Some(&fingerprint('a')));
    assert_eq!(cancellation.head_hash(), None);
    assert_eq!(cancellation.voter_count(), VoterCount::try_new(0).unwrap());
    assert_eq!(cancellation.requested_by(), actor);
    assert_eq!(cancellation.cancelled_at(), &cancelled_at);
    assert_eq!(cancellation.reason(), "Cancelamento aprovado pela comissão");
    assert_eq!(cancellation.version(), 1);
    assert_eq!(cancellation.kind(), "cancellation");
}

#[test]
fn cancels_open_election() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap();

    let cancellation = election
        .cancel(
            actor,
            cancelled_at,
            Some(fingerprint('b')),
            VoterCount::try_new(5).unwrap(),
            "Irregularidade identificada",
        )
        .unwrap()
        .clone();

    assert_eq!(election.status(), ElectionStatus::Cancelled);
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert!(cancellation.manifest_hash().is_some());
    assert_eq!(cancellation.schedule_hash(), Some(&fingerprint('a')));
    assert_eq!(cancellation.head_hash(), Some(&fingerprint('b')));
    assert_eq!(cancellation.voter_count(), VoterCount::try_new(5).unwrap());
    assert_eq!(cancellation.requested_by(), actor);
    assert_eq!(cancellation.cancelled_at(), &cancelled_at);
    assert_eq!(cancellation.reason(), "Irregularidade identificada");
    assert_eq!(cancellation.version(), 1);
    assert_eq!(cancellation.kind(), "cancellation");
}

#[test]
fn rejects_cancellation_before_opening() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = opened_at - chrono::Duration::seconds(1);

    let result = election.cancel(
        actor,
        cancelled_at,
        None,
        VoterCount::try_new(0).unwrap(),
        "Cancelamento solicitado",
    );

    assert!(matches!(
        result,
        Err(ElectionTransitionError::CancellationBeforeOpening)
    ));
    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.opened_at(), Some(opened_at));
    assert!(election.cancellation.is_none());
}

#[test]
fn accepts_cancellation_at_exact_opening_instant() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();

    let cancelled_at = election
        .cancel(
            actor,
            opened_at,
            None,
            VoterCount::try_new(0).unwrap(),
            "Cancelamento no instante da abertura",
        )
        .unwrap()
        .cancelled_at()
        .to_owned();

    assert_eq!(election.status(), ElectionStatus::Cancelled);
    assert_eq!(cancelled_at, opened_at);
}

#[test]
fn cancels_closed_election_preserving_closure() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    let closed_at = *election.current_period().ends_at();

    election.open(opened_at).unwrap();
    election
        .close(
            closed_at,
            fingerprint('c'),
            VoterCount::try_new(10).unwrap(),
            fingerprint('d'),
        )
        .unwrap();

    let original_closure = election.closure().unwrap().clone();
    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 20, 19, 0, 0).unwrap();

    election
        .cancel(
            actor,
            cancelled_at,
            Some(fingerprint('c')),
            VoterCount::try_new(10).unwrap(),
            "Cancelamento administrativo",
        )
        .unwrap();

    assert_eq!(election.status(), ElectionStatus::Cancelled);
    assert_eq!(election.closure(), Some(&original_closure));
    assert_eq!(election.tally_status(), TallyStatus::Pending);
}

#[test]
fn rejects_closed_election_cancellation_with_different_head_hash() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    let closed_at = *election.current_period().ends_at();

    election.open(opened_at).unwrap();
    election
        .close(
            closed_at,
            fingerprint('c'),
            VoterCount::try_new(10).unwrap(),
            fingerprint('d'),
        )
        .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 20, 19, 0, 0).unwrap();

    let result = election.cancel(
        actor,
        cancelled_at,
        Some(fingerprint('d')),
        VoterCount::try_new(10).unwrap(),
        "Cancelamento administrativo",
    );

    assert_eq!(
        result,
        Err(ElectionTransitionError::CancellationSnapshotMismatch)
    );
    assert_eq!(election.status(), ElectionStatus::Closed);
    assert!(election.cancellation().is_none());
}

#[test]
fn rejects_closed_election_cancellation_with_different_voter_count() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    let closed_at = *election.current_period().ends_at();

    election.open(opened_at).unwrap();
    election
        .close(
            closed_at,
            fingerprint('c'),
            VoterCount::try_new(10).unwrap(),
            fingerprint('d'),
        )
        .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 20, 19, 0, 0).unwrap();

    let result = election.cancel(
        actor,
        cancelled_at,
        Some(fingerprint('c')),
        VoterCount::try_new(9).unwrap(),
        "Cancelamento administrativo",
    );

    assert_eq!(
        result,
        Err(ElectionTransitionError::CancellationSnapshotMismatch)
    );
    assert_eq!(election.status(), ElectionStatus::Closed);
    assert!(election.cancellation().is_none());
}

#[test]
fn rejects_cancellation_before_existing_closure() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    let closed_at = *election.current_period().ends_at();

    election.open(opened_at).unwrap();
    election
        .close(
            closed_at,
            fingerprint('c'),
            VoterCount::try_new(10).unwrap(),
            fingerprint('d'),
        )
        .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = closed_at - chrono::Duration::seconds(1);

    let result = election.cancel(
        actor,
        cancelled_at,
        Some(fingerprint('c')),
        VoterCount::try_new(10).unwrap(),
        "Cancelamento administrativo",
    );

    assert_eq!(
        result,
        Err(ElectionTransitionError::CancellationBeforeClosure)
    );
    assert_eq!(election.status(), ElectionStatus::Closed);
    assert!(election.cancellation().is_none());
}

#[test]
fn repeated_cancellation_returns_existing_record_without_replacing_it() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let first_time = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();

    election
        .cancel(
            actor,
            first_time,
            None,
            VoterCount::try_new(0).unwrap(),
            "Primeiro motivo",
        )
        .unwrap();

    let original = election.cancellation().unwrap().clone();
    let second_time = Utc.with_ymd_and_hms(2026, 11, 17, 13, 0, 0).unwrap();

    let result = election.cancel(
        actor,
        second_time,
        Some(fingerprint('c')),
        VoterCount::try_new(4).unwrap(),
        "Segundo motivo",
    );

    assert_eq!(result, Ok(&original));
    assert_eq!(election.cancellation(), Some(&original));
    assert_eq!(original.reason(), "Primeiro motivo");
    assert_eq!(original.voter_count(), VoterCount::try_new(0).unwrap());
    assert_eq!(original.head_hash(), None);
}

#[test]
fn rejects_cancellation_with_empty_reason() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();

    let result = election.cancel(
        actor,
        cancelled_at,
        None,
        VoterCount::try_new(0).unwrap(),
        "",
    );

    assert_eq!(
        result,
        Err(ElectionTransitionError::InvalidCancellationReasonLength)
    );
    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert!(election.cancellation().is_none());
}

#[test]
fn rejects_cancellation_with_reason_longer_than_2000_characters() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();

    let result = election.cancel(
        actor,
        cancelled_at,
        None,
        VoterCount::try_new(0).unwrap(),
        "a".repeat(2001),
    );

    assert_eq!(
        result,
        Err(ElectionTransitionError::InvalidCancellationReasonLength)
    );
    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert!(election.cancellation().is_none());
}

#[test]
fn rejects_cancellation_with_fractional_seconds() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc
        .with_ymd_and_hms(2026, 11, 17, 12, 0, 0)
        .unwrap()
        .with_nanosecond(1)
        .unwrap();

    let result = election.cancel(
        actor,
        cancelled_at,
        None,
        VoterCount::try_new(0).unwrap(),
        "Motivo válido",
    );

    assert_eq!(result, Err(ElectionTransitionError::SubsecondPrecision));
    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert!(election.cancellation().is_none());
}

#[test]
fn rejects_closing_before_deadline() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let closed_at = Utc.with_ymd_and_hms(2026, 11, 20, 17, 59, 59).unwrap();
    let voter_count = VoterCount::try_new(10).unwrap();

    let result = election.close(closed_at, fingerprint('c'), voter_count, fingerprint('d'));

    assert_eq!(result, Err(ElectionTransitionError::ClosingBeforeDeadline));
    assert_eq!(election.status(), ElectionStatus::Open);
    assert!(election.closure().is_none());
}

#[test]
fn closes_election_at_exact_deadline() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let closed_at = *election.current_period().ends_at();
    let voter_count = VoterCount::try_new(10).unwrap();

    let closure = election
        .close(closed_at, fingerprint('c'), voter_count, fingerprint('d'))
        .unwrap()
        .clone();

    assert_eq!(election.status(), ElectionStatus::Closed);
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert_eq!(closure.election_id(), election.id());
    assert_eq!(closure.manifest_hash(), Some(&fingerprint('b')));
    assert_eq!(closure.schedule_hash(), &fingerprint('a'));
    assert_eq!(closure.head_hash(), Some(&fingerprint('c')));
    assert_eq!(closure.voter_count(), voter_count);
    assert_eq!(closure.closed_at(), &closed_at);
    assert_eq!(election.closure(), Some(&closure));
}

#[test]
fn rejects_closing_election_that_is_not_open() {
    let mut election = election_with_manifest();
    let closed_at = *election.current_period().ends_at();
    let voter_count = VoterCount::try_new(10).unwrap();

    let result = election.close(closed_at, fingerprint('c'), voter_count, fingerprint('d'));

    assert_eq!(result, Err(ElectionTransitionError::InvalidState));
    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert!(election.closure().is_none());
}

#[test]
fn rejects_closing_with_fractional_seconds() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let closed_at = election
        .current_period()
        .ends_at()
        .with_nanosecond(1)
        .unwrap();
    let voter_count = VoterCount::try_new(10).unwrap();

    let result = election.close(closed_at, fingerprint('c'), voter_count, fingerprint('d'));

    assert_eq!(result, Err(ElectionTransitionError::SubsecondPrecision));
    assert_eq!(election.status(), ElectionStatus::Open);
    assert!(election.closure().is_none());
}

#[test]
fn repeated_close_returns_existing_closure_without_replacing_it() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let first_closed_at = *election.current_period().ends_at();
    let first_voter_count = VoterCount::try_new(10).unwrap();

    let first_closure = election
        .close(
            first_closed_at,
            fingerprint('c'),
            first_voter_count,
            fingerprint('d'),
        )
        .unwrap()
        .clone();

    let second_closed_at = first_closed_at + chrono::Duration::seconds(60);
    let second_voter_count = VoterCount::try_new(20).unwrap();

    let second_closure = election
        .close(
            second_closed_at,
            fingerprint('d'),
            second_voter_count,
            fingerprint('d'),
        )
        .unwrap();

    assert_eq!(second_closure, &first_closure);
    assert_eq!(election.closure(), Some(&first_closure));
    assert_eq!(election.status(), ElectionStatus::Closed);
    assert_eq!(election.tally_status(), TallyStatus::Pending);
}

#[test]
fn opens_election_at_exact_start_instant() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();

    election.open(opened_at).unwrap();

    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert_eq!(election.opened_at(), Some(opened_at));
}

#[test]
fn rejects_opening_before_start() {
    let mut election = election_with_manifest();
    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 9, 59, 59).unwrap();

    let result = election.open(opened_at);

    assert_eq!(result, Err(ElectionTransitionError::OpeningBeforeStart));
    assert_eq!(election.status(), ElectionStatus::Preparing);
}

#[test]
fn rejects_opening_at_exact_end_instant() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().ends_at();

    let result = election.open(opened_at);

    assert_eq!(result, Err(ElectionTransitionError::OpeningAtOrAfterEnd));
    assert_eq!(election.status(), ElectionStatus::Preparing);
}

#[test]
fn rejects_opening_without_published_manifest() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let opened_at = *election.current_period().starts_at();
    let result = election.open(opened_at);

    assert_eq!(result, Err(ElectionTransitionError::ManifestRequired));
    assert_eq!(election.status(), ElectionStatus::Preparing);
}

#[test]
fn rejects_reopening_an_open_election() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();

    election.open(opened_at).unwrap();

    let result = election.open(opened_at);

    assert_eq!(result, Err(ElectionTransitionError::InvalidState));
    assert_eq!(election.status(), ElectionStatus::Open);
}

#[test]
fn rejects_opening_with_fractional_seconds() {
    let mut election = election_with_manifest();
    let opened_at = election
        .current_period()
        .starts_at()
        .with_nanosecond(1)
        .unwrap();

    let result = election.open(opened_at);

    assert_eq!(result, Err(ElectionTransitionError::SubsecondPrecision));
    assert_eq!(election.status(), ElectionStatus::Preparing);
}

#[test]
fn creates_election_in_preparing_state() {
    let id = election_id(1);
    let center_id = academic_center_id();
    let origin = calendar_origin(id);
    let origin_hash = fingerprint('a');

    let election = Election::try_new(
        id,
        center_id,
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        origin.clone(),
        origin_hash.clone(),
    )
    .unwrap();

    assert_eq!(election.id(), id);
    assert_eq!(election.academic_center_id(), center_id);
    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert_eq!(election.calendar_origin(), &origin);
    assert_eq!(election.calendar_origin_hash(), &origin_hash);
    assert_eq!(election.current_period(), origin.period());
    assert_eq!(election.current_schedule_hash(), &origin_hash);
    assert!(election.manifest().is_none());
    assert!(election.cancellation().is_none());
    assert!(election.closure().is_none());
}

#[test]
fn rejects_calendar_origin_from_another_election() {
    let id = election_id(1);
    let another_id = election_id(2);
    let origin = calendar_origin(another_id);

    let result = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        origin,
        fingerprint('a'),
    );

    assert_eq!(result, Err(InvalidElection::CalendarOriginElectionMismatch));
}

#[test]
fn preserves_initial_calendar_hash_as_current_schedule_hash() {
    let id = election_id(1);
    let origin_hash = fingerprint('b');

    let election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        origin_hash.clone(),
    )
    .unwrap();

    assert_eq!(election.calendar_origin_hash(), &origin_hash);
    assert_eq!(election.current_schedule_hash(), &origin_hash);
}

#[test]
fn preparing_accepts_schedule_change_before_manifest() {
    let id = election_id(1);
    let initial_hash = fingerprint('a');
    let next_hash = fingerprint('b');

    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        initial_hash.clone(),
    )
    .unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 21, 20, 0, 0).unwrap(),
    );

    let change = schedule_change(id, 1, None, initial_hash, new_period.clone());

    election.change_schedule(change, next_hash.clone()).unwrap();

    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert_eq!(election.current_period(), &new_period);
    assert_eq!(election.current_schedule_hash(), &next_hash);
    assert_eq!(election.schedule_revision(), 1);
    assert_eq!(election.schedule_changes().len(), 1);
    assert!(election.manifest().is_none());
}

#[test]
fn preparing_accepts_multiple_schedule_changes_with_new_start_dates() {
    let id = election_id(1);
    let first_hash = fingerprint('a');
    let second_hash = fingerprint('b');
    let third_hash = fingerprint('c');

    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        first_hash.clone(),
    )
    .unwrap();

    let first_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 21, 20, 0, 0).unwrap(),
    );

    let first_change = schedule_change(id, 1, None, first_hash, first_period.clone());

    election
        .change_schedule(first_change, second_hash.clone())
        .unwrap();

    let second_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 22, 9, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 22, 21, 0, 0).unwrap(),
    );

    let second_change = schedule_change(id, 2, None, second_hash, second_period.clone());

    election
        .change_schedule(second_change, third_hash.clone())
        .unwrap();

    assert_eq!(election.status(), ElectionStatus::Preparing);
    assert_eq!(election.current_period(), &second_period);
    assert_eq!(election.current_schedule_hash(), &third_hash);
    assert_eq!(election.schedule_revision(), 2);
    assert_eq!(election.schedule_changes().len(), 2);

    // Cada revisão preserva o hash produzido pelo codec.
    assert_eq!(
        election.schedule_changes()[0].resulting_schedule_hash(),
        &fingerprint('b')
    );
    assert_eq!(
        election.schedule_changes()[1].resulting_schedule_hash(),
        &fingerprint('c')
    );

    // A segunda revisão referencia o hash produzido pela primeira.
    assert_eq!(
        election.schedule_changes()[1].previous_schedule_hash(),
        &fingerprint('b')
    );

    // O hash da última revisão coincide com o calendário vigente.
    assert_eq!(
        election.schedule_changes()[1].resulting_schedule_hash(),
        election.current_schedule_hash()
    );
}

#[test]
fn preparing_rejects_schedule_change_with_wrong_previous_hash() {
    let id = election_id(1);
    let initial_hash = fingerprint('a');

    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        initial_hash,
    )
    .unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 21, 20, 0, 0).unwrap(),
    );

    let change = schedule_change(id, 1, None, fingerprint('c'), new_period);

    let result = election.change_schedule(change, fingerprint('b'));

    assert_eq!(
        result,
        Err(ElectionTransitionError::PreviousScheduleHashMismatch)
    );
    assert_eq!(election.schedule_revision(), 0);
    assert!(election.schedule_changes().is_empty());
    assert_eq!(election.current_schedule_hash(), &fingerprint('a'));
}

#[test]
fn preparing_rejects_non_sequential_schedule_revision() {
    let id = election_id(1);
    let initial_hash = fingerprint('a');

    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        initial_hash.clone(),
    )
    .unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 21, 20, 0, 0).unwrap(),
    );

    let change = schedule_change(id, 2, None, initial_hash, new_period);

    let result = election.change_schedule(change, fingerprint('b'));

    assert_eq!(
        result,
        Err(ElectionTransitionError::InvalidScheduleRevision)
    );
    assert_eq!(election.schedule_revision(), 0);
    assert!(election.schedule_changes().is_empty());
}

#[test]
fn preparing_rejects_schedule_change_without_extending_end() {
    let id = election_id(1);
    let initial_hash = fingerprint('a');

    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        initial_hash.clone(),
    )
    .unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 11, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap(),
    );

    let change = schedule_change(id, 1, None, initial_hash, new_period);

    let result = election.change_schedule(change, fingerprint('b'));

    assert_eq!(result, Err(ElectionTransitionError::EndMustBeExtended));
    assert_eq!(election.schedule_revision(), 0);
    assert!(election.schedule_changes().is_empty());
}

#[test]
fn open_election_rejects_schedule_change_before_opening() {
    let id = election_id(1);
    let mut election = election_with_manifest();
    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    election.open(opened_at).unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 20, 20, 0, 0).unwrap(),
    );

    let change = schedule_change_at(
        id,
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        new_period,
        Utc.with_ymd_and_hms(2026, 11, 20, 9, 59, 59).unwrap(),
    );

    let result = election.change_schedule(change, fingerprint('c'));

    assert_eq!(
        result,
        Err(ElectionTransitionError::ScheduleChangeBeforeOpening)
    );
    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.opened_at(), Some(opened_at));
    assert_eq!(election.schedule_revision(), 0);
    assert!(election.schedule_changes().is_empty());
}

#[test]
fn open_election_accepts_schedule_change_at_opening_instant() {
    let id = election_id(1);
    let mut election = election_with_manifest();
    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    election.open(opened_at).unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 20, 20, 0, 0).unwrap(),
    );

    let change = schedule_change_at(
        id,
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        new_period.clone(),
        opened_at,
    );

    election.change_schedule(change, fingerprint('c')).unwrap();

    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.opened_at(), Some(opened_at));
    assert_eq!(election.current_period(), &new_period);
    assert_eq!(election.schedule_revision(), 1);
}

#[test]
fn open_election_accepts_schedule_extension_before_deadline() {
    let id = election_id(1);
    let mut election = election_with_manifest();
    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    election.open(opened_at).unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 20, 20, 0, 0).unwrap(),
    );
    let change = schedule_change_at(
        id,
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        new_period.clone(),
        Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap(),
    );

    election.change_schedule(change, fingerprint('c')).unwrap();

    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.current_period(), &new_period);
    assert_eq!(election.current_schedule_hash(), &fingerprint('c'));
    assert_eq!(election.schedule_revision(), 1);
    assert_eq!(election.schedule_changes().len(), 1);
}

#[test]
fn open_election_rejects_schedule_change_that_changes_start() {
    let id = election_id(1);
    let mut election = election_with_manifest();
    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    election.open(opened_at).unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 10, 1, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 20, 20, 0, 0).unwrap(),
    );
    let change = schedule_change_at(
        id,
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        new_period,
        Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap(),
    );

    let result = election.change_schedule(change, fingerprint('c'));

    assert_eq!(result, Err(ElectionTransitionError::StartCannotChange));
    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.current_period(), &period());
    assert_eq!(election.current_schedule_hash(), &fingerprint('a'));
    assert_eq!(election.schedule_revision(), 0);
    assert!(election.schedule_changes().is_empty());
}

#[test]
fn open_election_rejects_extension_at_exact_deadline() {
    let id = election_id(1);
    let mut election = election_with_manifest();
    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    election.open(opened_at).unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 20, 20, 0, 0).unwrap(),
    );
    let change = schedule_change_at(
        id,
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        new_period,
        Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap(),
    );

    let result = election.change_schedule(change, fingerprint('c'));

    assert_eq!(result, Err(ElectionTransitionError::ExtensionAfterDeadline));
    assert_eq!(election.status(), ElectionStatus::Open);
    assert_eq!(election.current_period(), &period());
    assert_eq!(election.current_schedule_hash(), &fingerprint('a'));
    assert_eq!(election.schedule_revision(), 0);
    assert!(election.schedule_changes().is_empty());
}

fn closed_election() -> (Election, Fingerprint) {
    let mut election = election_with_manifest();

    let opened_at = *election.current_period().starts_at();
    let closed_at = *election.current_period().ends_at();

    election.open(opened_at).unwrap();
    election
        .close(
            closed_at,
            fingerprint('c'),
            VoterCount::try_new(10).unwrap(),
            fingerprint('d'),
        )
        .unwrap();

    (election, fingerprint('d'))
}

fn tally_counts(slate_votes: u32, null_votes: u32) -> Vec<OptionCount> {
    vec![
        OptionCount::try_new(OptionId::from_uuid(Uuid::from_u128(30)), slate_votes).unwrap(),
        OptionCount::try_new(OptionId::from_uuid(Uuid::nil()), null_votes).unwrap(),
    ]
}

fn build_tally_result(
    status: ResultStatus,
    reason: Option<ResultReason>,
    counts: Option<Vec<OptionCount>>,
    missing_authorities: Vec<AuthorityId>,
    voter_count: u32,
    closure_hash: Fingerprint,
) -> ElectionResult {
    ElectionResult::try_new(
        election_id(1),
        Some(fingerprint('b')),
        Some(closure_hash),
        Some(fingerprint('e')),
        None,
        VoterCount::try_new(voter_count).unwrap(),
        status,
        reason,
        counts,
        missing_authorities,
    )
    .unwrap()
}

fn valid_tally(closure_hash: Fingerprint) -> ElectionResult {
    build_tally_result(
        ResultStatus::Valid,
        None,
        Some(tally_counts(8, 2)),
        vec![],
        10,
        closure_hash,
    )
}

#[test]
fn publishes_valid_tally_and_marks_it_final() {
    let (mut election, closure_hash) = closed_election();
    let result = valid_tally(closure_hash.clone());

    let published = election
        .publish_tally(verified_tally(result.clone(), &closure_hash))
        .unwrap();

    assert_eq!(published, &result);
    assert_eq!(election.status(), ElectionStatus::Closed);
    assert_eq!(election.tally_status(), TallyStatus::Final);
    assert_eq!(election.tally_result(), Some(&result));
}

#[test]
fn publishes_incomplete_tally_with_authority_from_manifest() {
    let (mut election, closure_hash) = closed_election();
    let authority = AuthorityId::try_from_uuid(Uuid::from_u128(33)).unwrap();

    let result = build_tally_result(
        ResultStatus::Incomplete,
        Some(ResultReason::MissingAuthority),
        None,
        vec![authority],
        10,
        closure_hash.clone(),
    );

    election
        .publish_tally(verified_tally(result.clone(), &closure_hash))
        .unwrap();

    assert_eq!(election.tally_status(), TallyStatus::Incomplete);
    assert_eq!(election.tally_result(), Some(&result));
}

#[test]
fn rejects_incomplete_tally_with_authority_outside_manifest() {
    let (mut election, closure_hash) = closed_election();
    let unknown_authority = AuthorityId::try_from_uuid(Uuid::from_u128(99)).unwrap();

    let result = build_tally_result(
        ResultStatus::Incomplete,
        Some(ResultReason::MissingAuthority),
        None,
        vec![unknown_authority],
        10,
        closure_hash.clone(),
    );

    let publication = election.publish_tally(verified_tally(result, &closure_hash));

    assert_eq!(
        publication,
        Err(ElectionTransitionError::TallyAuthorityMismatch)
    );
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert!(election.tally_result().is_none());
}

#[test]
fn updates_incomplete_tally_to_valid_final_tally() {
    let (mut election, closure_hash) = closed_election();
    let authority = AuthorityId::try_from_uuid(Uuid::from_u128(33)).unwrap();

    let incomplete = build_tally_result(
        ResultStatus::Incomplete,
        Some(ResultReason::MissingAuthority),
        None,
        vec![authority],
        10,
        closure_hash.clone(),
    );

    election
        .publish_tally(verified_tally(incomplete, &closure_hash))
        .unwrap();

    let valid = valid_tally(closure_hash.clone());

    election
        .publish_tally(verified_tally(valid.clone(), &closure_hash))
        .unwrap();

    assert_eq!(election.tally_status(), TallyStatus::Final);
    assert_eq!(election.tally_result(), Some(&valid));
}

#[test]
fn rejects_valid_tally_when_counts_do_not_sum_to_voter_count() {
    let (mut election, closure_hash) = closed_election();

    let result = build_tally_result(
        ResultStatus::Valid,
        None,
        Some(tally_counts(7, 2)),
        vec![],
        10,
        closure_hash.clone(),
    );

    let publication = election.publish_tally(verified_tally(result, &closure_hash));

    assert_eq!(
        publication,
        Err(ElectionTransitionError::TallyReasonMismatch)
    );
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert!(election.tally_result().is_none());
}

#[test]
fn rejects_tally_with_wrong_closure_hash() {
    let (mut election, closure_hash) = closed_election();
    let result = valid_tally(fingerprint('f'));

    let publication = election.publish_tally(verified_tally(result, &closure_hash));

    assert_eq!(
        publication,
        Err(ElectionTransitionError::TallyClosureMismatch)
    );
    assert_eq!(election.tally_status(), TallyStatus::Pending);
}

#[test]
fn rejects_tally_with_wrong_voter_count() {
    let (mut election, closure_hash) = closed_election();

    let result = build_tally_result(
        ResultStatus::Valid,
        None,
        Some(tally_counts(7, 2)),
        vec![],
        9,
        closure_hash.clone(),
    );

    let publication = election.publish_tally(verified_tally(result, &closure_hash));

    assert_eq!(
        publication,
        Err(ElectionTransitionError::TallyVoterCountMismatch)
    );
    assert_eq!(election.tally_status(), TallyStatus::Pending);
}

#[test]
fn accepts_annulled_tally_when_counts_sum_does_not_match() {
    let (mut election, closure_hash) = closed_election();

    let result = build_tally_result(
        ResultStatus::Annulled,
        Some(ResultReason::AggregateSumMismatch),
        Some(tally_counts(7, 2)),
        vec![],
        10,
        closure_hash.clone(),
    );

    election
        .publish_tally(verified_tally(result.clone(), &closure_hash))
        .unwrap();

    assert_eq!(election.tally_status(), TallyStatus::Final);
    assert_eq!(election.tally_result(), Some(&result));
}

#[test]
fn publishes_same_final_tally_idempotently() {
    let (mut election, closure_hash) = closed_election();
    let result = valid_tally(closure_hash.clone());

    election
        .publish_tally(verified_tally(result.clone(), &closure_hash))
        .unwrap();

    let repeated = election
        .publish_tally(verified_tally(result.clone(), &closure_hash))
        .unwrap();

    assert_eq!(repeated, &result);
    assert_eq!(election.tally_status(), TallyStatus::Final);
    assert_eq!(election.tally_result(), Some(&result));
}

#[test]
fn rejects_replacing_final_tally_with_different_result() {
    let (mut election, closure_hash) = closed_election();
    let first_result = valid_tally(closure_hash.clone());

    election
        .publish_tally(verified_tally(first_result.clone(), &closure_hash))
        .unwrap();

    let different_result = build_tally_result(
        ResultStatus::Valid,
        None,
        Some(tally_counts(6, 4)),
        vec![],
        10,
        closure_hash.clone(),
    );

    let publication = election.publish_tally(verified_tally(different_result, &closure_hash));

    assert_eq!(publication, Err(ElectionTransitionError::TallyAlreadyFinal));
    assert_eq!(election.tally_result(), Some(&first_result));
}

#[test]
fn preparing_cancellation_snapshot_round_trips() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();

    election
        .cancel(
            actor,
            cancelled_at,
            None,
            VoterCount::try_new(0).unwrap(),
            "Cancelamento aprovado pela comissão",
        )
        .unwrap();

    let snapshot = election.to_snapshot();
    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Cancelled);
    assert_eq!(restored.tally_status(), TallyStatus::Pending);
    assert!(restored.manifest().is_none());
    assert!(restored.closure().is_none());
    assert!(restored.cancellation().is_some());
}

#[test]
fn open_cancellation_snapshot_round_trips() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    election.open(opened_at).unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap();

    election
        .cancel(
            actor,
            cancelled_at,
            Some(fingerprint('b')),
            VoterCount::try_new(5).unwrap(),
            "Irregularidade identificada",
        )
        .unwrap();

    let snapshot = election.to_snapshot();
    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Cancelled);
    assert_eq!(restored.opened_at(), Some(opened_at));
    assert!(restored.manifest().is_some());
    assert!(restored.closure().is_none());
    assert!(restored.cancellation().is_some());
}

#[test]
fn closed_cancellation_snapshot_round_trips_preserving_closure() {
    let mut election = election_with_manifest();
    let opened_at = *election.current_period().starts_at();
    let closed_at = *election.current_period().ends_at();

    election.open(opened_at).unwrap();
    election
        .close(
            closed_at,
            fingerprint('c'),
            VoterCount::try_new(10).unwrap(),
            fingerprint('d'),
        )
        .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 20, 19, 0, 0).unwrap();

    election
        .cancel(
            actor,
            cancelled_at,
            Some(fingerprint('c')),
            VoterCount::try_new(10).unwrap(),
            "Cancelamento administrativo",
        )
        .unwrap();

    let snapshot = election.to_snapshot();
    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Cancelled);
    assert_eq!(restored.tally_status(), TallyStatus::Pending);
    assert!(restored.closure().is_some());
    assert!(restored.cancellation().is_some());
    assert!(restored.tally_result().is_none());
}

#[test]
fn incomplete_tally_snapshot_round_trips() {
    let (mut election, closure_hash) = closed_election();
    let authority = AuthorityId::try_from_uuid(Uuid::from_u128(33)).unwrap();
    let result = build_tally_result(
        ResultStatus::Incomplete,
        Some(ResultReason::MissingAuthority),
        None,
        vec![authority],
        10,
        closure_hash.clone(),
    );

    election
        .publish_tally(verified_tally(result.clone(), &closure_hash))
        .unwrap();

    let snapshot = election.to_snapshot();
    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Closed);
    assert_eq!(restored.tally_status(), TallyStatus::Incomplete);
    assert_eq!(restored.tally_result(), Some(&result));
}

#[test]
fn rejects_snapshot_with_configuration_from_another_modality() {
    let election = election_with_manifest();
    let mut snapshot = election.to_snapshot();

    snapshot.configuration = Some(ElectionConfiguration::Identified(
        ElectionOptions::try_new(election.options().unwrap().to_vec()).unwrap(),
    ));

    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(super::super::election_snapshot::ElectionReconstitutionError::InvalidConfiguration)
    );
}

#[test]
fn rejects_snapshot_with_schedule_revision_out_of_sync() {
    let election = election_with_manifest();
    let mut snapshot = election.to_snapshot();

    snapshot.schedule_revision = 1;

    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(super::super::election_snapshot::ElectionReconstitutionError::ScheduleRevisionMismatch)
    );
}

#[test]
fn rejects_preparing_snapshot_with_opening_instant() {
    let election = election_with_manifest();
    let mut snapshot = election.to_snapshot();

    snapshot.opened_at = Some(*snapshot.current_period.starts_at());

    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(super::super::election_snapshot::ElectionReconstitutionError::InvalidOpening)
    );
}

#[test]
fn rejects_cancelled_snapshot_without_cancellation_record() {
    let election = election_with_manifest();
    let mut snapshot = election.to_snapshot();

    snapshot.status = ElectionStatus::Cancelled;

    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(super::super::election_snapshot::ElectionReconstitutionError::InvalidLifecycleState)
    );
}

#[test]
fn rejects_snapshot_with_cancellation_from_another_election() {
    let id = election_id(1);
    let mut election = Election::try_new(
        id,
        academic_center_id(),
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        calendar_origin(id),
        fingerprint('a'),
    )
    .unwrap();

    let actor = ActorId::try_from_uuid(Uuid::from_u128(50)).unwrap();
    let cancelled_at = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();

    election
        .cancel(
            actor,
            cancelled_at,
            None,
            VoterCount::try_new(0).unwrap(),
            "Cancelamento aprovado pela comissão",
        )
        .unwrap();

    let mut snapshot = election.to_snapshot();
    snapshot
        .cancellation
        .as_mut()
        .expect("a eleição cancelada deve possuir registro")
        .election_id = election_id(2);

    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(super::super::election_snapshot::ElectionReconstitutionError::InvalidCancellation)
    );
}

#[test]
fn preparing_snapshot_round_trips_without_schedule_changes() {
    let election = election_with_manifest();

    let snapshot = election.to_snapshot();
    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Preparing);
    assert_eq!(restored.tally_status(), TallyStatus::Pending);
}

#[test]
fn rejects_opening_before_last_preparation_change() {
    let mut election = election_with_manifest();
    let change = schedule_change_at(
        election.id(),
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        period_with(
            Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 11, 22, 20, 0, 0).unwrap(),
        ),
        Utc.with_ymd_and_hms(2026, 11, 21, 12, 0, 0).unwrap(),
    );
    election.change_schedule(change, fingerprint('c')).unwrap();
    assert_eq!(
        election.open(Utc.with_ymd_and_hms(2026, 11, 21, 11, 0, 0).unwrap()),
        Err(ElectionTransitionError::NonChronologicalOperation)
    );
}

#[test]
fn open_schedule_change_round_trips_with_opening_revision() {
    let id = election_id(1);
    let mut election = election_with_manifest();

    let opened_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    election.open(opened_at).unwrap();

    let new_period = period_with(
        Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 11, 21, 20, 0, 0).unwrap(),
    );

    let changed_at = Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap();

    let change = schedule_change_at(
        id,
        1,
        Some(fingerprint('b')),
        fingerprint('a'),
        new_period,
        changed_at,
    );

    election.change_schedule(change, fingerprint('c')).unwrap();

    let snapshot = election.to_snapshot();

    assert_eq!(snapshot.opening_schedule_revision, Some(0));

    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Open);
    assert_eq!(restored.to_snapshot().opening_schedule_revision, Some(0));
}

#[test]
fn closed_election_with_final_tally_round_trips() {
    let (mut election, closure_hash) = closed_election();
    let result = valid_tally(closure_hash.clone());

    election
        .publish_tally(verified_tally(result, &closure_hash))
        .unwrap();

    let snapshot = election.to_snapshot();
    let restored = Election::try_reconstitute(snapshot).unwrap();

    assert_eq!(restored, election);
    assert_eq!(restored.status(), ElectionStatus::Closed);
    assert_eq!(restored.tally_status(), TallyStatus::Final);
    assert!(restored.closure().is_some());
    assert!(restored.tally_result().is_some());
}

/// Simula a porta criptográfica; os números não representam uma verificação real.
fn verified_tally(result: ElectionResult, closure_hash: &Fingerprint) -> VerifiedTally {
    use crate::modules::elections::domain::entities::{ballot::Ciphertext, tally::Aggregate};
    let authorities = [31, 32, 33]
        .into_iter()
        .map(|id| AuthorityId::try_from_uuid(Uuid::from_u128(id)).unwrap())
        .filter(|id| !result.missing_authorities().contains(id))
        .collect();
    let aggregate = Aggregate::try_new(
        election_id(1),
        fingerprint('b'),
        closure_hash.clone(),
        [30, 0]
            .into_iter()
            .map(|id| {
                Ciphertext::new(
                    OptionId::from_uuid(Uuid::from_u128(id)),
                    BigUInt::try_from_decimal("1").unwrap(),
                    BigUInt::try_from_decimal("1").unwrap(),
                )
            })
            .collect(),
    )
    .unwrap();
    VerifiedTally::from_verified_parts(result, aggregate, fingerprint('e'), authorities)
}
