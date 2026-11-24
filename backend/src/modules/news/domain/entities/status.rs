//! Estado editorial do artigo de notícia.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::modules::news::domain::NewsError;

/// Estados possíveis de um artigo de notícia conforme o modelo de dados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewsStatus {
    /// Notícia em preparação, sem publicação nem pedido de análise atual.
    Draft,
    /// Proposta submetida e aguardando homologação.
    PendingReview,
    /// Notícia liberada para os feeds autorizados.
    Published,
    /// Versão submetida recusada na análise.
    Rejected,
    /// Notícia retirada da publicação corrente, preservada conforme as regras de histórico.
    Archived,
}

impl NewsStatus {
    /// Retorna a representação textual do estado no padrão snake_case.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingReview => "pending_review",
            Self::Published => "published",
            Self::Rejected => "rejected",
            Self::Archived => "archived",
        }
    }
}

impl fmt::Display for NewsStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NewsStatus {
    type Err = NewsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "pending_review" => Ok(Self::PendingReview),
            "published" => Ok(Self::Published),
            "rejected" => Ok(Self::Rejected),
            "archived" => Ok(Self::Archived),
            other => Err(NewsError::InvalidStatusString(other.to_string())),
        }
    }
}
