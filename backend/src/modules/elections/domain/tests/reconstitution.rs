//! Snapshots adversariais e fronteiras entre contagem e evidências administrativas.

use super::*;
use crate::modules::elections::domain::entities::{
    closure::ElectionClosure, election_snapshot::ElectionSnapshot, tally::InvalidIdentifiedTally,
};

/// Cada mutação muda uma referência independente do estado válido de origem.
type SnapshotMutation = fn(&mut ElectionSnapshot);

#[test]
fn open_snapshots_reject_inconsistent_state_calendar_and_preparation() {
    let mutations: &[SnapshotMutation] = &[
        |s| s.schedule_revision = 1,
        |s| s.current_schedule_hash = fp('f'),
        |s| s.current_period = ElectionPeriod::try_new(at(20, 10), at(21, 18)).unwrap(),
        |s| s.opened_at = None,
        |s| s.opening_schedule_revision = None,
        |s| s.opening_schedule_revision = Some(1),
        |s| s.opening_configuration_count = None,
        |s| s.opening_configuration_count = Some(0),
        |s| s.opening_configuration_count = Some(99),
        |s| s.status = ElectionStatus::Closed,
        |s| s.tally_status = TallyStatus::Final,
        |s| s.closure_hash = Some(fp('f')),
        |s| s.configuration = None,
        |s| s.configuration_changes.clear(),
    ];
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let original = opened(mode);
        for (index, mutate) in mutations.iter().enumerate() {
            let mut snapshot = original.to_snapshot();
            mutate(&mut snapshot);
            assert!(
                Election::try_reconstitute(snapshot).is_err(),
                "{mode:?}: mutação {index}"
            );
        }
        let mut wrong_origin = original.to_snapshot();
        wrong_origin.calendar_origin = CalendarOrigin::new(
            ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap(),
            wrong_origin.calendar_origin.period().clone(),
        );
        assert_eq!(
            Election::try_reconstitute(wrong_origin),
            Err(ElectionReconstitutionError::CalendarOriginElectionMismatch)
        );
    }
}

#[test]
fn final_result_cannot_supply_its_own_closure_reference_during_reconstitution() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = closed(mode, 2);
        let result = match mode {
            ElectionModality::Identified => VerifiedTally::try_identified(
                ElectionResult::try_identified(
                    eid(),
                    fp('d'),
                    VoterCount::try_new(2).unwrap(),
                    counts(1, 1),
                )
                .unwrap(),
            )
            .unwrap(),
            ElectionModality::Confidential => confidential_tally(
                &election,
                ResultStatus::Valid,
                None,
                Some(counts(1, 1)),
                vec![],
            ),
        };
        election.publish_tally(result).unwrap();
        let mut altered = election.to_snapshot();
        altered.closure_hash = Some(fp('f'));
        assert_eq!(
            Election::try_reconstitute(altered),
            Err(ElectionReconstitutionError::InvalidTallyResult)
        );
        let mut missing = election.to_snapshot();
        missing.closure = None;
        missing.closure_hash = None;
        assert_eq!(
            Election::try_reconstitute(missing),
            Err(ElectionReconstitutionError::InvalidTallyResult)
        );
    }
}

#[test]
fn closure_reconstitution_checks_modality_election_schedule_and_snapshot_count() {
    let mut identified = closed(ElectionModality::Identified, 2).to_snapshot();
    identified.closure = Some(
        ElectionClosure::try_new(
            eid(),
            fp('b'),
            fp('a'),
            fp('c'),
            VoterCount::try_new(2).unwrap(),
            at(20, 18),
        )
        .unwrap(),
    );
    assert_eq!(
        Election::try_reconstitute(identified),
        Err(ElectionReconstitutionError::InvalidClosure)
    );
    for (id, schedule) in [
        (
            ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap(),
            fp('a'),
        ),
        (eid(), fp('f')),
    ] {
        let mut snapshot = closed(ElectionModality::Identified, 2).to_snapshot();
        snapshot.closure = Some(
            ElectionClosure::try_identified(
                id,
                schedule,
                VoterCount::try_new(2).unwrap(),
                at(20, 18),
            )
            .unwrap(),
        );
        assert_eq!(
            Election::try_reconstitute(snapshot),
            Err(ElectionReconstitutionError::InvalidClosure)
        );
    }
    let mut election = closed(ElectionModality::Identified, 2);
    let result = ElectionResult::try_identified(
        eid(),
        fp('d'),
        VoterCount::try_new(2).unwrap(),
        counts(1, 1),
    )
    .unwrap();
    election
        .publish_tally(VerifiedTally::try_identified(result).unwrap())
        .unwrap();
    let mut changed_count = election.to_snapshot();
    changed_count.closure = Some(
        ElectionClosure::try_identified(
            eid(),
            fp('a'),
            VoterCount::try_new(3).unwrap(),
            at(20, 18),
        )
        .unwrap(),
    );
    assert_eq!(
        Election::try_reconstitute(changed_count),
        Err(ElectionReconstitutionError::InvalidTallyResult)
    );
}

