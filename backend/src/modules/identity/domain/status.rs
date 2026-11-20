//! Estados reutilizados pelos cadastros institucionais e usuários.

use std::{error::Error, fmt, str::FromStr};

/// Erro de conversão de status de catálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogStatusError {
    /// Rótulo de status não reconhecido.
    InvalidStatus(String),
}

impl fmt::Display for CatalogStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStatus(s) => write!(f, "status de catálogo inválido: '{s}'"),
        }
    }
}

impl Error for CatalogStatusError {}

/// Disponibilidade de um registro no catálogo.
///
/// Este estado não comprova vínculo estudantil e não é usado atualmente como
/// critério de autorização pelo serviço de autenticação.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogStatus {
    /// Disponível para novos usos.
    #[default]
    Active,
    /// Indisponível para novos usos, preservando os registros históricos.
    Inactive,
}

impl CatalogStatus {
    /// Retorna a representação textual canônica do status de catálogo.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl fmt::Display for CatalogStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CatalogStatus {
    type Err = CatalogStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            other => Err(CatalogStatusError::InvalidStatus(other.to_string())),
        }
    }
}

/// Erro de conversão de status de usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserStatusError {
    /// Rótulo de status de usuário não reconhecido.
    InvalidStatus(String),
}

impl fmt::Display for UserStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStatus(s) => write!(f, "status de usuário inválido: '{s}'"),
        }
    }
}

impl Error for UserStatusError {}

/// Ciclo de vida interno da conta: incompleta, ativa ou desativada.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserStatus {
    /// Identidade provisionada, aguardando os dados e a credencial interna.
    #[default]
    Incomplete,
    /// Conta CApp habilitada e ativa.
    Active,
    /// Conta CApp desativada.
    Inactive,
}

impl UserStatus {
    /// Retorna a representação textual canônica do status do usuário.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserStatus {
    type Err = UserStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "incomplete" => Ok(Self::Incomplete),
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            other => Err(UserStatusError::InvalidStatus(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_and_parse_catalog_status() {
        assert_eq!(CatalogStatus::Active.as_str(), "active");
        assert_eq!(CatalogStatus::Inactive.as_str(), "inactive");
        assert_eq!(
            CatalogStatus::from_str("active").unwrap(),
            CatalogStatus::Active
        );
        assert_eq!(
            CatalogStatus::from_str("inactive").unwrap(),
            CatalogStatus::Inactive
        );
        assert!(CatalogStatus::from_str("desconhecido").is_err());
    }

    #[test]
    fn should_format_and_parse_user_status() {
        assert_eq!(UserStatus::Incomplete.as_str(), "incomplete");
        assert_eq!(UserStatus::Active.as_str(), "active");
        assert_eq!(UserStatus::Inactive.as_str(), "inactive");
        assert_eq!(
            UserStatus::from_str("incomplete").unwrap(),
            UserStatus::Incomplete
        );
        assert_eq!(UserStatus::from_str("active").unwrap(), UserStatus::Active);
        assert_eq!(
            UserStatus::from_str("inactive").unwrap(),
            UserStatus::Inactive
        );
        assert!(UserStatus::from_str("desconhecido").is_err());
    }
}
