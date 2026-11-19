//! Configuração pronta para abertura, distinta do calendário inicial.

use super::manifest::VerifiedManifest;
use crate::modules::elections::domain::value_objects::{
    modality::ElectionModality, option::ElectionOption, options::ElectionOptions,
};

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

/// Alteração de configuração aplicada durante a preparação.
///
/// A revisão indica quantas alterações de calendário já tinham sido aplicadas.
/// Alterações na mesma revisão preservam a ordem da coleção. Isso permite
/// reconstituir uma revisão que referenciava um manifesto posteriormente
/// invalidado, sem reescrever os registros assinados do calendário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationConfigurationChange {
    after_schedule_revision: u32,
    configuration: Option<ElectionConfiguration>,
}

impl PreparationConfigurationChange {
    /// Registra uma configuração já validada pelo agregado, ou sua invalidação.
    pub(super) const fn new(
        after_schedule_revision: u32,
        configuration: Option<ElectionConfiguration>,
    ) -> Self {
        Self {
            after_schedule_revision,
            configuration,
        }
    }

    /// Revisão de calendário vigente antes da alteração.
    pub const fn after_schedule_revision(&self) -> u32 {
        self.after_schedule_revision
    }

    /// Configuração resultante; `None` retira a prontidão para abertura.
    pub const fn configuration(&self) -> Option<&ElectionConfiguration> {
        self.configuration.as_ref()
    }
}
