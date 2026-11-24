//! Invariantes do cadastro de chaves, sem HTTP, SQL, Firebase ou criptografia.

use chrono::{DateTime, Utc};

/// Decisão de domínio sobre uma solicitação de cadastro eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationDecision {
    /// Reutiliza o cadastro existente, sem modificar a chave registrada.
    Reuse,
    /// Autoriza a criação de um novo cadastro.
    Insert,
}

/// Motivo pelo qual uma solicitação de cadastro eleitoral foi recusada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RegistrationRefusal {
    /// A configuração da eleição não exige cadastro prévio.
    #[error("Cadastro não exigido nesta eleição")]
    NotRequired,
    /// A chave apresentada difere da chave anteriormente cadastrada.
    #[error("Chave diferente da já cadastrada")]
    KeyConflict,
    /// O prazo para realizar um novo cadastro está encerrado.
    #[error("Prazo de cadastro encerrado")]
    Closed,
    /// A conta não possui vínculo de elegibilidade vigente.
    #[error("Eleitor sem elegibilidade vigente")]
    NotEligible,
}

/// Decide com estado estável obtido durante a transação.
///
/// Uma retransmissão da mesma chave recupera o registro anterior mesmo que
/// o período de cadastramento tenha acabado ou o vínculo tenha expirado.
pub fn registration_decision(
    requires_registration: bool,
    registered_key: Option<&[u8]>,
    submitted_key: &[u8],
    starts_at: DateTime<Utc>,
    closed: bool,
    now: DateTime<Utc>,
    eligible_until: Option<DateTime<Utc>>,
) -> Result<RegistrationDecision, RegistrationRefusal> {
    if !requires_registration {
        return Err(RegistrationRefusal::NotRequired);
    }
    if let Some(registered) = registered_key {
        return if registered == submitted_key {
            Ok(RegistrationDecision::Reuse)
        } else {
            Err(RegistrationRefusal::KeyConflict)
        };
    }
    if closed || now >= starts_at {
        return Err(RegistrationRefusal::Closed);
    }
    if eligible_until.is_none_or(|end| now >= end) {
        return Err(RegistrationRefusal::NotEligible);
    }
    Ok(RegistrationDecision::Insert)
}

/// Resultado de comparar uma chave da cédula com o cadastro obrigatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisteredKeyError {
    /// Não existe uma chave previamente cadastrada.
    Missing,
    /// A chave da cédula difere da chave cadastrada.
    Mismatch,
}

/// Verifica se a chave pública da cédula corresponde à chave
/// previamente cadastrada pelo eleitor.
///
/// Retorna [`RegisteredKeyError::Missing`] quando não existe cadastro
/// e [`RegisteredKeyError::Mismatch`] quando as chaves divergem.
pub fn validate_registered_key(
    registered_key: Option<&[u8]>,
    ballot_key: &[u8],
) -> Result<(), RegisteredKeyError> {
    let registered_key = registered_key.ok_or(RegisteredKeyError::Missing)?;
    if registered_key != ballot_key {
        return Err(RegisteredKeyError::Mismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn reuses_existing_key_after_deadline_and_rejects_key_changes() {
        let now = Utc::now();
        assert_eq!(
            registration_decision(true, Some(b"A"), b"A", now, true, now, None),
            Ok(RegistrationDecision::Reuse)
        );
        assert_eq!(
            registration_decision(true, Some(b"A"), b"B", now, true, now, None),
            Err(RegistrationRefusal::KeyConflict)
        );
    }

    #[test]
    fn new_key_requires_policy_time_and_eligibility() {
        let now = Utc::now();
        let starts = now + Duration::hours(1);
        let eligible = Some(now + Duration::minutes(30));
        assert_eq!(
            registration_decision(true, None, b"A", starts, false, now, eligible),
            Ok(RegistrationDecision::Insert)
        );
        assert_eq!(
            registration_decision(false, None, b"A", starts, false, now, eligible),
            Err(RegistrationRefusal::NotRequired)
        );
        assert_eq!(
            registration_decision(true, None, b"A", now, false, now, eligible),
            Err(RegistrationRefusal::Closed)
        );
        assert_eq!(
            registration_decision(true, None, b"A", starts, false, now, None),
            Err(RegistrationRefusal::NotEligible)
        );
    }
}
