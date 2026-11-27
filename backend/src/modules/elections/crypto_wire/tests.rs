//! Interoperabilidade com os vetores públicos e recusas adversariais do codec.

use super::*;

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

// Referência de teste independente do codec tipado de produção.
fn ballot_signature_input(payload: &Value) -> Result<Vec<u8>, CryptoWireError> {
    with_domain(BALLOT_DOMAIN, payload)
}

fn context(payload: &Value) -> BallotContext {
    BallotContext {
        election_id: Uuid::parse_str(payload["electionId"].as_str().unwrap()).unwrap(),
        manifest_hash: payload["manifestHash"].as_str().unwrap().to_owned(),
        account_id: AccountId::try_from_uuid(
            Uuid::parse_str(payload["accountId"].as_str().unwrap()).unwrap(),
        )
        .unwrap(),
        expected_public_key: Some(payload["publicKey"].as_str().unwrap().to_owned()),
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
    let expected = context(&payload);
    let message = ballot_signature_input(&payload).unwrap();
    let signature = signer
        .key
        .sign_with_rng(
            &mut OsRng,
            Pss::new_blinded_with_salt::<Sha256>(32),
            &Sha256::digest(message),
        )
        .unwrap();
    (
        json!({"payload":payload,"signature":URL_SAFE_NO_PAD.encode(signature)}),
        expected,
    )
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
        let is_ballot = case["payload"].get("kind").is_none();
        let digest = if is_ballot {
            hex_digest(&ballot_signature_input(&case["payload"]).unwrap())
        } else {
            object_digest(&case["payload"]).unwrap()
        };
        assert_eq!(digest, case["sha256"].as_str().unwrap(), "{name}");
        let key = if case["signer"] == "legacy" {
            &corpus()["verificationKey"]
        } else {
            &corpus()["keys"][case["signer"].as_str().unwrap()]
        };
        if is_ballot {
            let mut ctx = context(&case["payload"]);
            ctx.expected_public_key = Some(key.as_str().unwrap().to_owned());
            let verified = verify_ballot(&bytes(&envelope(case)), &ctx)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(verified.ballot_hash, digest, "{name}");
        } else {
            verify_signed_object(&envelope(case), key.as_str().unwrap())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
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
    let (signed, expected) = resigned_ballot(vector("valid/ballot-0")["payload"].clone());
    let raw = bytes(&signed);
    let ballot = verify_ballot(&raw, &expected).unwrap();
    assert_eq!(
        ballot.ballot_hash,
        hex_digest(&ballot_signature_input(&signed["payload"]).unwrap())
    );
    assert_eq!(ballot.ciphertexts.ciphertexts().len(), 3);
    for field in ["election", "voter", "manifest", "key", "order"] {
        let mut wrong = expected.clone();
        match field {
            "election" => wrong.election_id = Uuid::new_v4(),
            "voter" => wrong.account_id = AccountId::try_from_uuid(Uuid::new_v4()).unwrap(),
            "manifest" => wrong.manifest_hash = "0".repeat(64),
            "key" => {
                let other = corpus()["keys"]["server"].as_str().unwrap().to_owned();
                assert_ne!(
                    expected.expected_public_key.as_deref(),
                    Some(other.as_str())
                );
                wrong.expected_public_key = Some(other);
            }
            "order" => wrong.option_ids.swap(0, 1),
            _ => unreachable!(),
        }
        assert_eq!(
            verify_ballot(&raw, &wrong).unwrap_err(),
            CryptoWireError::Context,
            "{field}"
        );
    }
    let mut unsigned = signed;
    unsigned["payload"]["ciphertexts"][0]["b"] = json!("4");
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
            "uppercase" => payload["accountId"] = json!("AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA"),
            _ => unreachable!(),
        }
        let (signed, expected) = resigned_ballot(payload);
        assert!(
            verify_ballot(&bytes(&signed), &expected).is_err(),
            "{defect}"
        );
    }
    let (mut signed, _) = resigned_ballot(vector("valid/ballot-0")["payload"].clone());
    signed["extra"] = json!(false);
    assert!(verify_ballot(&bytes(&signed), &context(&signed["payload"])).is_err());
}

