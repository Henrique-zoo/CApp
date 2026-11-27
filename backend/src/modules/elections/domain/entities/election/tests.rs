//! Regressões do agregado com evidências criptográficas sintéticas e relógio controlado.

use super::*;

use crate::modules::elections::domain::{
    choices::{ElectionOption, ElectionOptions},
    counts::OptionCount,
    entities::{
        authority::{Authority, AuthorityRole, KeyProof},
        manifest::{Manifest, VerifiedManifest},
    },
    error::InvalidCountConsistency,
    identifiers::SlateId,
};
use crate::modules::elections::test_support::{
    actor, aid, at, closed, configured, fp, manifest, opened, preparing,
};
use chrono::TimeZone;
use crypto_bigint::U2048;
use uuid::Uuid;

fn election_id(value: u128) -> ElectionId {
    ElectionId::try_from_uuid(Uuid::from_u128(value)).unwrap()
}

fn academic_center_id() -> AcademicCenterId {
    AcademicCenterId::try_from_uuid(Uuid::from_u128(10)).unwrap()
}

fn period() -> ElectionPeriod {
    let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    let ends_at = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();

    ElectionPeriod::try_new(starts_at, ends_at).unwrap()
}

fn creation_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 19, 10, 0, 0).unwrap()
}

fn valid_manifest(id: ElectionId, center_id: AcademicCenterId) -> Manifest {
    let slate_id = SlateId::try_from_uuid(Uuid::from_u128(30)).unwrap();

    let options = vec![
        ElectionOption::try_slate(slate_id, "Chapa Horizonte").unwrap(),
        ElectionOption::Null,
    ];

    let make_authority = |id_value: u128, role: AuthorityRole, signing_key: &str| {
        Authority::try_new(
            AuthorityId::try_from_uuid(Uuid::from_u128(id_value)).unwrap(),
            role,
            signing_key.as_bytes().to_vec(),
            U2048::from_u128(id_value + 100),
            KeyProof::new(U2048::from_u32(456), U2048::from_u32(789)),
        )
        .unwrap()
    };

    let authorities = vec![
        make_authority(31, AuthorityRole::Server, "server-key"),
        make_authority(32, AuthorityRole::Commission, "commission-key"),
        make_authority(33, AuthorityRole::Slate(slate_id), "slate-key"),
    ];

    Manifest::try_new(
        id,
        center_id,
        options,
        authorities,
        U2048::from_u32(987_654),
        "server-signing-key".as_bytes().to_vec(),
    )
    .unwrap()
}

#[test]
fn an_election_can_be_created_without_a_ready_configuration() {
    for modality in [ElectionModality::Identified, ElectionModality::Confidential] {
        let period = period();
        let opened_at = *period.starts_at();
        let created_at = creation_time();
        let mut election = Election::new(
            election_id(1),
            academic_center_id(),
            period,
            modality,
            created_at,
        );

        assert_eq!(election.state(), &ElectionState::Preparing);
        assert_eq!(election.modality(), modality);
        assert_eq!(election.created_at, created_at);
        assert_eq!(
            election.advance(ElectionTransition::Open { opened_at }),
            Err(ElectionTransitionError::InvalidState)
        );
        assert!(election.configuration().is_none());
    }
}

#[test]
fn rejects_invalid_cancellation_requester_or_reason_without_changing_state() {
    let mut election = Election::new(
        election_id(1),
        academic_center_id(),
        period(),
        ElectionModality::Identified,
        creation_time(),
    );

    for (requested_by, reason, error) in [
        (
            Uuid::nil(),
            "Eleição criada por engano",
            ElectionTransitionError::InvalidCancellationRequester,
        ),
        (
            Uuid::from_u128(20),
            "",
            ElectionTransitionError::InvalidCancellationReasonLength,
        ),
    ] {
        let result = election.advance(ElectionTransition::Cancel {
            cancelled_at: election.created_at,
            requested_by,
            reason: reason.to_owned(),
        });

        assert_eq!(result, Err(error));
        assert_eq!(election.state(), &ElectionState::Preparing);
    }
}

