//! Erros da camada de aplicação do módulo de demandas.
//!
//! Centraliza as falhas de regras de negócio, autorização de acesso ao fórum,
//! validações de integridade e contratos de persistência.

use std::error::Error;
use std::fmt;
use uuid::Uuid;
use crate::modules::demands::domain::entities::DemandError;
/// Erros decorrentes de operações de persistência de demandas.
#[derive(Debug)]
pub enum DemandRepositoryError {
    /// Falha técnica de banco de dados ou conectividade.
    Internal(String),
}
impl fmt::Display for DemandRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "falha na persistência de demandas: {msg}"),
        }
    }
}
impl Error for DemandRepositoryError {}
/// Erros decorrentes da consulta de vínculo acadêmico ou representação de curso pelo CA.
#[derive(Debug)]
pub enum AffiliationError {
    /// Falha técnica ao consultar dados de vínculo ou Centro Acadêmico.
    Internal(String),
}
impl fmt::Display for AffiliationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => {
                write!(
                    f,
                    "falha ao consultar vínculo acadêmico ou Centro Acadêmico: {msg}"
                )
            }
        }
    }
}
impl Error for AffiliationError {}
/// Falhas de execução dos casos de uso de demandas.
#[derive(Debug)]
pub enum DemandApplicationError {
    /// O Centro Acadêmico indicado não foi encontrado.
    AcademicCenterNotFound(Uuid),
    /// O Centro Acadêmico indicado não está ativo.
    AcademicCenterInactive(Uuid),
    /// A demanda indicada não foi encontrada ou não pertence ao Centro Acadêmico informado.
    DemandNotFound(Uuid),
    /// O autor não possui vínculo estudantil ativo na instituição.
    StudentMembershipRequired,
    /// O autor não tem permissão para atuar no contexto solicitado do Centro Acadêmico.
    UserNotPermittedInCa,
    /// Demandas no fórum institucional exigem a indicação de um curso válido.
    CourseNameRequired,
    /// O curso informado não é representado pelo Centro Acadêmico indicado.
    CourseNotRepresentedByCa {
        /// Identificador do Centro Acadêmico.
        ca_id: Uuid,
        /// Nome do curso não representado.
        course_name: String,
    },
    /// A alteração de status exige justificativa ou parecer explicativo não vazio.
    OpinionRequired,
    /// Violação de regra de negócio ou invariante do domínio de demandas.
    InvalidDemand(DemandError),
    /// Falha de persistência no repositório de demandas.
    Repository(DemandRepositoryError),
    /// Falha na verificação de vínculo acadêmico ou representação do CA.
    Affiliation(AffiliationError),
    /// A demanda informada pertence a outro Centro Acadêmico.
    DemandBelongsToAnotherCa {
        /// Identificador da demanda.
        demand_id: Uuid,
        /// Centro Acadêmico consultado.
        ca_id: Uuid,
    },
    /// O usuário não tem permissão para comentar ou interagir na demanda informada.
    UserNotPermittedToComment,
}
impl fmt::Display for DemandApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AcademicCenterNotFound(id) => {
                write!(f, "Centro Acadêmico '{id}' não encontrado")
            }
            Self::AcademicCenterInactive(id) => {
                write!(f, "Centro Acadêmico '{id}' não está ativo")
            }
            Self::DemandNotFound(id) => {
                write!(f, "demanda '{id}' não encontrada")
            }
            Self::StudentMembershipRequired => {
                write!(
                    f,
                    "vínculo estudantil ativo é obrigatório para registrar demandas"
                )
            }
            Self::UserNotPermittedInCa => {
                write!(
                    f,
                    "usuário sem permissão para atuar no fórum do Centro Acadêmico informado"
                )
            }
            Self::CourseNameRequired => {
                write!(
                    f,
                    "demandas do fórum institucional exigem associação válida a um curso"
                )
            }
            Self::CourseNotRepresentedByCa { ca_id, course_name } => {
                write!(
                    f,
                    "o curso '{course_name}' não é representado pelo Centro Acadêmico '{ca_id}'"
                )
            }
            Self::OpinionRequired => {
                write!(
                    f,
                    "parecer ou justificativa é obrigatório para alterar o status da demanda"
                )
            }
            Self::InvalidDemand(err) => write!(f, "{err}"),
            Self::Repository(err) => write!(f, "{err}"),
            Self::Affiliation(err) => write!(f, "{err}"),
            Self::DemandBelongsToAnotherCa { demand_id, ca_id } => {
                write!(
                    f,
                    "a demanda '{demand_id}' pertence a outro Centro Acadêmico (não pertence a '{ca_id}')"
                )
            }
            Self::UserNotPermittedToComment => {
                write!(
                    f,
                    "usuário sem permissão para interagir ou visualizar comentários desta demanda"
                )
            }
        }
    }
}
impl Error for DemandApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDemand(err) => Some(err),
            Self::Repository(err) => Some(err),
            Self::Affiliation(err) => Some(err),
            _ => None,
        }
    }
}
impl From<DemandError> for DemandApplicationError {
    fn from(err: DemandError) -> Self {
        Self::InvalidDemand(err)
    }
}
impl From<DemandRepositoryError> for DemandApplicationError {
    fn from(err: DemandRepositoryError) -> Self {
        Self::Repository(err)
    }
}
impl From<AffiliationError> for DemandApplicationError {
    fn from(err: AffiliationError) -> Self {
        Self::Affiliation(err)
    }
}
