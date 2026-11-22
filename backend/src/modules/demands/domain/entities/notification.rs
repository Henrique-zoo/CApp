//! Entidade de notificação de demandas e tipos associados.
//!
//! Encapsula os avisos gerados por eventos relevantes no ciclo de vida das demandas,
//! tais como atualização de status e parecer emitido pela gestão do Centro Acadêmico.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::demands::domain::entities::DemandError;

/// Limite máximo em caracteres para o título da notificação.
pub const MAX_NOTIFICATION_TITLE_LENGTH: usize = 255;

/// Limite máximo em caracteres para o corpo ou mensagem da notificação.
pub const MAX_NOTIFICATION_MESSAGE_LENGTH: usize = 5000;

/// Tipo de evento que originou a notificação da demanda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemandNotificationEventType {
    /// Atualização de status da demanda com parecer da gestão.
    StatusUpdated,
    /// Nova resposta ou interação oficial registrada pelo Centro Acadêmico.
    OfficialResponse,
}

impl DemandNotificationEventType {
    /// Retorna a representação textual canônica do tipo de evento.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::StatusUpdated => "status_updated",
            Self::OfficialResponse => "official_response",
        }
    }
}

impl fmt::Display for DemandNotificationEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DemandNotificationEventType {
    type Err = DemandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "status_updated" | "status-updated" | "statusupdated" => Ok(Self::StatusUpdated),
            "official_response" | "official-response" | "officialresponse" => {
                Ok(Self::OfficialResponse)
            }
            other => Err(DemandError::InvalidNotificationEventType(other.to_string())),
        }
    }
}

