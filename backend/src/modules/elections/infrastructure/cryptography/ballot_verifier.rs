//! Verificação RSA-PSS de uma cédula tipada.
//!
//! Não decodifica HTTP ou JSON, não verifica elegibilidade e não persiste.
//! A única entrada é uma entidade `SignedBallot` criada na fronteira da API.

use rsa::Pss;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::modules::elections::{
    application::{
        board::{BoardError, RecordedBallot, SignedVoteEvidence},
        error::BallotVerificationError,
        ports::{BallotVerifier, VoteEvidenceSigner},
    },
    crypto_wire::ServerSigner,
    domain::entities::ballot::{SignedBallot, VerifiedBallot},
    infrastructure::codec::{encode_base64_url, encode_fingerprint},
};

use super::{
    ballot_codec::{ballot_payload, signing_message},
    rsa_key::decode_rsa_public_key,
};

/// Adaptador sem estado para assinatura RSA-PSS/SHA-256 do protocolo eleitoral.
#[derive(Debug, Default, Clone, Copy)]
pub struct RsaPssBallotVerifier;

impl BallotVerifier for RsaPssBallotVerifier {
    fn verify_ballot(
        &self,
        ballot: SignedBallot,
    ) -> Result<VerifiedBallot, BallotVerificationError> {
        let invalid = || BallotVerificationError::InvalidEvidence;
        let key = decode_rsa_public_key(ballot.public_key()).map_err(|_| invalid())?;

        let message = signing_message(&ballot)?;
        let digest: [u8; 32] = Sha256::digest(&message).into();
        key.verify(
            Pss::new_with_salt::<Sha256>(32),
            &digest,
            ballot.signature(),
        )
        .map_err(|_| invalid())?;

        Ok(VerifiedBallot::from_verified_parts(ballot, digest))
    }
}

impl VoteEvidenceSigner for ServerSigner {
    fn public_key(&self) -> Result<String, BoardError> {
        Ok(self.public_key_base64()?)
    }

