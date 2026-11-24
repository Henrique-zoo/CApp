//! Compatibilidade semântica com o corpus público da #377.
//!
//! JSON é lido somente no teste e convertido pelos construtores de domínio. As
//! assinaturas/bytes/provas são conferidos pela suíte Python do contrato e pelas
//! futuras implementações criptográficas; estes testes não implementam um codec.

use crate::modules::elections::{
    domain::{
        Fingerprint,
        choices::ElectionOption,
        counts::{OptionCount, VoterCount},
        cryptography::elgamal::ElGamalCiphertext,
        entities::{
            authority::{Authority, AuthorityRole, KeyProof},
            ballot::{BallotCiphertexts, BallotContent, Ciphertext, SignedBallot},
            configuration::ElectionConfiguration,
            election::{
                Election, ElectionModality, ElectionPeriod, ElectionState, ElectionTransition,
            },
            manifest::{Manifest, VerifiedManifest},
            result::{FinalTallyResult, TallyComputation},
        },
        error::ElectionTransitionError,
        identifiers::{AcademicCenterId, AccountId, AuthorityId, ElectionId, SlateId},
        rules::board_snapshot::validate_confidential_board_snapshot,
    },
    infrastructure::codec::{BASE64_URL_MAX_LENGTH, decode_fingerprint},
    test_support::{aid, at, decoded},
};
use chrono::{DateTime, Utc};
use crypto_bigint::U2048;
use serde_json::Value;
use uuid::Uuid;

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
    decode_fingerprint(text(value, key)).unwrap()
}
fn time(value: &Value, key: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text(value, key))
        .unwrap()
        .with_timezone(&Utc)
}
fn period(value: &Value) -> ElectionPeriod {
    ElectionPeriod::try_new(time(value, "startsAt"), time(value, "endsAt")).unwrap()
}
fn election_id(value: &Value) -> ElectionId {
    ElectionId::try_from_uuid(uuid(value, "electionId")).unwrap()
}
fn uint2048(value: &Value, key: &str) -> U2048 {
    U2048::from_str_radix_vartime(text(value, key), 10).unwrap()
}
fn ciphertexts(value: &Value) -> Vec<Ciphertext> {
    value["ciphertexts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            Ciphertext::new(
                uuid(c, "optionId"),
                ElGamalCiphertext {
                    a: uint2048(c, "a"),
                    b: uint2048(c, "b"),
                },
            )
        })
        .collect()
}

/// Reconstitui apenas os valores públicos estruturais; não valida provas no teste.
fn read_manifest(value: &Value) -> Manifest {
    assert_eq!(value["groupId"], "rfc3526-modp14");
    let options = value["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|option| match text(option, "kind") {
            "slate" => ElectionOption::try_slate(
                SlateId::try_from_uuid(uuid(option, "id")).unwrap(),
                text(option, "name"),
            )
            .unwrap(),
            "null" => {
                assert!(uuid(option, "id").is_nil());
                assert_eq!(text(option, "name"), ElectionOption::NULL_OPTION_NAME);
                ElectionOption::Null
            }
            kind => panic!("Opção desconhecida: {kind}"),
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
                    "slate" => {
                        AuthorityRole::Slate(SlateId::try_from_uuid(uuid(a, "slateId")).unwrap())
                    }
                    _ => panic!("Autoridade desconhecida"),
                },
                decoded(text(a, "signingKey")),
                uint2048(a, "publicValue"),
                KeyProof::new(
                    uint2048(&a["keyProof"], "commitment"),
                    uint2048(&a["keyProof"], "response"),
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
        uint2048(value, "combinedPublicKey"),
        decoded(text(value, "serverSigningKey")),
    )
    .unwrap()
}

fn read_counts(value: &Value) -> Option<Vec<OptionCount>> {
    value["counts"].as_array().map(|counts| {
        counts
            .iter()
            .map(|c| {
                OptionCount::try_new(
                    uuid(c, "optionId"),
                    c["count"].as_u64().unwrap().try_into().unwrap(),
                )
                .unwrap()
            })
            .collect()
    })
}

