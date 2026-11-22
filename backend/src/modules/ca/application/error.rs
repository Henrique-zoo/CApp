//! Erros da camada de aplicação do módulo de Centros Acadêmicos.
//!
//! Representa as falhas de regras de orquestração, autorização,
//! inexistência de recursos e inconsistências temporais.

use std::{error::Error, fmt};

use uuid::Uuid;

use crate::modules::ca::{
    application::ports::{AcademicCenterRepositoryError, MandateRepositoryError},
    domain::entities::MandateError,
};

/// Erros emitidos pelos casos de uso de mandato do Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateApplicationError {
    /// O Centro Acadêmico informado não foi encontrado.
    AcademicCenterNotFound(Uuid),

    /// O Centro Acadêmico encontra-se arquivado ou inativo.
    AcademicCenterInactive(Uuid),

    /// O usuário não possui permissão administrativa no respectivo Centro Acadêmico.
    UserNotPermittedInCa {
        /// Identificador do usuário solicitante.
        user_id: Uuid,
        /// Identificador do Centro Acadêmico.
        academic_center_id: Uuid,
    },

    /// As datas fornecidas para o mandato violam a coerência temporal.
    InvalidMandateDates(String),

    /// Já existe um mandato ativo registrado para o respectivo Centro Acadêmico.
    ActiveMandateAlreadyExists(Uuid),

    /// Não foi encontrado mandato ativo para o Centro Acadêmico.
    ActiveMandateNotFound(Uuid),

    /// Violação de invariante de negócio do domínio.
    Domain(MandateError),

    /// Falha técnica de persistência ou repositório.
    Repository(String),
}

impl fmt::Display for MandateApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AcademicCenterNotFound(id) => {
                write!(f, "Centro Acadêmico '{id}' não encontrado")
            }
            Self::AcademicCenterInactive(id) => {
                write!(
                    f,
                    "Centro Acadêmico '{id}' encontra-se inativo ou arquivado"
                )
            }
            Self::UserNotPermittedInCa {
                user_id,
                academic_center_id,
            } => {
                write!(
                    f,
                    "usuário '{user_id}' não possui permissão de gestão no Centro Acadêmico '{academic_center_id}'"
                )
            }
            Self::InvalidMandateDates(msg) => {
                write!(f, "datas de mandato inválidas: {msg}")
            }
            Self::ActiveMandateAlreadyExists(id) => {
                write!(
                    f,
                    "já existe mandato ativo configurado para o Centro Acadêmico '{id}'"
                )
            }
            Self::ActiveMandateNotFound(id) => {
                write!(
                    f,
                    "não existe mandato ativo registrado para o Centro Acadêmico '{id}'"
                )
            }
            Self::Domain(err) => write!(f, "violação de regra de negócio: {err}"),
            Self::Repository(msg) => write!(f, "erro de persistência: {msg}"),
        }
    }
}

impl Error for MandateApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Domain(err) => Some(err),
            _ => None,
        }
    }
}

impl From<MandateError> for MandateApplicationError {
    fn from(err: MandateError) -> Self {
        Self::Domain(err)
    }
}

impl From<AcademicCenterRepositoryError> for MandateApplicationError {
    fn from(err: AcademicCenterRepositoryError) -> Self {
        Self::Repository(err.to_string())
    }
}

impl From<MandateRepositoryError> for MandateApplicationError {
    fn from(err: MandateRepositoryError) -> Self {
        match err {
            MandateRepositoryError::Conflict(msg) => Self::InvalidMandateDates(msg),
            MandateRepositoryError::Internal(msg) => Self::Repository(msg),
        }
    }
}