#[test]
fn advance_closes_then_finalizes_an_identified_election() {
    let period = period();
    let opened_at = *period.starts_at();
    let closed_at = *period.ends_at();
    let id = election_id(1);
    let mut election = Election::new(
        id,
        academic_center_id(),
        period,
        ElectionModality::Identified,
        creation_time(),
    );
    let slate = Uuid::from_u128(30);
    let null = Uuid::nil();
    let options = ElectionOptions::try_new(vec![
        ElectionOption::try_slate(SlateId::try_from_uuid(slate).unwrap(), "Horizonte").unwrap(),
        ElectionOption::Null,
    ])
    .unwrap();

    election
        .advance(ElectionTransition::CompletePreparation {
            configuration: ElectionConfiguration::Identified(options),
        })
        .unwrap();
    election
        .advance(ElectionTransition::Open { opened_at })
        .unwrap();
    assert_eq!(
        election.advance(ElectionTransition::Close {
            closed_at: closed_at - chrono::TimeDelta::seconds(1),
            voter_count: VoterCount::ZERO,
        }),
        Err(ElectionTransitionError::ClosingBeforeDeadline)
    );
    assert!(matches!(election.state, ElectionState::Open(_)));

    let voter_count = VoterCount::try_new(1).unwrap();
    election
        .advance(ElectionTransition::Close {
            closed_at,
            voter_count,
        })
        .unwrap();
    election
        .advance(ElectionTransition::Close {
            closed_at: closed_at + chrono::TimeDelta::seconds(1),
            voter_count: VoterCount::ZERO,
        })
        .unwrap();
    assert!(matches!(
        election.state,
        ElectionState::Closed { closed_at: original, .. } if original == closed_at
    ));

    election
        .advance(ElectionTransition::StartCounting {
            contributing_authorities: vec![],
        })
        .unwrap();
    let counts = vec![
        OptionCount::try_new(slate, 1).unwrap(),
        OptionCount::try_new(null, 0).unwrap(),
    ];

    assert_eq!(
        election.advance(ElectionTransition::Finalize {
            computation: TallyComputation::Totals(vec![]),
        }),
        Err(ElectionTransitionError::InvalidTally(
            InvalidCountConsistency::OptionCountMismatch
        ))
    );
    assert_eq!(
        election.advance(ElectionTransition::Finalize {
            computation: TallyComputation::TotalOutOfRange,
        }),
        Err(ElectionTransitionError::ModalityMismatch)
    );
    assert!(matches!(
        election.state(),
        ElectionState::CountingVotes { .. }
    ));

    election
        .advance(ElectionTransition::Finalize {
            computation: TallyComputation::Totals(counts.clone()),
        })
        .unwrap();
    election
        .advance(ElectionTransition::Finalize {
            computation: TallyComputation::Totals(counts.clone()),
        })
        .unwrap();

    assert_eq!(
        election.advance(ElectionTransition::Finalize {
            computation: TallyComputation::TotalOutOfRange,
        }),
        Err(ElectionTransitionError::ModalityMismatch)
    );
    assert_eq!(
        election.advance(ElectionTransition::Cancel {
            cancelled_at: closed_at,
            requested_by: Uuid::from_u128(20),
            reason: "Cancelamento solicitado".to_owned(),
        }),
        Err(ElectionTransitionError::InvalidState)
    );
    assert!(matches!(
        election.state,
        ElectionState::Finalized { closed_at: original, result, voter_count: stored, .. }
            if original == closed_at && result == FinalTallyResult::Valid(counts) && stored == voter_count
    ));
}

