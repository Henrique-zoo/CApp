//! Entidade de domínio para vinculação e histórico de integrantes a cargos de gestão.
//!
//! A vinculação concede permissões ativas no contexto específico de um CA.
//! A desvinculação revoga imediatamente as permissões sem excluir o registro do histórico
//! administrativo, em estrita consonância com o Relatório de Features § 2.5 e § 2.6.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Erros de validação e integridade da vinculação de integrantes a cargos.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleAssignmentError {
    /// O identificador do usuário não pode ser nulo.
    #[error("O identificador do usuário não pode ser nulo.")]
    NilUserId,

    /// O identificador do Centro Acadêmico não pode ser nulo.
    #[error("O identificador do Centro Acadêmico não pode ser nulo.")]
    NilAcademicCenterId,

    /// O identificador do cargo não pode ser nulo.
    #[error("O identificador do cargo não pode ser nulo.")]
    NilRoleId,

    /// O nome do cargo não pode ser vazio.
    #[error("O nome do cargo não pode ser vazio.")]
    EmptyRoleName,

    /// Tentativa de revogação em uma vinculação que já se encontra inativa ou revogada.
    #[error("A vinculação já se encontra revogada ou inativa.")]
    AssignmentAlreadyRevoked,
}

/// Registro formal de vinculação de um usuário a um cargo em um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleAssignment {
    /// Identificador único da atribuição/registro histórico.
    pub id: Uuid,

    /// Identificador do usuário vinculado.
    pub user_id: Uuid,

    /// Identificador do Centro Acadêmico no qual o cargo é exercido.
    pub academic_center_id: Uuid,

    /// Identificador do cargo administrativo ocupado.
    pub role_id: Uuid,

    /// Nome descritivo do cargo à época da vinculação.
    pub role_name: String,

    /// Situação da vinculação: `true` se ativa (concedendo permissões); `false` se revogada.
    pub is_active: bool,

    /// Data e hora da posse ou vinculação inicial.
    pub assigned_at: DateTime<Utc>,

    /// Data e hora em que ocorreu a desvinculação formal, se houver.
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RoleAssignment {
    /// Cria um novo registro ativo de vinculação de integrante a um cargo.
    ///
    /// # Erros
    ///
    /// Retorna erro se qualquer identificador for nulo ou se o nome do cargo estiver vazio.
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        academic_center_id: Uuid,
        role_id: Uuid,
        role_name: impl Into<String>,
    ) -> Result<Self, RoleAssignmentError> {
        let role_name = role_name.into();
        let trimmed_name = role_name.trim();
        if trimmed_name.is_empty() {
            return Err(RoleAssignmentError::EmptyRoleName);
        }
        if user_id.is_nil() {
            return Err(RoleAssignmentError::NilUserId);
        }
        if academic_center_id.is_nil() {
            return Err(RoleAssignmentError::NilAcademicCenterId);
        }
        if role_id.is_nil() {
            return Err(RoleAssignmentError::NilRoleId);
        }

        Ok(Self {
            id,
            user_id,
            academic_center_id,
            role_id,
            role_name: trimmed_name.to_string(),
            is_active: true,
            assigned_at: Utc::now(),
            revoked_at: None,
        })
    }

    /// Revoga imediatamente a vinculação ativa, registrando o timestamp de término sem apagar o histórico.
    ///
    /// # Erros
    ///
    /// Retorna [`RoleAssignmentError::AssignmentAlreadyRevoked`] se a vinculação já estiver inativa.
    pub fn revoke(&mut self, revoked_at: DateTime<Utc>) -> Result<(), RoleAssignmentError> {
        if !self.is_active {
            return Err(RoleAssignmentError::AssignmentAlreadyRevoked);
        }
        self.is_active = false;
        self.revoked_at = Some(revoked_at);
        Ok(())
    }

    /// Verifica se a vinculação confere permissões ativas no momento presente.
    pub fn is_active(&self) -> bool {
        self.is_active && self.revoked_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
    const DUMMY_CA_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const DUMMY_ROLE_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0099);

    #[test]
    fn should_create_active_assignment_and_revoke_successfully() {
        let mut assignment = RoleAssignment::new(
            Uuid::new_v4(),
            DUMMY_USER_ID,
            DUMMY_CA_ID,
            DUMMY_ROLE_ID,
            "Presidente",
        )
        .expect("vinculação válida");

        assert!(assignment.is_active());
        assert_eq!(assignment.role_name, "Presidente");
        assert!(assignment.revoked_at.is_none());

        let now = Utc::now();
        assignment.revoke(now).expect("revogação com sucesso");

        assert!(!assignment.is_active());
        assert_eq!(assignment.revoked_at, Some(now));

        // Segunda tentativa de revogar deve falhar
        assert_eq!(
            assignment.revoke(now).unwrap_err(),
            RoleAssignmentError::AssignmentAlreadyRevoked
        );
    }
}
