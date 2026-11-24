//! Caso de uso: Encerramento manual de consulta estudantil pela gestão do CA.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::PollApplicationError,
        ports::{AcademicCenterRepository, PollAuthorizationPort, PollRepository},
    },
    domain::entities::{AcademicCenterStatus, StudentPoll},
};

/// Dados de entrada para o comando de encerramento de consulta.
#[derive(Debug, Clone)]
pub struct ClosePollCommand {
    /// Identificador do Centro Acadêmico contextual da rota.
    pub academic_center_id: Uuid,
    /// Identificador da consulta a ser encerrada.
    pub poll_id: Uuid,
    /// Identificador do usuário gestor que está solicitando o encerramento.
    pub closed_by_id: Uuid,
    /// Instante do encerramento.
    pub closed_at: DateTime<Utc>,
}

/// Executa o caso de uso de encerramento da consulta estudantil.
pub async fn execute_close_poll<CA, PR, AP>(
    ca_repo: &CA,
    poll_repo: &PR,
    auth_port: &AP,
    cmd: ClosePollCommand,
) -> Result<StudentPoll, PollApplicationError>
where
    CA: AcademicCenterRepository,
    PR: PollRepository,
    AP: PollAuthorizationPort,
{
    let ca = ca_repo.find_by_id(cmd.academic_center_id).await?.ok_or(
        PollApplicationError::AcademicCenterNotFound(cmd.academic_center_id),
    )?;

    if ca.status() != AcademicCenterStatus::Active {
        return Err(PollApplicationError::AcademicCenterInactive(
            cmd.academic_center_id,
        ));
    }

    let is_permitted = auth_port
        .is_user_permitted_to_manage(cmd.closed_by_id, &ca)
        .await?;

    if !is_permitted {
        return Err(PollApplicationError::UserNotPermittedInCa {
            user_id: cmd.closed_by_id,
            academic_center_id: cmd.academic_center_id,
        });
    }

    let poll = poll_repo
        .find_by_id(cmd.poll_id)
        .await?
        .ok_or(PollApplicationError::PollNotFound(cmd.poll_id))?;

    // Isolamento estrito por Centro Acadêmico
    if poll.academic_center_id() != cmd.academic_center_id {
        return Err(PollApplicationError::PollNotFound(cmd.poll_id));
    }

    if poll.status() == crate::modules::ca::domain::entities::PollStatus::Closed {
        return Err(PollApplicationError::PollClosed);
    }

    let closed_poll = poll_repo
        .close_poll(cmd.poll_id, cmd.closed_by_id, cmd.closed_at)
        .await?;

    Ok(closed_poll)
}
