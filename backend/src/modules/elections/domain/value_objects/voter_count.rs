//! Quantidade de eleitores registrada em uma eleição.

/// Maior quantidade de eleitores permitida pelo contrato eleitoral.
pub const MAX_VOTER_COUNT: u32 = 100_000;

/// Quantidade validada de eleitores distintos de uma eleição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoterCount(u32);

/// Erros encontrados ao construir uma quantidade de eleitores inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidVoterCount {
    /// A quantidade excede o limite definido pelo contrato.
    #[error("A quantidade de eleitores não pode exceder 100000")]
    ExceedsMaximum,
}

impl VoterCount {
    /// Constrói uma quantidade validando o limite definido pelo contrato.
    pub const fn try_new(value: u32) -> Result<Self, InvalidVoterCount> {
        if value > MAX_VOTER_COUNT {
            return Err(InvalidVoterCount::ExceedsMaximum);
        }

        Ok(Self(value))
    }

    /// Retorna a quantidade de eleitores.
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero_voters() {
        let count = VoterCount::try_new(0).unwrap();

        assert_eq!(count.value(), 0);
    }

    #[test]
    fn accepts_maximum_voter_count() {
        let count = VoterCount::try_new(MAX_VOTER_COUNT).unwrap();

        assert_eq!(count.value(), MAX_VOTER_COUNT);
    }

    #[test]
    fn accepts_count_inside_allowed_range() {
        let count = VoterCount::try_new(500).unwrap();

        assert_eq!(count.value(), 500);
    }

    #[test]
    fn rejects_count_above_maximum() {
        let result = VoterCount::try_new(MAX_VOTER_COUNT + 1);

        assert_eq!(result, Err(InvalidVoterCount::ExceedsMaximum));
    }
}
