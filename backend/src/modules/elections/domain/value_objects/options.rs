//! Coleção de opções compartilhada pelas duas modalidades eleitorais.

use std::collections::HashSet;

use super::option::{ElectionOption, OptionKind};

/// Uma chapa e a opção especial Nulo.
pub const MIN_OPTIONS: usize = 2;

/// Cinquenta chapas e a opção especial Nulo, conforme o perfil v1.
pub const MAX_OPTIONS: usize = 51;

/// Opções aprovadas fornecidas pela aplicação, na ordem da configuração.
///
/// A aprovação das candidaturas é externa a este valor. Uma a cinquenta chapas
/// são acompanhadas de exatamente um Nulo; não existe opção de branco.
/// A mesma ordem é usada pelo manifesto confidencial e pela contagem identificada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionOptions(Vec<ElectionOption>);

/// Violação estrutural da coleção de opções eleitorais.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionOptions {
    /// Exige uma a cinquenta chapas, além de Nulo.
    #[error("A eleição deve possuir entre 2 e 51 opções")]
    InvalidCount,
    /// IDs de opções são únicos na eleição.
    #[error("Os identificadores das opções devem ser distintos")]
    DuplicateId,
    /// Nulo é obrigatório e único.
    #[error("A eleição deve possuir exatamente uma opção Nulo")]
    InvalidNullCount,
}

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
            .filter(|o| o.kind() == OptionKind::Null)
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
