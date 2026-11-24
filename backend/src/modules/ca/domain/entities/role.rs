//! Entidade de domínio para cargos e perfis administrativos de um Centro Acadêmico.
//!
//! Cada cargo pertence com exclusividade a um `academic_center_id`.
//! O cargo de Presidente é irrevogável e obrigatório em conformidade com o Relatório
//! de Features § 2.3 e com os cenários de especificação BDD da UnB.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::permission::Permission;

/// Erros de validação e integridade de cargos administrativos.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleError {
    /// O identificador do cargo não pode ser o UUID nulo.
    #[error("O identificador do cargo não pode ser nulo.")]
    NilRoleId,

    /// O identificador do Centro Acadêmico não pode ser o UUID nulo.
    #[error("O identificador do Centro Acadêmico não pode ser nulo.")]
    NilAcademicCenterId,

    /// O nome do cargo não pode ser vazio ou composto exclusivamente de espaços.
    #[error("O nome do cargo não pode ser vazio.")]
    EmptyRoleName,

    /// Tentativa de exclusão ou alteração de integridade proibida no cargo de Presidente.
    #[error("O cargo de Presidente é obrigatório e não pode ser excluído.")]
    PresidentRoleCannotBeDeleted,
}

/// Representa um cargo ou perfil administrativo no âmbito de um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    /// Identificador único do cargo.
    pub id: Uuid,

    /// Identificador do Centro Acadêmico proprietário deste cargo.
    pub academic_center_id: Uuid,

    /// Nome oficial do cargo (ex: "Presidente", "Diretor de Comunicação").
    pub name: String,

    /// Descrição sumária das atribuições do cargo.
    pub description: Option<String>,

    /// Indica se é um dos cargos padrão criados automaticamente no cadastro do CA.
    pub is_standard: bool,

    /// Indica se o cargo é obrigatório para a existência da gestão (ex: Presidente).
    pub is_mandatory: bool,

    /// Conjunto de permissões ativas diretamente atribuídas ao cargo.
    pub permissions: HashSet<Permission>,

    /// Data e hora de criação do cargo no sistema.
    pub created_at: DateTime<Utc>,

    /// Data e hora da última atualização de suas permissões ou atributos.
    pub updated_at: DateTime<Utc>,
}

impl Role {
    /// Cria uma nova instância de cargo administrativo após validar invariantes fundamentais.
    ///
    /// # Erros
    ///
    /// Retorna [`RoleError::NilRoleId`] se `id` for nulo,
    /// [`RoleError::NilAcademicCenterId`] se `academic_center_id` for nulo, ou
    /// [`RoleError::EmptyRoleName`] se `name` estiver em branco.
    pub fn new(
        id: Uuid,
        academic_center_id: Uuid,
        name: impl Into<String>,
        permissions: HashSet<Permission>,
    ) -> Result<Self, RoleError> {
        let name = name.into();
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(RoleError::EmptyRoleName);
        }
        if id.is_nil() {
            return Err(RoleError::NilRoleId);
        }
        if academic_center_id.is_nil() {
            return Err(RoleError::NilAcademicCenterId);
        }

