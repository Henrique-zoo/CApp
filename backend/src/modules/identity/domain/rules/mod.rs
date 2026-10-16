//! Regras e invariantes de negócio para identidade, retenção e privacidade discente.

use chrono::{DateTime, Duration, Utc};

use super::entities::{IdentityRetentionStatus, StudentIdentity};
use crate::shared::retention::{PiiSanitizer, RetentionError};

/// Valida que a plataforma opera exclusivamente no contexto da UnB e rejeita `institution_id`.
pub fn validate_unb_single_institution<T>(institution_id: Option<T>) -> Result<(), RetentionError> {
    PiiSanitizer::validate_no_institution_id(institution_id)
}

/// Avalia se um discente inativo ultrapassou o período de tolerância recursal de 180 dias.
pub fn is_grace_period_expired(unlinked_at: DateTime<Utc>, current_time: DateTime<Utc>) -> bool {
    let grace_period = Duration::days(180);
    current_time >= (unlinked_at + grace_period)
}

/// Aplica as regras de anonimização irreversível sobre uma identidade discente.
pub fn anonymize_identity(
    identity: &mut StudentIdentity,
    current_time: DateTime<Utc>,
) -> Result<(), RetentionError> {
    identity.name = "Estudante Anonimizado".to_string();
    identity.email = format!("anonimizado-{}@anonimo.capp.unb.br", identity.id);
    identity.matricula = format!(
        "{}***{}",
        &identity.matricula[..2],
        &identity.matricula[7..]
    );
    identity.retention_status = IdentityRetentionStatus::Anonymized {
        anonymized_at: current_time,
    };

    Ok(())
}
