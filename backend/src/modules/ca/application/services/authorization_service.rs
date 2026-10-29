//! Serviço de aplicação para avaliação de autorização institucional.
//!
//! Agrega os cargos ativos do usuário em um Centro Acadêmico específico, resolve
//! permissões acumuladas e aplica as regras de negócio de governança do CA.

use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::modules::ca::{
    application::ports::{
        AuthorizationServiceError, AuthorizationServicePort, RoleAssignmentRepositoryPort,
        RoleRepositoryPort,
    },
    domain::{
        entities::{permission::Permission, role::Role, role_assignment::RoleAssignment},
        rules::{
            AuthorizationRuleError, check_authorization_in_ca, evaluate_effective_permissions,
            is_authorized_in_ca,
        },
    },
    infrastructure::repository::{InMemoryRoleAssignmentRepository, InMemoryRoleRepository},
};

/// Identificador didático padrão do Centro Acadêmico de Engenharia de Redes (CAER).
pub const CAER_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);

/// Identificador didático padrão do Centro Acadêmico de Ciência da Computação (CACC).
pub const CACC_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);

/// Identificador didático do estudante Presidente do CAER ("231012345@aluno.unb.br").
pub const PRESIDENTE_CAER_USER_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

/// Identificador didático do estudante Diretor de Comunicação ("241098765@aluno.unb.br").
pub const DIRETOR_COMUNICACAO_USER_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0012);

/// Identificador didático do estudante Diretor de Eventos ("222033333@aluno.unb.br").
pub const DIRETOR_EVENTOS_USER_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0013);

/// Identificador didático de estudante regular sem nenhum cargo administrativo.
pub const STUDENT_WITHOUT_ROLE_USER_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0021);

/// Identificador didático do administrador da plataforma ("200000001@unb.br").
pub const ADMIN_PLATAFORMA_USER_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0099);

/// Serviço responsável por avaliar autorizações institucionais e agregar privilégios por CA.
#[derive(Clone)]
pub struct AuthorizationService {
    role_repo: Arc<dyn RoleRepositoryPort>,
    assignment_repo: Arc<dyn RoleAssignmentRepositoryPort>,
}

impl AuthorizationService {
    /// Constrói uma nova instância de [`AuthorizationService`] com portas injetadas.
    pub fn new(
        role_repo: Arc<dyn RoleRepositoryPort>,
        assignment_repo: Arc<dyn RoleAssignmentRepositoryPort>,
    ) -> Self {
        Self {
            role_repo,
            assignment_repo,
        }
    }

    /// Cria uma instância vazia em memória, útil para cenários de testes controlados.
    pub fn with_in_memory() -> Self {
        let role_repo = Arc::new(InMemoryRoleRepository::new());
        let assignment_repo = Arc::new(InMemoryRoleAssignmentRepository::new());
        Self::new(role_repo, assignment_repo)
    }

    /// Cria uma instância preenchida com a estrutura institucional didática da UnB (CAER e CACC).
    pub async fn with_didactic_data() -> Self {
        let role_repo = Arc::new(InMemoryRoleRepository::with_didactic_roles(
            CAER_DIDACTIC_ID,
            CACC_DIDACTIC_ID,
        ));
        let assignment_repo = Arc::new(InMemoryRoleAssignmentRepository::new());

        // Localiza os cargos do CAER para vincular os integrantes didáticos
        if let Ok(roles) = role_repo.find_roles_by_ca_id(CAER_DIDACTIC_ID).await {
            for role in roles {
                if role.name == "Presidente" {
                    let _ = assignment_repo
                        .save_assignment(
                            &RoleAssignment::new(
                                Uuid::new_v4(),
                                PRESIDENTE_CAER_USER_ID,
                                CAER_DIDACTIC_ID,
                                role.id,
                                "Presidente",
                            )
                            .unwrap(),
                        )
                        .await;
                } else if role.name == "Diretor de Comunicação" {
                    let _ = assignment_repo
                        .save_assignment(
                            &RoleAssignment::new(
                                Uuid::new_v4(),
                                DIRETOR_COMUNICACAO_USER_ID,
                                CAER_DIDACTIC_ID,
                                role.id,
                                "Diretor de Comunicação",
                            )
                            .unwrap(),
                        )
                        .await;
                } else if role.name == "Diretor de Eventos" {
                    let _ = assignment_repo
                        .save_assignment(
                            &RoleAssignment::new(
                                Uuid::new_v4(),
                                DIRETOR_EVENTOS_USER_ID,
                                CAER_DIDACTIC_ID,
                                role.id,
                                "Diretor de Eventos",
                            )
                            .unwrap(),
                        )
                        .await;
                }
            }
        }

        Self::new(role_repo, assignment_repo)
    }

