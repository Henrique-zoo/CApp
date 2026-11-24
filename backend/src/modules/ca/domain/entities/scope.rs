//! Escopos de atuação e segregação de permissões na plataforma.
//!
//! Define a classificação formal das ações administrativas, separando as atribuições
//! da Gestão do Centro Acadêmico, da Comissão Eleitoral e da Administração Geral.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erros decorrentes da conversão ou validação de escopos de permissão.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScopeError {
    /// O identificador textual do escopo informado é desconhecido.
    #[error(
        "Escopo desconhecido: '{0}'. Escopos válidos: 'ca_management', 'electoral_commission', 'platform_administration'."
    )]
    UnknownScope(String),
}

/// Escopos hierárquicos e funcionais de autorização.
///
/// Garante a segregação estrita de responsabilidades:
/// - [`Scope::CaManagement`]: Ações exclusivas da diretoria e gestão do Centro Acadêmico.
/// - [`Scope::ElectoralCommission`]: Ações privativas da comissão eleitoral independente.
/// - [`Scope::PlatformAdministration`]: Ações globais de governança e sustentação da plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Permissões de gestão cotidiana do Centro Acadêmico (notícias, eventos, integrantes, finanças).
    CaManagement,

    /// Permissões relativas à condução autônoma do pleito eleitoral (chapas, regras, apuração).
    ElectoralCommission,

    /// Permissões de administração institucional da plataforma (aprovação de CAs, auditoria global).
    PlatformAdministration,
}

impl Scope {
    /// Retorna o identificador textual padronizado do escopo em formato snake_case.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CaManagement => "ca_management",
            Self::ElectoralCommission => "electoral_commission",
            Self::PlatformAdministration => "platform_administration",
        }
    }

    /// Retorna a descrição formal legível do escopo em língua portuguesa.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::CaManagement => "Gestão do Centro Acadêmico",
            Self::ElectoralCommission => "Comissão Eleitoral",
            Self::PlatformAdministration => "Administração Geral da Plataforma",
        }
    }

    /// Informa se o escopo refere-se à gestão administrativa do Centro Acadêmico.
    pub const fn is_ca_management(&self) -> bool {
        matches!(self, Self::CaManagement)
    }

    /// Informa se o escopo refere-se às atribuições eleitorais.
    pub const fn is_electoral_commission(&self) -> bool {
        matches!(self, Self::ElectoralCommission)
    }

    /// Informa se o escopo refere-se à governança geral da plataforma.
    pub const fn is_platform_administration(&self) -> bool {
        matches!(self, Self::PlatformAdministration)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Scope {
    type Err = ScopeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "ca_management"
            | "gestao_ca"
            | "gestão_ca"
            | "gestão do centro acadêmico"
            | "gestao do centro academico" => Ok(Self::CaManagement),
            "electoral_commission"
            | "comissao_eleitoral"
            | "comissão_eleitoral"
            | "comissão eleitoral"
            | "comissao eleitoral" => Ok(Self::ElectoralCommission),
            "platform_administration"
            | "administracao_plataforma"
            | "administração_plataforma"
            | "administração geral"
            | "administracao geral" => Ok(Self::PlatformAdministration),
            _ => Err(ScopeError::UnknownScope(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_and_parse_scopes() {
        assert_eq!(Scope::CaManagement.as_str(), "ca_management");
        assert_eq!(Scope::ElectoralCommission.as_str(), "electoral_commission");
        assert_eq!(
            Scope::PlatformAdministration.as_str(),
            "platform_administration"
        );

        assert_eq!(
            "ca_management".parse::<Scope>().unwrap(),
            Scope::CaManagement
        );
        assert_eq!(
            "comissão eleitoral".parse::<Scope>().unwrap(),
            Scope::ElectoralCommission
        );
        assert_eq!(
            "administração geral".parse::<Scope>().unwrap(),
            Scope::PlatformAdministration
        );
    }

    #[test]
    fn should_reject_unknown_scope() {
        assert!(matches!(
            "invalido".parse::<Scope>(),
            Err(ScopeError::UnknownScope(_))
        ));
    }
}
