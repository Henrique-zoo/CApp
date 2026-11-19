//! Compatibilidade semântica com o corpus público da #377.
//!
//! JSON é lido somente no teste e convertido pelos construtores de domínio. As
//! assinaturas/bytes/provas são conferidos pela suíte Python do contrato e pelas
//! futuras implementações criptográficas; estes testes não implementam um codec.

use super::*;
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/elections/vectors.json"
    )))
    .unwrap()
}
fn case<'a>(corpus: &'a Value, name: &str) -> &'a Value {
    corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == name)
        .unwrap()
}
fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap()
}
fn uuid(value: &Value, key: &str) -> Uuid {
    Uuid::parse_str(text(value, key)).unwrap()
}
fn hash(value: &Value, key: &str) -> Fingerprint {
    Fingerprint::try_from_hex(text(value, key)).unwrap()
}
fn optional_hash(value: &Value, key: &str) -> Option<Fingerprint> {
    value[key]
        .as_str()
        .map(|s| Fingerprint::try_from_hex(s).unwrap())
}
fn time(value: &Value, key: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text(value, key))
        .unwrap()
        .with_timezone(&Utc)
}
fn election_id(value: &Value) -> ElectionId {
    ElectionId::try_from_uuid(uuid(value, "electionId")).unwrap()
}
fn option_id(value: &Value, key: &str) -> OptionId {
    OptionId::from_uuid(uuid(value, key))
}
fn big(value: &Value, key: &str) -> BigUInt {
    BigUInt::try_from_decimal(text(value, key)).unwrap()
}
fn ciphertexts(value: &Value) -> Vec<Ciphertext> {
    value["ciphertexts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| Ciphertext::new(option_id(c, "optionId"), big(c, "a"), big(c, "b")))
        .collect()
}

/// Reconstitui apenas os valores públicos estruturais; não valida provas no teste.
fn read_manifest(value: &Value) -> Manifest {
    assert_eq!(value["groupId"], "rfc3526-modp14");
    let options = value["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            ElectionOption::try_new(
                option_id(o, "id"),
                match text(o, "kind") {
                    "slate" => OptionKind::Slate,
                    "null" => OptionKind::Null,
                    _ => panic!("Opção desconhecida"),
                },
                text(o, "name"),
            )
            .unwrap()
        })
        .collect();
    let authorities = value["authorities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            Authority::try_new(
                AuthorityId::try_from_uuid(uuid(a, "id")).unwrap(),
                match text(a, "role") {
                    "server" => AuthorityRole::Server,
                    "commission" => AuthorityRole::Commission,
                    "slate" => AuthorityRole::Slate,
                    _ => panic!("Autoridade desconhecida"),
                },
                a["slateId"]
                    .as_str()
                    .map(|id| SlateId::try_from_uuid(Uuid::parse_str(id).unwrap()).unwrap()),
                encoded(text(a, "signingKey")),
                big(a, "publicValue"),
                KeyProof::new(
                    big(&a["keyProof"], "commitment"),
                    big(&a["keyProof"], "response"),
                ),
            )
            .unwrap()
        })
        .collect();
    Manifest::try_new(
        election_id(value),
        AcademicCenterId::try_from_uuid(uuid(value, "academicCenterId")).unwrap(),
        options,
        authorities,
        big(value, "combinedPublicKey"),
        encoded(text(value, "serverSigningKey")),
        hash(value, "calendarOriginHash"),
    )
    .unwrap()
}
fn read_change(value: &Value) -> ScheduleChange {
    ScheduleChange::try_new(
        election_id(value),
        optional_hash(value, "manifestHash"),
        value["revision"].as_u64().unwrap().try_into().unwrap(),
        hash(value, "previousScheduleHash"),
        ElectionPeriod::try_new(time(value, "startsAt"), time(value, "endsAt")).unwrap(),
        ActorId::try_from_uuid(uuid(value, "changedBy")).unwrap(),
        time(value, "changedAt"),
        text(value, "reason"),
    )
    .unwrap()
}
fn read_result(value: &Value) -> ElectionResult {
    let status = match text(value, "status") {
        "valid" => ResultStatus::Valid,
        "annulled" => ResultStatus::Annulled,
        "incomplete" => ResultStatus::Incomplete,
        "cancelled" => ResultStatus::Cancelled,
        _ => panic!("Resultado desconhecido"),
    };
    let reason = value["reason"].as_str().map(|reason| match reason {
        "missing_authority" => ResultReason::MissingAuthority,
        "invalid_authority_share" => ResultReason::InvalidAuthorityShare,
        "aggregate_sum_mismatch" => ResultReason::AggregateSumMismatch,
        "tally_out_of_range" => ResultReason::TallyOutOfRange,
        "manual_cancellation" => ResultReason::ManualCancellation,
        _ => panic!("Motivo desconhecido"),
    });
    let counts = value["counts"].as_array().map(|counts| {
        counts
            .iter()
            .map(|c| {
                OptionCount::try_new(
                    option_id(c, "optionId"),
                    c["count"].as_u64().unwrap().try_into().unwrap(),
                )
                .unwrap()
            })
            .collect()
    });
    let missing = value["missingAuthorities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| {
            AuthorityId::try_from_uuid(Uuid::parse_str(id.as_str().unwrap()).unwrap()).unwrap()
        })
        .collect();
    ElectionResult::try_new(
        election_id(value),
        optional_hash(value, "manifestHash"),
        optional_hash(value, "closureHash"),
        optional_hash(value, "aggregateHash"),
        optional_hash(value, "cancellationHash"),
        VoterCount::try_new(value["voterCount"].as_u64().unwrap().try_into().unwrap()).unwrap(),
        status,
        reason,
        counts,
        missing,
    )
    .unwrap()
}

