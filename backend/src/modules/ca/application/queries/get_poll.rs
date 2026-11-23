//! Caso de uso: Consulta e apuração de resultados de consulta estudantil.

use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::PollApplicationError,
        ports::{AcademicCenterRepository, PollRepository},
    },
    domain::entities::{PollResults, StudentPoll},
};

/// Dados para a consulta detalhada com apuração de resultados.
#[derive(Debug, Clone)]
pub struct GetPollQuery {
    /// Identificador do Centro Acadêmico contextual da rota.
    pub academic_center_id: Uuid,
    /// Identificador da consulta a ser recuperada.
    pub poll_id: Uuid,
}

/// Executa a recuperação detalhada da consulta com apuração de resultados.
pub async fn execute_get_poll<CA, PR>(
    ca_repo: &CA,
    poll_repo: &PR,
    query: GetPollQuery,
) -> Result<(StudentPoll, PollResults), PollApplicationError>
where
    CA: AcademicCenterRepository,
    PR: PollRepository,
{
    let _ca = ca_repo.find_by_id(query.academic_center_id).await?.ok_or(
        PollApplicationError::AcademicCenterNotFound(query.academic_center_id),
    )?;

    let poll = poll_repo
        .find_by_id(query.poll_id)
        .await?
        .ok_or(PollApplicationError::PollNotFound(query.poll_id))?;

    // Isolamento estrito por Centro Acadêmico
    if poll.academic_center_id() != query.academic_center_id {
        return Err(PollApplicationError::PollNotFound(query.poll_id));
    }

    let vote_counts = poll_repo.count_votes_by_option(query.poll_id).await?;
    let results = poll.tally_results(&vote_counts);

    Ok((poll, results))
}
