//! Posições e referências do quadro confidencial, sem cálculo criptográfico.

use super::*;

#[test]
fn first_entry_is_rooted_in_the_ballot_manifest_and_preserves_the_signed_ballot() {
    let accepted = ballot(ElectionModality::Confidential, 60, 'e', 'f');
    let entry = BoardEntry::try_new(accepted.clone(), 1, fp('b'), fp('c')).unwrap();
    assert_eq!(entry.election_id(), eid());
    assert_eq!(entry.manifest_hash(), &fp('b'));
    assert_eq!(entry.ballot(), &accepted);
    assert_eq!(entry.index(), 1);
    assert_eq!(entry.previous_hash(), &fp('b'));
    assert_eq!(entry.entry_hash(), &fp('c'));
    assert_eq!(
        BoardEntry::try_new(accepted, 1, fp('a'), fp('c')),
        Err(InvalidBoardEntry::InvalidOrigin)
    );
}

#[test]
fn board_indices_obey_the_same_profile_limit_as_receipts() {
    let accepted = ballot(ElectionModality::Confidential, 60, 'e', 'f');
    for index in [0, 100001, u32::MAX] {
        assert_eq!(
            BoardEntry::try_new(accepted.clone(), index, fp('b'), fp('c')),
            Err(InvalidBoardEntry::InvalidIndex)
        );
    }
    assert!(BoardEntry::try_new(accepted, 100000, fp('c'), fp('d')).is_ok());
    assert_eq!(
        BoardEntry::try_new(
            ballot(ElectionModality::Identified, 60, 'e', 'f'),
            1,
            fp('b'),
            fp('c')
        ),
        Err(InvalidBoardEntry::IdentifiedBallot)
    );
}

#[test]
fn consecutive_entries_require_the_same_context_and_previous_digest() {
    let accepted = ballot(ElectionModality::Confidential, 60, 'e', 'f');
    let first = BoardEntry::try_new(accepted.clone(), 1, fp('b'), fp('c')).unwrap();
    let second = BoardEntry::try_new(accepted.clone(), 2, fp('c'), fp('d')).unwrap();
    assert_eq!(second.validate_successor_of(&first), Ok(()));
    for entry in [
        first.clone(),
        BoardEntry::try_new(accepted.clone(), 3, fp('c'), fp('d')).unwrap(),
        BoardEntry::try_new(accepted.clone(), 2, fp('a'), fp('d')).unwrap(),
    ] {
        assert_eq!(
            entry.validate_successor_of(&first),
            Err(InvalidBoardEntry::InvalidPredecessor)
        );
    }
    for (election_id, manifest_hash) in [
        (
            ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
            fp('b'),
        ),
        (eid(), fp('a')),
    ] {
        let other = SignedBallot::try_new(
            election_id,
            voter(40),
            SubmissionId::try_from_uuid(Uuid::from_u128(61)).unwrap(),
            encoded("YQ"),
            encoded("Yg"),
            BallotContent::Confidential {
                manifest_hash,
                ciphertexts: match accepted.ballot().content() {
                    BallotContent::Confidential { ciphertexts, .. } => ciphertexts.clone(),
                    _ => unreachable!(),
                },
            },
        )
        .unwrap();
        let other = VerifiedBallot::from_verified_parts(other, fp('e'), fp('f'));
        let entry = BoardEntry::try_new(other, 2, fp('c'), fp('d')).unwrap();
        assert_eq!(
            entry.validate_successor_of(&first),
            Err(InvalidBoardEntry::InvalidPredecessor)
        );
    }
    let last = BoardEntry::try_new(accepted, 100000, fp('c'), fp('d')).unwrap();
    assert_eq!(
        second.validate_successor_of(&last),
        Err(InvalidBoardEntry::InvalidPredecessor)
    );
}
