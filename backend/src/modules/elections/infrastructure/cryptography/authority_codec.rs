//! Codificação canônica de credenciais e contribuições públicas.
//!
//! A API já converteu as representações de entrada para valores tipados.
//! Estes codecs reconstroem o formato público existente para assinatura,
//! verificação, persistência e auditoria, sem interpretar requisições HTTP.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};

use crate::modules::elections::{
    application::{preparation::AuthorityRegistrationProof, tally::ShareSubmission},
    domain::entities::authority::AuthorityRole,
};

use super::encoding::{decimal, lower_hex};

pub(in crate::modules::elections::infrastructure) fn registration_envelope(
    proof: &AuthorityRegistrationProof,
) -> Value {
    let authority = &proof.authority;
    let role = match authority.role() {
        AuthorityRole::Commission => "commission",
        AuthorityRole::Slate(_) => "slate",
        AuthorityRole::Server => "server",
    };
    json!({
        "payload": {
            "version": 1,
            "kind": "authority-registration",
            "electionId": proof.election.into_uuid(),
            "authority": {
                "id": authority.id().into_uuid(),
                "role": role,
                "slateId": authority.slate_id().map(|id| id.into_uuid()),
                "signingKey": URL_SAFE_NO_PAD.encode(authority.signing_key()),
                "publicValue": decimal(authority.public_value()),
                "keyProof": {
                    "commitment": decimal(authority.key_proof().commitment()),
                    "response": decimal(authority.key_proof().response()),
                },
            },
        },
        "signature": URL_SAFE_NO_PAD.encode(&proof.signature),
    })
}

pub(in crate::modules::elections::infrastructure) fn share_envelope(
    submission: &ShareSubmission,
) -> Value {
    let shares: Vec<_> = submission
        .shares
        .iter()
        .map(|item| {
            json!({
                "optionId": item.option_id,
                "value": decimal(&item.share.value),
                "proof": {
                    "commitmentG": decimal(&item.share.commitment_g),
                    "commitmentA": decimal(&item.share.commitment_a),
                    "response": decimal(&item.share.response),
                },
            })
        })
        .collect();
    json!({
        "payload": {
            "version": 1,
            "kind": "decryption-share",
            "electionId": submission.election.into_uuid(),
            "manifestHash": lower_hex(&submission.manifest_hash),
            "closureHash": lower_hex(&submission.closure_hash),
            "aggregateHash": lower_hex(&submission.aggregate_hash),
            "authorityId": submission.authority.into_uuid(),
            "shares": shares,
        },
        "signature": URL_SAFE_NO_PAD.encode(&submission.signature),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::{
        application::tally::OptionShare,
        crypto_wire::{canonical_payload, signature_input},
        domain::{
            cryptography::elgamal::DecryptionShare,
            entities::authority::{Authority, KeyProof},
            identifiers::{AuthorityId, ElectionId},
        },
    };
    use crypto_bigint::U2048;
    use uuid::Uuid;

    #[test]
    fn registration_preserves_public_signing_format() {
        let election = ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap();
        let authority_id = AuthorityId::try_from_uuid(Uuid::from_u128(2)).unwrap();
        let authority = Authority::try_new(
            authority_id,
            AuthorityRole::Commission,
            vec![1],
            U2048::from_u8(4),
            KeyProof::new(U2048::from_u8(2), U2048::ZERO),
        )
        .unwrap();
        let envelope = registration_envelope(&AuthorityRegistrationProof {
            election,
            authority,
            signature: vec![2],
        });
        let expected = json!({
            "version": 1, "kind": "authority-registration", "electionId": election.into_uuid(),
            "authority": {"id": authority_id.into_uuid(), "role": "commission", "slateId": null,
                "signingKey": "AQ", "publicValue": "4", "keyProof": {"commitment": "2", "response": "0"}},
        });
        assert_eq!(envelope, json!({"payload": expected, "signature": "Ag"}));
        assert_eq!(
            signature_input(&envelope["payload"]).unwrap(),
            signature_input(&expected).unwrap()
        );
    }

    #[test]
    fn share_preserves_public_signing_format_and_order() {
        let election = ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap();
        let authority = AuthorityId::try_from_uuid(Uuid::from_u128(2)).unwrap();
        let submission = ShareSubmission {
            election,
            authority,
            manifest_hash: [0xab; 32],
            closure_hash: [0xcd; 32],
            aggregate_hash: [0xef; 32],
            signature: vec![2],
            shares: [Uuid::from_u128(3), Uuid::nil()]
                .into_iter()
                .map(|option_id| OptionShare {
                    option_id,
                    share: DecryptionShare {
                        value: U2048::ONE,
                        commitment_g: U2048::from_u8(2),
                        commitment_a: U2048::ONE,
                        response: U2048::ZERO,
                    },
                })
                .collect(),
        };
        let envelope = share_envelope(&submission);
        let expected = json!({
            "version": 1, "kind": "decryption-share", "electionId": election.into_uuid(),
            "authorityId": authority.into_uuid(), "manifestHash": "ab".repeat(32),
            "closureHash": "cd".repeat(32), "aggregateHash": "ef".repeat(32),
            "shares": [
                {"optionId": Uuid::from_u128(3), "value": "1", "proof": {"commitmentG":"2", "commitmentA":"1", "response":"0"}},
                {"optionId": Uuid::nil(), "value": "1", "proof": {"commitmentG":"2", "commitmentA":"1", "response":"0"}},
            ],
        });
        assert_eq!(envelope, json!({"payload": expected, "signature": "Ag"}));
        assert_eq!(
            canonical_payload(&envelope["payload"]).unwrap(),
            canonical_payload(&expected).unwrap()
        );
    }
}
