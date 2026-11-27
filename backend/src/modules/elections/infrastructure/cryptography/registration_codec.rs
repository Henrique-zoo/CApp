//! Codificação determinística da prova de cadastro, independente de HTTP.
//!
//! A conta e a eleição vêm exclusivamente do contexto autenticado. A mesma
//! representação deve ser reconstruída pelo cliente antes de assinar.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::json;

use crate::modules::elections::{
    application::error::RegistrationError,
    domain::identifiers::{AccountId, ElectionId},
};

pub(super) const REGISTRATION_SIGNATURE_DOMAIN: &[u8] = b"CAPP-ELECTIONS/REGISTRATION/1\0";

/// Reconstrói a mensagem RSA-PSS de cadastro (JCS de dados tipados).
pub(super) fn registration_message(
    election_id: ElectionId,
    account_id: AccountId,
    public_key: &[u8],
) -> Result<Vec<u8>, RegistrationError> {
    let payload = json!({
        "accountId": account_id.into_uuid(),
        "electionId": election_id.into_uuid(),
        "publicKey": URL_SAFE_NO_PAD.encode(public_key),
    });
    let canonical = serde_jcs::to_vec(&payload).map_err(|_| RegistrationError::InvalidProof)?;
    let mut message = Vec::with_capacity(REGISTRATION_SIGNATURE_DOMAIN.len() + canonical.len());
    message.extend_from_slice(REGISTRATION_SIGNATURE_DOMAIN);
    message.extend_from_slice(&canonical);
    Ok(message)
}
