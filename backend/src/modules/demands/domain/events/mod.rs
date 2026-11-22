//! Eventos de domínio do módulo de demandas.
//!
//! Representam os fatos relevantes ocorridos no ciclo de vida das demandas,
//! tais como abertura de solicitações e transições de status.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::demands::domain::entities::{
    DemandError, DemandNotification, DemandNotificationEventType, DemandStatus,
};

/// Evento emitido quando o status de uma demanda é atualizado pela gestão do CA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandStatusUpdatedEvent {
    /// Identificador da demanda alterada.
    pub demand_id: Uuid,
    /// Identificador do Centro Acadêmico responsável.
    pub academic_center_id: Uuid,
    /// Identificador do estudante autor da solicitação.
    pub author_id: Uuid,
    /// Identificador do gestor que realizou a alteração.
    pub changed_by_id: Uuid,
    /// Status anterior, se houver.
    pub from_status: Option<DemandStatus>,
    /// Novo status assumido pela demanda.
    pub to_status: DemandStatus,
    /// Parecer ou justificativa registrada pela gestão.
    pub opinion: String,
    /// Instante da ocorrência do evento em UTC.
    pub occurred_at: DateTime<Utc>,
}

impl DemandStatusUpdatedEvent {
    /// Converte o evento de transição de status em notificação para o estudante autor.
    ///
    /// # Regra de Negócio
    /// Nunca notificar o próprio usuário por uma ação executada por ele mesmo.
    /// Retorna `None` se o autor da ação for o próprio autor da demanda.
    pub fn to_notification(
        &self,
        notification_id: Uuid,
    ) -> Option<Result<DemandNotification, DemandError>> {
        if self.changed_by_id == self.author_id {
            return None;
        }

        let status_label = match self.to_status {
            DemandStatus::Open => "aberta",
            DemandStatus::InAnalysis => "em análise",
            DemandStatus::ForwardedToCoordination => "encaminhada à coordenação",
            DemandStatus::Completed => "concluída",
        };

        let title = format!("Demanda {status_label}");
        let message = format!(
            "O status da sua demanda foi alterado para '{status_label}'. Parecer da gestão: {}",
            self.opinion.trim()
        );

        Some(DemandNotification::new(
            notification_id,
            self.demand_id,
            self.author_id,
            self.academic_center_id,
            DemandNotificationEventType::StatusUpdated,
            title,
            message,
            self.occurred_at,
        ))
    }
}

/// Evento emitido quando uma resposta ou comunicação oficial do CA é registrada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandOfficialResponseEvent {
    /// Identificador da demanda associada.
    pub demand_id: Uuid,
    /// Identificador do Centro Acadêmico responsável.
    pub academic_center_id: Uuid,
    /// Identificador do estudante autor da demanda.
    pub author_id: Uuid,
    /// Identificador do membro do CA que respondeu.
    pub responder_id: Uuid,
    /// Conteúdo da resposta oficial.
    pub response_text: String,
    /// Instante da ocorrência do evento em UTC.
    pub occurred_at: DateTime<Utc>,
}

impl DemandOfficialResponseEvent {
    /// Converte a resposta oficial em notificação para o autor.
    ///
    /// # Regra de Negócio
    /// Retorna `None` se o membro que respondeu for o próprio autor da demanda.
    pub fn to_notification(
        &self,
        notification_id: Uuid,
    ) -> Option<Result<DemandNotification, DemandError>> {
        if self.responder_id == self.author_id {
            return None;
        }

        Some(DemandNotification::new(
            notification_id,
            self.demand_id,
            self.author_id,
            self.academic_center_id,
            DemandNotificationEventType::OfficialResponse,
            "Nova resposta oficial do CA".to_string(),
            format!(
                "O Centro Acadêmico respondeu à sua demanda: {}",
                self.response_text.trim()
            ),
            self.occurred_at,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_notification_when_manager_updates_student_demand() {
        let demand_id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let now = Utc::now();

        let event = DemandStatusUpdatedEvent {
            demand_id,
            academic_center_id: ca_id,
            author_id,
            changed_by_id: manager_id,
            from_status: Some(DemandStatus::Open),
            to_status: DemandStatus::InAnalysis,
            opinion: "Iniciada vistoria técnica".to_string(),
            occurred_at: now,
        };

        let notif_id = Uuid::new_v4();
        let maybe_notif = event.to_notification(notif_id);
        assert!(maybe_notif.is_some());
        let notif = maybe_notif.unwrap().expect("notificação válida");

        assert_eq!(notif.id(), notif_id);
        assert_eq!(notif.demand_id(), demand_id);
        assert_eq!(notif.recipient_user_id(), author_id);
        assert_eq!(notif.academic_center_id(), ca_id);
        assert_eq!(
            notif.event_type(),
            DemandNotificationEventType::StatusUpdated
        );
        assert_eq!(notif.title(), "Demanda em análise");
        assert!(notif.message().contains("Iniciada vistoria técnica"));
        assert!(!notif.is_read());
    }

    #[test]
    fn should_not_generate_notification_when_author_updates_own_demand() {
        let author_id = Uuid::new_v4();
        let event = DemandStatusUpdatedEvent {
            demand_id: Uuid::new_v4(),
            academic_center_id: Uuid::new_v4(),
            author_id,
            changed_by_id: author_id, // Mesmo usuário!
            from_status: Some(DemandStatus::Open),
            to_status: DemandStatus::InAnalysis,
            opinion: "Atualizando própria demanda".to_string(),
            occurred_at: Utc::now(),
        };

        let maybe_notif = event.to_notification(Uuid::new_v4());
        assert!(
            maybe_notif.is_none(),
            "nunca deve notificar o próprio autor"
        );
    }

    #[test]
    fn should_generate_notification_for_official_response() {
        let author_id = Uuid::new_v4();
        let responder_id = Uuid::new_v4();
        let event = DemandOfficialResponseEvent {
            demand_id: Uuid::new_v4(),
            academic_center_id: Uuid::new_v4(),
            author_id,
            responder_id,
            response_text: "O material foi adquirido e entregue".to_string(),
            occurred_at: Utc::now(),
        };

        let notif = event
            .to_notification(Uuid::new_v4())
            .expect("deve gerar notificação")
            .expect("válida");

        assert_eq!(notif.recipient_user_id(), author_id);
        assert_eq!(
            notif.event_type(),
            DemandNotificationEventType::OfficialResponse
        );
        assert_eq!(notif.title(), "Nova resposta oficial do CA");
    }
}
