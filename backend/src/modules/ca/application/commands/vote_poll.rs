//! Caso de uso: Registro de voto único de estudante na consulta estudantil.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::PollApplicationError,
        ports::{PollAuthorizationPort, AcademicCenterRepository, PollRepository},
    },
    domain::entities::{AcademicCenterStatus, PollVote},
};

/// Dados de entrada para o comando de registro de voto.
#[derive(Debug, Clone)]
pub struct VotePollCommand {
    /// Identificador do Centro Acadêmico contextual da rota.
    pub academic_center_id: Uuid,
    /// Identificador da consulta/enquete votada.
    pub poll_id: Uuid,
    /// Identificador da opção de voto selecionada.
    pub option_id: Uuid,
    /// Identificador do estudante votante autenticado.
    pub user_id: Uuid,
    /// Instante do voto.
    pub voted_at: DateTime<Utc>,
}

/// Executa o caso de uso de registro de voto.
pub async fn execute_vote_poll<CA, PR, AP>(
    ca_repo: &CA,
    poll_repo: &PR,
    auth_port: &AP,
    cmd: VotePollCommand,
) -> Result<PollVote, PollApplicationError>
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

    let poll = poll_repo
        .find_by_id(cmd.poll_id)
        .await?
        .ok_or(PollApplicationError::PollNotFound(cmd.poll_id))?;

    // Isolamento estrito por Centro Acadêmico
    if poll.academic_center_id() != cmd.academic_center_id {
        return Err(PollApplicationError::PollNotFound(cmd.poll_id));
    }

    // Valida período e status da consulta
    poll.can_vote(cmd.voted_at)?;

    // Valida existência da opção na consulta
    if !poll.has_option(cmd.option_id) {
        return Err(PollApplicationError::OptionNotFound(cmd.option_id));
    }

    // Valida elegibilidade do estudante
    let eligible = auth_port
        .is_student_eligible_to_vote(cmd.user_id, &poll)
        .await?;

    if !eligible {
        return Err(PollApplicationError::StudentNotEligible {
            user_id: cmd.user_id,
            poll_id: cmd.poll_id,
        });
    }

    // Valida unicidade de voto
    let already_voted = poll_repo.has_user_voted(cmd.poll_id, cmd.user_id).await?;
    if already_voted {
        return Err(PollApplicationError::DuplicateVote {
            user_id: cmd.user_id,
            poll_id: cmd.poll_id,
        });
    }

    let vote = PollVote::new(cmd.poll_id, cmd.option_id, cmd.user_id, cmd.voted_at);
    poll_repo.save_vote(&vote).await?;

    Ok(vote)
}
