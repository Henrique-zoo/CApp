use super::*;

use crate::modules::elections::{
    application::{
        error::{ElectionReaderError, ElectionStoreError},
        ports::{ElectionReader, ElectionStore, VersionedElection},
    },
    domain::{
        entities::election::{Election, ElectionStatus},
        value_objects::{
            calendar_origin::CalendarOrigin,
            election_period::ElectionPeriod,
            fingerprint::Fingerprint,
            identifiers::{AcademicCenterId, ElectionId},
        },
    },
};

use chrono::{TimeZone, Utc};
use uuid::Uuid;

struct FakeElectionStore {
    result: Result<Option<VersionedElection>, ElectionReaderError>,
}

#[async_trait::async_trait]
impl ElectionReader for FakeElectionStore {
    async fn find_by_id(
        &self,
        _election_id: ElectionId,
    ) -> Result<Option<VersionedElection>, ElectionReaderError> {
        self.result.clone()
    }
}

#[async_trait::async_trait]
impl ElectionStore for FakeElectionStore {
    async fn save(
        &self,
        _election: &Election,
        expected_version: u64,
    ) -> Result<u64, ElectionStoreError> {
        Ok(expected_version + 1)
    }
}

fn sample_election() -> Election {
    let election_id = ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap();

    let center_id = AcademicCenterId::try_from_uuid(Uuid::from_u128(2)).unwrap();

    let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
    let ends_at = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();

    let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();
    let origin = CalendarOrigin::new(election_id, period);
    let origin_hash = Fingerprint::try_from_hex("a".repeat(64)).unwrap();

    Election::try_new(
        election_id,
        center_id,
        crate::modules::elections::domain::value_objects::modality::ElectionModality::Confidential,
        origin,
        origin_hash,
    )
    .unwrap()
}

#[tokio::test]
async fn returns_election_when_it_exists() {
    let election = sample_election();
    let election_id = election.id();

    let store = FakeElectionStore {
        result: Ok(Some(VersionedElection::new(election, 7))),
    };

    let use_case = GetElection::new(&store);
    let result = use_case.execute(election_id).await.unwrap();

    assert_eq!(result.id(), election_id);
    assert_eq!(result.status(), ElectionStatus::Preparing);
}

#[tokio::test]
async fn returns_not_found_when_election_does_not_exist() {
    let election_id = ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap();

    let store = FakeElectionStore { result: Ok(None) };

    let use_case = GetElection::new(&store);
    let result = use_case.execute(election_id).await;

    assert!(matches!(result, Err(GetElectionError::NotFound)));
}

#[tokio::test]
async fn preserves_storage_error() {
    let election_id = ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap();

    let store = FakeElectionStore {
        result: Err(ElectionReaderError::Internal),
    };

    let use_case = GetElection::new(&store);
    let result = use_case.execute(election_id).await;

    assert!(matches!(
        result,
        Err(GetElectionError::Reader(ElectionReaderError::Internal))
    ));
}

#[tokio::test]
async fn rejects_a_stored_election_with_another_identifier() {
    let requested = ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap();
    let store = FakeElectionStore {
        result: Ok(Some(VersionedElection::new(sample_election(), 7))),
    };
    assert_eq!(
        GetElection::new(&store).execute(requested).await,
        Err(GetElectionError::Reader(
            ElectionReaderError::InvalidStoredData
        ))
    );
}