    fn sign_vote(
        &self,
        ballot: &VerifiedBallot,
        index: u32,
        previous_hash: &str,
    ) -> Result<SignedVoteEvidence, BoardError> {
        let envelope = json!({
            "payload": ballot_payload(ballot.ballot()),
            "signature": encode_base64_url(ballot.ballot().signature())
                .map_err(|_| BoardError::InvalidArtifact("assinatura da cédula"))?,
        });
        let hash = encode_fingerprint(ballot.ballot_hash());
        let signed = self.issue_record(&envelope, &hash, index, previous_hash)?;

        Ok(SignedVoteEvidence {
            entry_hash: signed.entry_hash,
            record: RecordedBallot {
                entry: signed.board_entry,
                receipt: signed.receipt,
                recovered: false,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::{
        domain::{
            entities::ballot::BallotContent,
            identifiers::{AccountId, ElectionId},
        },
        infrastructure::cryptography::ballot_codec::{ballot_payload, signing_message},
    };
    use rand::rngs::OsRng;
    use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey};
    use uuid::Uuid;

    fn ids() -> (ElectionId, AccountId) {
        (
            ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap(),
            AccountId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
        )
    }

    fn ballot_for(key: &RsaPrivateKey, signature: Vec<u8>, choice: Uuid) -> SignedBallot {
        let (election, account) = ids();
        let der = RsaPublicKey::from(key).to_public_key_der().unwrap();
        SignedBallot::new(
            election,
            account,
            der.as_bytes().to_vec(),
            signature,
            BallotContent::Identified(choice),
        )
    }

    #[test]
    fn signature_does_not_change_ballot_hash() {
        let mut rng = OsRng;
        let private = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let choice = Uuid::from_u128(3);
        let template = ballot_for(&private, vec![], choice);
        let message = signing_message(&template).unwrap();
        let digest = Sha256::digest(&message);

        let sign = |rng: &mut OsRng| {
            private
                .sign_with_rng(rng, Pss::new_blinded_with_salt::<Sha256>(32), &digest)
                .unwrap()
        };
        let signature_a = sign(&mut rng);
        let signature_b = sign(&mut rng);
        assert_ne!(signature_a, signature_b);

        let a = RsaPssBallotVerifier
            .verify_ballot(ballot_for(&private, signature_a.clone(), choice))
            .unwrap();
        let b = RsaPssBallotVerifier
            .verify_ballot(ballot_for(&private, signature_b, choice))
            .unwrap();
        assert_eq!(a.ballot_hash(), b.ballot_hash());

        // Alterar o conteúdo mas manter uma assinatura válida do conteúdo anterior falha.
        let tampered = ballot_for(&private, signature_a, Uuid::from_u128(4));
        assert!(matches!(
            RsaPssBallotVerifier.verify_ballot(tampered),
            Err(BallotVerificationError::InvalidEvidence)
        ));
    }

    #[test]
    fn confidential_payload_covers_manifest_and_ciphertexts() {
        use crate::modules::elections::domain::{
            cryptography::elgamal::ElGamalCiphertext,
            entities::ballot::{BallotCiphertexts, Ciphertext},
        };
        use crypto_bigint::U2048;
        let mut rng = OsRng;
        let private = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let (election, account) = ids();
        let key = RsaPublicKey::from(&private).to_public_key_der().unwrap();
        let ciphertexts = BallotCiphertexts::try_new(vec![
            Ciphertext::new(
                Uuid::from_u128(3),
                ElGamalCiphertext {
                    a: U2048::from_u8(2),
                    b: U2048::from_u8(4),
                },
            ),
            Ciphertext::new(
                Uuid::nil(),
                ElGamalCiphertext {
                    a: U2048::from_u8(8),
                    b: U2048::from_u8(16),
                },
            ),
        ])
        .unwrap();
        let ballot = SignedBallot::new(
            election,
            account,
            key.as_bytes().to_vec(),
            vec![],
            BallotContent::Confidential {
                manifest_hash: [0xab; 32],
                ciphertexts,
            },
        );
        let payload = ballot_payload(&ballot);
        assert_eq!(payload["manifestHash"], "ab".repeat(32));
        assert_eq!(payload["ciphertexts"][0]["a"], "2");
        assert_eq!(
            payload["ciphertexts"][1]["optionId"],
            Uuid::nil().to_string()
        );
        assert!(payload.get("submissionId").is_none());
        assert!(payload.get("version").is_none());
        assert!(payload.get("kind").is_none());
        assert!(payload.get("voterId").is_none());
        assert_eq!(payload["accountId"], account.into_uuid().to_string());

        // Todas as modalidades usam o mesmo domínio de assinatura.
        let message = signing_message(&ballot).unwrap();
        assert!(message.starts_with(super::super::ballot_codec::BALLOT_SIGNATURE_DOMAIN));
        let canonical = serde_jcs::to_vec(&payload).unwrap();
        assert_eq!(
            &message[super::super::ballot_codec::BALLOT_SIGNATURE_DOMAIN.len()..],
            canonical.as_slice()
        );

        // A verificação real da assinatura confidencial usa esse mesmo codec.
        let digest = Sha256::digest(&message);
        let signature = private
            .sign_with_rng(&mut rng, Pss::new_blinded_with_salt::<Sha256>(32), &digest)
            .unwrap();
        let signed = SignedBallot::new(
            election,
            account,
            key.as_bytes().to_vec(),
            signature,
            ballot.content().clone(),
        );
        assert!(RsaPssBallotVerifier.verify_ballot(signed).is_ok());
    }

    #[test]
    fn cryptographic_payload_has_no_redundant_transport_fields() {
        let mut rng = OsRng;
        let private = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let ballot = ballot_for(&private, vec![], Uuid::from_u128(3));
        let payload = ballot_payload(&ballot);
        assert!(payload.get("kind").is_none());
        assert!(payload.get("version").is_none());
        assert!(payload.get("submissionId").is_none());
        assert_eq!(
            payload["electionId"],
            ballot.election_id().into_uuid().to_string()
        );
        assert_eq!(
            payload["accountId"],
            ballot.account_id().into_uuid().to_string()
        );
    }
}
