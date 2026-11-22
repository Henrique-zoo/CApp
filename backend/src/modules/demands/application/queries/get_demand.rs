//! Consulta de detalhes de uma demanda específica do Centro Acadêmico.

use uuid::Uuid;

/// Parâmetros para consulta detalhada de uma demanda com histórico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetDemandQuery {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,

    /// Identificador estável da demanda.
    pub demand_id: Uuid,

    /// Identificador do usuário que realiza a consulta.
    pub user_id: Uuid,
}

impl GetDemandQuery {
    /// Constrói uma nova consulta de detalhes de demanda.
    pub fn new(academic_center_id: Uuid, demand_id: Uuid, user_id: Uuid) -> Self {
        Self {
            academic_center_id,
            demand_id,
            user_id,
        }
    }
}
