//! Total de participantes de uma eleição e contagens por escolha.

use uuid::Uuid;

use super::{
    error::{InvalidOptionCount, InvalidVoterCount},
    profile::MAX_VOTER_COUNT,
};

/// Quantidade validada de eleitores distintos de uma eleição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoterCount(u32);

impl VoterCount {
    /// Total de participantes quando a eleição foi encerrada sem votos.
    pub const ZERO: Self = Self(0);

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

/// Contagem validada de votos atribuída a uma opção eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptionCount {
    option_id: Uuid,
    count: VoterCount,
}

impl OptionCount {
    /// Constrói uma contagem validando o limite de votos do contrato.
    pub const fn try_new(option_id: Uuid, count: u32) -> Result<Self, InvalidOptionCount> {
        let count = match VoterCount::try_new(count) {
            Ok(count) => count,
            Err(error) => return Err(InvalidOptionCount::InvalidCount(error)),
        };

        Ok(Self { option_id, count })
    }

    /// Retorna o identificador da opção eleitoral.
    pub const fn option_id(&self) -> Uuid {
        self.option_id
    }

    /// Retorna a quantidade de votos contabilizados.
    pub const fn count(&self) -> VoterCount {
        self.count
    }
}

#[cfg(test)]
mod voter_count_tests {
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

#[cfg(test)]
mod option_count_tests {
    use super::*;

    #[test]
    fn accepts_valid_option_count() {
        let option_id = Uuid::new_v4();

        let option_count = OptionCount::try_new(option_id, 42).unwrap();

        assert_eq!(option_count.option_id(), option_id);
        assert_eq!(option_count.count().value(), 42);
    }

    #[test]
    fn accepts_zero_votes() {
        let option_id = Uuid::new_v4();

        let option_count = OptionCount::try_new(option_id, 0).unwrap();

        assert_eq!(option_count.count().value(), 0);
    }

    #[test]
    fn accepts_null_option() {
        let option_id = Uuid::nil();

        let option_count = OptionCount::try_new(option_id, 3).unwrap();

        assert!(option_count.option_id().is_nil());
        assert_eq!(option_count.count().value(), 3);
    }

    #[test]
    fn accepts_maximum_vote_count() {
        let option_id = Uuid::new_v4();

        let option_count = OptionCount::try_new(option_id, 100_000).unwrap();

        assert_eq!(option_count.count().value(), 100_000);
    }

    #[test]
    fn rejects_vote_count_above_contract_limit() {
        let option_id = Uuid::new_v4();

        let result = OptionCount::try_new(option_id, 100_001);

        assert_eq!(
            result,
            Err(InvalidOptionCount::InvalidCount(
                InvalidVoterCount::ExceedsMaximum
            ))
        );
    }
}
