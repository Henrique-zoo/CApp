//! Regras de negócio para avaliação de autorizações e isolamento por Centro Acadêmico.
//!
//! Implementa os algoritmos de resolução de permissões e as invariantes estritas
//! do Relatório de Features § 2.4:
//! 1. Isolamento estrito por Centro Acadêmico.
//! 2. Implicação automática de homologação para publicação direta no mesmo domínio.
//! 3. Proibição de homologação de propostas alheias por detentores de apenas publicação direta.
//! 4. Revogação com preservação de histórico.

use std::collections::HashSet;

use thiserror::Error;
use uuid::Uuid;

use crate::modules::ca::domain::entities::{
    permission::Permission, role::Role, role_assignment::RoleAssignment,
};

/// Erros decorrentes da violação de regras de autorização de domínio.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorizationRuleError {
    /// O cargo pertence a um Centro Acadêmico divergente do recurso requisitado.
    #[error(
        "Isolamento violado: o cargo pertence ao CA '{role_ca_id}', mas o recurso solicitado pertence ao CA '{target_ca_id}'."
    )]
    AcademicCenterMismatch {
        /// CA ao qual o cargo do usuário está vinculado.
        role_ca_id: Uuid,
        /// CA do recurso alvo da operação.
        target_ca_id: Uuid,
    },

    /// O usuário não possui a permissão requerida no Centro Acadêmico alvo.
    #[error("Permissão '{required}' não concedida no contexto do Centro Acadêmico '{ca_id}'.")]
    PermissionDenied {
        /// Permissão que era exigida para a operação.
        required: Permission,
        /// Identificador do CA onde a ação foi tentada.
        ca_id: Uuid,
    },

    /// O usuário não possui cargos ativos no Centro Acadêmico informado.
    #[error(
        "O usuário '{user_id}' não possui nenhum cargo ativo na gestão do Centro Acadêmico '{ca_id}'."
    )]
    NoActiveRolesInAcademicCenter {
        /// Identificador do usuário.
        user_id: Uuid,
        /// Identificador do CA.
        ca_id: Uuid,
    },
}

/// Agrega o conjunto completo de permissões efetivas a partir dos cargos do usuário.
///
/// Expande automaticamente permissões acessórias, como a regra que concede
/// publicação direta a quem possui homologação editorial (§ 2.4).
pub fn evaluate_effective_permissions(roles: &[Role]) -> HashSet<Permission> {
    let mut effective = HashSet::new();
    for role in roles {
        for perm in &role.permissions {
            effective.insert(*perm);
            for implied in perm.implied_permissions() {
                effective.insert(implied);
            }
        }
    }
    effective
}

/// Avalia se a lista de cargos ativos do usuário confere a permissão requerida no CA alvo.
///
/// # Regras Invariantes
/// 1. Somente cargos com `academic_center_id == target_ca_id` são considerados.
/// 2. As permissões de cargos pertencentes a outros CAs são estritamente ignoradas.
/// 3. Avalia se a permissão requerida está no conjunto de permissões efetivas consolidadas.
pub fn is_authorized_in_ca(
    roles: &[Role],
    target_ca_id: Uuid,
    required_permission: &Permission,
) -> bool {
    let relevant_roles: Vec<Role> = roles
        .iter()
        .filter(|r| r.academic_center_id == target_ca_id)
        .cloned()
        .collect();

    if relevant_roles.is_empty() {
        return false;
    }

    let effective_permissions = evaluate_effective_permissions(&relevant_roles);
    required_permission.is_satisfied_by_any(&effective_permissions)
}

/// Valida rigorosamente a autorização, retornando [`AuthorizationRuleError`] em caso de recusa.
pub fn check_authorization_in_ca(
    user_id: Uuid,
    roles: &[Role],
    target_ca_id: Uuid,
    required_permission: &Permission,
) -> Result<(), AuthorizationRuleError> {
    let relevant_roles: Vec<&Role> = roles
        .iter()
        .filter(|r| r.academic_center_id == target_ca_id)
        .collect();

    if relevant_roles.is_empty() {
        // Se o usuário possui cargos em outro CA, especifica o isolamento
        if let Some(other_role) = roles.first() {
            return Err(AuthorizationRuleError::AcademicCenterMismatch {
                role_ca_id: other_role.academic_center_id,
                target_ca_id,
            });
        }
        return Err(AuthorizationRuleError::NoActiveRolesInAcademicCenter {
            user_id,
            ca_id: target_ca_id,
        });
    }

    let cloned_roles: Vec<Role> = relevant_roles.into_iter().cloned().collect();
    let effective = evaluate_effective_permissions(&cloned_roles);

    if required_permission.is_satisfied_by_any(&effective) {
        Ok(())
    } else {
        Err(AuthorizationRuleError::PermissionDenied {
            required: *required_permission,
            ca_id: target_ca_id,
        })
    }
}

/// Valida se um conjunto de vinculações ativas confere a autorização ao usuário.
pub fn filter_active_assignments(assignments: &[RoleAssignment]) -> Vec<&RoleAssignment> {
    assignments.iter().filter(|a| a.is_active()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);
    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[test]
    fn should_isolate_permissions_between_different_academic_centers() {
        // Usuário é Presidente em CAER
        let caer_pres = Role::presidente(CAER_ID);
        let roles = vec![caer_pres];

        // Tem autorização para administrar membros no CAER
        assert!(is_authorized_in_ca(
            &roles,
            CAER_ID,
            &Permission::ManageMembers
        ));

        // NÃO tem autorização para administrar membros no CACC (isolamento)
        assert!(!is_authorized_in_ca(
            &roles,
            CACC_ID,
            &Permission::ManageMembers
        ));

        assert!(matches!(
            check_authorization_in_ca(USER_ID, &roles, CACC_ID, &Permission::ManageMembers),
            Err(AuthorizationRuleError::AcademicCenterMismatch { .. })
        ));
    }

    #[test]
    fn should_allow_homologator_to_publish_directly_in_same_domain() {
        // Usuário é Diretor de Comunicação com ApproveNews
        let comm_role = Role::new(
            Uuid::new_v4(),
            CAER_ID,
            "Homologador de Notícias",
            HashSet::from([Permission::ApproveNews]),
        )
        .unwrap();

        let roles = vec![comm_role];

        // Pode homologar notícias
        assert!(is_authorized_in_ca(
            &roles,
            CAER_ID,
            &Permission::ApproveNews
        ));
        // E PODE publicar diretamente por implicação
        assert!(is_authorized_in_ca(
            &roles,
            CAER_ID,
            &Permission::PublishNewsDirect
        ));
    }

    #[test]
    fn should_refuse_direct_publisher_trying_to_homologate() {
        // Usuário possui APENAS permissão de publicação direta
        let pub_role = Role::new(
            Uuid::new_v4(),
            CAER_ID,
            "Publicador de Notícias",
            HashSet::from([Permission::PublishNewsDirect]),
        )
        .unwrap();

        let roles = vec![pub_role];

        // Pode publicar diretamente
        assert!(is_authorized_in_ca(
            &roles,
            CAER_ID,
            &Permission::PublishNewsDirect
        ));
        // NÃO pode homologar
        assert!(!is_authorized_in_ca(
            &roles,
            CAER_ID,
            &Permission::ApproveNews
        ));

        let check = check_authorization_in_ca(USER_ID, &roles, CAER_ID, &Permission::ApproveNews);
        assert!(matches!(
            check,
            Err(AuthorizationRuleError::PermissionDenied {
                required: Permission::ApproveNews,
                ..
            })
        ));
    }
}
