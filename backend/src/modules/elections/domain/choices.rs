//! Chapas e voto Nulo na ordem compartilhada pela configuração e pela apuração.

use std::collections::HashSet;

use uuid::Uuid;

use super::{
    error::{InvalidElectionOption, InvalidElectionOptions},
    identifiers::SlateId,
    profile::{MAX_OPTIONS, MIN_OPTIONS},
};

/// Nome e identificador validados de uma chapa disponível para voto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlateOption {
    id: SlateId,
    name: String,
}

/// Escolha eleitoral. A lista completa determina a ordem e a unicidade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectionOption {
    /// Chapa com nome validado e identificador diferente de zero.
    Slate(SlateOption),

    /// Voto Nulo, representado pelo UUID zero no protocolo.
    Null,
}

impl ElectionOption {
    /// Nome reservado pelo protocolo para a opção Nulo.
    pub const NULL_OPTION_NAME: &'static str = "Nulo";

    /// Constrói a escolha de uma chapa com identificador validado.
    ///
    /// O nome precisa conter texto e ter até 200 caracteres.
    pub fn try_slate(id: SlateId, name: impl Into<String>) -> Result<Self, InvalidElectionOption> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(InvalidElectionOption::EmptyName);
        }

        if name.chars().count() > 200 {
            return Err(InvalidElectionOption::NameTooLong);
        }

        Ok(Self::Slate(SlateOption { id, name }))
    }

    /// Retorna o identificador da chapa ou o UUID zero para Nulo.
    pub const fn id(&self) -> Uuid {
        match self {
            Self::Slate(slate) => slate.id.into_uuid(),
            Self::Null => Uuid::nil(),
        }
    }

    /// Identificador da chapa quando a escolha corresponde a uma candidatura.
    pub const fn slate_id(&self) -> Option<SlateId> {
        match self {
            Self::Slate(slate) => Some(slate.id),
            Self::Null => None,
        }
    }

    /// Nome cadastrado da chapa ou o nome fixo da opção Nulo.
    pub fn name(&self) -> &str {
        match self {
            Self::Slate(slate) => &slate.name,
            Self::Null => Self::NULL_OPTION_NAME,
        }
    }
}

/// Opções aprovadas fornecidas pela aplicação, na ordem da configuração.
///
/// A aprovação das candidaturas é externa a este valor. Uma a cinquenta chapas
/// são acompanhadas de exatamente um Nulo; não existe opção de branco.
/// A mesma ordem é usada pelo manifesto confidencial e pela contagem identificada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionOptions(Vec<ElectionOption>);

impl ElectionOptions {
    /// Valida limites, unicidade e presença de Nulo sem ordenar a entrada.
    /// Somente o manifesto confidencial exige Nulo na última posição.
    pub fn try_new(options: Vec<ElectionOption>) -> Result<Self, InvalidElectionOptions> {
        if !(MIN_OPTIONS..=MAX_OPTIONS).contains(&options.len()) {
            return Err(InvalidElectionOptions::InvalidCount);
        }
        let mut ids = HashSet::with_capacity(options.len());
        for option in &options {
            if !ids.insert(option.id()) {
                return Err(InvalidElectionOptions::DuplicateId);
            }
        }
        if options
            .iter()
            .filter(|option| matches!(option, ElectionOption::Null))
            .count()
            != 1
        {
            return Err(InvalidElectionOptions::InvalidNullCount);
        }
        Ok(Self(options))
    }

    /// Retorna a coleção na ordem aprovada.
    pub fn as_slice(&self) -> &[ElectionOption] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_slate_option() {
        let id = SlateId::try_from_uuid(Uuid::new_v4()).unwrap();

        let option = ElectionOption::try_slate(id, "Chapa Horizonte").unwrap();

        assert_eq!(option.id(), id.into_uuid());
        assert_eq!(option.slate_id(), Some(id));
        assert_eq!(option.name(), "Chapa Horizonte");
    }

    #[test]
    fn accepts_valid_null_option() {
        let option = ElectionOption::Null;

        assert!(option.id().is_nil());
        assert_eq!(option.slate_id(), None);
        assert_eq!(option.name(), "Nulo");
    }

    #[test]
    fn rejects_empty_name() {
        let id = SlateId::try_from_uuid(Uuid::new_v4()).unwrap();

        assert_eq!(
            ElectionOption::try_slate(id, ""),
            Err(InvalidElectionOption::EmptyName)
        );
    }

    #[test]
    fn rejects_name_composed_only_of_whitespace() {
        let id = SlateId::try_from_uuid(Uuid::new_v4()).unwrap();

        assert_eq!(
            ElectionOption::try_slate(id, "   \t  "),
            Err(InvalidElectionOption::EmptyName)
        );
    }

    #[test]
    fn rejects_name_longer_than_200_characters() {
        let id = SlateId::try_from_uuid(Uuid::new_v4()).unwrap();
        let name = "a".repeat(201);

        assert_eq!(
            ElectionOption::try_slate(id, name),
            Err(InvalidElectionOption::NameTooLong)
        );
    }

    #[test]
    fn accepts_name_with_exactly_200_characters() {
        let id = SlateId::try_from_uuid(Uuid::new_v4()).unwrap();
        let name = "a".repeat(200);

        let option = ElectionOption::try_slate(id, name).unwrap();

        assert_eq!(option.name().chars().count(), 200);
    }

    #[test]
    fn identified_options_require_a_slate_and_unique_ids_within_the_limit() {
        let slate_id = SlateId::try_from_uuid(Uuid::from_u128(1)).unwrap();
        let slate = ElectionOption::try_slate(slate_id, "Aurora").unwrap();

        assert!(ElectionOptions::try_new(vec![ElectionOption::Null]).is_err());
        assert!(
            ElectionOptions::try_new(vec![slate.clone(), slate, ElectionOption::Null]).is_err()
        );

        for slates in [50, 51] {
            let mut choices: Vec<_> = (1..=slates)
                .map(|id| {
                    let slate_id = SlateId::try_from_uuid(Uuid::from_u128(id)).unwrap();
                    ElectionOption::try_slate(slate_id, format!("Chapa {id}")).unwrap()
                })
                .collect();
            choices.push(ElectionOption::Null);
            assert_eq!(ElectionOptions::try_new(choices).is_ok(), slates == 50);
        }
    }
}
