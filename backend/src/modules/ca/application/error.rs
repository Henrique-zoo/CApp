//! Erros da camada de aplicação do módulo de Centros Acadêmicos.
//!
//! Representa as falhas de regras de orquestração, autorização,
//! inexistência de recursos e inconsistências temporais.

use crate::modules::ca::application::ports::AcademicCenterRepositoryError;
use crate::modules::ca::application::ports::MandateRepositoryError;
use crate::modules::ca::application::ports::PollRepositoryError;
use crate::modules::ca::domain::entities::MandateError;
use crate::modules::ca::domain::entities::PollError;
use chrono::DateTime;
use chrono::Utc;
use std::error::Error;
use std::fmt;
use uuid::Uuid;
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

    /// O mandato informado não foi encontrado.
    MandateNotFound(Uuid),

    /// O integrante informado não foi encontrado na gestão.
    MemberNotFound(Uuid),

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
                    "não existe gestão vigente registrada para o Centro Acadêmico '{id}'"
                )
            }
            Self::MandateNotFound(id) => {
                write!(f, "mandato '{id}' não encontrado")
            }
            Self::MemberNotFound(id) => {
                write!(f, "integrante '{id}' não encontrado na gestão")
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
/// Erros emitidos pelos casos de uso de consultas e enquetes estudantis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollApplicationError {
    /// O Centro Acadêmico informado não foi encontrado.
    AcademicCenterNotFound(Uuid),

    /// O Centro Acadêmico encontra-se arquivado ou inativo.
    AcademicCenterInactive(Uuid),

    /// A consulta estudantil informada não foi encontrada.
    PollNotFound(Uuid),

    /// O usuário não possui permissão de gestão no Centro Acadêmico.
    UserNotPermittedInCa {
        /// Identificador do usuário solicitante.
        user_id: Uuid,
        /// Identificador do Centro Acadêmico.
        academic_center_id: Uuid,
    },

    /// O estudante não é elegível para participar desta consulta.
    StudentNotEligible {
        /// Identificador do estudante.
        user_id: Uuid,
        /// Identificador da consulta.
        poll_id: Uuid,
    },

    /// O estudante já registrou seu voto nesta consulta.
    DuplicateVote {
        /// Identificador do estudante.
        user_id: Uuid,
        /// Identificador da consulta.
        poll_id: Uuid,
    },

    /// O período de votação ainda não foi iniciado.
    VotingNotStarted {
        /// Data/hora de início da votação.
        starts_at: DateTime<Utc>,
    },

    /// O período de votação já terminou.
    VotingEnded {
        /// Data/hora de término da votação.
        ends_at: DateTime<Utc>,
    },

    /// A consulta estudantil está encerrada.
    PollClosed,

    /// A opção de voto indicada não foi encontrada na consulta.
    OptionNotFound(Uuid),

    /// As opções informadas para a consulta são inválidas.
    InvalidPollOptions(String),

    /// As datas informadas para a consulta são inválidas.
    InvalidPollDates(String),

    /// Violação de regra de negócio do domínio.
    Domain(PollError),

    /// Falha técnica de persistência ou repositório.
    Repository(String),
}
impl fmt::Display for PollApplicationError {
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
            Self::PollNotFound(id) => {
                write!(f, "consulta estudantil '{id}' não encontrada")
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
            Self::StudentNotEligible { user_id, poll_id } => {
                write!(
                    f,
                    "estudante '{user_id}' não é elegível para votar na consulta '{poll_id}'"
                )
            }
            Self::DuplicateVote { user_id, poll_id } => {
                write!(
                    f,
                    "estudante '{user_id}' já registrou voto na consulta '{poll_id}'"
                )
            }
            Self::VotingNotStarted { starts_at } => {
                write!(
                    f,
                    "o período de votação desta consulta ainda não começou (início em: {starts_at})"
                )
            }
            Self::VotingEnded { ends_at } => {
                write!(
                    f,
                    "o período de votação desta consulta já terminou (término em: {ends_at})"
                )
            }
            Self::PollClosed => write!(f, "esta consulta estudantil encontra-se encerrada"),
            Self::OptionNotFound(id) => {
                write!(f, "opção de voto '{id}' não encontrada nesta consulta")
            }
            Self::InvalidPollOptions(msg) => {
                write!(f, "opções de enquete inválidas: {msg}")
            }
            Self::InvalidPollDates(msg) => {
                write!(f, "período de votação inválido: {msg}")
            }
            Self::Domain(err) => write!(f, "violação de regra de negócio: {err}"),
            Self::Repository(msg) => write!(f, "erro de persistência: {msg}"),
        }
    }
}
impl Error for PollApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Domain(err) => Some(err),
            _ => None,
        }
    }
}
impl From<PollError> for PollApplicationError {
    fn from(err: PollError) -> Self {
        match err {
            PollError::PollClosed => Self::PollClosed,
            PollError::VotingNotStarted { starts_at } => Self::VotingNotStarted { starts_at },
            PollError::VotingEnded { ends_at } => Self::VotingEnded { ends_at },
            PollError::DuplicateVote { poll_id, user_id } => {
                Self::DuplicateVote { poll_id, user_id }
            }
            PollError::OptionNotFound(id) => Self::OptionNotFound(id),
            PollError::InvalidPeriod { starts_at, ends_at } => Self::InvalidPollDates(format!(
                "início ({starts_at}) deve preceder encerramento ({ends_at})"
            )),
            PollError::TooFewOptions { min, found } => Self::InvalidPollOptions(format!(
                "mínimo de {min} opções exigidas, encontradas {found}"
            )),
            PollError::DuplicateOptionTitles(opt) => {
                Self::InvalidPollOptions(format!("opção duplicada: '{opt}'"))
            }
            other => Self::Domain(other),
        }
    }
}
impl From<AcademicCenterRepositoryError> for PollApplicationError {
    fn from(err: AcademicCenterRepositoryError) -> Self {
        Self::Repository(err.to_string())
    }
}
impl From<PollRepositoryError> for PollApplicationError {
    fn from(err: PollRepositoryError) -> Self {
        match err {
            PollRepositoryError::Conflict(msg) => Self::InvalidPollDates(msg),
            PollRepositoryError::NotFound(msg) => Self::Repository(msg),
            PollRepositoryError::Internal(msg) => Self::Repository(msg),
        }
    }
}
