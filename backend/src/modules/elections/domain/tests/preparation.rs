//! Edição da preparação preservando manifestos e referências históricas.

use super::*;
use crate::modules::elections::domain::entities::{
    configuration::ElectionConfiguration, election::InvalidElection,
    election_snapshot::PreparationConfigurationChangeSnapshot,
};

/// Simula um manifesto revalidado após edição das opções na preparação.
fn revised_manifest() -> Manifest {
    let previous = manifest();
    let mut options = previous.options().to_vec();
    options[0] = ElectionOption::try_new(option(30), OptionKind::Slate, "Aurora revisada").unwrap();
    Manifest::try_new(
        eid(),
        aid(),
        options,
        previous.authorities().to_vec(),
        previous.combined_public_key().clone(),
        previous.server_signing_key().clone(),
        fp('a'),
    )
    .unwrap()
}

#[test]
fn invalidating_configuration_preserves_calendar_and_requires_new_preparation() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let mut election = configured(mode);
        let calendar = election.current_period().clone();
        election.invalidate_configuration().unwrap();
        assert!(election.options().is_none());
        assert!(election.manifest_hash().is_none());
        assert_eq!(election.current_period(), &calendar);
        assert_eq!(election.current_schedule_hash(), &fp('a'));
        assert_eq!(election.configuration_changes().len(), 2);
        assert!(
            election.configuration_changes()[0]
                .configuration()
                .is_some()
        );
        assert!(election.open(at(20, 10)).is_err());
        assert_eq!(
            Election::try_reconstitute(election.to_snapshot()).unwrap(),
            election
        );
    }
}

#[test]
fn repeated_invalidation_and_identical_identified_options_are_idempotent() {
    let mut election = configured(ElectionModality::Identified);
    election
        .configure_identified(ElectionOptions::try_new(options()).unwrap())
        .unwrap();
    assert_eq!(election.configuration_changes().len(), 1);
    election.invalidate_configuration().unwrap();
    let invalidated = election.to_snapshot();
    election.invalidate_configuration().unwrap();
    assert_eq!(election.to_snapshot(), invalidated);
}

#[test]
fn republishing_a_manifest_requires_explicit_invalidation() {
    let mut election = configured(ElectionModality::Confidential);
    let previous = election.clone();
    assert_eq!(
        election.publish_manifest(VerifiedManifest::from_verified_parts(
            revised_manifest(),
            fp('f')
        )),
        Err(InvalidElection::ManifestAlreadyPublished)
    );
    assert_eq!(election, previous);
    election.invalidate_configuration().unwrap();
    election
        .publish_manifest(VerifiedManifest::from_verified_parts(
            revised_manifest(),
            fp('f'),
        ))
        .unwrap();
    assert_eq!(election.manifest_hash(), Some(&fp('f')));
    assert_eq!(election.options().unwrap()[0].name(), "Aurora revisada");
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
}

#[test]
fn old_manifest_calendar_revisions_survive_invalidation_republication_and_tally() {
    let mut election = configured(ElectionModality::Confidential);
    let first = change(&election, at(21, 10), at(21, 18), at(19, 10));
    election.change_schedule(first, fp('c')).unwrap();
    election.invalidate_configuration().unwrap();
    let without_manifest = change(&election, at(22, 10), at(22, 18), at(19, 11));
    election.change_schedule(without_manifest, fp('d')).unwrap();
    election
        .publish_manifest(VerifiedManifest::from_verified_parts(
            revised_manifest(),
            fp('f'),
        ))
        .unwrap();
    election.open(at(22, 10)).unwrap();
    let extension = change(&election, at(22, 10), at(23, 18), at(22, 12));
    election.change_schedule(extension, fp('e')).unwrap();
    assert_eq!(
        election.schedule_changes()[0]
            .schedule_change()
            .manifest_hash(),
        Some(&fp('b'))
    );
    assert_eq!(
        election.schedule_changes()[1]
            .schedule_change()
            .manifest_hash(),
        None
    );
    assert_eq!(
        election.schedule_changes()[2]
            .schedule_change()
            .manifest_hash(),
        Some(&fp('f'))
    );
    assert!(
        ballot(ElectionModality::Confidential, 60, '1', '2')
            .ballot()
            .validate_for(&election)
            .is_err()
    );
    election
        .close(
            at(23, 18),
            fp('7'),
            VoterCount::try_new(2).unwrap(),
            fp('9'),
        )
        .unwrap();
    let result = ElectionResult::try_new(
        eid(),
        Some(fp('f')),
        Some(fp('9')),
        Some(fp('8')),
        None,
        VoterCount::try_new(2).unwrap(),
        ResultStatus::Valid,
        None,
        Some(counts(1, 1)),
        vec![],
    )
    .unwrap();
    let aggregate = Aggregate::try_new(
        eid(),
        fp('f'),
        fp('9'),
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
        .collect();
    election
        .publish_tally(VerifiedTally::from_verified_parts(
            result,
            aggregate,
            fp('8'),
            authorities,
        ))
        .unwrap();
    let restored = Election::try_reconstitute(election.to_snapshot()).unwrap();
    assert_eq!(restored, election);
    assert_eq!(restored.result_status(), Some(ResultStatus::Valid));
    assert_eq!(restored.configuration_changes().len(), 3);
}

#[test]
fn cancellation_after_invalidation_retains_previous_public_evidence() {
    let mut election = configured(ElectionModality::Confidential);
    let update = change(&election, at(21, 10), at(21, 18), at(19, 10));
    election.change_schedule(update, fp('c')).unwrap();
    election.invalidate_configuration().unwrap();
    election
        .cancel(
            actor(),
            at(19, 11),
            None,
            VoterCount::try_new(0).unwrap(),
            "Configuração incorreta",
        )
        .unwrap();
    assert!(
        election
            .cancelled_result(fp('d'))
            .unwrap()
            .manifest_hash()
            .is_none()
    );
    assert_eq!(election.configuration_changes().len(), 2);
    let restored = Election::try_reconstitute(election.to_snapshot()).unwrap();
    assert_eq!(
        restored.schedule_changes()[0]
            .schedule_change()
            .manifest_hash(),
        Some(&fp('b'))
    );
    assert_eq!(restored, election);
}

#[test]
fn opening_or_terminal_state_prevents_invalidation_in_both_modalities() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        for mut election in [opened(mode), closed(mode, 2)] {
            let before = election.clone();
            assert_eq!(
                election.invalidate_configuration(),
                Err(InvalidElection::ConfigurationAfterPreparation)
            );
            assert_eq!(election, before);
        }
        let mut cancelled = configured(mode);
        cancelled
            .cancel(
                actor(),
                at(19, 10),
                cancelled.manifest_hash().cloned(),
                VoterCount::try_new(0).unwrap(),
                "Cancelamento",
            )
            .unwrap();
        assert_eq!(
            cancelled.invalidate_configuration(),
            Err(InvalidElection::ConfigurationAfterPreparation)
        );
    }
}

