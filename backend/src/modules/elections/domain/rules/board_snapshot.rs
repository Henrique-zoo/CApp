//! Presença e origem das referências de uma fotografia confidencial.
//!
//! Cancelamento e encerramento usam a mesma regra do perfil v1. Conferir o quadro
//! completo, seus digests e a contagem transacional pertence aos adaptadores.

use crate::modules::elections::domain::value_objects::{
    fingerprint::Fingerprint, voter_count::VoterCount,
};

/// Referências incompatíveis com a existência do manifesto e a contagem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBoardSnapshot {
    /// Sem manifesto, não pode existir cabeça de quadro nem participação.
    #[error("Quadro ou participação sem manifesto")]
    ManifestRequired,
    /// Publicado o manifesto, a fotografia precisa identificar a cabeça do quadro.
    #[error("A fotografia confidencial exige a cabeça do quadro")]
    HeadRequired,
    /// A origem do quadro representa exatamente zero participantes.
    #[error("A cabeça do quadro é incompatível com a contagem de participantes")]
    InvalidEmptyBoard,
}

/// Confere referências confidenciais sem calcular hashes ou percorrer o quadro.
///
/// Antes do manifesto, somente a fotografia vazia sem cabeça é permitida. Com
/// manifesto, exige cabeça; a origem do quadro corresponde exatamente à
/// contagem zero. A modalidade identificada não usa esta regra nem exige hashes.
///
/// # Erros
///
/// Retorna [`InvalidBoardSnapshot`] quando faltar uma referência necessária ou
/// houver evidência incompatível com a ausência de manifesto/participantes.
pub fn validate_confidential_board_snapshot(
    manifest_hash: Option<&Fingerprint>,
    head_hash: Option<&Fingerprint>,
    voter_count: VoterCount,
) -> Result<(), InvalidBoardSnapshot> {
    match (manifest_hash, head_hash) {
        (None, None) if voter_count.value() == 0 => Ok(()),
        (None, _) => Err(InvalidBoardSnapshot::ManifestRequired),
        (Some(_), None) => Err(InvalidBoardSnapshot::HeadRequired),
        (Some(manifest), Some(head)) if (voter_count.value() == 0) != (head == manifest) => {
            Err(InvalidBoardSnapshot::InvalidEmptyBoard)
        }
        (Some(_), Some(_)) => Ok(()),
    }
}
