//! Regras de aceite, reconstituição e repetição de um voto.

use super::*;
use crate::modules::elections::{
    domain::{
        counts::VoterCount,
        entities::{ballot::SignedBallot, election::ElectionTransition},
    },
    test_support::{account, actor, at, ballot, configured, decoded, eid, fp, opened, option},
};
use uuid::Uuid;

#[test]
fn first_acceptance_binds_key_without_prior_registration_in_both_modalities() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let election = opened(mode);
        let vote = Vote::try_accept(
            &election,
            VoteKey::new(eid(), account(50)),
            ballot(mode, 'e'),
            at(20, 11),
        )
        .unwrap();
        assert_eq!(vote.key(), VoteKey::new(eid(), account(50)));
        assert_eq!(vote.accepted_ballot().ballot().account_id(), account(50));
        assert_eq!(vote.accepted_ballot().ballot().public_key(), b"a");
        assert_eq!(vote.accepted_at(), at(20, 11));
    }
}

#[test]
fn identified_participation_preserves_the_elector_choice_association() {
    let election = opened(ElectionModality::Identified);
    let vote = Vote::try_accept(
        &election,
        VoteKey::new(eid(), account(50)),
        ballot(ElectionModality::Identified, 'e'),
        at(20, 11),
    )
    .unwrap();
    assert_eq!(
        vote.accepted_ballot().ballot().content(),
        &BallotContent::Identified(option(30))
    );
    assert_eq!(vote.key().account_id(), account(50));
}

#[test]
fn changed_content_does_not_create_a_second_vote() {
    let mode = ElectionModality::Identified;
    let election = opened(mode);
    let vote = Vote::try_accept(
        &election,
        VoteKey::new(eid(), account(50)),
        ballot(mode, 'e'),
        at(20, 11),
    )
    .unwrap();
    assert_eq!(
        vote.retry(vote.key(), &ballot(mode, 'a')),
        Err(VoteError::AlreadyParticipated)
    );
    assert_ne!(vote.key(), VoteKey::new(eid(), account(51)));
    assert_ne!(
        vote.key(),
        VoteKey::new(
            ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
            account(50)
        )
    );
}

#[test]
fn retry_and_historical_reconstitution_work_after_closure_or_cancellation() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = opened(mode);
        let original = ballot(mode, 'e');
        let vote = Vote::try_accept(
            &election,
            VoteKey::new(eid(), account(50)),
            original.clone(),
            at(20, 11),
        )
        .unwrap();
        election
            .advance(ElectionTransition::Close {
                closed_at: at(20, 18),
                voter_count: VoterCount::try_new(1).unwrap(),
            })
            .unwrap();
        assert_eq!(vote.retry(vote.key(), &original), Ok(&vote));
        assert_eq!(
            Vote::try_reconstitute(
                &election,
                VoteKey::new(eid(), account(50)),
                original.clone(),
                at(20, 11)
            )
            .unwrap(),
            vote
        );
        assert_eq!(
            Vote::try_accept(
                &election,
                VoteKey::new(eid(), account(51)),
                original.clone(),
                at(20, 11)
            ),
            Err(VoteError::ElectionNotOpen)
        );
        election
            .advance(ElectionTransition::Cancel {
                requested_by: actor(),
                cancelled_at: at(20, 19),
                reason: "Cancelamento".to_owned(),
            })
            .unwrap();
        assert_eq!(vote.retry(vote.key(), &original), Ok(&vote));
        assert_eq!(
            Vote::try_reconstitute(
                &election,
                VoteKey::new(eid(), account(50)),
                original,
                at(20, 11)
            )
            .unwrap(),
            vote
        );
    }
}

