//! Contexto imutável da configuração eleitoral consultada antes do aceite.

use uuid::Uuid;

use super::super::domain::Fingerprint;

/// Contexto obtido do manifesto oficial e da identidade autenticada.
#[derive(Debug, Clone)]
pub struct BallotVerificationContext {
    /// Digest do manifesto oficial.
    pub manifest_hash: Fingerprint,
    /// Ordem das opções no manifesto, incluindo Nulo.
    pub option_ids: Vec<Uuid>,
}