#[test]
fn snapshot_cannot_erase_or_reorder_configuration_publications() {
    let mut election = configured(ElectionModality::Confidential);
    let update = change(&election, at(21, 10), at(21, 18), at(19, 10));
    election.change_schedule(update, fp('c')).unwrap();
    election.invalidate_configuration().unwrap();
    election
        .publish_manifest(VerifiedManifest::from_verified_parts(
            revised_manifest(),
            fp('f'),
        ))
        .unwrap();
    let mut removed = election.to_snapshot();
    removed.configuration_changes.remove(0);
    assert!(Election::try_reconstitute(removed).is_err());
    let mut reordered = election.to_snapshot();
    reordered.configuration_changes.reverse();
    assert!(Election::try_reconstitute(reordered).is_err());
    let mut future = election.to_snapshot();
    future.configuration_changes[2].after_schedule_revision = 2;
    assert_eq!(
        Election::try_reconstitute(future),
        Err(ElectionReconstitutionError::InvalidConfigurationHistory)
    );
}

#[test]
fn configuration_history_cannot_invalidate_after_opening_or_inject_another_modality() {
    let mut election = opened(ElectionModality::Confidential);
    let extension = change(&election, at(20, 10), at(21, 18), at(20, 12));
    election.change_schedule(extension, fp('c')).unwrap();
    let mut invalidated = election.to_snapshot();
    invalidated
        .configuration_changes
        .push(PreparationConfigurationChangeSnapshot {
            after_schedule_revision: 1,
            configuration: None,
        });
    invalidated.configuration = None;
    assert_eq!(
        Election::try_reconstitute(invalidated),
        Err(ElectionReconstitutionError::InvalidConfigurationHistory)
    );
    let mut wrong_mode = configured(ElectionModality::Confidential).to_snapshot();
    wrong_mode.configuration_changes[0].configuration = Some(ElectionConfiguration::Identified(
        ElectionOptions::try_new(options()).unwrap(),
    ));
    assert_eq!(
        Election::try_reconstitute(wrong_mode),
        Err(ElectionReconstitutionError::InvalidConfigurationHistory)
    );
}

#[test]
fn malformed_replacement_never_restores_readiness() {
    let mut election = configured(ElectionModality::Confidential);
    election.invalidate_configuration().unwrap();
    let original = manifest();
    let wrong = Manifest::try_new(
        ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap(),
        aid(),
        options(),
        original.authorities().to_vec(),
        original.combined_public_key().clone(),
        original.server_signing_key().clone(),
        fp('a'),
    )
    .unwrap();
    assert_eq!(
        election.publish_manifest(VerifiedManifest::from_verified_parts(wrong, fp('f'))),
        Err(InvalidElection::ManifestElectionMismatch)
    );
    assert!(election.options().is_none());
    assert_eq!(election.configuration_changes().len(), 2);
    assert_eq!(
        election.open(at(20, 10)),
        Err(ElectionTransitionError::ManifestRequired)
    );
}

#[test]
fn opening_freezes_configuration_position_even_in_the_same_calendar_revision() {
    for mode in [ElectionModality::Identified, ElectionModality::Confidential] {
        let election = opened(mode);
        assert_eq!(election.to_snapshot().opening_configuration_count, Some(1));
        let mut appended = election.to_snapshot();
        appended
            .configuration_changes
            .push(PreparationConfigurationChangeSnapshot {
                after_schedule_revision: 0,
                configuration: None,
            });
        appended
            .configuration_changes
            .push(PreparationConfigurationChangeSnapshot {
                after_schedule_revision: 0,
                configuration: election.to_snapshot().configuration,
            });
        assert_eq!(
            Election::try_reconstitute(appended),
            Err(ElectionReconstitutionError::InvalidOpening)
        );
    }
}
