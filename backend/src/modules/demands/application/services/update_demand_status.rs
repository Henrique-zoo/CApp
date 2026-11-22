//! Caso de uso: atualização de status de demandas pela gestão do Centro Acadêmico.
//!
//! Coordena autorização estrita de gestão no CA, validação de justificativa/parecer,
//! bloqueio de encaminhamento de espaço físico à coordenação, transição no domínio
//! e persistência transacional com histórico.

use chrono::Utc;
use uuid::Uuid;

use crate::modules::demands::{
    application::{
        commands::UpdateDemandStatusCommand,
        error::DemandApplicationError,
        ports::{
            AcademicAffiliationPort, DemandNotificationRepository, DemandRepository,
            NoopNotificationRepository,
        },
    },
    domain::{
        entities::{Demand, DemandError, DemandForum, DemandStatus},
        events::DemandStatusUpdatedEvent,
        rules::validate_comment,
    },
};

/// Serviço para atualização de status de demandas pela gestão do Centro Acadêmico.
#[derive(Debug, Clone)]
pub struct UpdateDemandStatusUseCase<R, A, N = NoopNotificationRepository> {
    demand_repository: R,
    affiliation_port: A,
    notification_repository: N,
}

impl<R, A> UpdateDemandStatusUseCase<R, A, NoopNotificationRepository> {
    /// Inicializa o caso de uso com os adaptadores de repositório e afiliação (sem disparo de notificação).
    pub fn new(demand_repository: R, affiliation_port: A) -> Self {
        Self {
            demand_repository,
            affiliation_port,
            notification_repository: NoopNotificationRepository,
        }
    }
}

impl<R, A, N> UpdateDemandStatusUseCase<R, A, N> {
    /// Inicializa o caso de uso com captura e persistência de notificações.
    pub fn with_notifications(
        demand_repository: R,
        affiliation_port: A,
        notification_repository: N,
    ) -> Self {
        Self {
            demand_repository,
            affiliation_port,
            notification_repository,
        }
    }
}