    /// Vincula um usuário a um cargo no Centro Acadêmico, concedendo as permissões associadas.
    ///
    /// # Erros
    ///
    /// Retorna erro se o cargo não existir, se pertencer a outro CA ou se houver falha de persistência.
    pub async fn assign_role(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        role_id: Uuid,
    ) -> Result<RoleAssignment, AuthorizationServiceError> {
        let role = self
            .role_repo
            .find_role_by_id(role_id)
            .await
            .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?
            .ok_or_else(|| {
                AuthorizationServiceError::Forbidden(format!("Cargo '{role_id}' não encontrado."))
            })?;

        if role.academic_center_id != ca_id {
            return Err(AuthorizationServiceError::AcademicCenterMismatch {
                role_ca_id: role.academic_center_id,
                target_ca_id: ca_id,
            });
        }

        let assignment = RoleAssignment::new(Uuid::new_v4(), user_id, ca_id, role_id, &role.name)
            .map_err(|e| AuthorizationServiceError::Forbidden(e.to_string()))?;

        self.assignment_repo
            .save_assignment(&assignment)
            .await
            .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?;

        Ok(assignment)
    }

    /// Desvincula um integrante revogando suas permissões sem apagar o registro histórico.
    pub async fn revoke_assignment(
        &self,
        assignment_id: Uuid,
    ) -> Result<(), AuthorizationServiceError> {
        self.assignment_repo
            .revoke_assignment(assignment_id, Utc::now())
            .await
            .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))
    }

    /// Desvincula um usuário de um cargo específico em um Centro Acadêmico.
    pub async fn revoke_user_role(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AuthorizationServiceError> {
        let active_assignments = self
            .assignment_repo
            .find_active_by_user_and_ca(user_id, ca_id)
            .await
            .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?;

        let now = Utc::now();
        for assignment in active_assignments {
            if assignment.role_id == role_id {
                self.assignment_repo
                    .revoke_assignment(assignment.id, now)
                    .await
                    .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AuthorizationServicePort for AuthorizationService {
    async fn check_permission(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        permission: &Permission,
    ) -> Result<(), AuthorizationServiceError> {
        let active_assignments = self
            .assignment_repo
            .find_active_by_user_and_ca(user_id, ca_id)
            .await
            .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?;

        if active_assignments.is_empty() {
            // Verifica se o usuário possui cargo ativo em outro CA (regra de isolamento multi-CA)
            let other_active = self
                .assignment_repo
                .find_active_by_user(user_id)
                .await
                .unwrap_or_default();

            if let Some(other) = other_active
                .into_iter()
                .find(|a| a.academic_center_id != ca_id)
            {
                return Err(AuthorizationServiceError::AcademicCenterMismatch {
                    role_ca_id: other.academic_center_id,
                    target_ca_id: ca_id,
                });
            }

            // Verifica se o usuário já fez parte da gestão deste CA e foi desvinculado
            let all_history = self
                .assignment_repo
                .find_history_by_user_and_ca(user_id, ca_id)
                .await
                .unwrap_or_default();

            if all_history.iter().any(|a| !a.is_active()) {
                return Err(AuthorizationServiceError::Forbidden(
                    "O usuário foi desvinculado da gestão e suas permissões foram revogadas."
                        .to_string(),
                ));
            }

            return Err(AuthorizationServiceError::NoActiveRoleInCa { user_id, ca_id });
        }

        let mut roles = Vec::with_capacity(active_assignments.len());
        for assignment in &active_assignments {
            if let Some(role) = self
                .role_repo
                .find_role_by_id(assignment.role_id)
                .await
                .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?
            {
                roles.push(role);
            }
        }

        check_authorization_in_ca(user_id, &roles, ca_id, permission).map_err(|err| match err {
            AuthorizationRuleError::AcademicCenterMismatch {
                role_ca_id,
                target_ca_id,
            } => AuthorizationServiceError::AcademicCenterMismatch {
                role_ca_id,
                target_ca_id,
            },
            AuthorizationRuleError::PermissionDenied { required, ca_id } => {
                AuthorizationServiceError::Forbidden(format!(
                    "Permissão '{}' não concedida para o Centro Acadêmico '{}'.",
                    required, ca_id
                ))
            }
            AuthorizationRuleError::NoActiveRolesInAcademicCenter { user_id, ca_id } => {
                AuthorizationServiceError::NoActiveRoleInCa { user_id, ca_id }
            }
        })
    }

    async fn has_permission(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        permission: &Permission,
    ) -> Result<bool, AuthorizationServiceError> {
        let roles = self.list_active_roles(user_id, ca_id).await?;
        Ok(is_authorized_in_ca(&roles, ca_id, permission))
    }

    async fn get_effective_permissions(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<HashSet<Permission>, AuthorizationServiceError> {
        let roles = self.list_active_roles(user_id, ca_id).await?;
        Ok(evaluate_effective_permissions(&roles))
    }

    async fn list_active_roles(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<Vec<Role>, AuthorizationServiceError> {
        let active_assignments = self
            .assignment_repo
            .find_active_by_user_and_ca(user_id, ca_id)
            .await
            .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?;

        let mut roles = Vec::with_capacity(active_assignments.len());
        for assignment in active_assignments {
            if let Some(role) = self
                .role_repo
                .find_role_by_id(assignment.role_id)
                .await
                .map_err(|e| AuthorizationServiceError::RepositoryFailure(e.to_string()))?
            {
                roles.push(role);
            }
        }
        Ok(roles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_refuse_user_without_role_in_ca() {
        let service = AuthorizationService::with_didactic_data().await;

        let result = service
            .check_permission(
                STUDENT_WITHOUT_ROLE_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::ManageMembers,
            )
            .await;

        assert_eq!(
            result.unwrap_err(),
            AuthorizationServiceError::NoActiveRoleInCa {
                user_id: STUDENT_WITHOUT_ROLE_USER_ID,
                ca_id: CAER_DIDACTIC_ID,
            }
        );

        let has_perm = service
            .has_permission(
                STUDENT_WITHOUT_ROLE_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::ManageMembers,
            )
            .await
            .expect("avaliação sem erro de transporte");
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn should_refuse_user_with_role_in_another_ca() {
        let service = AuthorizationService::with_didactic_data().await;

        // PRESIDENTE_CAER_USER_ID é Presidente em CAER, mas tenta atuar em CACC
        let result = service
            .check_permission(
                PRESIDENTE_CAER_USER_ID,
                CACC_DIDACTIC_ID,
                &Permission::ManageMembers,
            )
            .await;

        assert_eq!(
            result.unwrap_err(),
            AuthorizationServiceError::AcademicCenterMismatch {
                role_ca_id: CAER_DIDACTIC_ID,
                target_ca_id: CACC_DIDACTIC_ID,
            }
        );

        let has_perm = service
            .has_permission(
                PRESIDENTE_CAER_USER_ID,
                CACC_DIDACTIC_ID,
                &Permission::ManageMembers,
            )
            .await
            .unwrap();
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn should_allow_homologator_to_publish_directly_in_same_domain() {
        let service = AuthorizationService::with_didactic_data().await;

        // DIRETOR_COMUNICACAO_USER_ID possui ApproveNews e PublishNewsDirect
        // Verifica que ele pode homologar notícias
        service
            .check_permission(
                DIRETOR_COMUNICACAO_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::ApproveNews,
            )
            .await
            .expect("homologador pode homologar");

        // Verifica que ele pode publicar diretamente
        service
            .check_permission(
                DIRETOR_COMUNICACAO_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::PublishNewsDirect,
            )
            .await
            .expect("homologador pode publicar direto");

        assert!(
            service
                .has_permission(
                    DIRETOR_COMUNICACAO_USER_ID,
                    CAER_DIDACTIC_ID,
                    &Permission::PublishNewsDirect,
                )
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn should_refuse_direct_publisher_trying_to_homologate() {
        let service = AuthorizationService::with_didactic_data().await;

        // DIRETOR_EVENTOS_USER_ID possui apenas PublishEventsDirect
        // Ele pode publicar eventos diretamente
        service
            .check_permission(
                DIRETOR_EVENTOS_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::PublishEventsDirect,
            )
            .await
            .expect("pode publicar eventos diretamente");

        // Mas NÃO pode homologar eventos de terceiros
        let result = service
            .check_permission(
                DIRETOR_EVENTOS_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::ApproveEvents,
            )
            .await;

        assert!(matches!(
            result.unwrap_err(),
            AuthorizationServiceError::Forbidden(_)
        ));

        let has_perm = service
            .has_permission(
                DIRETOR_EVENTOS_USER_ID,
                CAER_DIDACTIC_ID,
                &Permission::ApproveEvents,
            )
            .await
            .unwrap();
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn should_revoke_member_immediately_while_preserving_history() {
        let service = AuthorizationService::with_in_memory();
        let user_id = Uuid::new_v4();
        let ca_id = CAER_DIDACTIC_ID;

        // 1. Cria um cargo de Ouvidor com ManageDemands
        let role = Role::new(
            Uuid::new_v4(),
            ca_id,
            "Ouvidor Geral",
            HashSet::from([Permission::ManageDemands]),
        )
        .unwrap();
        service.role_repo.save_role(&role).await.unwrap();

        // 2. Vincula o usuário ao cargo
        let assignment = service.assign_role(user_id, ca_id, role.id).await.unwrap();

        // 3. Valida que o usuário possui permissão ativa
        assert!(
            service
                .has_permission(user_id, ca_id, &Permission::ManageDemands)
                .await
                .unwrap()
        );
        service
            .check_permission(user_id, ca_id, &Permission::ManageDemands)
            .await
            .expect("permissão ativa");

        // 4. Desvincula o integrante
        service.revoke_assignment(assignment.id).await.unwrap();

        // 5. Permissão ativa é revogada imediatamente
        assert!(
            !service
                .has_permission(user_id, ca_id, &Permission::ManageDemands)
                .await
                .unwrap()
        );

        let err = service
            .check_permission(user_id, ca_id, &Permission::ManageDemands)
            .await
            .unwrap_err();
        assert!(matches!(err, AuthorizationServiceError::Forbidden(_)));

        // 6. O histórico administrativo permanece preservado
        let history = service
            .assignment_repo
            .find_history_by_user_and_ca(user_id, ca_id)
            .await
            .unwrap();
        assert_eq!(history.len(), 1);
        assert!(!history[0].is_active);
        assert!(history[0].revoked_at.is_some());
    }

    #[tokio::test]
    async fn should_aggregate_effective_permissions_from_multiple_roles() {
        let service = AuthorizationService::with_in_memory();
        let user_id = Uuid::new_v4();
        let ca_id = CAER_DIDACTIC_ID;

        // Cargo 1: Notícias
        let role1 = Role::new(
            Uuid::new_v4(),
            ca_id,
            "Comunicação",
            HashSet::from([Permission::PublishNewsDirect]),
        )
        .unwrap();
        service.role_repo.save_role(&role1).await.unwrap();
        service.assign_role(user_id, ca_id, role1.id).await.unwrap();

        // Cargo 2: Finanças
        let role2 = Role::new(
            Uuid::new_v4(),
            ca_id,
            "Tesouraria",
            HashSet::from([Permission::PublishFinancialStatements]),
        )
        .unwrap();
        service.role_repo.save_role(&role2).await.unwrap();
        service.assign_role(user_id, ca_id, role2.id).await.unwrap();

        // Consolida permissões de ambos os cargos
        let effective = service
            .get_effective_permissions(user_id, ca_id)
            .await
            .unwrap();
        assert!(effective.contains(&Permission::PublishNewsDirect));
        assert!(effective.contains(&Permission::PublishFinancialStatements));
        assert!(!effective.contains(&Permission::ManageMembers));
    }
}