/// Entidade de domínio representando uma notificação de demanda entregue a um estudante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandNotification {
    id: Uuid,
    demand_id: Uuid,
    recipient_user_id: Uuid,
    academic_center_id: Uuid,
    event_type: DemandNotificationEventType,
    title: String,
    message: String,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl DemandNotification {
    /// Cria uma nova notificação não lida com validação de invariantes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        demand_id: Uuid,
        recipient_user_id: Uuid,
        academic_center_id: Uuid,
        event_type: DemandNotificationEventType,
        title: String,
        message: String,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DemandError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(DemandError::EmptyNotificationTitle);
        }
        if trimmed_title.chars().count() > MAX_NOTIFICATION_TITLE_LENGTH {
            return Err(DemandError::NotificationTitleTooLong {
                max: MAX_NOTIFICATION_TITLE_LENGTH,
            });
        }

        let trimmed_message = message.trim();
        if trimmed_message.is_empty() {
            return Err(DemandError::EmptyNotificationMessage);
        }
        if trimmed_message.chars().count() > MAX_NOTIFICATION_MESSAGE_LENGTH {
            return Err(DemandError::NotificationMessageTooLong {
                max: MAX_NOTIFICATION_MESSAGE_LENGTH,
            });
        }

        Ok(Self {
            id,
            demand_id,
            recipient_user_id,
            academic_center_id,
            event_type,
            title: trimmed_title.to_string(),
            message: trimmed_message.to_string(),
            read_at: None,
            created_at,
        })
    }

    /// Reconstitui uma notificação a partir dos dados persistidos.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        demand_id: Uuid,
        recipient_user_id: Uuid,
        academic_center_id: Uuid,
        event_type: DemandNotificationEventType,
        title: String,
        message: String,
        read_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DemandError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(DemandError::EmptyNotificationTitle);
        }
        if trimmed_title.chars().count() > MAX_NOTIFICATION_TITLE_LENGTH {
            return Err(DemandError::NotificationTitleTooLong {
                max: MAX_NOTIFICATION_TITLE_LENGTH,
            });
        }

        let trimmed_message = message.trim();
        if trimmed_message.is_empty() {
            return Err(DemandError::EmptyNotificationMessage);
        }
        if trimmed_message.chars().count() > MAX_NOTIFICATION_MESSAGE_LENGTH {
            return Err(DemandError::NotificationMessageTooLong {
                max: MAX_NOTIFICATION_MESSAGE_LENGTH,
            });
        }

        if read_at.is_some_and(|r_at| r_at < created_at) {
            return Err(DemandError::InconsistentTimestamps {
                created_at,
                updated_at: read_at.unwrap(),
            });
        }

        Ok(Self {
            id,
            demand_id,
            recipient_user_id,
            academic_center_id,
            event_type,
            title: trimmed_title.to_string(),
            message: trimmed_message.to_string(),
            read_at,
            created_at,
        })
    }

    /// Marca a notificação como lida no instante especificado, preservando a data caso já estivesse lida.
    pub fn mark_as_read(&mut self, at: DateTime<Utc>) {
        if self.read_at.is_none() {
            self.read_at = Some(at);
        }
    }

    /// Identificador único da notificação.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador da demanda vinculada.
    pub fn demand_id(&self) -> Uuid {
        self.demand_id
    }

    /// Usuário destinatário da notificação.
    pub fn recipient_user_id(&self) -> Uuid {
        self.recipient_user_id
    }

    /// Centro Acadêmico responsável pelo contexto da demanda.
    pub fn academic_center_id(&self) -> Uuid {
        self.academic_center_id
    }

    /// Tipo de evento gerador da notificação.
    pub fn event_type(&self) -> DemandNotificationEventType {
        self.event_type
    }

    /// Título ou cabeçalho do aviso.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Conteúdo detalhado da notificação.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Instante UTC de leitura pelo estudante, se já lida.
    pub fn read_at(&self) -> Option<DateTime<Utc>> {
        self.read_at
    }

    /// Instante UTC de criação da notificação.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Indica se a notificação já foi lida pelo destinatário.
    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_notification_with_unread_status() {
        let id = Uuid::new_v4();
        let demand_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let now = Utc::now();

        let notif = DemandNotification::new(
            id,
            demand_id,
            recipient_id,
            ca_id,
            DemandNotificationEventType::StatusUpdated,
            "Demanda em análise".to_string(),
            "O status da sua demanda foi atualizado para em análise".to_string(),
            now,
        )
        .expect("deve criar notificação válida");

        assert_eq!(notif.id(), id);
        assert_eq!(notif.demand_id(), demand_id);
        assert_eq!(notif.recipient_user_id(), recipient_id);
        assert_eq!(notif.academic_center_id(), ca_id);
        assert_eq!(
            notif.event_type(),
            DemandNotificationEventType::StatusUpdated
        );
        assert_eq!(notif.title(), "Demanda em análise");
        assert_eq!(
            notif.message(),
            "O status da sua demanda foi atualizado para em análise"
        );
        assert!(!notif.is_read());
        assert_eq!(notif.read_at(), None);
        assert_eq!(notif.created_at(), now);
    }

    #[test]
    fn should_mark_notification_as_read() {
        let now = Utc::now();
        let mut notif = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            "Título".to_string(),
            "Mensagem".to_string(),
            now,
        )
        .unwrap();

        assert!(!notif.is_read());
        let read_time = now + chrono::Duration::minutes(5);
        notif.mark_as_read(read_time);

        assert!(notif.is_read());
        assert_eq!(notif.read_at(), Some(read_time));

        // Segunda chamada preserva primeira data
        let later = read_time + chrono::Duration::minutes(10);
        notif.mark_as_read(later);
        assert_eq!(notif.read_at(), Some(read_time));
    }

    #[test]
    fn should_reject_empty_or_whitespace_title() {
        let result = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            "   \t  ".to_string(),
            "Mensagem válida".to_string(),
            Utc::now(),
        );

        assert_eq!(result, Err(DemandError::EmptyNotificationTitle));
    }

    #[test]
    fn should_reject_title_exceeding_max_length() {
        let long_title = "a".repeat(MAX_NOTIFICATION_TITLE_LENGTH + 1);
        let result = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            long_title,
            "Mensagem válida".to_string(),
            Utc::now(),
        );

        assert_eq!(
            result,
            Err(DemandError::NotificationTitleTooLong {
                max: MAX_NOTIFICATION_TITLE_LENGTH
            })
        );
    }

    #[test]
    fn should_reject_empty_or_whitespace_message() {
        let result = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            "Título válido".to_string(),
            "   \n\t  ".to_string(),
            Utc::now(),
        );

        assert_eq!(result, Err(DemandError::EmptyNotificationMessage));
    }

    #[test]
    fn should_reject_message_exceeding_max_length() {
        let long_msg = "m".repeat(MAX_NOTIFICATION_MESSAGE_LENGTH + 1);
        let result = DemandNotification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            "Título válido".to_string(),
            long_msg,
            Utc::now(),
        );

        assert_eq!(
            result,
            Err(DemandError::NotificationMessageTooLong {
                max: MAX_NOTIFICATION_MESSAGE_LENGTH
            })
        );
    }

    #[test]
    fn should_reject_reconstitution_when_read_at_is_before_created_at() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);

        let result = DemandNotification::reconstitute(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            DemandNotificationEventType::StatusUpdated,
            "Título".to_string(),
            "Mensagem".to_string(),
            Some(past),
            now,
        );

        assert_eq!(
            result,
            Err(DemandError::InconsistentTimestamps {
                created_at: now,
                updated_at: past
            })
        );
    }

    #[test]
    fn should_parse_event_types_from_string() {
        assert_eq!(
            "status_updated"
                .parse::<DemandNotificationEventType>()
                .unwrap(),
            DemandNotificationEventType::StatusUpdated
        );
        assert_eq!(
            "official_response"
                .parse::<DemandNotificationEventType>()
                .unwrap(),
            DemandNotificationEventType::OfficialResponse
        );
        assert!(
            "invalid_type"
                .parse::<DemandNotificationEventType>()
                .is_err()
        );
    }
}