        let now = Utc::now();
        Ok(Self {
            id,
            academic_center_id,
            name: trimmed_name.to_string(),
            description: None,
            is_standard: false,
            is_mandatory: false,
            permissions,
            created_at: now,
            updated_at: now,
        })
    }

    /// Constrói o cargo padrão e obrigatório de "Presidente" para o CA informado.
    ///
    /// O cargo de Presidente nasce com poderes administrativos amplos sobre a gestão.
    pub fn presidente(academic_center_id: Uuid) -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::ManageMembers);
        permissions.insert(Permission::RequestCaVerification);
        permissions.insert(Permission::PublishNewsDirect);
        permissions.insert(Permission::ApproveNews);
        permissions.insert(Permission::ShareNews);
        permissions.insert(Permission::PublishEventsDirect);
        permissions.insert(Permission::ApproveEvents);
        permissions.insert(Permission::ManageDemands);
        permissions.insert(Permission::ManageMaterials);
        permissions.insert(Permission::ManageElectoralCommission);
        permissions.insert(Permission::ManageFaq);
        permissions.insert(Permission::ManageMentorship);
        permissions.insert(Permission::PublishFinancialStatements);
        permissions.insert(Permission::CreateStudentPolls);
        permissions.insert(Permission::RegisterStudentStrikes);
        permissions.insert(Permission::RegisterCaStrikes);
        permissions.insert(Permission::ConfigureStrikeRules);

        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            academic_center_id,
            name: "Presidente".to_string(),
            description: Some(
                "Representante legal e administrador geral da gestão do Centro Acadêmico."
                    .to_string(),
            ),
            is_standard: true,
            is_mandatory: true,
            permissions,
            created_at: now,
            updated_at: now,
        }
    }

    /// Constrói o cargo padrão de "Vice-Presidente" para o CA informado.
    pub fn vice_presidente(academic_center_id: Uuid) -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::PublishNewsDirect);
        permissions.insert(Permission::PublishEventsDirect);
        permissions.insert(Permission::ManageDemands);
        permissions.insert(Permission::ManageMaterials);
        permissions.insert(Permission::ManageFaq);

        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            academic_center_id,
            name: "Vice-Presidente".to_string(),
            description: Some("Co-gestor auxiliar da administração do CA.".to_string()),
            is_standard: true,
            is_mandatory: false,
            permissions,
            created_at: now,
            updated_at: now,
        }
    }

    /// Constrói o cargo padrão de "Tesoureiro" para o CA informado.
    pub fn tesoureiro(academic_center_id: Uuid) -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::PublishFinancialStatements);

        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            academic_center_id,
            name: "Tesoureiro".to_string(),
            description: Some(
                "Responsável pela gestão financeira e prestação de contas do CA.".to_string(),
            ),
            is_standard: true,
            is_mandatory: false,
            permissions,
            created_at: now,
            updated_at: now,
        }
    }

    /// Constrói um cargo de "Diretor de Comunicação" com poderes de homologação e publicação de notícias.
    pub fn diretor_comunicacao(academic_center_id: Uuid) -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::ApproveNews);
        permissions.insert(Permission::PublishNewsDirect);
        permissions.insert(Permission::ShareNews);

        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            academic_center_id,
            name: "Diretor de Comunicação".to_string(),
            description: Some(
                "Responsável editorial e publicação de comunicados oficiais do CA.".to_string(),
            ),
            is_standard: false,
            is_mandatory: false,
            permissions,
            created_at: now,
            updated_at: now,
        }
    }

    /// Constrói um cargo de "Diretor de Eventos" com permissão de publicação e gestão direta de eventos.
    pub fn diretor_eventos(academic_center_id: Uuid) -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::PublishEventsDirect);

        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            academic_center_id,
            name: "Diretor de Eventos".to_string(),
            description: Some(
                "Organizador e gestor de eventos acadêmicos e extracurriculares.".to_string(),
            ),
            is_standard: false,
            is_mandatory: false,
            permissions,
            created_at: now,
            updated_at: now,
        }
    }

    /// Avalia se este cargo confere a permissão solicitada, considerando implicações editoriais.
    pub fn has_permission(&self, permission: &Permission) -> bool {
        permission.is_satisfied_by_any(&self.permissions)
    }

    /// Adiciona uma permissão ao cargo, atualizando o timestamp de modificação.
    pub fn add_permission(&mut self, permission: Permission) {
        if self.permissions.insert(permission) {
            self.updated_at = Utc::now();
        }
    }

    /// Remove uma permissão do cargo, atualizando o timestamp de modificação.
    pub fn remove_permission(&mut self, permission: &Permission) {
        if self.permissions.remove(permission) {
            self.updated_at = Utc::now();
        }
    }

    /// Retorna o conjunto expandido de todas as permissões efetivas concedidas por este cargo.
    pub fn effective_permissions(&self) -> HashSet<Permission> {
        let mut set = self.permissions.clone();
        for p in &self.permissions {
            for implied in p.implied_permissions() {
                set.insert(implied);
            }
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_CA_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);

    #[test]
    fn should_create_valid_custom_role() {
        let role = Role::new(
            Uuid::new_v4(),
            DUMMY_CA_ID,
            "Coordenador de Apoio",
            HashSet::from([Permission::ManageFaq]),
        )
        .expect("cargo válido");

        assert_eq!(role.name, "Coordenador de Apoio");
        assert!(role.has_permission(&Permission::ManageFaq));
        assert!(!role.has_permission(&Permission::ManageMembers));
    }

    #[test]
    fn should_reject_role_with_empty_name_or_nil_ids() {
        assert_eq!(
            Role::new(Uuid::new_v4(), DUMMY_CA_ID, "   ", HashSet::new()).unwrap_err(),
            RoleError::EmptyRoleName
        );
        assert_eq!(
            Role::new(Uuid::nil(), DUMMY_CA_ID, "Válido", HashSet::new()).unwrap_err(),
            RoleError::NilRoleId
        );
        assert_eq!(
            Role::new(Uuid::new_v4(), Uuid::nil(), "Válido", HashSet::new()).unwrap_err(),
            RoleError::NilAcademicCenterId
        );
    }

    #[test]
    fn should_create_standard_presidente_with_mandatory_invariants() {
        let pres = Role::presidente(DUMMY_CA_ID);
        assert_eq!(pres.name, "Presidente");
        assert!(pres.is_standard);
        assert!(pres.is_mandatory);
        assert!(pres.has_permission(&Permission::ManageMembers));
        assert!(pres.has_permission(&Permission::ApproveNews));
        // Implicação automática de homologação -> publicação direta
        assert!(pres.has_permission(&Permission::PublishNewsDirect));
    }
}
