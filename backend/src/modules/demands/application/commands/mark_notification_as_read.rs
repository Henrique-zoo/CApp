//! Comando para marcação de leitura de uma notificação de demanda.

use uuid::Uuid;

/// Dados necessários para marcar uma notificação como lida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkDemandNotificationAsReadCommand {
    /// Identificador da notificação a ser marcada.
    pub notification_id: Uuid,
    /// Identificador do estudante autenticado solicitante.
    pub user_id: Uuid,
}

impl MarkDemandNotificationAsReadCommand {
    /// Cria uma nova instância do comando.
    pub fn new(notification_id: Uuid, user_id: Uuid) -> Self {
        Self {
            notification_id,
            user_id,
        }
    }
}