#[test]
fn public_protocol_scenarios_preserve_domain_outcomes_and_reconstitution() {
    let corpus = corpus();
    for scenario in corpus["scenarios"].as_array().unwrap() {
        let name = text(scenario, "name");
        let initial = &case(&corpus, &format!("{name}/calendar-origin"))["payload"];
        let manifest_case = case(&corpus, &format!("{name}/manifest"));
        let manifest = read_manifest(&manifest_case["payload"]);
        let mut election = Election::try_new(
            manifest.election_id(),
            manifest.academic_center_id(),
            ElectionModality::Confidential,
            CalendarOrigin::new(
                election_id(initial),
                ElectionPeriod::try_new(time(initial, "startsAt"), time(initial, "endsAt"))
                    .unwrap(),
            ),
            hash(&manifest_case["payload"], "calendarOriginHash"),
        )
        .unwrap();
        let delay = case(&corpus, &format!("{name}/delay"));
        election
            .change_schedule(read_change(&delay["payload"]), hash(delay, "sha256"))
            .unwrap();
        election
            .publish_manifest(VerifiedManifest::from_verified_parts(
                manifest,
                hash(manifest_case, "sha256"),
            ))
            .unwrap();
        election
            .open(*election.current_period().starts_at())
            .unwrap();
        let extension = case(&corpus, &format!("{name}/extension"));
        election
            .change_schedule(
                read_change(&extension["payload"]),
                hash(extension, "sha256"),
            )
            .unwrap();

        // As mesmas cédulas do corpus preservam dimensão, ordem e contexto.
        for ballot in corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| text(c, "name").starts_with(&format!("{name}/ballot-")))
        {
            let payload = &ballot["payload"];
            let signed = SignedBallot::try_new(
                election_id(payload),
                VoterId::try_from_uuid(uuid(payload, "voterId")).unwrap(),
                SubmissionId::try_from_uuid(uuid(payload, "submissionId")).unwrap(),
                encoded(text(payload, "publicKey")),
                encoded(text(ballot, "signature")),
                BallotContent::Confidential {
                    manifest_hash: hash(payload, "manifestHash"),
                    ciphertexts: ciphertexts(payload),
                },
            )
            .unwrap();
            signed.validate_for(&election).unwrap();
        }
        let closure_case = case(&corpus, &format!("{name}/closure"));
        let closure = &closure_case["payload"];
        let n = VoterCount::try_new(closure["voterCount"].as_u64().unwrap().try_into().unwrap())
            .unwrap();
        election
            .close(
                time(closure, "closedAt"),
                hash(closure, "headHash"),
                n,
                hash(closure_case, "sha256"),
            )
            .unwrap();
        let aggregate_case = case(&corpus, &format!("{name}/aggregate"));
        let payload = &aggregate_case["payload"];
        let aggregate = Aggregate::try_new(
            election_id(payload),
            hash(payload, "manifestHash"),
            hash(payload, "closureHash"),
            ciphertexts(payload),
        )
        .unwrap();
        let result = read_result(&case(&corpus, &format!("{name}/result"))["payload"]);
        let shares: Vec<_> = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| text(c, "name").starts_with(&format!("{name}/share-")))
            .map(|c| AuthorityId::try_from_uuid(uuid(&c["payload"], "authorityId")).unwrap())
            .collect();
        if name == "valid" {
            let pending = read_result(&case(&corpus, "valid/incomplete")["payload"]);
            let contributions = shares
                .iter()
                .copied()
                .filter(|id| !pending.missing_authorities().contains(id))
                .collect();
            election
                .publish_tally(VerifiedTally::from_verified_parts(
                    pending,
                    aggregate.clone(),
                    hash(aggregate_case, "sha256"),
                    contributions,
                ))
                .unwrap();
            assert_eq!(election.tally_status(), TallyStatus::Incomplete);
        }
        election
            .publish_tally(VerifiedTally::from_verified_parts(
                result.clone(),
                aggregate,
                hash(aggregate_case, "sha256"),
                shares,
            ))
            .unwrap();
        assert_eq!(election.result_status(), Some(result.status()), "{name}");
        assert_eq!(
            Election::try_reconstitute(election.to_snapshot()).unwrap(),
            election,
            "{name}"
        );
        if let Some(counts) = result.counts() {
            let totals: Vec<_> = counts.iter().map(|c| c.count().value()).collect();
            assert_eq!(
                serde_json::json!(totals),
                scenario["expectedTotals"],
                "{name}"
            );
        }
        if name == "compensated" {
            assert_eq!(scenario["assumptionViolation"], true);
            assert_eq!(
                result.status(),
                ResultStatus::Valid,
                "Igualdade agregada não é prova de validade individual"
            );
        }
        if name == "out-of-range" {
            assert_eq!(result.reason(), Some(ResultReason::TallyOutOfRange));
        }
    }
}

