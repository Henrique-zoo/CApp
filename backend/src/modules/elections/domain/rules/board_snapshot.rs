//! Presença e origem das referências de uma fotografia confidencial.
//!
//! Cancelamento e encerramento usam a mesma regra do perfil v1. Conferir o quadro
//! completo, seus digests e a contagem transacional pertence aos adaptadores.

use crate::modules::elections::domain::{
    Fingerprint, counts::VoterCount, error::InvalidBoardSnapshot,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::test_support::fp;

    #[test]
    fn a_board_without_a_manifest_contains_no_head_or_participants() {
        let head = fp('b');
        let zero = VoterCount::try_new(0).unwrap();
        let one = VoterCount::try_new(1).unwrap();

        assert_eq!(
            validate_confidential_board_snapshot(None, None, zero),
            Ok(())
        );
        assert_eq!(
            validate_confidential_board_snapshot(None, Some(&head), zero),
            Err(InvalidBoardSnapshot::ManifestRequired)
        );
        assert_eq!(
            validate_confidential_board_snapshot(None, None, one),
            Err(InvalidBoardSnapshot::ManifestRequired)
        );
    }

    #[test]
    fn a_board_with_a_manifest_requires_a_head_consistent_with_the_count() {
        let manifest = fp('b');
        let other = fp('c');
        let zero = VoterCount::try_new(0).unwrap();
        let one = VoterCount::try_new(1).unwrap();

        assert_eq!(
            validate_confidential_board_snapshot(Some(&manifest), None, zero),
            Err(InvalidBoardSnapshot::HeadRequired)
        );
        assert_eq!(
            validate_confidential_board_snapshot(Some(&manifest), Some(&manifest), zero),
            Ok(())
        );
        assert_eq!(
            validate_confidential_board_snapshot(Some(&manifest), Some(&other), one),
            Ok(())
        );
        for (head, count) in [(&other, zero), (&manifest, one)] {
            assert_eq!(
                validate_confidential_board_snapshot(Some(&manifest), Some(head), count),
                Err(InvalidBoardSnapshot::InvalidEmptyBoard)
            );
        }
    }
}
