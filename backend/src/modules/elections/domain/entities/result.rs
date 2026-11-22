//! Contagens calculadas e desfecho definitivo da apuração eleitoral.
//!
//! A eleição recebe uma computação depois que a apuração começa e determina
//! o desfecho usando o total de participantes fixado no encerramento. Esse
//! desfecho fica disponível quando a eleição chega ao estado finalizado.

use crate::modules::elections::domain::{
    choices::ElectionOption,
    counts::{OptionCount, VoterCount},
    error::InvalidCountConsistency,
    rules::result_consistency::{CountConsistency, validate_result_counts},
};

/// Dados calculados na apuração para a eleição que será finalizada.
///
/// A aplicação usa os votos aceitos até o encerramento. Na votação secreta, os
/// totais vêm do agregado e das contribuições verificadas.
#[derive(Debug)]
pub enum TallyComputation {
    /// Contagem por opção, na ordem definida pela eleição.
    Totals(Vec<OptionCount>),

    /// O ElGamal encontrou uma contagem que não pôde ser recuperada em `[0, N]`.
    TotalOutOfRange,
}

/// Resultado definitivo determinado ao finalizar a apuração.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalTallyResult {
    /// Contagens consistentes com o número de participantes.
    Valid(Vec<OptionCount>),

    /// Apuração anulada porque a soma das contagens diverge do número de participantes.
    AnnulledBySumMismatch(Vec<OptionCount>),

    /// Apuração secreta anulada porque o ElGamal não recuperou um total em `[0, N]`.
    AnnulledOutOfRange,
}

impl FinalTallyResult {
    /// Confere as opções e os limites e determina o resultado usando o total de
    /// participantes fixado no encerramento. Uma soma divergente preserva as
    /// contagens para publicação junto do motivo da anulação.
    pub(super) fn from_computation(
        computation: TallyComputation,
        options: &[ElectionOption],
        voter_count: VoterCount,
    ) -> Result<Self, InvalidCountConsistency> {
        match computation {
            TallyComputation::Totals(counts) => {
                match validate_result_counts(options, &counts, voter_count)? {
                    CountConsistency::Consistent => Ok(Self::Valid(counts)),
                    CountConsistency::SumMismatch { .. } => Ok(Self::AnnulledBySumMismatch(counts)),
                }
            }
            TallyComputation::TotalOutOfRange => Ok(Self::AnnulledOutOfRange),
        }
    }
}
