//! Modalidades, prontidão, calendário, resultados e snapshots adversariais.

use super::*;

#[test]
fn identified_lifecycle_needs_no_confidential_artifacts() {
    let mut election = opened(ElectionModality::Identified);
    assert!(election.manifest().is_none());
    assert_eq!(election.options().unwrap().len(), 2);
    election
        .close_identified(at(20, 18), VoterCount::try_new(3).unwrap(), fp('d'))
        .unwrap();
    let result = ElectionResult::try_identified(
        eid(),
        fp('d'),
        VoterCount::try_new(3).unwrap(),
        counts(2, 1),
    )
    .unwrap();
    election
        .publish_tally(VerifiedTally::try_identified(result).unwrap())
        .unwrap();
    assert_eq!(election.result_status(), Some(ResultStatus::Valid));
    assert_eq!(election.tally_status(), TallyStatus::Final);
    assert!(election.closure().unwrap().manifest_hash().is_none());
    assert!(election.closure().unwrap().head_hash().is_none());
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
}

#[test]
fn preparation_does_not_require_options_keys_or_electors() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let election = preparing(mode);
        assert!(election.options().is_none());
        assert_eq!(
            Election::try_reconstitute(election.to_snapshot()).unwrap(),
            election
        );
    }
}

#[test]
fn incomplete_configuration_cannot_open_either_modality() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = preparing(mode);
        assert!(election.open(at(20, 10)).is_err());
        assert_eq!(election.status(), ElectionStatus::Preparing);
    }
}

#[test]
fn identified_configuration_is_mutable_only_before_opening() {
    let mut election = configured(ElectionModality::Identified);
    let mut reordered = options();
    reordered.reverse();
    election
        .configure_identified(ElectionOptions::try_new(reordered.clone()).unwrap())
        .unwrap();
    election.open(at(20, 10)).unwrap();
    assert!(
        election
            .configure_identified(ElectionOptions::try_new(options()).unwrap())
            .is_err()
    );
    assert_eq!(election.options().unwrap(), reordered);
}

#[test]
fn configuration_and_closure_cannot_cross_modalities() {
    let mut identified = configured(ElectionModality::Identified);
    assert!(
        identified
            .publish_manifest(VerifiedManifest::from_verified_parts(manifest(), fp('b')))
            .is_err()
    );
    identified.open(at(20, 10)).unwrap();
    assert_eq!(
        identified.close(
            at(20, 18),
            fp('c'),
            VoterCount::try_new(0).unwrap(),
            fp('d')
        ),
        Err(ElectionTransitionError::ModalityMismatch)
    );
    let mut confidential = opened(ElectionModality::Confidential);
    assert!(
        confidential
            .configure_identified(ElectionOptions::try_new(options()).unwrap())
            .is_err()
    );
    assert_eq!(
        confidential.close_identified(at(20, 18), VoterCount::try_new(0).unwrap(), fp('d')),
        Err(ElectionTransitionError::ModalityMismatch)
    );
}

#[test]
fn identified_count_mismatch_annuls_and_zero_votes_are_consistent() {
    for (n, slate, null, expected) in [
        (0, 0, 0, ResultStatus::Valid),
        (3, 1, 1, ResultStatus::Annulled),
        (2, 2, 1, ResultStatus::Annulled),
    ] {
        let mut election = closed(ElectionModality::Identified, n);
        let result = ElectionResult::try_identified(
            eid(),
            fp('d'),
            VoterCount::try_new(n).unwrap(),
            counts(slate, null),
        )
        .unwrap();
        election
            .publish_tally(VerifiedTally::try_identified(result).unwrap())
            .unwrap();
        assert_eq!(election.result_status(), Some(expected));
    }
}

#[test]
fn counts_outside_snapshot_range_are_not_publishable() {
    assert_eq!(
        ElectionResult::try_identified(
            eid(),
            fp('d'),
            VoterCount::try_new(2).unwrap(),
            counts(3, 0)
        ),
        Err(InvalidElectionResult::CountOutOfRange)
    );
    assert!(
        super::super::rules::result_consistency::validate_result_counts(
            &options(),
            &counts(3, 0),
            VoterCount::try_new(2).unwrap()
        )
        .is_err()
    );
}

