//! Caso de uso: marcação de notificação de demanda como lida pelo estudante destinatário.

use chrono::Utc;

use crate::modules::demands::{
    application::{
        commands::MarkDemandNotificationAsReadCommand, error::DemandApplicationError,
        ports::DemandNotificationRepository,
    },
    domain::entities::DemandNotification,
};

/// Marca uma notificação como lida garantindo autorização estrita de destinatário.
///
/// # Regras de Negócio e Segurança
/// 1. A notificação deve existir no repositório ([`DemandApplicationError::DemandNotificationNotFound`]).
/// 2. Somente o próprio estudante destinatário da notificação pode marcá-la como lida
///    ([`DemandApplicationError::DemandNotificationForbidden`]).
/// 3. A operação é idempotente: caso já esteja lida, preserva o timestamp original de leitura.
pub async fn mark_demand_notification_as_read<N>(
    notification_repo: N,
    command: MarkDemandNotificationAsReadCommand,
) -> Result<DemandNotification, DemandApplicationError>
where
    N: DemandNotificationRepository,
{
    let mut notification = notification_repo
        .find_by_id(command.notification_id)
        .await?
        .ok_or(DemandApplicationError::DemandNotificationNotFound(
            command.notification_id,
        ))?;

    if notification.recipient_user_id() != command.user_id {
        return Err(DemandApplicationError::DemandNotificationForbidden);
    }

    if notification.read_at().is_none() {
        notification.mark_as_read(Utc::now());
        notification_repo.update(&notification).await?;
    }

    Ok(notification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::modules::demands::{
        domain::entities::DemandNotificationEventType,
        infrastructure::InMemoryDemandNotificationRepository,
    };

    #[tokio::test]
    async fn should_mark_notification_as_read_successfully_by_recipient() {
        let repo = InMemoryDemandNotificationRepository::new();
        let student_id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let notif_id = Uuid::new_v4();

        let notif = DemandNotification::new(
            notif_id,
            Uuid::new_v4(),
            student_id,
            ca_id,
            DemandNotificationEventType::StatusUpdated,
            "Aviso de atualização".to_string(),
            "Mensagem".to_string(),
            Utc::now(),
        )
        .unwrap();

        repo.create(&notif).await.unwrap();

        let command = MarkDemandNotificationAsReadCommand::new(notif_id, student_id);
        let updated = mark_demand_notification_as_read(&repo, command)
            .await
            .expect("deve marcar como lida com sucesso");

        assert!(updated.is_read());
        assert!(updated.read_at().is_some());

        // Verifica persistência no repositório
        let reloaded = repo.find_by_id(notif_id).await.unwrap().unwrap();
        assert!(reloaded.is_read());
    }

    #[tokio::test]
    async fn should_reject_marking_as_read_when_notification_belongs_to_another_student() {
        let repo = InMemoryDemandNotificationRepository::new();
        let owner_student = Uuid::new_v4();
        let intruder_student = Uuid::new_v4();
        let notif_id = Uuid::new_v4();

        let notif = DemandNotification::new(
            notif_id,
            Uuid::new_v4(),
            owner_student,
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            "Aviso de atualização".to_string(),
            "Mensagem".to_string(),
            Utc::now(),
        )
        .unwrap();

        repo.create(&notif).await.unwrap();

        let command = MarkDemandNotificationAsReadCommand::new(notif_id, intruder_student);
        let result = mark_demand_notification_as_read(&repo, command).await;

        match result {
            Err(DemandApplicationError::DemandNotificationForbidden) => (),
            other => panic!("esperava DemandNotificationForbidden, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_return_not_found_when_notification_does_not_exist() {
        let repo = InMemoryDemandNotificationRepository::new();
        let non_existent_id = Uuid::new_v4();
        let student_id = Uuid::new_v4();

        let command = MarkDemandNotificationAsReadCommand::new(non_existent_id, student_id);
        let result = mark_demand_notification_as_read(&repo, command).await;

        match result {
            Err(DemandApplicationError::DemandNotificationNotFound(id)) => {
                assert_eq!(id, non_existent_id);
            }
            other => panic!("esperava DemandNotificationNotFound, obteve {other:?}"),
        }
    }
}
