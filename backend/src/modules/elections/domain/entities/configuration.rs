//! Configuração pronta para abertura, distinta do calendário inicial.

use super::{election::ElectionModality, manifest::VerifiedManifest};
use crate::modules::elections::domain::choices::{ElectionOption, ElectionOptions};

/// Material público necessário à modalidade escolhida.
///
/// A ausência desta configuração no agregado representa preparação incompleta.
/// Nenhuma variante contém cadastro prévio de eleitores ou segredos privados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectionConfiguration {
    /// Opções de voto identificado, sem cerimônia ou manifesto ElGamal.
    Identified(ElectionOptions),
    /// Manifesto e digest conferidos pelas portas criptográficas da aplicação.
    Confidential(Box<VerifiedManifest>),
}

impl ElectionConfiguration {
    /// Retorna a modalidade correspondente ao material configurado.
    pub const fn modality(&self) -> ElectionModality {
        match self {
            Self::Identified(_) => ElectionModality::Identified,
            Self::Confidential(_) => ElectionModality::Confidential,
        }
    }

    /// Retorna as opções preservando a ordem configurada.
    pub fn options(&self) -> &[ElectionOption] {
        match self {
            Self::Identified(options) => options.as_slice(),
            Self::Confidential(verified) => verified.manifest().options(),
        }
    }
}