#[test]
fn missing_or_invalid_authority_can_be_completed_without_replacing_aggregate() {
    for reason in [
        ResultReason::MissingAuthority,
        ResultReason::InvalidAuthorityShare,
    ] {
        let mut election = closed(ElectionModality::Confidential, 2);
        let authority = election.manifest().unwrap().authorities()[2].id();
        let pending = confidential_tally(
            &election,
            ResultStatus::Incomplete,
            Some(reason),
            None,
            vec![authority],
        );
        election.publish_tally(pending).unwrap();
        assert_eq!(election.result_status(), Some(ResultStatus::Incomplete));
        let complete = confidential_tally(
            &election,
            ResultStatus::Valid,
            None,
            Some(counts(1, 1)),
            vec![],
        );
        election.publish_tally(complete).unwrap();
        assert_eq!(election.result_status(), Some(ResultStatus::Valid));
    }
}

#[test]
fn declaring_valid_does_not_replace_missing_contributions() {
    let mut election = closed(ElectionModality::Confidential, 2);
    let valid = confidential_tally(
        &election,
        ResultStatus::Valid,
        None,
        Some(counts(1, 1)),
        vec![],
    );
    for authorities in [
        vec![],
        vec![AuthorityId::try_from_uuid(Uuid::from_u128(999)).unwrap()],
        vec![election.manifest().unwrap().authorities()[0].id(); 3],
    ] {
        let forged = VerifiedTally::from_verified_parts(
            valid.result().clone(),
            valid.aggregate().unwrap().clone(),
            fp('e'),
            authorities,
        );
        assert_eq!(
            election.publish_tally(forged),
            Err(ElectionTransitionError::TallyContributionsMismatch)
        );
    }
    assert_eq!(election.tally_status(), TallyStatus::Pending);
}

#[test]
fn out_of_range_result_is_annulled_without_published_counts() {
    let mut election = closed(ElectionModality::Confidential, 2);
    let result = confidential_tally(
        &election,
        ResultStatus::Annulled,
        Some(ResultReason::TallyOutOfRange),
        None,
        vec![],
    );
    election.publish_tally(result).unwrap();
    assert_eq!(election.result_status(), Some(ResultStatus::Annulled));
    assert!(election.tally_result().unwrap().counts().is_none());
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
}

#[test]
fn aggregate_from_another_closure_or_digest_is_rejected() {
    let mut election = closed(ElectionModality::Confidential, 2);
    let valid = confidential_tally(
        &election,
        ResultStatus::Valid,
        None,
        Some(counts(1, 1)),
        vec![],
    );
    for hash in ['d', 'f'] {
        let aggregate = Aggregate::try_new(
            eid(),
            fp('b'),
            fp('f'),
            valid.aggregate().unwrap().ciphertexts().to_vec(),
        )
        .unwrap();
        let forged = VerifiedTally::from_verified_parts(
            valid.result().clone(),
            aggregate,
            fp(hash),
            valid.contributing_authorities().to_vec(),
        );
        assert_eq!(
            election.publish_tally(forged),
            Err(ElectionTransitionError::TallyAggregateMismatch)
        );
    }
}

#[test]
fn result_cannot_redefine_the_independent_closure_hash() {
    let mut election = closed(ElectionModality::Identified, 2);
    let result = ElectionResult::try_identified(
        eid(),
        fp('f'),
        VoterCount::try_new(2).unwrap(),
        counts(1, 1),
    )
    .unwrap();
    assert_eq!(
        election.publish_tally(VerifiedTally::try_identified(result).unwrap()),
        Err(ElectionTransitionError::TallyClosureMismatch)
    );
    let mut snapshot = election.to_snapshot();
    snapshot.closure_hash = None;
    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(ElectionReconstitutionError::InvalidClosure)
    );
}

#[test]
fn cancellation_remains_distinct_and_preserves_incomplete_evidence() {
    let mut election = closed(ElectionModality::Confidential, 2);
    let authority = election.manifest().unwrap().authorities()[0].id();
    let pending = confidential_tally(
        &election,
        ResultStatus::Incomplete,
        Some(ResultReason::MissingAuthority),
        None,
        vec![authority],
    );
    election.publish_tally(pending).unwrap();
    election
        .cancel(
            actor(),
            at(20, 19),
            Some(fp('c')),
            VoterCount::try_new(2).unwrap(),
            "Cancelamento",
        )
        .unwrap();
    assert_eq!(election.result_status(), Some(ResultStatus::Cancelled));
    assert_eq!(
        election.tally_result().unwrap().status(),
        ResultStatus::Incomplete
    );
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
    assert!(election.open(at(20, 10)).is_err());
}