#[test]
fn advance_counts_confidential_votes_after_all_authorities_contribute() {
    let period = period();
    let opened_at = *period.starts_at();
    let closed_at = *period.ends_at();
    let id = election_id(1);
    let center_id = academic_center_id();
    let mut election = Election::new(
        id,
        center_id,
        period,
        ElectionModality::Confidential,
        creation_time(),
    );
    let manifest = valid_manifest(id, center_id);
    let authorities: Vec<_> = manifest.authorities().iter().map(Authority::id).collect();
    let verified = VerifiedManifest::from_verified_parts(manifest, fp('b'));

    election
        .advance(ElectionTransition::CompletePreparation {
            configuration: ElectionConfiguration::Confidential(Box::new(verified)),
        })
        .unwrap();
    election
        .advance(ElectionTransition::Open { opened_at })
        .unwrap();
    assert_eq!(
        election.advance(ElectionTransition::StartCounting {
            contributing_authorities: authorities.clone(),
        }),
        Err(ElectionTransitionError::InvalidState)
    );

    election
        .advance(ElectionTransition::Close {
            closed_at,
            voter_count: VoterCount::ZERO,
        })
        .unwrap();
    for contributions in [
        vec![authorities[0]],
        vec![authorities[0]; authorities.len()],
    ] {
        assert_eq!(
            election.advance(ElectionTransition::StartCounting {
                contributing_authorities: contributions,
            }),
            Err(ElectionTransitionError::ContributionsMismatch)
        );
        assert!(matches!(election.state, ElectionState::Closed { .. }));
    }

    election
        .advance(ElectionTransition::StartCounting {
            contributing_authorities: authorities.into_iter().rev().collect(),
        })
        .unwrap();
    assert!(matches!(
        election.state,
        ElectionState::CountingVotes { closed_at: original, .. } if original == closed_at
    ));
}

#[test]
fn published_manifest_is_immutable_while_waiting_to_open() {
    let mut election = configured(ElectionModality::Confidential);
    let original = election.configuration().unwrap().clone();
    let configured_again = ElectionConfiguration::Confidential(Box::new(
        VerifiedManifest::from_verified_parts(manifest(), fp('e')),
    ));

    assert_eq!(
        election.advance(ElectionTransition::CompletePreparation {
            configuration: configured_again
        }),
        Err(ElectionTransitionError::InvalidState)
    );
    election
        .change_schedule(
            ElectionPeriod::try_new(at(20, 11), at(20, 19)).unwrap(),
            at(20, 9),
        )
        .unwrap();
    assert!(matches!(election.state(), ElectionState::WaitingStart(_)));
    assert_eq!(election.configuration(), Some(&original));
}

#[test]
fn configuration_must_match_mode_and_election() {
    let mut identified = preparing(ElectionModality::Identified);
    let verified = VerifiedManifest::from_verified_parts(manifest(), fp('b'));
    assert_eq!(
        identified.advance(ElectionTransition::CompletePreparation {
            configuration: ElectionConfiguration::Confidential(Box::new(verified.clone())),
        }),
        Err(ElectionTransitionError::ModalityMismatch)
    );
    let mut confidential = Election::new(
        ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
        aid(),
        identified.current_period().clone(),
        ElectionModality::Confidential,
        at(19, 10),
    );
    assert_eq!(
        confidential.advance(ElectionTransition::CompletePreparation {
            configuration: ElectionConfiguration::Confidential(Box::new(verified)),
        }),
        Err(ElectionTransitionError::InvalidConfiguration)
    );
}

#[test]
fn opening_accepts_the_start_and_excludes_the_end() {
    let mut election = configured(ElectionModality::Identified);
    let before = election.clone();
    for attempted_at in [at(20, 9), at(20, 18)] {
        assert_eq!(
            election.advance(ElectionTransition::Open {
                opened_at: attempted_at
            }),
            Err(ElectionTransitionError::OpeningOutsidePeriod {
                attempted_at,
                starts_at: at(20, 10),
                ends_at: at(20, 18),
            })
        );
        assert_eq!(election, before);
    }

    election
        .advance(ElectionTransition::Open {
            opened_at: at(20, 10),
        })
        .unwrap();
    assert!(matches!(election.state(), ElectionState::Open(_)));
}

#[test]
fn rescheduling_never_moves_either_boundary_earlier() {
    let mut election = preparing(ElectionModality::Identified);
    assert_eq!(
        election.change_schedule(
            ElectionPeriod::try_new(at(20, 9), at(20, 19)).unwrap(),
            at(20, 8)
        ),
        Err(ElectionTransitionError::StartCannotMoveEarlier)
    );
    for ends_at in [at(20, 17), at(20, 18)] {
        assert_eq!(
            election.change_schedule(
                ElectionPeriod::try_new(at(20, 11), ends_at).unwrap(),
                at(20, 8)
            ),
            Err(ElectionTransitionError::EndMustBeExtended)
        );
    }
}

