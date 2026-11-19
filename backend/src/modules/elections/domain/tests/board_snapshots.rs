//! A mesma fotografia confidencial é exigida no cancelamento e encerramento.

use super::*;
use crate::modules::elections::domain::entities::closure::{
    ElectionClosure, InvalidElectionClosure,
};

#[test]
fn cancellation_without_manifest_cannot_claim_a_board_head() {
    let mut invalidated = configured(ElectionModality::Confidential);
    invalidated.invalidate_configuration().unwrap();
    for mut election in [preparing(ElectionModality::Confidential), invalidated] {
        let previous = election.clone();
        assert_eq!(
            election.cancel(
                actor(),
                at(19, 12),
                Some(fp('c')),
                VoterCount::try_new(0).unwrap(),
                "Cancelamento"
            ),
            Err(ElectionTransitionError::CancellationSnapshotMismatch)
        );
        assert_eq!(election, previous);
    }
}

#[test]
fn confidential_cancellation_requires_the_board_head_once_a_manifest_exists() {
    for mut election in [
        configured(ElectionModality::Confidential),
        opened(ElectionModality::Confidential),
    ] {
        for count in [0, 1] {
            let previous = election.clone();
            assert_eq!(
                election.cancel(
                    actor(),
                    at(20, 12),
                    None,
                    VoterCount::try_new(count).unwrap(),
                    "Cancelamento"
                ),
                Err(ElectionTransitionError::CancellationSnapshotMismatch)
            );
            assert_eq!(election, previous);
        }
    }
}

#[test]
fn empty_confidential_cancellation_must_preserve_the_manifest_as_head() {
    for mut election in [
        configured(ElectionModality::Confidential),
        opened(ElectionModality::Confidential),
    ] {
        let previous = election.clone();
        assert_eq!(
            election.cancel(
                actor(),
                at(20, 12),
                Some(fp('c')),
                VoterCount::try_new(0).unwrap(),
                "Cancelamento"
            ),
            Err(ElectionTransitionError::InvalidEmptyBoard)
        );
        assert_eq!(election, previous);
        election
            .cancel(
                actor(),
                at(20, 12),
                Some(fp('b')),
                VoterCount::try_new(0).unwrap(),
                "Cancelamento",
            )
            .unwrap();
        assert_eq!(
            Election::try_reconstitute(election.to_snapshot()),
            Ok(election)
        );
    }
}

#[test]
fn empty_closure_cannot_bypass_the_board_origin_via_direct_construction() {
    assert_eq!(
        ElectionClosure::try_new(
            eid(),
            fp('b'),
            fp('a'),
            fp('c'),
            VoterCount::try_new(0).unwrap(),
            at(20, 18)
        ),
        Err(InvalidElectionClosure::InvalidEmptyBoard)
    );
    assert!(
        ElectionClosure::try_new(
            eid(),
            fp('b'),
            fp('a'),
            fp('b'),
            VoterCount::try_new(0).unwrap(),
            at(20, 18)
        )
        .is_ok()
    );
}

#[test]
fn cancellation_snapshot_cannot_inject_or_erase_confidential_board_references() {
    let mut unconfigured = preparing(ElectionModality::Confidential);
    unconfigured
        .cancel(
            actor(),
            at(19, 12),
            None,
            VoterCount::try_new(0).unwrap(),
            "Cancelamento",
        )
        .unwrap();
    let mut forged = unconfigured.to_snapshot();
    forged.cancellation.as_mut().unwrap().head_hash = Some(fp('c'));
    assert_eq!(
        Election::try_reconstitute(forged),
        Err(ElectionReconstitutionError::InvalidCancellation)
    );

    let mut empty = opened(ElectionModality::Confidential);
    empty
        .cancel(
            actor(),
            at(20, 12),
            Some(fp('b')),
            VoterCount::try_new(0).unwrap(),
            "Cancelamento",
        )
        .unwrap();
    for head in [None, Some(fp('c'))] {
        let mut forged = empty.to_snapshot();
        forged.cancellation.as_mut().unwrap().head_hash = head;
        assert_eq!(
            Election::try_reconstitute(forged),
            Err(ElectionReconstitutionError::InvalidCancellation)
        );
    }
}

#[test]
fn nonempty_snapshot_cannot_use_the_manifest_as_head() {
    let count = VoterCount::try_new(1).unwrap();
    assert_eq!(
        ElectionClosure::try_new(eid(), fp('b'), fp('a'), fp('b'), count, at(20, 18)),
        Err(InvalidElectionClosure::InvalidEmptyBoard)
    );
    let mut election = opened(ElectionModality::Confidential);
    let previous = election.clone();
    assert_eq!(
        election.close(at(20, 18), fp('b'), count, fp('d')),
        Err(ElectionTransitionError::InvalidEmptyBoard)
    );
    assert_eq!(election, previous);
    assert_eq!(
        election.cancel(actor(), at(20, 12), Some(fp('b')), count, "Cancelamento"),
        Err(ElectionTransitionError::InvalidEmptyBoard)
    );
    assert_eq!(election, previous);
}

#[test]
fn nonempty_cancellation_reconstitution_cannot_claim_an_empty_board() {
    let mut election = opened(ElectionModality::Confidential);
    election
        .cancel(
            actor(),
            at(20, 12),
            Some(fp('c')),
            VoterCount::try_new(1).unwrap(),
            "Cancelamento",
        )
        .unwrap();
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()),
        Ok(election.clone())
    );
    let mut forged = election.to_snapshot();
    forged.cancellation.as_mut().unwrap().head_hash = Some(fp('b'));
    assert_eq!(
        Election::try_reconstitute(forged),
        Err(ElectionReconstitutionError::InvalidCancellation)
    );
}
