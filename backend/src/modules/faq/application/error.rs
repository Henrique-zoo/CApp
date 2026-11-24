//! Erros da camada de aplicação do módulo de perguntas frequentes (FAQ).

use std::{error::Error, fmt};

use uuid::Uuid;

use crate::modules::faq::domain::entities::faq_entry::FaqError;

/// Falhas técnicas originadas no adaptador de persistência de FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaqRepositoryError {
    /// Erro de acesso ao banco de dados ou execução SQL.
    Database(String),
}

impl fmt::Display for FaqRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "falha de persistência no repositório de FAQ: {msg}"),
        }
    }
}

impl Error for FaqRepositoryError {}

/// Erros emitidos pelos casos de uso do módulo de FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaqApplicationError {
    /// O Centro Acadêmico informado não foi encontrado.
    AcademicCenterNotFound(Uuid),

    /// O Centro Acadêmico informado não está ativo.
    AcademicCenterInactive(Uuid),

    /// O usuário não possui cargo ativo de gestão no Centro Acadêmico especificado.
    UserNotPermittedInCa,

    /// Violação de invariantes e validações da entidade de domínio.
    InvalidEntry(FaqError),

    /// Falha técnica no repositório de dados.
    Repository(FaqRepositoryError),

    /// Erro interno de processamento.
    Internal(String),
}

impl fmt::Display for FaqApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AcademicCenterNotFound(id) => {
                write!(f, "Centro Acadêmico '{id}' não encontrado")
            }
            Self::AcademicCenterInactive(id) => {
                write!(f, "Centro Acadêmico '{id}' não está ativo")
            }
            Self::UserNotPermittedInCa => {
                write!(
                    f,
                    "usuário sem permissão para cadastrar FAQ neste Centro Acadêmico"
                )
            }
            Self::InvalidEntry(err) => write!(f, "{err}"),
            Self::Repository(err) => write!(f, "{err}"),
            Self::Internal(msg) => write!(f, "erro interno: {msg}"),
        }
    }
}

impl Error for FaqApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidEntry(err) => Some(err),
            Self::Repository(err) => Some(err),
            _ => None,
        }
    }
}

impl From<FaqError> for FaqApplicationError {
    fn from(err: FaqError) -> Self {
        Self::InvalidEntry(err)
    }
}

impl From<FaqRepositoryError> for FaqApplicationError {
    fn from(err: FaqRepositoryError) -> Self {
        Self::Repository(err)
    }
}
