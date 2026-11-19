//! Primeiro aceite, identidade interna, repetição, cédulas e recibos.

use super::*;
use crate::modules::elections::{
    application::ballot_ports::{InvalidRecordedBallot, RecordedBallot},
    domain::value_objects::receipt::{InvalidReceiptIndex, Receipt},
};

#[test]
fn first_acceptance_binds_key_without_prior_registration_in_both_modalities() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let election = opened(mode);
        let participation = Participation::try_accept(
            &election,
            account(50),
            voter(40),
            ballot(mode, 60, 'e', 'f'),
            at(20, 11),
        )
        .unwrap();
        assert_eq!(
            participation.key(),
            ParticipationKey::new(eid(), account(50))
        );
        assert_eq!(participation.accepted_ballot().key_fingerprint(), &fp('f'));
        assert_eq!(participation.accepted_at(), at(20, 11));
    }
}

#[test]
fn identified_participation_preserves_the_elector_choice_association() {
    let election = opened(ElectionModality::Identified);
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        ballot(ElectionModality::Identified, 60, 'e', 'f'),
        at(20, 11),
    )
    .unwrap();
    assert_eq!(
        participation.accepted_ballot().ballot().content(),
        &BallotContent::Identified(option(30))
    );
    assert_eq!(participation.key().account_id(), account(50));
}

#[test]
fn confidential_participation_contains_only_the_ciphertext_vector() {
    let election = opened(ElectionModality::Confidential);
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        ballot(ElectionModality::Confidential, 60, 'e', 'f'),
        at(20, 11),
    )
    .unwrap();
    match participation.accepted_ballot().ballot().content() {
        BallotContent::Confidential {
            manifest_hash,
            ciphertexts,
        } => {
            assert_eq!(manifest_hash, &fp('b'));
            assert_eq!(
                ciphertexts
                    .iter()
                    .map(Ciphertext::option_id)
                    .collect::<Vec<_>>(),
                vec![option(30), option(0)]
            );
        }
        _ => panic!("Uma participação confidencial não pode conter escolha identificada"),
    }
}

#[test]
fn switching_key_or_submission_does_not_change_account_election_uniqueness() {
    let mode = ElectionModality::Identified;
    let election = opened(mode);
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        ballot(mode, 60, 'e', 'f'),
        at(20, 11),
    )
    .unwrap();
    assert_eq!(
        participation.retry(participation.key(), &ballot(mode, 61, 'a', 'b')),
        Err(ParticipationError::AlreadyParticipated)
    );
    assert_eq!(
        participation.retry(participation.key(), &ballot(mode, 60, 'a', 'b')),
        Err(ParticipationError::SubmissionConflict)
    );
    assert_ne!(
        participation.key(),
        ParticipationKey::new(eid(), account(51))
    );
    assert_ne!(
        participation.key(),
        ParticipationKey::new(
            ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
            account(50)
        )
    );
}

#[test]
fn retry_and_historical_reconstitution_work_after_closure_or_cancellation() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = opened(mode);
        let original = ballot(mode, 60, 'e', 'f');
        let participation = Participation::try_accept(
            &election,
            account(50),
            voter(40),
            original.clone(),
            at(20, 11),
        )
        .unwrap();
        match mode {
            ElectionModality::Identified => {
                election
                    .close_identified(at(20, 18), VoterCount::try_new(1).unwrap(), fp('d'))
                    .unwrap();
            }
            ElectionModality::Confidential => {
                election
                    .close(
                        at(20, 18),
                        fp('c'),
                        VoterCount::try_new(1).unwrap(),
                        fp('d'),
                    )
                    .unwrap();
            }
        }
        assert_eq!(
            participation.retry(participation.key(), &original),
            Ok(&participation)
        );
        assert_eq!(
            Participation::try_reconstitute(
                &election,
                account(50),
                voter(40),
                original.clone(),
                at(20, 11)
            )
            .unwrap(),
            participation
        );
        assert_eq!(
            Participation::try_accept(
                &election,
                account(51),
                voter(40),
                original.clone(),
                at(20, 11)
            ),
            Err(ParticipationError::ElectionNotOpen)
        );
        election
            .cancel(
                actor(),
                at(20, 19),
                if mode == ElectionModality::Confidential {
                    Some(fp('c'))
                } else {
                    None
                },
                VoterCount::try_new(1).unwrap(),
                "Cancelamento",
            )
            .unwrap();
        assert_eq!(
            participation.retry(participation.key(), &original),
            Ok(&participation)
        );
        assert_eq!(
            Participation::try_reconstitute(
                &election,
                account(50),
                voter(40),
                original,
                at(20, 11)
            )
            .unwrap(),
            participation
        );
    }
}

