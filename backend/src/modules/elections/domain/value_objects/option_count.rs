//! Quantidade de votos contabilizados para uma opção eleitoral.

use crate::modules::elections::domain::value_objects::{
    identifiers::OptionId,
    voter_count::{InvalidVoterCount, VoterCount},
};

/// Contagem validada de votos atribuída a uma opção eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptionCount {
    option_id: OptionId,
    count: VoterCount,
}

/// Erros encontrados ao construir uma contagem de opção inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidOptionCount {
    /// A quantidade de votos excede o limite do contrato.
    #[error("A quantidade de votos da opção excede 100000")]
    InvalidCount(#[from] InvalidVoterCount),
}

impl OptionCount {
    /// Constrói uma contagem validando o limite de votos do contrato.
    pub const fn try_new(option_id: OptionId, count: u32) -> Result<Self, InvalidOptionCount> {
        let count = match VoterCount::try_new(count) {
            Ok(count) => count,
            Err(error) => return Err(InvalidOptionCount::InvalidCount(error)),
        };

        Ok(Self { option_id, count })
    }

    /// Retorna o identificador da opção eleitoral.
    pub const fn option_id(&self) -> OptionId {
        self.option_id
    }

    /// Retorna a quantidade de votos contabilizados.
    pub const fn count(&self) -> VoterCount {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn accepts_valid_option_count() {
        let option_id = OptionId::from_uuid(Uuid::new_v4());

        let option_count = OptionCount::try_new(option_id, 42).unwrap();

        assert_eq!(option_count.option_id(), option_id);
        assert_eq!(option_count.count().value(), 42);
    }

    #[test]
    fn accepts_zero_votes() {
        let option_id = OptionId::from_uuid(Uuid::new_v4());

        let option_count = OptionCount::try_new(option_id, 0).unwrap();

        assert_eq!(option_count.count().value(), 0);
    }

    #[test]
    fn accepts_null_option() {
        let option_id = OptionId::from_uuid(Uuid::nil());

        let option_count = OptionCount::try_new(option_id, 3).unwrap();

        assert!(option_count.option_id().is_null_option());
        assert_eq!(option_count.count().value(), 3);
    }

    #[test]
    fn accepts_maximum_vote_count() {
        let option_id = OptionId::from_uuid(Uuid::new_v4());

        let option_count = OptionCount::try_new(option_id, 100_000).unwrap();

        assert_eq!(option_count.count().value(), 100_000);
    }

    #[test]
    fn rejects_vote_count_above_contract_limit() {
        let option_id = OptionId::from_uuid(Uuid::new_v4());

        let result = OptionCount::try_new(option_id, 100_001);

        assert_eq!(
            result,
            Err(InvalidOptionCount::InvalidCount(
                InvalidVoterCount::ExceedsMaximum
            ))
        );
    }
}
