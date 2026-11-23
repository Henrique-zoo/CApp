//! Comando para atualização de status de uma demanda no Centro Acadêmico.

use uuid::Uuid;

use crate::modules::demands::domain::entities::DemandStatus;

/// Parâmetros de comando para alteração de status de uma demanda pela gestão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateDemandStatusCommand {
    /// Identificador do Centro Acadêmico responsável pela demanda.
    pub academic_center_id: Uuid,

    /// Identificador estável da demanda a ser atualizada.
    pub demand_id: Uuid,

    /// Identificador do membro da gestão que realiza a transição.
    pub manager_id: Uuid,

    /// Novo status pretendido no ciclo de vida da demanda.
    pub new_status: DemandStatus,

    /// Parecer ou justificativa obrigatória registrada para a transição.
    pub opinion: String,
}

impl UpdateDemandStatusCommand {
    /// Constrói um novo comando de atualização de status.
    pub fn new(
        academic_center_id: Uuid,
        demand_id: Uuid,
        manager_id: Uuid,
        new_status: DemandStatus,
        opinion: String,
    ) -> Self {
        Self {
            academic_center_id,
            demand_id,
            manager_id,
            new_status,
            opinion,
        }
    }
}