#[test]
fn retry_uses_payload_hash_rather_than_randomized_pss_signature() {
    let mode = ElectionModality::Identified;
    let election = opened(mode);
    let original = ballot(mode, 60, 'e', 'f');
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        original.clone(),
        at(20, 11),
    )
    .unwrap();
    // A porta afirma duas assinaturas válidas do mesmo payload canônico.
    let resigned = SignedBallot::try_new(
        eid(),
        voter(40),
        original.ballot().submission_id(),
        encoded("YQ"),
        encoded("Yw"),
        content(mode),
    )
    .unwrap();
    let verified = VerifiedBallot::from_verified_parts(resigned, fp('e'), fp('f'));
    assert_eq!(
        participation.retry(participation.key(), &verified),
        Ok(&participation)
    );
}

#[test]
fn another_account_or_public_voter_cannot_recover_the_participation() {
    let mode = ElectionModality::Identified;
    let election = opened(mode);
    let original = ballot(mode, 60, 'e', 'f');
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        original.clone(),
        at(20, 11),
    )
    .unwrap();
    assert_eq!(
        participation.retry(ParticipationKey::new(eid(), account(51)), &original),
        Err(ParticipationError::IdentityMismatch)
    );
    assert_eq!(
        Participation::try_accept(&election, account(50), voter(41), original, at(20, 11)),
        Err(ParticipationError::IdentityMismatch)
    );
}

#[test]
fn new_acceptance_respects_actual_opening_and_half_open_interval() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = configured(mode);
        election.open(at(20, 11)).unwrap();
        for time in [at(20, 10), at(20, 18), at(21, 10)] {
            assert_eq!(
                Participation::try_accept(
                    &election,
                    account(50),
                    voter(40),
                    ballot(mode, 60, 'e', 'f'),
                    time
                ),
                Err(ParticipationError::InvalidAcceptanceTime)
            );
        }
        assert!(
            Participation::try_accept(
                &election,
                account(50),
                voter(40),
                ballot(mode, 60, 'e', 'f'),
                at(20, 11)
            )
            .is_ok()
        );
    }
}

#[test]
fn historical_acceptance_cannot_be_added_after_cancellation() {
    let mode = ElectionModality::Identified;
    let mut election = opened(mode);
    election
        .cancel(
            actor(),
            at(20, 12),
            None,
            VoterCount::try_new(0).unwrap(),
            "Cancelamento",
        )
        .unwrap();
    assert_eq!(
        Participation::try_reconstitute(
            &election,
            account(50),
            voter(40),
            ballot(mode, 60, 'e', 'f'),
            at(20, 13)
        ),
        Err(ParticipationError::InvalidAcceptanceTime)
    );
}

#[test]
fn ballot_cannot_cross_modalities_or_choose_an_unknown_slate() {
    let identified = opened(ElectionModality::Identified);
    assert!(
        ballot(ElectionModality::Confidential, 60, 'e', 'f')
            .ballot()
            .validate_for(&identified)
            .is_err()
    );
    let confidential = opened(ElectionModality::Confidential);
    assert!(
        ballot(ElectionModality::Identified, 60, 'e', 'f')
            .ballot()
            .validate_for(&confidential)
            .is_err()
    );
    let unknown = SignedBallot::try_new(
        eid(),
        voter(40),
        SubmissionId::try_from_uuid(Uuid::from_u128(60)).unwrap(),
        encoded("YQ"),
        encoded("Yg"),
        BallotContent::Identified(option(999)),
    )
    .unwrap();
    assert!(unknown.validate_for(&identified).is_err());
}

