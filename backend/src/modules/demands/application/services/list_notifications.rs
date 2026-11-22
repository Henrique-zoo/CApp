//! Caso de uso: listagem de notificações de demandas para o estudante destinatário.

use crate::modules::demands::{
    application::{
        error::DemandApplicationError, ports::DemandNotificationRepository,
        queries::ListDemandNotificationsQuery,
    },
    domain::entities::DemandNotification,
};

/// Lista as notificações de demandas destinadas ao estudante autenticado.
pub async fn list_demand_notifications<N>(
    notification_repo: N,
    query: ListDemandNotificationsQuery,
) -> Result<Vec<DemandNotification>, DemandApplicationError>
where
    N: DemandNotificationRepository,
{
    let notifications = notification_repo
        .list_by_recipient(
            query.recipient_user_id,
            query.unread_only,
            query.limit,
            query.offset,
        )
        .await?;

    Ok(notifications)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    use crate::modules::demands::{
        domain::entities::DemandNotificationEventType,
        infrastructure::InMemoryDemandNotificationRepository,
    };

    #[tokio::test]
    async fn should_list_notifications_only_for_recipient() {
        let repo = InMemoryDemandNotificationRepository::new();
        let student_a = Uuid::new_v4();
        let student_b = Uuid::new_v4();
        let ca_id = Uuid::new_v4();

        let notif_a = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            student_a,
            ca_id,
            DemandNotificationEventType::StatusUpdated,
            "Notificação do A".to_string(),
            "Mensagem A".to_string(),
            Utc::now(),
        )
        .unwrap();

        let notif_b = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            student_b,
            ca_id,
            DemandNotificationEventType::StatusUpdated,
            "Notificação do B".to_string(),
            "Mensagem B".to_string(),
            Utc::now(),
        )
        .unwrap();

        repo.create(&notif_a).await.unwrap();
        repo.create(&notif_b).await.unwrap();

        let query = ListDemandNotificationsQuery::new(student_a, false, None, None);
        let list = list_demand_notifications(&repo, query).await.unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id(), notif_a.id());
    }

    #[tokio::test]
    async fn should_filter_unread_notifications_when_requested() {
        let repo = InMemoryDemandNotificationRepository::new();
        let student_id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let now = Utc::now();

        let mut notif_read = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            student_id,
            ca_id,
            DemandNotificationEventType::StatusUpdated,
            "Lida".to_string(),
            "Mensagem lida".to_string(),
            now,
        )
        .unwrap();
        notif_read.mark_as_read(now + chrono::Duration::minutes(1));

        let notif_unread = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            student_id,
            ca_id,
            DemandNotificationEventType::StatusUpdated,
            "Não lida".to_string(),
            "Mensagem não lida".to_string(),
            now + chrono::Duration::minutes(2),
        )
        .unwrap();

        repo.create(&notif_read).await.unwrap();
        repo.create(&notif_unread).await.unwrap();

        let query_all = ListDemandNotificationsQuery::new(student_id, false, None, None);
        let all = list_demand_notifications(&repo, query_all).await.unwrap();
        assert_eq!(all.len(), 2);

        let query_unread = ListDemandNotificationsQuery::new(student_id, true, None, None);
        let unread = list_demand_notifications(&repo, query_unread)
            .await
            .unwrap();
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].id(), notif_unread.id());
    }
}
