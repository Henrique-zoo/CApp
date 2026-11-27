//! Compatibilidade entre calendário, aceite de votos e acumulador ElGamal.

use crate::modules::elections::{
    domain::{
        cryptography::elgamal::{accumulate_ballot, empty_accumulator},
        entities::{
            ballot::BallotContent,
            election::{ElectionModality, ElectionPeriod},
            vote::{Vote, VoteKey},
        },
    },
    test_support::{account, at, ballot, ciphertext, fp, opened, option},
};

#[test]
fn confidential_participation_contains_only_the_ciphertext_vector() {
    let election = opened(ElectionModality::Confidential);
    let vote = Vote::try_accept(
        &election,
        VoteKey::new(election.id(), account(50)),
        ballot(ElectionModality::Confidential, 'e'),
        at(20, 11),
    )
    .unwrap();
    match vote.accepted_ballot().ballot().content() {
        BallotContent::Confidential {
            manifest_hash,
            ciphertexts,
        } => {
            assert_eq!(manifest_hash, &fp('b'));
            assert_eq!(ciphertexts.option_ids(), &[option(30), option(0)]);
            let encrypted = ciphertexts.encrypted_ballot();
            let accumulator = empty_accumulator(ciphertexts.option_ids().len()).unwrap();
            let accumulated = accumulate_ballot(accumulator, encrypted).unwrap();

            assert_eq!(
                accumulated,
                [
                    *ciphertext(option(30), 2, 4).components(),
                    *ciphertext(option(0), 8, 16).components(),
                ]
            );
        }
        _ => panic!("Uma participação confidencial não pode conter escolha identificada"),
    }
}

#[test]
fn extension_preserves_historical_votes_and_accepts_new_votes_up_to_the_revised_end() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = opened(mode);
        let original = Vote::try_accept(
            &election,
            VoteKey::new(election.id(), account(50)),
            ballot(mode, 'e'),
            at(20, 11),
        )
        .unwrap();
        election
            .change_schedule(
                ElectionPeriod::try_new(at(20, 10), at(20, 20)).unwrap(),
                at(20, 12),
            )
            .unwrap();
        assert_eq!(
            Vote::try_reconstitute(
                &election,
                VoteKey::new(election.id(), account(50)),
                ballot(mode, 'e'),
                at(20, 11)
            ),
            Ok(original)
        );
        for time in [at(20, 12), at(20, 19)] {
            assert!(
                Vote::try_accept(
                    &election,
                    VoteKey::new(election.id(), account(50)),
                    ballot(mode, 'e'),
                    time
                )
                .is_ok()
            );
        }
    }
}