#[test]
fn ciphertext_dimension_order_ids_manifest_and_degenerate_components_are_checked() {
    let election = opened(ElectionModality::Confidential);
    let make = |ciphertexts, manifest_hash| {
        SignedBallot::try_new(
            eid(),
            voter(40),
            SubmissionId::try_from_uuid(Uuid::from_u128(60)).unwrap(),
            encoded("YQ"),
            encoded("Yg"),
            BallotContent::Confidential {
                manifest_hash,
                ciphertexts,
            },
        )
    };
    assert!(
        make(
            vec![Ciphertext::new(option(30), number(2), number(4))],
            fp('b')
        )
        .is_err()
    );
    assert!(
        make(
            vec![Ciphertext::new(option(30), number(2), number(4)); 2],
            fp('b')
        )
        .is_err()
    );
    for a in [0, 1] {
        assert!(
            make(
                vec![
                    Ciphertext::new(option(30), number(a), number(4)),
                    Ciphertext::new(option(0), number(2), number(4))
                ],
                fp('b')
            )
            .is_err()
        );
    }
    let reversed = vec![
        Ciphertext::new(option(0), number(2), number(4)),
        Ciphertext::new(option(30), number(2), number(4)),
    ];
    assert!(
        make(reversed, fp('b'))
            .unwrap()
            .validate_for(&election)
            .is_err()
    );
    let correct = vec![
        Ciphertext::new(option(30), number(2), number(4)),
        Ciphertext::new(option(0), number(2), number(4)),
    ];
    assert!(
        make(correct, fp('f'))
            .unwrap()
            .validate_for(&election)
            .is_err()
    );
}

#[test]
fn null_is_a_regular_valid_choice_in_both_modalities() {
    let identified = opened(ElectionModality::Identified);
    let ballot = SignedBallot::try_new(
        eid(),
        voter(40),
        SubmissionId::try_from_uuid(Uuid::from_u128(60)).unwrap(),
        encoded("YQ"),
        encoded("Yg"),
        BallotContent::Identified(option(0)),
    )
    .unwrap();
    assert!(ballot.validate_for(&identified).is_ok());
    assert_eq!(
        opened(ElectionModality::Confidential)
            .options()
            .unwrap()
            .last()
            .unwrap()
            .id(),
        option(0)
    );
}

#[test]
fn confirmed_confidential_receipt_must_match_ballot_and_manifest() {
    let mode = ElectionModality::Confidential;
    let election = opened(mode);
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        ballot(mode, 60, 'e', 'f'),
        at(20, 11),
    )
    .unwrap();
    assert_eq!(
        RecordedBallot::try_new(participation.clone(), None, None),
        Err(InvalidRecordedBallot)
    );
    for (manifest, ballot_hash) in [('b', 'e'), ('f', 'e'), ('b', 'f')] {
        let receipt = Receipt::try_new(
            eid(),
            fp(manifest),
            fp(ballot_hash),
            1,
            fp('c'),
            encoded("Yg"),
        )
        .unwrap();
        assert_eq!(
            RecordedBallot::try_new(
                participation.clone(),
                Some(
                    BoardEntry::try_new(
                        participation.accepted_ballot().clone(),
                        1,
                        fp('b'),
                        fp('c')
                    )
                    .unwrap()
                ),
                Some(receipt)
            )
            .is_ok(),
            manifest == 'b' && ballot_hash == 'e'
        );
    }
}