#[test]
fn public_protocol_scenarios_preserve_domain_outcomes() {
    let corpus = corpus();
    for scenario in corpus["scenarios"].as_array().unwrap() {
        let name = text(scenario, "name");
        let voting_period = period(&corpus["periods"][name]);
        let opened_at = *voting_period.starts_at();
        let manifest_case = case(&corpus, &format!("{name}/manifest"));
        let manifest = read_manifest(&manifest_case["payload"]);
        let mut election = Election::new(
            manifest.election_id(),
            manifest.academic_center_id(),
            voting_period.clone(),
            ElectionModality::Confidential,
            at(14, 10),
        );
        let configuration = ElectionConfiguration::Confidential(Box::new(
            VerifiedManifest::from_verified_parts(manifest, hash(manifest_case, "sha256")),
        ));
        election
            .advance(ElectionTransition::CompletePreparation { configuration })
            .unwrap();

        election
            .advance(ElectionTransition::Open { opened_at })
            .unwrap();
        // As mesmas cédulas do corpus preservam dimensão, ordem e contexto.
        for ballot in corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| text(c, "name").starts_with(&format!("{name}/ballot-")))
        {
            let payload = &ballot["payload"];
            let signed = SignedBallot::new(
                election_id(payload),
                AccountId::try_from_uuid(uuid(payload, "accountId")).unwrap(),
                decoded(text(payload, "publicKey")),
                decoded(text(ballot, "signature")),
                BallotContent::Confidential {
                    manifest_hash: hash(payload, "manifestHash"),
                    ciphertexts: BallotCiphertexts::try_new(ciphertexts(payload)).unwrap(),
                },
            );
            signed.validate_for(&election).unwrap();
        }
        let closure_case = case(&corpus, &format!("{name}/closure"));
        let closure = &closure_case["payload"];
        let n = VoterCount::try_new(closure["voterCount"].as_u64().unwrap().try_into().unwrap())
            .unwrap();
        let manifest_hash = hash(closure, "manifestHash");
        let head_hash = hash(closure, "headHash");
        validate_confidential_board_snapshot(Some(&manifest_hash), Some(&head_hash), n).unwrap();
        assert_eq!(election_id(closure), election.id());
        assert_eq!(manifest_hash, hash(manifest_case, "sha256"));
        let closed_at = time(closure, "closedAt");
        election
            .advance(ElectionTransition::Close {
                closed_at,
                voter_count: n,
            })
            .unwrap();
        let aggregate_case = case(&corpus, &format!("{name}/aggregate"));
        let aggregate = &aggregate_case["payload"];
        let result = &case(&corpus, &format!("{name}/result"))["payload"];
        let options = election.configuration().unwrap().options();
        assert!(
            ciphertexts(aggregate)
                .iter()
                .map(Ciphertext::option_id)
                .eq(options.iter().map(ElectionOption::id))
        );
        assert_eq!(election_id(aggregate), election.id());
        assert_eq!(hash(aggregate, "closureHash"), hash(closure_case, "sha256"));
        assert_eq!(hash(result, "closureHash"), hash(aggregate, "closureHash"));
        assert_eq!(
            hash(result, "manifestHash"),
            hash(aggregate, "manifestHash")
        );
        assert_eq!(
            hash(result, "aggregateHash"),
            hash(aggregate_case, "sha256")
        );
        assert_eq!(result["voterCount"], n.value());

        let shares: Vec<_> = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| text(c, "name").starts_with(&format!("{name}/share-")))
            .map(|c| AuthorityId::try_from_uuid(uuid(&c["payload"], "authorityId")).unwrap())
            .collect();
        if name == "valid" {
            let pending = &case(&corpus, "valid/incomplete")["payload"];
            let missing: Vec<_> = pending["missingAuthorities"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    AuthorityId::try_from_uuid(Uuid::parse_str(id.as_str().unwrap()).unwrap())
                        .unwrap()
                })
                .collect();
            let contributing_authorities = shares
                .iter()
                .copied()
                .filter(|id| !missing.contains(id))
                .collect();
            assert_eq!(pending["status"], "incomplete");
            assert!(pending["counts"].is_null());
            assert_eq!(
                election.advance(ElectionTransition::StartCounting {
                    contributing_authorities
                }),
                Err(ElectionTransitionError::ContributionsMismatch)
            );
            assert!(matches!(election.state(), ElectionState::Closed { .. }));
        }
        election
            .advance(ElectionTransition::StartCounting {
                contributing_authorities: shares,
            })
            .unwrap();
        let counts = read_counts(result);
        let (computation, outcome) = match (
            text(result, "status"),
            result["reason"].as_str(),
            counts.as_ref(),
        ) {
            ("valid", None, Some(counts)) => (
                TallyComputation::Totals(counts.clone()),
                FinalTallyResult::Valid(counts.clone()),
            ),
            ("annulled", Some("aggregate_sum_mismatch"), Some(counts)) => (
                TallyComputation::Totals(counts.clone()),
                FinalTallyResult::AnnulledBySumMismatch(counts.clone()),
            ),
            ("annulled", Some("tally_out_of_range"), None) => (
                TallyComputation::TotalOutOfRange,
                FinalTallyResult::AnnulledOutOfRange,
            ),
            _ => panic!("O cenário {name} deve ter uma apuração final válida ou anulada"),
        };
        election
            .advance(ElectionTransition::Finalize { computation })
            .unwrap();
        let ElectionState::Finalized {
            voting,
            closed_at: actual_closed_at,
            voter_count,
            result: actual_result,
        } = election.state()
        else {
            panic!("O cenário {name} deve terminar com a apuração finalizada");
        };
        assert_eq!(actual_result, &outcome, "{name}");
        assert_eq!(voting.opened_at, opened_at, "{name}");
        assert_eq!(*actual_closed_at, closed_at, "{name}");
        assert_eq!(*voter_count, n, "{name}");
        assert_eq!(election.current_period(), &voting_period, "{name}");

        if let Some(counts) = counts {
            let totals: Vec<_> = counts.iter().map(|c| c.count().value()).collect();
            assert_eq!(
                serde_json::json!(totals),
                scenario["expectedTotals"],
                "{name}"
            );
        }
        if name == "compensated" {
            assert_eq!(scenario["assumptionViolation"], true);
            assert!(
                matches!(actual_result, FinalTallyResult::Valid(_)),
                "Igualdade agregada não é prova de validade individual"
            );
        }
        if name == "out-of-range" {
            assert_eq!(actual_result, &FinalTallyResult::AnnulledOutOfRange);
        }
    }
}

