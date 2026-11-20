//! Configuração institucional consultada a partir do tenant Microsoft.

use std::{error::Error, fmt};
use uuid::Uuid;

use crate::modules::identity::domain::{
    CatalogStatus,
    rules::institution::{validate_institution_acronym, validate_institution_name},
};

/// Erros de validação e criação de instituição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstitutionError {
    /// O nome da instituição não pode ser vazio ou em branco.
    EmptyName,
    /// A sigla da instituição não pode ser vazia ou em branco quando informada.
    EmptyAcronym,
}

impl fmt::Display for InstitutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "o nome da instituição não pode ser vazio"),
            Self::EmptyAcronym => write!(f, "a sigla da instituição não pode ser vazia"),
        }
    }
}

impl Error for InstitutionError {}

/// Instituição cadastrada e configuração de verificação de estudantes.
///
/// Corresponde à tabela `institution`; o tenant possui unicidade no PostgreSQL.
/// O grupo é um identificador externo e sua correspondência com o tenant deve ser
/// garantida por quem configura a instituição. Não contém credenciais Microsoft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Institution {
    /// Identificador interno da instituição no CApp.
    id: Uuid,
    /// Nome público da instituição.
    name: String,
    /// Sigla oficial da instituição.
    acronym: Option<String>,
    /// UUID do diretório Microsoft; identifica unicamente a instituição cadastrada.
    microsoft_tenant_id: Uuid,
    /// UUID do grupo cuja participação comprova a elegibilidade discente exigida.
    microsoft_student_group_id: Uuid,
    /// Disponibilidade no catálogo institucional.
    status: CatalogStatus,
}

impl Institution {
    /// Constrói uma nova instituição aplicando validações de domínio.
    pub fn new(
        id: Uuid,
        name: String,
        acronym: Option<String>,
        microsoft_tenant_id: Uuid,
        microsoft_student_group_id: Uuid,
        status: CatalogStatus,
    ) -> Result<Self, InstitutionError> {
        let validated_name = validate_institution_name(&name)?;
        let validated_acronym = validate_institution_acronym(acronym.as_deref())?;

        Ok(Self {
            id,
            name: validated_name,
            acronym: validated_acronym,
            microsoft_tenant_id,
            microsoft_student_group_id,
            status,
        })
    }

    /// Reconstrói uma instituição persistida no banco.
    pub fn reconstitute(
        id: Uuid,
        name: String,
        acronym: Option<String>,
        microsoft_tenant_id: Uuid,
        microsoft_student_group_id: Uuid,
        status: CatalogStatus,
    ) -> Result<Self, InstitutionError> {
        Self::new(
            id,
            name,
            acronym,
            microsoft_tenant_id,
            microsoft_student_group_id,
            status,
        )
    }

    /// Identificador interno da instituição no CApp.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Nome público da instituição.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sigla oficial cadastrada da instituição.
    pub fn acronym(&self) -> Option<&str> {
        self.acronym.as_deref()
    }

    /// UUID do diretório/tenant Microsoft da instituição.
    pub fn microsoft_tenant_id(&self) -> Uuid {
        self.microsoft_tenant_id
    }

    /// UUID do grupo institucional de discentes.
    pub fn microsoft_student_group_id(&self) -> Uuid {
        self.microsoft_student_group_id
    }

    /// Situação no catálogo de instituições.
    pub fn status(&self) -> CatalogStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_institution() {
        let inst = Institution::new(
            Uuid::new_v4(),
            "  Universidade de Brasília  ".into(),
            Some("  UnB  ".into()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            CatalogStatus::Active,
        )
        .expect("should create valid institution");

        assert_eq!(inst.name(), "Universidade de Brasília");
        assert_eq!(inst.acronym(), Some("UnB"));
        assert_eq!(inst.status(), CatalogStatus::Active);
    }

    #[test]
    fn should_reject_empty_name() {
        let result = Institution::new(
            Uuid::new_v4(),
            "   ".into(),
            None,
            Uuid::new_v4(),
            Uuid::new_v4(),
            CatalogStatus::Active,
        );

        assert_eq!(result.unwrap_err(), InstitutionError::EmptyName);
    }

    #[test]
    fn should_reject_empty_acronym() {
        let result = Institution::new(
            Uuid::new_v4(),
            "Universidade de Brasília".into(),
            Some("   ".into()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            CatalogStatus::Active,
        );

        assert_eq!(result.unwrap_err(), InstitutionError::EmptyAcronym);
    }
}
