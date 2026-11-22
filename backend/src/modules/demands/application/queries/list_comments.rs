//! Consulta para listagem de comentários de uma demanda.

use uuid::Uuid;

/// Parâmetros da consulta para listagem dos comentários de uma demanda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListDemandCommentsQuery {
    /// Identificador do Centro Acadêmico responsável pela demanda.
    pub academic_center_id: Uuid,

    /// Identificador da demanda alvo.
    pub demand_id: Uuid,

    /// Identificador do usuário solicitante (para validação de autorização).
    pub user_id: Uuid,
}

impl ListDemandCommentsQuery {
    /// Cria uma nova consulta para listagem de comentários.
    pub fn new(academic_center_id: Uuid, demand_id: Uuid, user_id: Uuid) -> Self {
        Self {
            academic_center_id,
            demand_id,
            user_id,
        }
    }
}