#[test]
fn cancellation_reconstitution_rejects_backdating_reason_and_snapshot_substitution() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = closed(mode, 2);
        election
            .cancel(
                actor(),
                at(20, 19),
                if mode == ElectionModality::Confidential {
                    Some(fp('c'))
                } else {
                    None
                },
                VoterCount::try_new(2).unwrap(),
                "Cancelamento",
            )
            .unwrap();
        let mutations: &[SnapshotMutation] = &[
            |s| s.cancellation.as_mut().unwrap().cancelled_at = at(20, 17),
            |s| s.cancellation.as_mut().unwrap().reason.clear(),
            |s| s.cancellation.as_mut().unwrap().schedule_hash = Some(fp('f')),
            |s| s.cancellation.as_mut().unwrap().voter_count = VoterCount::try_new(3).unwrap(),
            |s| {
                s.cancellation.as_mut().unwrap().election_id =
                    ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap()
            },
        ];
        for mutate in mutations {
            let mut snapshot = election.to_snapshot();
            mutate(&mut snapshot);
            assert_eq!(
                Election::try_reconstitute(snapshot),
                Err(ElectionReconstitutionError::InvalidCancellation)
            );
        }
    }
}

#[test]
fn identified_tally_constructor_cannot_bypass_confidential_evidence_or_administrative_state() {
    let confidential = closed(ElectionModality::Confidential, 2);
    let tally = confidential_tally(
        &confidential,
        ResultStatus::Valid,
        None,
        Some(counts(1, 1)),
        vec![],
    );
    assert_eq!(
        VerifiedTally::try_identified(tally.result().clone()),
        Err(InvalidIdentifiedTally::ConfidentialResult)
    );
    let cancellation = ElectionResult::try_cancelled(
        eid(),
        ElectionModality::Identified,
        None,
        fp('f'),
        VoterCount::try_new(0).unwrap(),
    )
    .unwrap();
    assert_eq!(
        VerifiedTally::try_identified(cancellation),
        Err(InvalidIdentifiedTally::AdministrativeCancellation)
    );
}

#[test]
fn identified_tally_requires_closure_and_the_configured_option_order() {
    let mut election = opened(ElectionModality::Identified);
    let result = ElectionResult::try_identified(
        eid(),
        fp('d'),
        VoterCount::try_new(2).unwrap(),
        counts(1, 1),
    )
    .unwrap();
    assert_eq!(
        election.publish_tally(VerifiedTally::try_identified(result).unwrap()),
        Err(ElectionTransitionError::TallyBeforeClosure)
    );
    election
        .close_identified(at(20, 18), VoterCount::try_new(2).unwrap(), fp('d'))
        .unwrap();
    for values in [
        vec![
            OptionCount::try_new(option(99), 1).unwrap(),
            OptionCount::try_new(option(0), 1).unwrap(),
        ],
        vec![
            OptionCount::try_new(option(0), 1).unwrap(),
            OptionCount::try_new(option(30), 1).unwrap(),
        ],
    ] {
        let result =
            ElectionResult::try_identified(eid(), fp('d'), VoterCount::try_new(2).unwrap(), values)
                .unwrap();
        assert_eq!(
            election.publish_tally(VerifiedTally::try_identified(result).unwrap()),
            Err(ElectionTransitionError::TallyOptionsMismatch)
        );
    }
    assert_eq!(election.tally_status(), TallyStatus::Pending);
    assert!(election.tally_result().is_none());
}