#[test]
fn preparation_cannot_move_start_backwards() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = configured(mode);
        let update = change(&election, at(20, 9), at(20, 20), at(19, 10));
        assert_eq!(
            election.change_schedule(update, fp('f')),
            Err(ElectionTransitionError::StartCannotMoveEarlier)
        );
    }
}

#[test]
fn postponed_preparation_can_open_late_but_cannot_be_backdated() {
    let mut election = configured(ElectionModality::Identified);
    let update = change(&election, at(21, 10), at(22, 18), at(21, 11));
    election.change_schedule(update, fp('f')).unwrap();
    assert_eq!(
        election.open(at(21, 10)),
        Err(ElectionTransitionError::NonChronologicalOperation)
    );
    election.open(at(21, 12)).unwrap();
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
}

#[test]
fn snapshot_cannot_hide_an_open_extension_as_preparation() {
    let mut election = opened(ElectionModality::Identified);
    let update = change(&election, at(20, 10), at(21, 18), at(20, 12));
    election.change_schedule(update, fp('f')).unwrap();
    let mut snapshot = election.to_snapshot();
    snapshot.opening_schedule_revision = Some(1);
    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(ElectionReconstitutionError::InvalidOpening)
    );
}

#[test]
fn snapshot_opening_must_fit_calendar_at_opening_not_only_extended_calendar() {
    let mut election = opened(ElectionModality::Identified);
    let update = change(&election, at(20, 10), at(21, 18), at(20, 12));
    election.change_schedule(update, fp('f')).unwrap();
    let mut snapshot = election.to_snapshot();
    snapshot.opened_at = Some(at(21, 10));
    assert_eq!(
        Election::try_reconstitute(snapshot),
        Err(ElectionReconstitutionError::InvalidOpening)
    );
}

#[test]
fn snapshot_cannot_remove_configuration_from_cancelled_open_election() {
    let mut election = opened(ElectionModality::Confidential);
    election
        .cancel(
            actor(),
            at(20, 12),
            Some(fp('b')),
            VoterCount::try_new(0).unwrap(),
            "Cancelamento",
        )
        .unwrap();
    let mut snapshot = election.to_snapshot();
    snapshot.configuration = None;
    snapshot.cancellation.as_mut().unwrap().manifest_hash = None;
    assert!(Election::try_reconstitute(snapshot).is_err());
}

#[test]
fn same_second_preparation_and_opening_have_an_unambiguous_revision() {
    let mut election = configured(ElectionModality::Identified);
    let update = change(&election, at(20, 10), at(21, 18), at(20, 10));
    election.change_schedule(update, fp('f')).unwrap();
    election.open(at(20, 10)).unwrap();
    assert_eq!(election.to_snapshot().opening_schedule_revision, Some(1));
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
}

#[test]
fn administrative_result_preserves_both_modalities_without_early_counts() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = opened(mode);
        assert!(election.cancelled_result(fp('e')).is_err());
        election
            .cancel(
                actor(),
                at(20, 12),
                None,
                VoterCount::try_new(0).unwrap(),
                "Cancelamento",
            )
            .unwrap();
        let result = election.cancelled_result(fp('e')).unwrap();
        assert_eq!(result.modality(), mode);
        assert_eq!(result.status(), ResultStatus::Cancelled);
        assert!(result.counts().is_none());
        assert!(result.closure_hash().is_none());
        assert!(result.aggregate_hash().is_none());
        assert_eq!(result.cancellation_hash(), Some(&fp('e')));
    }
}

#[test]
fn final_result_blocks_cancellation_and_any_reopening() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = closed(mode, 2);
        let tally = match mode {
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
        election.publish_tally(tally).unwrap();
        assert!(
            election
                .cancel(
                    actor(),
                    at(20, 19),
                    None,
                    VoterCount::try_new(2).unwrap(),
                    "Cancelamento"
                )
                .is_err()
        );
        assert!(election.open(at(20, 10)).is_err());
    }
}