impl<R, A, N> UpdateDemandStatusUseCase<R, A, N>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
    N: DemandNotificationRepository,
{
    /// Executa a atualização de status da demanda e dispara notificação caso o autor não seja o próprio gestor.
    ///
    /// # Regras de Negócio e Segurança
    ///
    /// 1. A justificativa ou parecer (`opinion`) não pode ser vazio nem ultrapassar o limite máximo.
    /// 2. O Centro Acadêmico deve existir e estar ativo.
    /// 3. O usuário autenticado deve possuir vínculo estudantil ativo.
    /// 4. Autorização estrita: apenas membros com cargo de gestão ativo no Centro Acadêmico
    ///    possuem permissão para alterar o status das demandas daquele CA ([`DemandApplicationError::UserNotPermittedInCa`]).
    /// 5. A demanda solicitada deve existir e pertencer estritamente ao `academic_center_id` informado
    ///    ([`DemandApplicationError::DemandNotFound`]), garantindo isolamento total entre CAs.
    /// 6. Demandas do fórum de espaço físico nunca podem ser encaminhadas à coordenação do curso
    ///    ([`DemandError::CoordinationForwardingNotAllowed`]).
    /// 7. A transição deve ser válida perante o ciclo de vida da demanda e seu histórico é registrado.
    /// 8. A persistência atualiza o status da demanda e grava atomicamente o registro de histórico.
    /// 9. Caso o gestor que atualizou não seja o próprio autor da demanda, gera uma notificação de
    ///    atualização de status destinada ao estudante autor.
    pub async fn execute(
        &self,
        command: UpdateDemandStatusCommand,
    ) -> Result<Demand, DemandApplicationError> {
        let trimmed_opinion = command.opinion.trim();
        if trimmed_opinion.is_empty() {
            return Err(DemandApplicationError::OpinionRequired);
        }
        validate_comment(trimmed_opinion)?;

        let affiliation = self
            .affiliation_port
            .get_affiliation(command.manager_id, command.academic_center_id)
            .await?;

        if !affiliation.academic_center_exists {
            return Err(DemandApplicationError::AcademicCenterNotFound(
                command.academic_center_id,
            ));
        }

        if !affiliation.academic_center_active {
            return Err(DemandApplicationError::AcademicCenterInactive(
                command.academic_center_id,
            ));
        }

        if !affiliation.has_active_student_membership {
            return Err(DemandApplicationError::StudentMembershipRequired);
        }

        if !affiliation.is_ca_manager {
            return Err(DemandApplicationError::UserNotPermittedInCa);
        }

        let mut demand = self
            .demand_repository
            .find_by_id(command.demand_id)
            .await?
            .ok_or(DemandApplicationError::DemandNotFound(command.demand_id))?;

        if demand.academic_center_id() != command.academic_center_id {
            return Err(DemandApplicationError::DemandNotFound(command.demand_id));
        }

        if demand.forum() == DemandForum::PhysicalSpace
            && command.new_status == DemandStatus::ForwardedToCoordination
        {
            return Err(DemandApplicationError::InvalidDemand(
                DemandError::CoordinationForwardingNotAllowed,
            ));
        }

        let now = Utc::now();
        let from_status = Some(demand.status());

        demand.transition_to(
            command.new_status,
            command.manager_id,
            Some(trimmed_opinion.to_string()),
            now,
        )?;

        self.demand_repository.update(&demand).await?;

        // Notificação de atualização de status
        // Regra de negócio: nunca notificar o próprio usuário por uma ação executada por ele mesmo.
        let status_event = DemandStatusUpdatedEvent {
            demand_id: demand.id(),
            academic_center_id: demand.academic_center_id(),
            author_id: demand.author_id(),
            changed_by_id: command.manager_id,
            from_status,
            to_status: command.new_status,
            opinion: trimmed_opinion.to_string(),
            occurred_at: now,
        };

        if let Some(notification_res) = status_event.to_notification(Uuid::new_v4()) {
            let notification = notification_res?;
            self.notification_repository.create(&notification).await?;
        }

        Ok(demand)
    }
}

/// Função de conveniência para execução direta do caso de uso de atualização de status.
pub async fn update_demand_status<R, A>(
    demand_repository: R,
    affiliation_port: A,
    command: UpdateDemandStatusCommand,
) -> Result<Demand, DemandApplicationError>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
{
    UpdateDemandStatusUseCase::new(demand_repository, affiliation_port)
        .execute(command)
        .await
}

