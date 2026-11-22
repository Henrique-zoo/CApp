//! Regras de consistência das contagens de uma apuração eleitoral.

use crate::modules::elections::domain::{
    choices::ElectionOption,
    counts::{OptionCount, VoterCount},
    error::InvalidCountConsistency,
};

/// Resultado da verificação aritmética das contagens eleitorais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountConsistency {
    /// A soma das contagens corresponde ao total de eleitores.
    Consistent,

    /// A soma das contagens diverge do total de eleitores.
    SumMismatch {
        /// Total de eleitores registrado no fechamento.
        expected: u32,

        /// Soma efetiva das contagens por opção.
        actual: u64,
    },
}

/// Verifica a correspondência entre as opções do manifesto e suas contagens.
///
/// A função preserva a ordem publicada no manifesto, inclusive a posição
/// final reservada à opção Nulo.
///
/// Uma divergência na soma não é tratada como erro estrutural: ela é
/// representada por [`CountConsistency::SumMismatch`].
pub fn validate_result_counts(
    options: &[ElectionOption],
    counts: &[OptionCount],
    voter_count: VoterCount,
) -> Result<CountConsistency, InvalidCountConsistency> {
    if options.len() != counts.len() {
        return Err(InvalidCountConsistency::OptionCountMismatch);
    }

    for (option, count) in options.iter().zip(counts.iter()) {
        if option.id() != count.option_id() {
            return Err(InvalidCountConsistency::OptionOrderMismatch);
        }
        if count.count().value() > voter_count.value() {
            return Err(InvalidCountConsistency::CountOutOfRange);
        }
    }

    Ok(compare_count_sum(counts, voter_count))
}

/// Compara a soma das contagens com o total de participantes.
fn compare_count_sum(counts: &[OptionCount], voter_count: VoterCount) -> CountConsistency {
    let actual = counts
        .iter()
        .map(|count| u64::from(count.count().value()))
        .sum::<u64>();

    let expected = voter_count.value();

    if actual == u64::from(expected) {
        CountConsistency::Consistent
    } else {
        CountConsistency::SumMismatch { expected, actual }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::domain::identifiers::SlateId;
    use uuid::Uuid;

    fn slate_option(id: u128, name: &str) -> ElectionOption {
        ElectionOption::try_slate(SlateId::try_from_uuid(Uuid::from_u128(id)).unwrap(), name)
            .unwrap()
    }

    fn null_option() -> ElectionOption {
        ElectionOption::Null
    }

    fn options() -> Vec<ElectionOption> {
        vec![
            slate_option(1, "Chapa A"),
            slate_option(2, "Chapa B"),
            null_option(),
        ]
    }

    fn counts(values: [(u128, u32); 3]) -> Vec<OptionCount> {
        values
            .into_iter()
            .map(|(id, count)| OptionCount::try_new(Uuid::from_u128(id), count).unwrap())
            .collect()
    }

    #[test]
    fn accepts_counts_matching_manifest_and_voter_total() {
        let options = options();
        let counts = counts([(1, 4), (2, 3), (0, 1)]);

        let result = validate_result_counts(&options, &counts, VoterCount::try_new(8).unwrap());

        assert_eq!(result, Ok(CountConsistency::Consistent));
    }

    #[test]
    fn identifies_sum_mismatch() {
        let options = options();
        let counts = counts([(1, 4), (2, 3), (0, 1)]);

        let result = validate_result_counts(&options, &counts, VoterCount::try_new(9).unwrap());

        assert_eq!(
            result,
            Ok(CountConsistency::SumMismatch {
                expected: 9,
                actual: 8,
            })
        );
    }

    #[test]
    fn rejects_different_number_of_counts() {
        let options = options();
        let counts = counts([(1, 4), (2, 3), (0, 1)]);
        let counts = &counts[..2];

        let result = validate_result_counts(&options, counts, VoterCount::try_new(7).unwrap());

        assert_eq!(result, Err(InvalidCountConsistency::OptionCountMismatch));
    }

    #[test]
    fn rejects_counts_in_different_order() {
        let options = options();
        let counts = counts([(2, 4), (1, 3), (0, 1)]);

        let result = validate_result_counts(&options, &counts, VoterCount::try_new(8).unwrap());

        assert_eq!(result, Err(InvalidCountConsistency::OptionOrderMismatch));
    }

    #[test]
    fn validates_null_option_in_last_position() {
        let options = options();
        let counts = counts([(1, 4), (2, 3), (0, 1)]);

        let result = validate_result_counts(&options, &counts, VoterCount::try_new(8).unwrap());

        assert_eq!(result, Ok(CountConsistency::Consistent));
        assert!(options.last().unwrap().id().is_nil());
        assert!(counts.last().unwrap().option_id().is_nil());
    }

    #[test]
    fn accepts_zero_counts_when_voter_total_is_zero() {
        let options = options();
        let counts = counts([(1, 0), (2, 0), (0, 0)]);

        let result = validate_result_counts(&options, &counts, VoterCount::try_new(0).unwrap());

        assert_eq!(result, Ok(CountConsistency::Consistent));
    }
}
