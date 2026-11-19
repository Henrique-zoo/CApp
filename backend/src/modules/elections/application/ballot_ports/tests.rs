//! Identidade fornecida por porta e recuperação sem renovação da elegibilidade.

use super::*;
use crate::modules::elections::domain::{
    entities::{
        ballot::BallotContent,
        election::{Election, ElectionStatus},
        participation::ParticipationError,
    },
    value_objects::{
        calendar_origin::CalendarOrigin,
        election_period::ElectionPeriod,
        fingerprint::Fingerprint,
        identifiers::{AcademicCenterId, AccountId, ElectionId, OptionId, SubmissionId, VoterId},
        modality::ElectionModality,
        option::{ElectionOption, OptionKind},
        options::ElectionOptions,
        voter_count::VoterCount,
    },
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

/// Fixture identificada: nenhuma chave ou eleitor é preparado antes da abertura.
fn opened() -> Election {
    let id = ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap();
    let start = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();
    let mut election = Election::try_new(
        id,
        AcademicCenterId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
        ElectionModality::Identified,
        CalendarOrigin::new(id, ElectionPeriod::try_new(start, end).unwrap()),
        Fingerprint::try_from_hex("a".repeat(64)).unwrap(),
    )
    .unwrap();
    election
        .configure_identified(
            ElectionOptions::try_new(vec![
                ElectionOption::try_new(
                    OptionId::from_uuid(Uuid::from_u128(6)),
                    OptionKind::Slate,
                    "Aurora",
                )
                .unwrap(),
                ElectionOption::try_new(OptionId::from_uuid(Uuid::nil()), OptionKind::Null, "Nulo")
                    .unwrap(),
            ])
            .unwrap(),
        )
        .unwrap();
    election.open(start).unwrap();
    election
}

/// Simula o retorno da verificação; não implementa assinatura no teste.
fn verified_ballot(election_id: ElectionId, submission: u128, digest: char) -> VerifiedBallot {
    let ballot = SignedBallot::try_new(
        election_id,
        VoterId::try_from_uuid(Uuid::from_u128(3)).unwrap(),
        SubmissionId::try_from_uuid(Uuid::from_u128(submission)).unwrap(),
        Base64Url::try_new("YQ").unwrap(),
        Base64Url::try_new("Yg").unwrap(),
        BallotContent::Identified(OptionId::from_uuid(Uuid::from_u128(6))),
    )
    .unwrap();
    VerifiedBallot::from_verified_parts(
        ballot,
        Fingerprint::try_from_hex(digest.to_string().repeat(64)).unwrap(),
        Fingerprint::try_from_hex("b".repeat(64)).unwrap(),
    )
}

fn context(election_id: ElectionId, account: u128, voter: u128) -> AuthenticatedElectorContext {
    AuthenticatedElectorContext::new(
        AccountId::try_from_uuid(Uuid::from_u128(account)).unwrap(),
        VoterId::try_from_uuid(Uuid::from_u128(voter)).unwrap(),
        election_id,
    )
}

#[test]
fn new_acceptance_keeps_the_authenticated_account_and_public_voter_together() {
    let election = opened();
    let authenticated = context(election.id(), 4, 3);
    let accepted = authenticated
        .prepare_participation(
            &election,
            verified_ballot(election.id(), 5, 'c'),
            *election.current_period().starts_at(),
        )
        .unwrap();
    assert_eq!(accepted.key(), authenticated.participation_key());
    assert_eq!(
        accepted.key().account_id(),
        AccountId::try_from_uuid(Uuid::from_u128(4)).unwrap()
    );
    assert_eq!(
        accepted.accepted_ballot().ballot().voter_id(),
        authenticated.voter_id()
    );
}

#[test]
fn another_election_or_electoral_identity_cannot_be_combined_with_the_ballot() {
    let election = opened();
    let other = ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap();
    for authenticated in [context(other, 4, 3), context(election.id(), 4, 99)] {
        assert_eq!(
            authenticated.prepare_participation(
                &election,
                verified_ballot(election.id(), 5, 'c'),
                *election.current_period().starts_at()
            ),
            Err(ParticipationError::IdentityMismatch)
        );
    }
    assert_eq!(
        context(election.id(), 4, 3).prepare_participation(
            &election,
            verified_ballot(other, 5, 'c'),
            *election.current_period().starts_at()
        ),
        Err(ParticipationError::IdentityMismatch)
    );
}

#[test]
fn record_recovery_does_not_require_a_new_acceptance_after_closure() {
    let mut election = opened();
    let authenticated = context(election.id(), 4, 3);
    let ballot = verified_ballot(election.id(), 5, 'c');
    let participation = authenticated
        .prepare_participation(
            &election,
            ballot.clone(),
            *election.current_period().starts_at(),
        )
        .unwrap();
    let record = RecordedBallot::try_new(participation, None, None).unwrap();
    election
        .close_identified(
            *election.current_period().ends_at(),
            VoterCount::try_new(1).unwrap(),
            Fingerprint::try_from_hex("d".repeat(64)).unwrap(),
        )
        .unwrap();
    assert_eq!(election.status(), ElectionStatus::Closed);
    assert_eq!(
        record.recover(authenticated.participation_key(), &ballot),
        Ok(&record)
    );
    assert_eq!(
        authenticated.prepare_participation(
            &election,
            ballot,
            *election.current_period().starts_at()
        ),
        Err(ParticipationError::ElectionNotOpen)
    );
}

#[test]
fn record_recovery_rejects_another_account_operation_or_payload() {
    let election = opened();
    let authenticated = context(election.id(), 4, 3);
    let ballot = verified_ballot(election.id(), 5, 'c');
    let participation = authenticated
        .prepare_participation(
            &election,
            ballot.clone(),
            *election.current_period().starts_at(),
        )
        .unwrap();
    let record = RecordedBallot::try_new(participation, None, None).unwrap();
    assert_eq!(
        record.recover(context(election.id(), 99, 3).participation_key(), &ballot),
        Err(ParticipationError::IdentityMismatch)
    );
    assert_eq!(
        record.recover(
            authenticated.participation_key(),
            &verified_ballot(election.id(), 6, 'c')
        ),
        Err(ParticipationError::AlreadyParticipated)
    );
    assert_eq!(
        record.recover(
            authenticated.participation_key(),
            &verified_ballot(election.id(), 5, 'd')
        ),
        Err(ParticipationError::SubmissionConflict)
    );
}