#[test]
fn record_and_receipt_bind_nested_signature_chain_and_original_ballot() {
    let (signed, ctx) = resigned_ballot(vector("valid/ballot-0")["payload"].clone());
    let ballot = verify_ballot(&bytes(&signed), &ctx).unwrap();
    let signer = signer();
    let key = signer.public_key_base64().unwrap();
    let record = signer
        .issue_record(&ballot.envelope, &ballot.ballot_hash, 1, &ctx.manifest_hash)
        .unwrap();
    verify_record(
        &bytes(&record.board_entry),
        &bytes(&record.receipt),
        &ctx,
        1,
        &ctx.manifest_hash,
        &key,
    )
    .unwrap();
    assert_eq!(record.receipt["payload"]["ballotHash"], ballot.ballot_hash);
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
fn board_vectors_validate_with_the_current_ballot_protocol() {
    let server = corpus()["keys"]["server"].as_str().unwrap();
    let manifest = verify_manifest(&bytes(&envelope(vector("valid/manifest"))), server).unwrap();
    assert_eq!(
        manifest.manifest_hash,
        vector("valid/manifest")["sha256"].as_str().unwrap()
    );
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
fn closure_rejects_a_noncanonical_time_even_when_server_signed_it() {
    let signer = signer();
    let key = signer.public_key_base64().unwrap();
    let mut manifest_payload = vector("valid/manifest")["payload"].clone();
    manifest_payload["serverSigningKey"] = json!(key);
    let manifest = verify_manifest(
        &bytes(&signer.sign_payload(&manifest_payload).unwrap()),
        &key,
    )
    .unwrap();
    let mut payload = vector("valid/closure")["payload"].clone();
    payload["manifestHash"] = json!(manifest.manifest_hash);
    payload["closedAt"] = json!("2026-11-26T12:00:00+00:00");
    let head = payload["headHash"].as_str().unwrap().to_owned();
    let signed = signer.sign_payload(&payload).unwrap();
    assert_eq!(
        verify_closure(&bytes(&signed), &manifest, &head, 2, &key).unwrap_err(),
        CryptoWireError::Encoding
    );
}

#[test]
fn closure_requires_exact_snapshot() {
    let server = corpus()["keys"]["server"].as_str().unwrap();
    let manifest = verify_manifest(&bytes(&envelope(vector("valid/manifest"))), server).unwrap();
    let closure = envelope(vector("valid/closure"));
    let head = closure["payload"]["headHash"].as_str().unwrap();
    verify_closure(&bytes(&closure), &manifest, head, 2, server).unwrap();
    assert!(verify_closure(&bytes(&closure), &manifest, head, 1, server).is_err());
    assert!(
        verify_closure(
            &bytes(&closure),
            &manifest,
            &manifest.manifest_hash,
            2,
            server
        )
        .is_err()
    );
}

fn tally_board(scope: &str) -> crate::modules::elections::application::board::BoardSnapshot {
    let periods = &corpus()["periods"][scope];
    let closure = envelope(vector(&format!("{scope}/closure")));
    let count = closure["payload"]["voterCount"].as_u64().unwrap();
    crate::modules::elections::application::board::BoardSnapshot {
        starts_at: timestamp(periods["startsAt"].as_str().unwrap()).unwrap(),
        ends_at: timestamp(periods["endsAt"].as_str().unwrap()).unwrap(),
        manifest: envelope(vector(&format!("{scope}/manifest"))),
        cancellation: None,
        entries: (0..count)
            .map(|index| envelope(vector(&format!("{scope}/entry-{index}"))))
            .collect(),
        checkpoint: (count != 0)
            .then(|| envelope(vector(&format!("{scope}/receipt-{}", count - 1)))),
        closure: Some(closure),
        aggregate: Some(envelope(vector(&format!("{scope}/aggregate")))),
    }
}

#[test]
fn tally_vectors_verify_and_recover_expected_results() {
    let server = corpus()["keys"]["server"].as_str().unwrap();
    for scenario in corpus()["scenarios"].as_array().unwrap() {
        let scope = scenario["name"].as_str().unwrap();
        let board = tally_board(scope);
        let tally = tally::VerifiedTally::from_board(&board, server)
            .unwrap_or_else(|e| panic!("{scope}: {e}"));
        let shares: Vec<_> = (0..scenario["shareHashes"].as_array().unwrap().len())
            .map(|i| {
                tally
                    .verify_share(&envelope(vector(&format!("{scope}/share-{i}"))))
                    .unwrap_or_else(|e| panic!("{scope}/share-{i}: {e}"))
            })
            .collect();
        assert_eq!(
            tally.result_payload(&shares).unwrap(),
            vector(&format!("{scope}/result"))["payload"],
            "{scope}"
        );
        if scope == "valid" {
            assert_eq!(
                tally.result_payload(&shares[..shares.len() - 1]).unwrap(),
                vector("valid/incomplete")["payload"]
            );
        }
    }
}

#[test]
fn tally_rejects_tampered_publication() {
    let server = corpus()["keys"]["server"].as_str().unwrap();
    let mut board = tally_board("valid");
    assert!(tally::VerifiedTally::from_board(&board, server).is_ok());
    board.entries.pop();
    assert!(tally::VerifiedTally::from_board(&board, server).is_err());
    let mut board = tally_board("valid");
    board.aggregate = Some(envelope(vector("empty/aggregate")));
    assert!(tally::VerifiedTally::from_board(&board, server).is_err());
}
