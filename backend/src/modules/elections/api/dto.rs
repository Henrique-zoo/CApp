//! Representações HTTP da API eleitoral.
//!
//! O estado do cadastro sai como JSON privado da própria conta. A conversão
//! codifica a chave SPKI em Base64url para o transporte.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Serialize;
use uuid::Uuid;

use crate::modules::elections::application::registration::RegistrationStatus;

/// Situação do cadastro ou do voto já confirmado para a conta autenticada.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RegistrationResponse {
    requires_registration: bool,
    voter_id: Option<Uuid>,
    public_key: Option<String>,
    participated: bool,
}

impl From<RegistrationStatus> for RegistrationResponse {
    fn from(status: RegistrationStatus) -> Self {
        Self {
            requires_registration: status.requires_registration,
            voter_id: status.voter_id,
            public_key: status.public_key.map(|key| URL_SAFE_NO_PAD.encode(key)),
            participated: status.participated,
        }
    }
}
