//! Dados do cadastro privado e regras para confirmar ou recuperar uma chave.
//!
//! A aplicação usa a conta autenticada e o horário do banco. O adaptador
//! carrega a situação do quadro e do eleitor sob o mesmo bloqueio usado no
//! aceite do voto, para que a política não possa mudar durante a operação.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{board::BoardError, error::RegistrationError};

/// Conta e vínculo de identidade obtidos da credencial autenticada.
#[derive(Debug, Clone, Copy)]
pub struct VoterIdentity {
    /// UUID interno do CApp, também usado como identificador público em novos votos.
    pub account_id: Uuid,
    /// Objeto Microsoft vinculado à sessão, necessário para comprovar o vínculo.
    pub microsoft_object_id: Option<Uuid>,
}

/// Estado da própria conta, inclusive após perder a elegibilidade ou o prazo.
#[derive(Debug, PartialEq, Eq)]
pub struct RegistrationStatus {
    /// Exigência escolhida pela comissão e congelada com o quadro.
    pub requires_registration: bool,
    /// UUID público já vinculado à participação ou à chave cadastrada.
    pub voter_id: Option<Uuid>,
    /// Chave pública SPKI em DER.
    pub public_key: Option<Vec<u8>>,
    /// Indica aceite de voto e emissão de recibo já confirmados.
    pub participated: bool,
}

pub(crate) enum RegistrationAction {
    Reuse,
    Insert,
}

/// Decide o cadastro com o estado lido na transação. A repetição da mesma
/// chave recupera o registro existente, mesmo após a abertura. Uma chave
/// inédita exige que o cadastro seja obrigatório e que o vínculo esteja vigente.
pub(crate) fn registration_action(
    status: &RegistrationStatus,
    submitted_key: &[u8],
    starts_at: DateTime<Utc>,
    closed: bool,
    now: DateTime<Utc>,
    eligible_until: Option<DateTime<Utc>>,
) -> Result<RegistrationAction, RegistrationError> {
    if !status.requires_registration {
        return Err(RegistrationError::NotRequired);
    }

    if let Some(key) = status.public_key.as_deref() {
        return if key == submitted_key {
            Ok(RegistrationAction::Reuse)
        } else {
            Err(RegistrationError::KeyConflict)
        };
    }

    if closed || now >= starts_at {
        return Err(RegistrationError::Closed);
    }

    if eligible_until.is_none_or(|end| now >= end) {
        return Err(RegistrationError::NotEligible);
    }

    Ok(RegistrationAction::Insert)
}

/// Confere a chave de uma nova cédula em eleição com cadastro prévio.
pub(crate) fn validate_registered_key(
    registered_key: Option<&[u8]>,
    ballot_key: &[u8],
) -> Result<(), BoardError> {
    let registered_key = registered_key.ok_or(BoardError::RegistrationRequired)?;
    if registered_key != ballot_key {
        return Err(BoardError::RegisteredKeyMismatch);
    }

    Ok(())
}