#[test]
fn an_open_election_can_only_extend_its_end_before_the_deadline() {
    let mut election = opened(ElectionModality::Identified);
    let extension = ElectionPeriod::try_new(at(20, 10), at(20, 20)).unwrap();
    assert_eq!(
        election.change_schedule(extension.clone(), at(20, 9)),
        Err(ElectionTransitionError::ScheduleChangeBeforeOpening)
    );
    assert_eq!(
        election.change_schedule(extension.clone(), at(20, 18)),
        Err(ElectionTransitionError::ExtensionAfterDeadline)
    );
    assert_eq!(
        election.change_schedule(
            ElectionPeriod::try_new(at(20, 11), at(20, 20)).unwrap(),
            at(20, 15)
        ),
        Err(ElectionTransitionError::StartCannotChange)
    );
    election
        .change_schedule(extension.clone(), at(20, 15))
        .unwrap();
    assert_eq!(*election.current_period(), extension);
    assert_eq!(
        election.advance(ElectionTransition::Close {
            closed_at: at(20, 18),
            voter_count: VoterCount::ZERO,
        }),
        Err(ElectionTransitionError::ClosingBeforeDeadline)
    );
    election
        .advance(ElectionTransition::Close {
            closed_at: at(20, 20),
            voter_count: VoterCount::ZERO,
        })
        .unwrap();
    assert_eq!(
        election.change_schedule(
            ElectionPeriod::try_new(at(20, 10), at(20, 21)).unwrap(),
            at(20, 20)
        ),
        Err(ElectionTransitionError::InvalidState)
    );
}

#[test]
fn cancelling_after_closure_preserves_the_first_cancellation() {
    let mut election = closed(ElectionModality::Identified);
    let cancellation = |cancelled_at| ElectionTransition::Cancel {
        cancelled_at,
        requested_by: actor(),
        reason: "Cancelamento registrado".to_owned(),
    };

    assert_eq!(
        election.advance(cancellation(at(20, 17))),
        Err(ElectionTransitionError::CancellationBeforeClosure)
    );
    election.advance(cancellation(at(20, 19))).unwrap();
    let cancelled = election.clone();

    election.advance(cancellation(at(20, 20))).unwrap();
    assert_eq!(election, cancelled);
}

fn instant(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 16, hour, minute, second)
        .single()
        .unwrap()
}

#[test]
fn accepts_valid_period() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0);

    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

    assert_eq!(period.starts_at(), &starts_at);
    assert_eq!(period.ends_at(), &ends_at);
}

#[test]
fn rejects_equal_start_and_end() {
    let value = instant(10, 0, 0);

    assert_eq!(
        ElectionPeriod::try_new(value, value),
        Err(InvalidElectionPeriod::StartMustPrecedeEnd)
    );
}

#[test]
fn rejects_start_after_end() {
    let starts_at = instant(12, 0, 0);
    let ends_at = instant(10, 0, 0);

    assert_eq!(
        ElectionPeriod::try_new(starts_at, ends_at),
        Err(InvalidElectionPeriod::StartMustPrecedeEnd)
    );
}

#[test]
fn rejects_fractional_seconds_in_start() {
    let starts_at = instant(10, 0, 0).with_nanosecond(1).unwrap();
    let ends_at = instant(12, 0, 0);

    assert_eq!(
        ElectionPeriod::try_new(starts_at, ends_at),
        Err(InvalidElectionPeriod::SubsecondPrecision)
    );
}

#[test]
fn rejects_fractional_seconds_in_end() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0).with_nanosecond(500_000_000).unwrap();

    assert_eq!(
        ElectionPeriod::try_new(starts_at, ends_at),
        Err(InvalidElectionPeriod::SubsecondPrecision)
    );
}

#[test]
fn contains_start_instant() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0);
    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

    assert!(period.contains(&starts_at));
}

#[test]
fn excludes_end_instant() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0);
    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

    assert!(!period.contains(&ends_at));
}

#[test]
fn contains_instant_inside_period() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0);
    let inside = instant(11, 0, 0);
    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

    assert!(period.contains(&inside));
}

#[test]
fn excludes_instant_before_period() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0);
    let before = instant(9, 59, 59);
    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

    assert!(!period.contains(&before));
}

#[test]
fn excludes_instant_after_period() {
    let starts_at = instant(10, 0, 0);
    let ends_at = instant(12, 0, 0);
    let after = instant(12, 0, 1);
    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

    assert!(!period.contains(&after));
}