#[test]
fn an_incomplete_tally_cannot_switch_to_a_different_aggregate() {
    let mut election = closed(ElectionModality::Confidential, 2);
    let missing = vec![election.manifest().unwrap().authorities()[0].id()];
    let pending = confidential_tally(
        &election,
        ResultStatus::Incomplete,
        Some(ResultReason::MissingAuthority),
        None,
        missing,
    );
    election.publish_tally(pending).unwrap();
    let complete = confidential_tally(
        &election,
        ResultStatus::Valid,
        None,
        Some(counts(1, 1)),
        vec![],
    );
    let altered = Aggregate::try_new(
        eid(),
        fp('b'),
        fp('d'),
        [30, 0]
            .into_iter()
            .map(|id| Ciphertext::new(option(id), number(2), number(4)))
            .collect(),
    )
    .unwrap();
    let evidence = VerifiedTally::from_verified_parts(
        complete.result().clone(),
        altered,
        fp('e'),
        complete.contributing_authorities().to_vec(),
    );
    assert_eq!(
        election.publish_tally(evidence),
        Err(ElectionTransitionError::TallyAggregateMismatch)
    );
    assert_eq!(election.tally_status(), TallyStatus::Incomplete);
}

#[test]
fn structural_manifest_rejects_duplicate_public_values_and_degenerate_keys() {
    use super::super::entities::{authority::InvalidAuthority, manifest::InvalidManifest};
    let original = manifest();
    let mut authorities = original.authorities().to_vec();
    let a = authorities[1].clone();
    authorities[1] = Authority::try_new(
        a.id(),
        a.role(),
        a.slate_id(),
        a.signing_key().clone(),
        authorities[0].public_value().clone(),
        a.key_proof().clone(),
    )
    .unwrap();
    assert_eq!(
        Manifest::try_new(
            eid(),
            aid(),
            options(),
            authorities,
            number(8),
            encoded("ZA"),
            fp('a')
        ),
        Err(InvalidManifest::DuplicateAuthorityPublicValue)
    );
    for value in [0, 1] {
        let a = &original.authorities()[0];
        assert_eq!(
            Authority::try_new(
                a.id(),
                a.role(),
                None,
                a.signing_key().clone(),
                number(value),
                a.key_proof().clone()
            ),
            Err(InvalidAuthority::InvalidPublicValue)
        );
        assert_eq!(
            Manifest::try_new(
                eid(),
                aid(),
                options(),
                original.authorities().to_vec(),
                number(value),
                encoded("ZA"),
                fp('a')
            ),
            Err(InvalidManifest::InvalidCombinedPublicKey)
        );
    }
}

#[test]
fn identified_option_limits_and_duplicate_ids_follow_the_profile() {
    assert!(ElectionOptions::try_new(vec![options()[1].clone()]).is_err());
    assert!(
        ElectionOptions::try_new(vec![
            options()[0].clone(),
            options()[0].clone(),
            options()[1].clone()
        ])
        .is_err()
    );
    for slates in [50, 51] {
        let mut choices: Vec<_> = (1..=slates)
            .map(|id| {
                ElectionOption::try_new(option(id), OptionKind::Slate, format!("Chapa {id}"))
                    .unwrap()
            })
            .collect();
        choices.push(options()[1].clone());
        assert_eq!(ElectionOptions::try_new(choices).is_ok(), slates == 50);
    }
    assert!(AccountId::try_from_uuid(Uuid::nil()).is_err());
}

#[test]
fn cancellation_in_preparation_cannot_claim_preexisting_participants() {
    let mut election = preparing(ElectionModality::Identified);
    assert_eq!(
        election.cancel(
            actor(),
            at(19, 12),
            None,
            VoterCount::try_new(1).unwrap(),
            "Cancelamento"
        ),
        Err(ElectionTransitionError::CancellationSnapshotMismatch)
    );
}

#[test]
fn zero_ballots_freeze_manifest_hash_as_the_board_head() {
    let mut election = opened(ElectionModality::Confidential);
    assert_eq!(
        election.close(
            at(20, 18),
            fp('c'),
            VoterCount::try_new(0).unwrap(),
            fp('d')
        ),
        Err(ElectionTransitionError::InvalidEmptyBoard)
    );
    election
        .close(
            at(20, 18),
            fp('b'),
            VoterCount::try_new(0).unwrap(),
            fp('d'),
        )
        .unwrap();
    assert_eq!(election.closure().unwrap().head_hash(), Some(&fp('b')));
}