/// Função de conveniência para execução com disparo e persistência de notificações.
pub async fn update_demand_status_with_notifications<R, A, N>(
    demand_repository: R,
    affiliation_port: A,
    notification_repository: N,
    command: UpdateDemandStatusCommand,
) -> Result<Demand, DemandApplicationError>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
    N: DemandNotificationRepository,
{
    UpdateDemandStatusUseCase::with_notifications(
        demand_repository,
        affiliation_port,
        notification_repository,
    )
    .execute(command)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::demands::{
        application::ports::AcademicAffiliation,
        domain::rules::MAX_COMMENT_LENGTH,
        infrastructure::{InMemoryAcademicAffiliationPort, InMemoryDemandRepository},
    };
    use uuid::Uuid;

    fn sample_ca_id() -> Uuid {
        Uuid::new_v4()
    }

    fn sample_manager_id() -> Uuid {
        Uuid::new_v4()
    }

    fn sample_student_id() -> Uuid {
        Uuid::new_v4()
    }

    fn create_test_demand(ca_id: Uuid, author_id: Uuid, forum: DemandForum) -> Demand {
        Demand::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            forum,
            Some("Título da Demanda".to_string()),
            "Descrição da demanda para testes".to_string(),
            Some("Geral".to_string()),
            if forum == DemandForum::Institutional {
                Some("Engenharia de Redes".to_string())
            } else {
                None
            },
            Utc::now(),
        )
        .expect("demanda válida")
    }

    #[tokio::test]
    async fn should_update_physical_space_demand_status_successfully_by_ca_manager() {
        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia de Redes".to_string()),
                vec!["Engenharia de Redes".to_string()],
                true, // is_ca_manager = true
            ),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::InAnalysis,
            "Análise iniciada pela diretoria de patrimônio".to_string(),
        );

        let updated = update_demand_status(&repo, &affiliation_port, command)
            .await
            .expect("deve atualizar com sucesso");

        assert_eq!(updated.status(), DemandStatus::InAnalysis);
        assert_eq!(updated.status_history().len(), 1);
        let entry = &updated.status_history()[0];
        assert_eq!(entry.from_status(), Some(DemandStatus::Open));
        assert_eq!(entry.to_status(), DemandStatus::InAnalysis);
        assert_eq!(entry.changed_by_id(), manager_id);
        assert_eq!(
            entry.comment(),
            Some("Análise iniciada pela diretoria de patrimônio")
        );

        // Verifica persistência no repositório
        let reloaded = repo.find_by_id(demand_id).await.unwrap().unwrap();
        assert_eq!(reloaded.status(), DemandStatus::InAnalysis);
    }

    #[tokio::test]
    async fn should_forward_institutional_demand_to_coordination_by_ca_manager() {
        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let mut demand = create_test_demand(ca_id, author_id, DemandForum::Institutional);
        demand
            .transition_to(
                DemandStatus::InAnalysis,
                manager_id,
                Some("Triagem inicial".to_string()),
                Utc::now(),
            )
            .unwrap();
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia de Redes".to_string()),
                vec!["Engenharia de Redes".to_string()],
                true,
            ),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::ForwardedToCoordination,
            "Encaminhado à coordenação via SEI".to_string(),
        );

        let updated = update_demand_status(&repo, &affiliation_port, command)
            .await
            .expect("deve encaminhar à coordenação");

        assert_eq!(updated.status(), DemandStatus::ForwardedToCoordination);
        assert_eq!(updated.status_history().len(), 2);
    }

    #[tokio::test]
    async fn should_reject_forwarding_physical_space_demand_to_coordination() {
        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia de Redes".to_string()),
                vec!["Engenharia de Redes".to_string()],
                true,
            ),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::ForwardedToCoordination,
            "Tentando enviar reparo à coordenação".to_string(),
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::InvalidDemand(
                DemandError::CoordinationForwardingNotAllowed,
            )) => (),
            other => panic!("esperava CoordinationForwardingNotAllowed, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_status_update_without_opinion() {
        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::InAnalysis,
            "   \t \n  ".to_string(),
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::OpinionRequired) => (),
            other => panic!("esperava OpinionRequired, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_opinion_too_long() {
        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        let long_opinion = "a".repeat(MAX_COMMENT_LENGTH + 1);
        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::InAnalysis,
            long_opinion,
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::InvalidDemand(DemandError::CommentTooLong { max })) => {
                assert_eq!(max, MAX_COMMENT_LENGTH);
            }
            other => panic!("esperava CommentTooLong, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_update_when_user_is_not_ca_manager() {
        let ca_id = sample_ca_id();
        let non_manager_id = sample_student_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            non_manager_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia de Redes".to_string()),
                vec!["Engenharia de Redes".to_string()],
                false, // is_ca_manager = false
            ),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            non_manager_id,
            DemandStatus::InAnalysis,
            "Tentativa não autorizada".to_string(),
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::UserNotPermittedInCa) => (),
            other => panic!("esperava UserNotPermittedInCa, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_update_when_manager_belongs_to_different_ca() {
        let ca_1 = sample_ca_id();
        let _ca_2 = sample_ca_id();
        let manager_ca_2 = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_1, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        // Usuário é gestor do CA 2, mas tenta atuar no CA 1 onde não é gestor
        affiliation_port.set_affiliation(
            manager_ca_2,
            ca_1,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Computação".to_string()),
                vec!["Engenharia de Redes".to_string()],
                false, // no CA 1 não é gestor
            ),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_1,
            demand_id,
            manager_ca_2,
            DemandStatus::InAnalysis,
            "Tentativa cruzada".to_string(),
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::UserNotPermittedInCa) => (),
            other => panic!("esperava UserNotPermittedInCa, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_update_when_demand_belongs_to_another_ca() {
        let ca_1 = sample_ca_id();
        let ca_2 = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        // Demanda cadastrada no CA 2
        let demand = create_test_demand(ca_2, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        // Gestor tem cargo no CA 1
        affiliation_port.set_affiliation(
            manager_id,
            ca_1,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        // Mas tenta atualizar no contexto do CA 1
        let command = UpdateDemandStatusCommand::new(
            ca_1,
            demand_id,
            manager_id,
            DemandStatus::InAnalysis,
            "Tentativa isolamento".to_string(),
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::DemandNotFound(id)) => assert_eq!(id, demand_id),
            other => panic!("esperava DemandNotFound, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_reopening_completed_institutional_demand() {
        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let repo = InMemoryDemandRepository::new();
        let mut demand = create_test_demand(ca_id, author_id, DemandForum::Institutional);
        demand
            .transition_to(
                DemandStatus::Completed,
                manager_id,
                Some("Conclusão anterior".to_string()),
                Utc::now(),
            )
            .unwrap();
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::Open,
            "Tentando reabrir diretamente".to_string(),
        );

        let result = update_demand_status(&repo, &affiliation_port, command).await;
        match result {
            Err(DemandApplicationError::InvalidDemand(DemandError::InvalidStatusTransition {
                ..
            })) => (),
            other => panic!("esperava InvalidStatusTransition, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_dispatch_notification_to_student_author_when_manager_updates_status() {
        use crate::modules::demands::infrastructure::InMemoryDemandNotificationRepository;

        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();
        let author_id = sample_student_id();

        let demand_repo = InMemoryDemandRepository::new();
        let notif_repo = InMemoryDemandNotificationRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        demand_repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::InAnalysis,
            "A vistoria foi agendada".to_string(),
        );

        let updated = update_demand_status_with_notifications(
            &demand_repo,
            &affiliation_port,
            &notif_repo,
            command,
        )
        .await
        .expect("deve atualizar com sucesso");

        assert_eq!(updated.status(), DemandStatus::InAnalysis);

        // Verifica notificação gerada
        let notifs = notif_repo.all();
        assert_eq!(notifs.len(), 1);
        let notif = &notifs[0];
        assert_eq!(notif.demand_id(), demand_id);
        assert_eq!(notif.recipient_user_id(), author_id);
        assert_eq!(notif.academic_center_id(), ca_id);
        assert_eq!(notif.title(), "Demanda em análise");
        assert!(notif.message().contains("A vistoria foi agendada"));
        assert!(!notif.is_read());
    }

    #[tokio::test]
    async fn should_not_dispatch_notification_when_manager_updates_own_demand() {
        use crate::modules::demands::infrastructure::InMemoryDemandNotificationRepository;

        let ca_id = sample_ca_id();
        let manager_id = sample_manager_id();

        let demand_repo = InMemoryDemandRepository::new();
        let notif_repo = InMemoryDemandNotificationRepository::new();
        // O próprio gestor é o autor da demanda!
        let demand = create_test_demand(ca_id, manager_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        demand_repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        let command = UpdateDemandStatusCommand::new(
            ca_id,
            demand_id,
            manager_id,
            DemandStatus::InAnalysis,
            "Eu mesmo estou analisando".to_string(),
        );

        let updated = update_demand_status_with_notifications(
            &demand_repo,
            &affiliation_port,
            &notif_repo,
            command,
        )
        .await
        .expect("deve atualizar com sucesso");

        assert_eq!(updated.status(), DemandStatus::InAnalysis);

        // Regra de negócio: nunca notificar o próprio usuário por ação executada por ele mesmo
        let notifs = notif_repo.all();
        assert!(
            notifs.is_empty(),
            "nenhuma notificação deve ser gerada para ação própria"
        );
    }
}
