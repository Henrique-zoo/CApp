//! Codificação determinística da mensagem criptográfica da cédula.
//!
//! Não desserializa JSON nem conhece DTOs. O protocolo de assinatura inclui
//! o identificador da eleição e o da conta autenticada, mesmo que esses
//! identificadores não sejam campos do corpo HTTP.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::modules::elections::{
    application::error::BallotVerificationError,
    domain::{
        cryptography::elgamal::ElGamalCiphertext,
        entities::ballot::{BallotContent, SignedBallot},
        profile::MAX_SIGNED_MESSAGE_BYTES,
    },
};

use super::encoding::{decimal, lower_hex};

/// Separação criptográfica da assinatura eleitoral, incluindo a versão.
/// Mudá-la torna assinaturas da versão anterior incompatíveis.
pub(super) const BALLOT_SIGNATURE_DOMAIN: &[u8] = b"CAPP-ELECTIONS/BALLOT/1\0";

/// Reconstrói o payload autenticado, exclusivamente de valores de domínio.
///
/// A representação aqui é a entrada do algoritmo JCS, não a DTO HTTP.
/// Esta função deve ser reproduzida byte a byte pelo cliente e pelo auditor.
pub(super) fn ballot_payload(ballot: &SignedBallot) -> Value {
    let election_id = ballot.election_id().into_uuid();
    let account_id = ballot.account_id().into_uuid();
    let public_key = URL_SAFE_NO_PAD.encode(ballot.public_key());

    match ballot.content() {
        BallotContent::Identified(choice_id) => json!({
            "electionId": election_id,
            "accountId": account_id,
            "publicKey": public_key,
            "choiceId": choice_id,
        }),
        BallotContent::Confidential {
            manifest_hash,
            ciphertexts,
        } => {
            json!({
                "electionId": election_id,
                "accountId": account_id,
                "manifestHash": lower_hex(manifest_hash),
                "publicKey": public_key,
                "ciphertexts": ciphertexts_payload(ciphertexts.option_ids(), ciphertexts.encrypted_ballot().ciphertexts()),
            })
        }
    }
}

/// Codifica os pares ElGamal na ordem das escolhas usadas para assinar o objeto.
pub(super) fn ciphertexts_payload(ids: &[Uuid], ciphertexts: &[ElGamalCiphertext]) -> Vec<Value> {
    ids.iter()
        .zip(ciphertexts)
        .map(|(option_id, components)| {
            json!({
                "optionId": option_id,
                "a": decimal(&components.a),
                "b": decimal(&components.b),
            })
        })
        .collect()
}

/// Bytes assinados e usados no cálculo do hash da cédula.
///
/// Não incluem a assinatura RSA-PSS: assinaturas diferentes de um mesmo
/// conteúdo resultam no mesmo `ballot_hash`, permitindo retransmissões.
pub(super) fn signing_message(ballot: &SignedBallot) -> Result<Vec<u8>, BallotVerificationError> {
    let canonical = serde_jcs::to_vec(&ballot_payload(ballot))
        .map_err(|_| BallotVerificationError::InvalidEvidence)?;
    if canonical.len() > MAX_SIGNED_MESSAGE_BYTES {
        return Err(BallotVerificationError::InvalidEvidence);
    }
    let mut message = Vec::with_capacity(BALLOT_SIGNATURE_DOMAIN.len() + canonical.len());
    message.extend_from_slice(BALLOT_SIGNATURE_DOMAIN);
    message.extend_from_slice(&canonical);
    Ok(message)
}
