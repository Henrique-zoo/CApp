//! Opções ordenadas que podem integrar o manifesto eleitoral.

use crate::modules::elections::domain::value_objects::identifiers::OptionId;

/// Categoria de uma opção eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionKind {
    /// Opção correspondente a uma chapa.
    Slate,

    /// Opção reservada para voto nulo.
    Null,
}

/// Opção eleitoral validada individualmente.
///
/// A posição da opção Nulo e a unicidade dos IDs são verificadas pelo manifesto,
/// pois dependem da coleção completa de opções.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionOption {
    id: OptionId,
    kind: OptionKind,
    name: String,
}

/// Erro ao construir uma opção eleitoral inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionOption {
    /// Nome vazio ou composto somente por espaços.
    #[error("O nome da opção não pode ser vazio")]
    EmptyName,

    /// Nome maior que o limite do contrato.
    #[error("O nome da opção excede 200 caracteres")]
    NameTooLong,

    /// A opção Nulo exige UUID zero e nome exatamente "Nulo".
    #[error("A opção Nulo exige UUID zero e nome Nulo")]
    InvalidNullOption,

    /// Uma opção de chapa não pode usar UUID zero.
    #[error("Uma opção de chapa não pode usar UUID zero")]
    InvalidSlateOption,
}

impl ElectionOption {
    /// Nome reservado pelo protocolo para a opção Nulo.
    pub const NULL_OPTION_NAME: &'static str = "Nulo";

    /// Constrói uma opção eleitoral verificando suas invariantes individuais.
    pub fn try_new(
        id: OptionId,
        kind: OptionKind,
        name: impl Into<String>,
    ) -> Result<Self, InvalidElectionOption> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(InvalidElectionOption::EmptyName);
        }

        if name.chars().count() > 200 {
            return Err(InvalidElectionOption::NameTooLong);
        }

        match kind {
            OptionKind::Null => {
                if !id.is_null_option() || name != Self::NULL_OPTION_NAME {
                    return Err(InvalidElectionOption::InvalidNullOption);
                }
            }
            OptionKind::Slate => {
                if id.is_null_option() {
                    return Err(InvalidElectionOption::InvalidSlateOption);
                }
            }
        }

        Ok(Self { id, kind, name })
    }

    /// Retorna o identificador da opção.
    pub const fn id(&self) -> OptionId {
        self.id
    }

    /// Retorna a categoria da opção.
    pub const fn kind(&self) -> OptionKind {
        self.kind
    }

    /// Retorna o nome da opção.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn accepts_valid_slate_option() {
        let id = OptionId::from_uuid(Uuid::new_v4());

        let option = ElectionOption::try_new(id, OptionKind::Slate, "Chapa Horizonte").unwrap();

        assert_eq!(option.id(), id);
        assert_eq!(option.kind(), OptionKind::Slate);
        assert_eq!(option.name(), "Chapa Horizonte");
    }

    #[test]
    fn accepts_valid_null_option() {
        let id = OptionId::from_uuid(Uuid::nil());

        let option =
            ElectionOption::try_new(id, OptionKind::Null, ElectionOption::NULL_OPTION_NAME)
                .unwrap();

        assert!(option.id().is_null_option());
        assert_eq!(option.kind(), OptionKind::Null);
        assert_eq!(option.name(), "Nulo");
    }

    #[test]
    fn rejects_null_option_with_non_nil_id() {
        let id = OptionId::from_uuid(Uuid::new_v4());

        assert_eq!(
            ElectionOption::try_new(id, OptionKind::Null, "Nulo"),
            Err(InvalidElectionOption::InvalidNullOption)
        );
    }

    #[test]
    fn rejects_null_option_with_different_name() {
        let id = OptionId::from_uuid(Uuid::nil());

        assert_eq!(
            ElectionOption::try_new(id, OptionKind::Null, "Voto nulo"),
            Err(InvalidElectionOption::InvalidNullOption)
        );
    }

    #[test]
    fn rejects_slate_with_nil_id() {
        let id = OptionId::from_uuid(Uuid::nil());

        assert_eq!(
            ElectionOption::try_new(id, OptionKind::Slate, "Chapa"),
            Err(InvalidElectionOption::InvalidSlateOption)
        );
    }

    #[test]
    fn rejects_empty_name() {
        let id = OptionId::from_uuid(Uuid::new_v4());

        assert_eq!(
            ElectionOption::try_new(id, OptionKind::Slate, ""),
            Err(InvalidElectionOption::EmptyName)
        );
    }

    #[test]
    fn rejects_name_composed_only_of_whitespace() {
        let id = OptionId::from_uuid(Uuid::new_v4());

        assert_eq!(
            ElectionOption::try_new(id, OptionKind::Slate, "   \t  "),
            Err(InvalidElectionOption::EmptyName)
        );
    }

    #[test]
    fn rejects_name_longer_than_200_characters() {
        let id = OptionId::from_uuid(Uuid::new_v4());
        let name = "a".repeat(201);

        assert_eq!(
            ElectionOption::try_new(id, OptionKind::Slate, name),
            Err(InvalidElectionOption::NameTooLong)
        );
    }

    #[test]
    fn accepts_name_with_exactly_200_characters() {
        let id = OptionId::from_uuid(Uuid::new_v4());
        let name = "a".repeat(200);

        let option = ElectionOption::try_new(id, OptionKind::Slate, name).unwrap();

        assert_eq!(option.name().chars().count(), 200);
    }
}