#[test]
fn confidential_retry_preserves_the_exact_confirmed_receipt_after_closure() {
    let mode = ElectionModality::Confidential;
    let mut election = opened(mode);
    let accepted = ballot(mode, 60, 'e', 'f');
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        accepted.clone(),
        at(20, 11),
    )
    .unwrap();
    let key = participation.key();
    let receipt = Receipt::try_new(eid(), fp('b'), fp('e'), 1, fp('c'), encoded("Yg")).unwrap();
    let entry = BoardEntry::try_new(accepted.clone(), 1, fp('b'), fp('c')).unwrap();
    let record =
        RecordedBallot::try_new(participation, Some(entry.clone()), Some(receipt.clone())).unwrap();
    election
        .close(
            at(20, 18),
            fp('c'),
            VoterCount::try_new(1).unwrap(),
            fp('d'),
        )
        .unwrap();
    let recovered = record.recover(key, &accepted).unwrap();
    assert_eq!(recovered.receipt(), Some(&receipt));
    assert_eq!(recovered.board_entry(), Some(&entry));
    assert_eq!(recovered.receipt().unwrap().signature().as_str(), "Yg");
    assert_eq!(recovered.receipt().unwrap().index(), 1);
    assert_eq!(
        record.recover(key, &ballot(mode, 60, 'd', 'f')),
        Err(ParticipationError::SubmissionConflict)
    );
}

#[test]
fn receipt_indices_follow_profile_limits_and_identified_needs_no_secret_board() {
    for index in [0, 100001] {
        assert_eq!(
            Receipt::try_new(eid(), fp('b'), fp('e'), index, fp('c'), encoded("Yg")),
            Err(InvalidReceiptIndex)
        );
    }
    assert!(Receipt::try_new(eid(), fp('b'), fp('e'), 100000, fp('c'), encoded("Yg")).is_ok());
    let election = opened(ElectionModality::Identified);
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        ballot(ElectionModality::Identified, 60, 'e', 'f'),
        at(20, 11),
    )
    .unwrap();
    assert!(RecordedBallot::try_new(participation, None, None).is_ok());
}

#[test]
fn confirmed_record_requires_the_same_ballot_position_and_digest_as_the_receipt() {
    let mode = ElectionModality::Confidential;
    let election = opened(mode);
    let accepted = ballot(mode, 60, 'e', 'f');
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        accepted.clone(),
        at(20, 11),
    )
    .unwrap();
    let receipt = Receipt::try_new(eid(), fp('b'), fp('e'), 1, fp('c'), encoded("Yg")).unwrap();
    let correct = BoardEntry::try_new(accepted.clone(), 1, fp('b'), fp('c')).unwrap();
    for (entry, receipt) in [
        (None, Some(receipt.clone())),
        (Some(correct.clone()), None),
        (
            Some(BoardEntry::try_new(accepted.clone(), 2, fp('a'), fp('c')).unwrap()),
            Some(receipt.clone()),
        ),
        (
            Some(BoardEntry::try_new(accepted, 1, fp('b'), fp('d')).unwrap()),
            Some(receipt.clone()),
        ),
        (
            Some(BoardEntry::try_new(ballot(mode, 61, 'e', 'f'), 1, fp('b'), fp('c')).unwrap()),
            Some(receipt.clone()),
        ),
        (
            Some(correct),
            Some(
                Receipt::try_new(
                    ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
                    fp('b'),
                    fp('e'),
                    1,
                    fp('c'),
                    encoded("Yg"),
                )
                .unwrap(),
            ),
        ),
    ] {
        assert_eq!(
            RecordedBallot::try_new(participation.clone(), entry, receipt),
            Err(InvalidRecordedBallot)
        );
    }
}

#[test]
fn identified_record_rejects_confidential_evidence() {
    let participation = Participation::try_accept(
        &opened(ElectionModality::Identified),
        account(50),
        voter(40),
        ballot(ElectionModality::Identified, 60, 'e', 'f'),
        at(20, 11),
    )
    .unwrap();
    let entry = BoardEntry::try_new(
        ballot(ElectionModality::Confidential, 60, 'e', 'f'),
        1,
        fp('b'),
        fp('c'),
    )
    .unwrap();
    let receipt = Receipt::try_new(eid(), fp('b'), fp('e'), 1, fp('c'), encoded("Yg")).unwrap();
    for (entry, receipt) in [
        (Some(entry.clone()), None),
        (None, Some(receipt.clone())),
        (Some(entry), Some(receipt)),
    ] {
        assert_eq!(
            RecordedBallot::try_new(participation.clone(), entry, receipt),
            Err(InvalidRecordedBallot)
        );
    }
}

