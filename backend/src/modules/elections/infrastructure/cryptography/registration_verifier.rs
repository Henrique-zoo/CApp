//! Verificação tipada RSA-PSS da prova de posse da chave eleitoral.
//!
//! Não interpreta JSON/HTTP, não consulta o cadastro e não decide elegibilidade.

use rsa::Pss;
use sha2::{Digest, Sha256};

use crate::modules::elections::{
    application::{
        error::RegistrationError,
        ports::VoterKeyProofVerifier,
        registration::{RegistrationProof, VerifiedRegistrationKey},
    },
    domain::identifiers::{AccountId, ElectionId},
};

use super::{registration_codec::registration_message, rsa_key::decode_rsa_public_key};

/// Implementação RSA-PSS/SHA-256 da verificação de provas de posse
/// das chaves públicas dos eleitores.
///
/// Reconstrói a mensagem assinada a partir da prova de cadastro e
/// dos identificadores confiáveis fornecidos pela aplicação.
#[derive(Debug, Default, Clone, Copy)]
pub struct RsaPssVoterKeyProofVerifier;

impl VoterKeyProofVerifier for RsaPssVoterKeyProofVerifier {
    fn verify(
        &self,
        proof: RegistrationProof,
        election_id: ElectionId,
        account_id: AccountId,
    ) -> Result<VerifiedRegistrationKey, RegistrationError> {
        let key = decode_rsa_public_key(proof.public_key())
            .map_err(|_| RegistrationError::InvalidProof)?;
        let message = registration_message(election_id, account_id, proof.public_key())?;
        let digest = Sha256::digest(&message);
        key.verify(Pss::new_with_salt::<Sha256>(32), &digest, proof.signature())
            .map_err(|_| RegistrationError::InvalidProof)?;
        Ok(VerifiedRegistrationKey::from_verified_proof(proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use uuid::Uuid;

    #[test]
    fn verifies_reference_registration_and_rejects_mismatched_context() {
        let raw = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../packages/api-contracts/examples/election-voter-registration.json"
        ));
        let envelope: serde_json::Value = serde_json::from_str(raw).unwrap();
        let key = URL_SAFE_NO_PAD
            .decode(envelope["payload"]["publicKey"].as_str().unwrap())
            .unwrap();
        let signature = URL_SAFE_NO_PAD
            .decode(envelope["signature"].as_str().unwrap())
            .unwrap();
        let election = ElectionId::try_from_uuid(
            Uuid::parse_str("a7000000-0000-4000-8000-000000000137").unwrap(),
        )
        .unwrap();
        let account = AccountId::try_from_uuid(
            Uuid::parse_str("a7000000-0000-4000-8000-000000000001").unwrap(),
        )
        .unwrap();
        let verifier = RsaPssVoterKeyProofVerifier;
        assert_eq!(
            verifier
                .verify(
                    RegistrationProof::new(key.clone(), signature.clone()),
                    election,
                    account
                )
                .unwrap()
                .as_bytes(),
            key.as_slice(),
        );
        assert!(
            verifier
                .verify(
                    RegistrationProof::new(key.clone(), signature.clone()),
                    ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap(),
                    account
                )
                .is_err()
        );
        assert!(
            verifier
                .verify(
                    RegistrationProof::new(key.clone(), signature.clone()),
                    election,
                    AccountId::try_from_uuid(Uuid::from_u128(99)).unwrap()
                )
                .is_err()
        );
        let mut tampered = signature;
        tampered[0] ^= 1;
        assert!(
            verifier
                .verify(RegistrationProof::new(key, tampered), election, account)
                .is_err()
        );
    }
}