#[test]
fn retry_uses_payload_hash_rather_than_randomized_pss_signature() {
    let mode = ElectionModality::Identified;
    let election = opened(mode);
    let original = ballot(mode, 'e');
    let vote = Vote::try_accept(
        &election,
        VoteKey::new(eid(), account(50)),
        original.clone(),
        at(20, 11),
    )
    .unwrap();
    // A porta afirma duas assinaturas válidas do mesmo payload canônico.
    let original = original.ballot();
    let resigned = SignedBallot::new(
        original.election_id(),
        original.account_id(),
        original.public_key().to_vec(),
        decoded("Yw"),
        original.content().clone(),
    );
    let verified = VerifiedBallot::from_verified_parts(resigned, fp('e'));
    assert_eq!(vote.retry(vote.key(), &verified), Ok(&vote));
}

#[test]
fn accepts_and_recovers_only_the_vote_for_the_authenticated_account_and_election() {
    let mode = ElectionModality::Identified;
    let election = opened(mode);
    let original = ballot(mode, 'e');
    let vote = Vote::try_accept(
        &election,
        VoteKey::new(eid(), account(50)),
        original.clone(),
        at(20, 11),
    )
    .unwrap();
    let other_election = ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap();
    for other_key in [
        VoteKey::new(eid(), account(51)),
        VoteKey::new(other_election, account(50)),
    ] {
        assert_eq!(
            vote.retry(other_key, &original),
            Err(VoteError::IdentityMismatch)
        );
        assert_eq!(
            Vote::try_accept(&election, other_key, original.clone(), at(20, 11)),
            Err(VoteError::IdentityMismatch)
        );
    }

    let signed = original.ballot();
    for (election_id, account_id) in [(eid(), account(51)), (other_election, account(50))] {
        let other = SignedBallot::new(
            election_id,
            account_id,
            signed.public_key().to_vec(),
            signed.signature().to_vec(),
            signed.content().clone(),
        );
        let other = VerifiedBallot::from_verified_parts(other, fp('e'));
        assert_eq!(
            vote.retry(vote.key(), &other),
            Err(VoteError::IdentityMismatch)
        );
        assert_eq!(
            Vote::try_accept(&election, vote.key(), other, at(20, 11)),
            Err(VoteError::IdentityMismatch)
        );
    }
}

#[test]
fn new_acceptance_respects_actual_opening_and_half_open_interval() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = configured(mode);
        let half_second = chrono::TimeDelta::milliseconds(500);
        let opened_at = at(20, 11) + half_second;
        election
            .advance(ElectionTransition::Open { opened_at })
            .unwrap();
        for time in [at(20, 11), at(20, 18), at(20, 18) + half_second] {
            assert_eq!(
                Vote::try_accept(
                    &election,
                    VoteKey::new(eid(), account(50)),
                    ballot(mode, 'e'),
                    time
                ),
                Err(VoteError::InvalidAcceptanceTime)
            );
        }
        for time in [opened_at, at(20, 18) - half_second] {
            assert!(
                Vote::try_accept(
                    &election,
                    VoteKey::new(eid(), account(50)),
                    ballot(mode, 'e'),
                    time,
                )
                .is_ok()
            );
        }
    }
}

#[test]
fn historical_acceptance_cannot_be_added_after_cancellation() {
    let mode = ElectionModality::Identified;
    let mut election = opened(mode);
    election
        .advance(ElectionTransition::Cancel {
            requested_by: actor(),
            cancelled_at: at(20, 12),
            reason: "Cancelamento".to_owned(),
        })
        .unwrap();
    assert_eq!(
        Vote::try_reconstitute(
            &election,
            VoteKey::new(eid(), account(50)),
            ballot(mode, 'e'),
            at(20, 13)
        ),
        Err(VoteError::InvalidAcceptanceTime)
    );
}

#[test]
fn receipt_indices_follow_profile_limits() {
    for index in [0, 100001] {
        assert_eq!(
            Receipt::try_new(eid(), fp('b'), fp('e'), index, fp('c'), decoded("Yg")),
            Err(InvalidReceiptIndex)
        );
    }
    assert!(Receipt::try_new(eid(), fp('b'), fp('e'), 100000, fp('c'), decoded("Yg")).is_ok());
}