#[test]
fn public_cancellation_before_manifest_has_no_tally() {
    let corpus = corpus();
    let cancellation = &case(&corpus, "cancelled/cancellation")["payload"];
    let mut election = Election::new(
        election_id(cancellation),
        aid(),
        period(&corpus["periods"]["cancelled"]),
        ElectionModality::Confidential,
        at(14, 10),
    );

    let requested_by = uuid(cancellation, "requestedBy");
    let cancelled_at = time(cancellation, "cancelledAt");
    let reason = text(cancellation, "reason");

    election
        .advance(ElectionTransition::Cancel {
            requested_by,
            cancelled_at,
            reason: reason.to_owned(),
        })
        .unwrap();
    let ElectionState::Cancelled {
        requested_by: actual_requester,
        cancelled_at: actual_cancelled_at,
        reason: actual_reason,
    } = election.state()
    else {
        panic!("A eleição deve estar cancelada");
    };
    assert_eq!(*actual_requester, requested_by);
    assert_eq!(*actual_cancelled_at, cancelled_at);
    assert_eq!(actual_reason, reason);
    assert!(election.configuration().is_none());
    let public_record = &case(&corpus, "cancelled/result")["payload"];
    assert_eq!(public_record["status"], "cancelled");
    assert!(public_record["counts"].is_null());
}

#[test]
fn domain_limits_match_the_versioned_profile() {
    let profile: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/elections/profile.json"
    )))
    .unwrap();
    use super::super::profile::{
        MAX_AUTHORITIES, MAX_OPTIONS, MAX_SIGNED_MESSAGE_BYTES, MAX_VOTER_COUNT, MIN_OPTIONS,
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
        serde_json::json!(MAX_SIGNED_MESSAGE_BYTES),
        profile["limits"]["maxSignedMessageBytes"]
    );
    assert_eq!(
        serde_json::json!(BASE64_URL_MAX_LENGTH),
        schema["$defs"]["Base64Url"]["maxLength"]
    );
}
