//! Interoperabilidade com os vetores públicos e recusas adversariais do codec.

use super::*;

#[test]
fn independent_registration_vector_binds_account_election_kind_and_key() {
    let raw = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/examples/election-voter-registration.json"
    ));
    let election = Uuid::parse_str("a7000000-0000-4000-8000-000000000137").unwrap();
    let account = Uuid::parse_str("a7000000-0000-4000-8000-000000000001").unwrap();
    let envelope = parse_strict_json(raw).unwrap();
    assert_eq!(
        verify_registration(raw, election, account).unwrap(),
        envelope["payload"]["publicKey"]
    );
    assert!(verify_registration(raw, account, election).is_err());
    let mut other = envelope.clone();
    other["payload"]["kind"] = json!("ballot");
    assert!(verify_registration(&serde_json::to_vec(&other).unwrap(), election, account).is_err());
    let mut padded = envelope;
    padded["signature"] = json!(format!("{}=", padded["signature"].as_str().unwrap()));
    assert!(verify_registration(&serde_json::to_vec(&padded).unwrap(), election, account).is_err());
}

fn corpus() -> &'static Value {
    static CORPUS: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    CORPUS.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../packages/api-contracts/elections/vectors.json"
        )))
        .unwrap()
    })
}

fn vector(name: &str) -> &'static Value {
    corpus()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap()
}

fn envelope(case: &Value) -> Value {
    json!({"payload":case["payload"],"signature":case["signature"]})
}

fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

fn context(payload: &Value) -> BallotContext {
    BallotContext {
        election_id: Uuid::parse_str(payload["electionId"].as_str().unwrap()).unwrap(),
        manifest_hash: payload["manifestHash"].as_str().unwrap().to_owned(),
        voter_id: Uuid::parse_str(payload["voterId"].as_str().unwrap()).unwrap(),
        registered_public_key: Some(payload["publicKey"].as_str().unwrap().to_owned()),
        option_ids: payload["ciphertexts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| Uuid::parse_str(c["optionId"].as_str().unwrap()).unwrap())
            .collect(),
    }
}

fn signer() -> ServerSigner {
    ServerSigner::from_pkcs8_pem(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/jwt/test-only-rsa-private.pem"
    )))
    .unwrap()
}

fn resigned_ballot(mut payload: Value) -> (Value, BallotContext) {
    let signer = signer();
    payload["publicKey"] = Value::String(signer.public_key_base64().unwrap());
    let context = context(&payload);
    (signer.sign_payload(&payload).unwrap(), context)
}