#[test]
fn randomized_ballot_signature_does_not_replace_the_confirmed_board_entry() {
    let mode = ElectionModality::Confidential;
    let election = opened(mode);
    let accepted = ballot(mode, 60, 'e', 'f');
    let participation = Participation::try_accept(
        &election,
        account(50),
        voter(40),
        accepted.clone(),
        at(20, 11),
    )
    .unwrap();
    let original = accepted.ballot();
    let resigned = SignedBallot::try_new(
        original.election_id(),
        original.voter_id(),
        original.submission_id(),
        original.public_key().clone(),
        encoded("Yw"),
        original.content().clone(),
    )
    .unwrap();
    let resigned = VerifiedBallot::from_verified_parts(resigned, fp('e'), fp('f'));
    let receipt = Receipt::try_new(eid(), fp('b'), fp('e'), 1, fp('c'), encoded("Yg")).unwrap();
    // A assinatura externa não muda ballotHash, mas está aninhada no board-entry.
    // Recuperar o aceite não autoriza reconstruir/trocar esse payload persistido.
    let replacement = BoardEntry::try_new(resigned.clone(), 1, fp('b'), fp('c')).unwrap();
    assert_eq!(
        RecordedBallot::try_new(
            participation.clone(),
            Some(replacement),
            Some(receipt.clone())
        ),
        Err(InvalidRecordedBallot)
    );
    let entry = BoardEntry::try_new(accepted, 1, fp('b'), fp('c')).unwrap();
    let record = RecordedBallot::try_new(
        participation.clone(),
        Some(entry.clone()),
        Some(receipt.clone()),
    )
    .unwrap();
    let recovered = record.recover(participation.key(), &resigned).unwrap();
    assert_eq!(recovered.board_entry(), Some(&entry));
    assert_eq!(recovered.receipt(), Some(&receipt));
}

#[test]
fn new_acceptance_cannot_predate_the_current_extension_but_history_can() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = opened(mode);
        let original = Participation::try_accept(
            &election,
            account(50),
            voter(40),
            ballot(mode, 60, 'e', 'f'),
            at(20, 11),
        )
        .unwrap();
        election
            .change_schedule(
                ScheduleChange::try_new(
                    eid(),
                    election.manifest_hash().cloned(),
                    1,
                    fp('a'),
                    ElectionPeriod::try_new(at(20, 10), at(20, 20)).unwrap(),
                    actor(),
                    at(20, 12),
                    "Prorrogação",
                )
                .unwrap(),
                fp('c'),
            )
            .unwrap();
        assert_eq!(
            Participation::try_accept(
                &election,
                account(50),
                voter(40),
                ballot(mode, 60, 'e', 'f'),
                at(20, 11)
            ),
            Err(ParticipationError::InvalidAcceptanceTime)
        );
        assert_eq!(
            Participation::try_reconstitute(
                &election,
                account(50),
                voter(40),
                ballot(mode, 60, 'e', 'f'),
                at(20, 11)
            ),
            Ok(original)
        );
        for time in [at(20, 12), at(20, 19)] {
            assert!(
                Participation::try_accept(
                    &election,
                    account(50),
                    voter(40),
                    ballot(mode, 60, 'e', 'f'),
                    time
                )
                .is_ok()
            );
        }
    }
}

#[test]
fn empty_terminal_snapshot_cannot_reconstitute_a_participation() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let closed = closed(mode, 0);
        let mut cancelled = opened(mode);
        cancelled
            .cancel(
                actor(),
                at(20, 12),
                None,
                VoterCount::try_new(0).unwrap(),
                "Cancelamento",
            )
            .unwrap();
        for election in [&closed, &cancelled] {
            assert_eq!(
                Participation::try_reconstitute(
                    election,
                    account(50),
                    voter(40),
                    ballot(mode, 60, 'e', 'f'),
                    at(20, 11)
                ),
                Err(ParticipationError::EmptyParticipationSnapshot)
            );
        }
    }
}