#[test]
fn public_cancellation_before_manifest_has_no_tally() {
    let corpus = corpus();
    let initial = case(&corpus, "cancelled/calendar-origin");
    let payload = &initial["payload"];
    let mut election = Election::try_new(
        election_id(payload),
        aid(),
        ElectionModality::Confidential,
        CalendarOrigin::new(
            election_id(payload),
            ElectionPeriod::try_new(time(payload, "startsAt"), time(payload, "endsAt")).unwrap(),
        ),
        hash(initial, "sha256"),
    )
    .unwrap();
    let cancellation = &case(&corpus, "cancelled/cancellation")["payload"];
    election
        .cancel(
            ActorId::try_from_uuid(uuid(cancellation, "requestedBy")).unwrap(),
            time(cancellation, "cancelledAt"),
            None,
            VoterCount::try_new(0).unwrap(),
            text(cancellation, "reason"),
        )
        .unwrap();
    assert_eq!(election.result_status(), Some(ResultStatus::Cancelled));
    assert!(election.tally_result().is_none());
    assert_eq!(
        read_result(&case(&corpus, "cancelled/result")["payload"]).status(),
        ResultStatus::Cancelled
    );
    assert_eq!(
        Election::try_reconstitute(election.to_snapshot()).unwrap(),
        election
    );
}

#[test]
fn domain_limits_match_the_versioned_profile() {
    let profile: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/elections/profile.json"
    )))
    .unwrap();
    use super::super::entities::manifest::{MAX_AUTHORITIES, MAX_OPTIONS, MIN_OPTIONS};
    use super::super::value_objects::{
        base64_url::BASE64_URL_MAX_LENGTH, big_uint::BIG_UINT_MAX_LENGTH,
        voter_count::MAX_VOTER_COUNT,
    };
    let schema: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/elections/schemas.json"
    )))
    .unwrap();
    assert_eq!(
        MIN_OPTIONS as u64,
        profile["limits"]["minSlates"].as_u64().unwrap() + 1
    );
    assert_eq!(
        MAX_OPTIONS as u64,
        profile["limits"]["maxSlates"].as_u64().unwrap() + 1
    );
    assert_eq!(
        serde_json::json!(MAX_AUTHORITIES),
        profile["limits"]["maxAuthorities"]
    );
    assert_eq!(
        serde_json::json!(MAX_VOTER_COUNT),
        profile["limits"]["maxVoters"]
    );
    assert_eq!(
        serde_json::json!(BIG_UINT_MAX_LENGTH),
        schema["$defs"]["BigUInt"]["maxLength"]
    );
    assert_eq!(
        serde_json::json!(BASE64_URL_MAX_LENGTH),
        schema["$defs"]["Base64Url"]["maxLength"]
    );
}