#[test]
fn all_contract_vectors_match_jcs_hash_and_rsa_signatures() {
    for case in corpus()["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        assert_eq!(
            canonical_payload(&case["payload"]).unwrap(),
            case["canonical"].as_str().unwrap().as_bytes(),
            "{name}"
        );
        assert_eq!(
            object_digest(&case["payload"]).unwrap(),
            case["sha256"].as_str().unwrap(),
            "{name}"
        );
        let key = if case["signer"] == "legacy" {
            &corpus()["verificationKey"]
        } else {
            &corpus()["keys"][case["signer"].as_str().unwrap()]
        };
        verify_signed_object(&envelope(case), key.as_str().unwrap())
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn strict_json_rejects_duplicates_at_any_depth_and_noninteger_literals() {
    for raw in [
        r#"{"version":1,"version":1}"#,
        r#"{"payload":{"version":1,"\u0076ersion":1}}"#,
        r#"{"x":[{"a":1,"a":2}]}"#,
        r#"{"version":1.0}"#,
        r#"{"version":1e0}"#,
        r#"{"index":-0}"#,
        r#"{"index":9007199254740992}"#,
        r#"{} {}"#,
        r#"{"text":"\ud800"}"#,
    ] {
        assert!(parse_strict_json(raw.as_bytes()).is_err(), "{raw}");
    }
    assert!(parse_strict_json(&vec![b' '; MAX_SIGNED_MESSAGE_BYTES + 1]).is_err());
    assert_eq!(
        parse_strict_json(br#" { "b": [1], "a": "\u0061" } "#).unwrap(),
        json!({"a":"a","b":[1]})
    );
}

#[test]
fn canonicalization_orders_property_names_by_utf16() {
    assert_eq!(
        String::from_utf8(canonical_payload(&json!({"\u{e000}":1,"\u{10000}":2})).unwrap())
            .unwrap(),
        "{\"𐀀\":2,\"\":1}"
    );
}

#[test]
fn rsa_profile_rejects_noncanonical_spki_base64_and_wrong_parameters() {
    let public = signer().public_key_base64().unwrap();
    assert!(validate_public_key(&public).is_ok());
    assert!(validate_public_key(&(public.clone() + "=")).is_err());
    let mut der = URL_SAFE_NO_PAD.decode(&public).unwrap();
    der.push(0);
    assert!(validate_public_key(&URL_SAFE_NO_PAD.encode(der)).is_err());
    let (key, _) = decode_public_key(&public).unwrap();
    let wrong_exponent = RsaPublicKey::new(key.n().clone(), BigUint::from(3_u8)).unwrap();
    assert!(
        validate_public_key(
            &URL_SAFE_NO_PAD.encode(wrong_exponent.to_public_key_der().unwrap().as_bytes())
        )
        .is_err()
    );
    let small = RsaPublicKey::new(
        (BigUint::from(1_u8) << 1023) + BigUint::from(1_u8),
        BigUint::from(65537_u32),
    )
    .unwrap();
    assert!(
        validate_public_key(&URL_SAFE_NO_PAD.encode(small.to_public_key_der().unwrap().as_bytes()))
            .is_err()
    );
}

#[test]
fn signatures_require_domain_prefix_and_exact_salt_length() {
    let signer = signer();
    let public = signer.public_key_base64().unwrap();
    let payload = &vector("codec-receipt")["payload"];
    for (input, salt) in [
        (canonical_payload(payload).unwrap(), 32),
        (signature_input(payload).unwrap(), 31),
    ] {
        let sig = signer
            .key
            .sign_with_rng(
                &mut OsRng,
                Pss::new_blinded_with_salt::<Sha256>(salt),
                &Sha256::digest(input),
            )
            .unwrap();
        let signed = json!({"payload":payload,"signature":URL_SAFE_NO_PAD.encode(sig)});
        assert_eq!(
            verify_signed_object(&signed, &public),
            Err(CryptoWireError::Signature)
        );
    }
    verify_signed_object(&signer.sign_payload(payload).unwrap(), &public).unwrap();
}

#[test]
fn ballot_vectors_validate_against_trusted_context_and_detect_substitution() {
    let case = vector("valid/ballot-0");
    let raw = bytes(&envelope(case));
    let expected = context(&case["payload"]);
    let ballot = verify_ballot(&raw, &expected).unwrap();
    assert_eq!(ballot.ballot_hash, case["sha256"].as_str().unwrap());
    assert_eq!(ballot.ciphertexts.len(), 3);
    for field in ["election", "voter", "manifest", "key", "order"] {
        let mut wrong = expected.clone();
        match field {
            "election" => wrong.election_id = Uuid::new_v4(),
            "voter" => wrong.voter_id = Uuid::new_v4(),
            "manifest" => wrong.manifest_hash = "0".repeat(64),
            "key" => wrong.registered_public_key = Some(signer().public_key_base64().unwrap()),
            "order" => wrong.option_ids.swap(0, 1),
            _ => unreachable!(),
        }
        assert_eq!(
            verify_ballot(&raw, &wrong).unwrap_err(),
            CryptoWireError::Context,
            "{field}"
        );
    }
    let mut unsigned = envelope(case);
    unsigned["payload"]["submissionId"] = Value::String(Uuid::new_v4().to_string());
    assert_eq!(
        verify_ballot(&bytes(&unsigned), &expected).unwrap_err(),
        CryptoWireError::Signature
    );
}

#[test]
fn signed_but_malformed_ballots_are_rejected() {
    for defect in [
        "unknown",
        "decimal",
        "number",
        "identity",
        "duplicate",
        "uppercase",
    ] {
        let mut payload = vector("valid/ballot-0")["payload"].clone();
        match defect {
            "unknown" => payload["extra"] = json!(true),
            "decimal" => payload["ciphertexts"][0]["a"] = json!("02"),
            "number" => payload["ciphertexts"][0]["a"] = json!(2),
            "identity" => payload["ciphertexts"][0]["a"] = json!("1"),
            "duplicate" => {
                payload["ciphertexts"][0]["optionId"] =
                    payload["ciphertexts"][1]["optionId"].clone()
            }
            "uppercase" => payload["submissionId"] = json!("AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA"),
            _ => unreachable!(),
        }
        let (signed, expected) = resigned_ballot(payload);
        assert!(
            verify_ballot(&bytes(&signed), &expected).is_err(),
            "{defect}"
        );
    }
    let mut signed = envelope(vector("valid/ballot-0"));
    signed["extra"] = json!(false);
    assert!(verify_ballot(&bytes(&signed), &context(&signed["payload"])).is_err());
}

#[test]
fn record_and_receipt_bind_nested_signature_chain_and_original_ballot() {
    let ballot_case = vector("valid/ballot-0");
    let ctx = context(&ballot_case["payload"]);
    let ballot = verify_ballot(&bytes(&envelope(ballot_case)), &ctx).unwrap();
    let signer = signer();
    let key = signer.public_key_base64().unwrap();
    let record = signer.issue_record(&ballot, 1, &ctx.manifest_hash).unwrap();
    verify_record(
        &bytes(&record.board_entry),
        &bytes(&record.receipt),
        &ctx,
        1,
        &ctx.manifest_hash,
        &key,
    )
    .unwrap();
    assert_eq!(
        record.receipt["payload"]["ballotHash"],
        ballot_case["sha256"]
    );
    let mut changed = record.receipt["payload"].clone();
    changed["index"] = json!(2);
    let changed = signer.sign_payload(&changed).unwrap();
    assert_eq!(
        verify_record(
            &bytes(&record.board_entry),
            &bytes(&changed),
            &ctx,
            1,
            &ctx.manifest_hash,
            &key
        )
        .unwrap_err(),
        CryptoWireError::Context
    );
    let mut changed = record.board_entry["payload"].clone();
    changed["ballot"]["signature"] = record.board_entry["signature"].clone();
    assert_ne!(object_digest(&changed).unwrap(), record.entry_hash);
    assert!(
        verify_record(
            &bytes(&record.board_entry),
            &bytes(&record.receipt),
            &ctx,
            2,
            &ctx.manifest_hash,
            &key
        )
        .is_err()
    );
}

#[test]
fn existing_manifest_calendar_and_board_vectors_are_accepted() {
    let server = corpus()["keys"]["server"].as_str().unwrap();
    let manifest = verify_manifest(&bytes(&envelope(vector("valid/manifest"))), server).unwrap();
    assert_eq!(
        manifest.manifest_hash,
        vector("valid/manifest")["sha256"].as_str().unwrap()
    );
    verify_calendar_origin(
        &bytes(&envelope(vector("valid/calendar-origin"))),
        &manifest,
        server,
    )
    .unwrap();
    let ctx = context(&vector("valid/ballot-0")["payload"]);
    verify_record(
        &bytes(&envelope(vector("valid/entry-0"))),
        &bytes(&envelope(vector("valid/receipt-0"))),
        &ctx,
        1,
        &ctx.manifest_hash,
        server,
    )
    .unwrap();
    assert!(
        verify_manifest(
            &bytes(&envelope(vector("valid/manifest"))),
            &signer().public_key_base64().unwrap()
        )
        .is_err()
    );
}

#[test]
fn manifest_rejects_invalid_key_proof_even_with_valid_server_signature() {
    let signer = signer();
    let key = signer.public_key_base64().unwrap();
    let mut payload = vector("valid/manifest")["payload"].clone();
    payload["serverSigningKey"] = json!(key);
    verify_manifest(&bytes(&signer.sign_payload(&payload).unwrap()), &key).unwrap();
    payload["authorities"][0]["keyProof"]["response"] = json!("0");
    assert_eq!(
        verify_manifest(&bytes(&signer.sign_payload(&payload).unwrap()), &key).unwrap_err(),
        CryptoWireError::Signature
    );
}

#[test]
fn signed_manifests_reject_proof_rebinding_invalid_groups_and_duplicate_authorities() {
    let signer = signer();
    let key = signer.public_key_base64().unwrap();
    for defect in [
        "authority-id",
        "signing-key",
        "group",
        "joint-key",
        "duplicate-key",
        "response-range",
        "identity-commitment",
        "unknown-proof-field",
    ] {
        let mut payload = vector("valid/manifest")["payload"].clone();
        payload["serverSigningKey"] = json!(key);
        match defect {
            "authority-id" => payload["authorities"][0]["id"] = json!(Uuid::new_v4().to_string()),
            "signing-key" => payload["authorities"][0]["signingKey"] = json!(key),
            "group" => payload["groupId"] = json!("rfc3526-modp15"),
            "joint-key" => payload["combinedPublicKey"] = json!("2"),
            "duplicate-key" => {
                payload["authorities"][1]["signingKey"] =
                    payload["authorities"][0]["signingKey"].clone()
            }
            "response-range" => {
                payload["authorities"][0]["keyProof"]["response"] =
                    json!(group_parameters().1.to_str_radix(10))
            }
            "identity-commitment" => {
                payload["authorities"][0]["keyProof"]["commitment"] = json!("1")
            }
            "unknown-proof-field" => payload["authorities"][0]["keyProof"]["ignored"] = json!(true),
            _ => unreachable!(),
        }
        let signed = signer.sign_payload(&payload).unwrap();
        assert!(verify_manifest(&bytes(&signed), &key).is_err(), "{defect}");
    }
}

#[test]
fn closure_rejects_an_early_time_even_when_server_signed_it() {
    let signer = signer();
    let key = signer.public_key_base64().unwrap();
    let mut manifest_payload = vector("valid/manifest")["payload"].clone();
    manifest_payload["serverSigningKey"] = json!(key);
    let manifest = verify_manifest(
        &bytes(&signer.sign_payload(&manifest_payload).unwrap()),
        &key,
    )
    .unwrap();
    let schedule = signer
        .sign_payload(&vector("valid/extension")["payload"])
        .unwrap();
    let mut payload = vector("valid/closure")["payload"].clone();
    payload["manifestHash"] = json!(manifest.manifest_hash);
    payload["closedAt"] = schedule["payload"]["startsAt"].clone();
    let head = payload["headHash"].as_str().unwrap().to_owned();
    let signed = signer.sign_payload(&payload).unwrap();
    assert_eq!(
        verify_closure(&bytes(&signed), &manifest, &schedule, &head, 2, &key).unwrap_err(),
        CryptoWireError::Context
    );
}

#[test]
fn closure_requires_exact_snapshot_and_effective_deadline() {
    let server = corpus()["keys"]["server"].as_str().unwrap();
    let manifest = verify_manifest(&bytes(&envelope(vector("valid/manifest"))), server).unwrap();
    let closure = envelope(vector("valid/closure"));
    let schedule = envelope(vector("valid/extension"));
    let head = closure["payload"]["headHash"].as_str().unwrap();
    verify_closure(&bytes(&closure), &manifest, &schedule, head, 2, server).unwrap();
    assert!(verify_closure(&bytes(&closure), &manifest, &schedule, head, 1, server).is_err());
    assert!(
        verify_closure(
            &bytes(&closure),
            &manifest,
            &schedule,
            &manifest.manifest_hash,
            2,
            server
        )
        .is_err()
    );
}
