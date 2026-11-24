//! Parâmetros de consulta para listagem de notificações de demandas do estudante.

use uuid::Uuid;

/// Consulta para listar as notificações destinadas a um estudante autenticado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListDemandNotificationsQuery {
    /// Identificador do estudante destinatário das notificações.
    pub recipient_user_id: Uuid,
    /// Se verdadeiro, retorna apenas notificações ainda não lidas (`read_at IS NULL`).
    pub unread_only: bool,
    /// Limite máximo de registros retornados.
    pub limit: Option<usize>,
    /// Deslocamento inicial de paginação.
    pub offset: Option<usize>,
}

impl ListDemandNotificationsQuery {
    /// Constrói uma nova consulta com os parâmetros informados.
    pub fn new(
        recipient_user_id: Uuid,
        unread_only: bool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Self {
        Self {
            recipient_user_id,
            unread_only,
            limit,
            offset,
        }
    }
}
