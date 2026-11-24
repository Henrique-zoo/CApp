//! Contratos de persistência e integração do módulo de Centros Acadêmicos.
//!
//! Os casos de uso dependem exclusivamente destas traits, sem acoplamento a SQLx,
//! drivers de banco de dados ou detalhes de infraestrutura HTTP.

use std::{error::Error, fmt};

use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::ca::{
    application::error::MandateApplicationError,
    domain::entities::{AcademicCenter, Mandate},
};

/// Erros originados nas operações de persistência de Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademicCenterRepositoryError {
    /// Falha técnica interna no adaptador de persistência.
    Internal(String),
}

impl fmt::Display for AcademicCenterRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "falha no repositório de Centro Acadêmico: {msg}"),
        }
    }
}

impl Error for AcademicCenterRepositoryError {}

/// Erros originados nas operações de persistência de mandato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateRepositoryError {
    /// Falha técnica interna no adaptador de persistência.
    Internal(String),
    /// Conflito de integridade relacional ou unicidade de mandato ativo.
    Conflict(String),
}

impl fmt::Display for MandateRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "falha no repositório de mandatos: {msg}"),
            Self::Conflict(msg) => write!(f, "conflito no repositório de mandatos: {msg}"),
        }
    }
}

impl Error for MandateRepositoryError {}

/// Contrato de persistência para recuperação e consulta de Centros Acadêmicos.
#[async_trait]
pub trait AcademicCenterRepository: Send + Sync {
    /// Busca um Centro Acadêmico pelo seu identificador único.
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<AcademicCenter>, AcademicCenterRepositoryError>;
}

/// Contrato de persistência para armazenamento e consulta de mandatos de gestão.
#[async_trait]
pub trait MandateRepository: Send + Sync {
    /// Persiste um novo mandato de gestão ou atualiza suas informações.
    async fn save(&self, mandate: &Mandate) -> Result<(), MandateRepositoryError>;

    /// Recupera um mandato pelo seu identificador único.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, MandateRepositoryError>;

    /// Recupera o mandato atualmente ativo de um Centro Acadêmico, se houver.
    async fn find_active_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Option<Mandate>, MandateRepositoryError>;

    /// Lista todos os mandatos (histórico) associados a um Centro Acadêmico.
    async fn find_all_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<Mandate>, MandateRepositoryError>;
}

/// Contrato de verificação de autorização de gestão no Centro Acadêmico.
#[async_trait]
pub trait AcademicCenterAuthorizationPort: Send + Sync {
    /// Determina se o usuário possui permissão administrativa no Centro Acadêmico indicado.
    async fn is_user_permitted(
        &self,
        user_id: Uuid,
        ca: &AcademicCenter,
    ) -> Result<bool, MandateApplicationError>;
}

/// Política padrão de autorização: o criador / responsável inicial do CA possui autorização.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultAcademicCenterAuthorization;

#[async_trait]
impl AcademicCenterAuthorizationPort for DefaultAcademicCenterAuthorization {
    async fn is_user_permitted(
        &self,
        user_id: Uuid,
        ca: &AcademicCenter,
    ) -> Result<bool, MandateApplicationError> {
        Ok(ca.created_by_id() == Some(user_id))
    }
}
