//! Caso de uso: Listagem de consultas estudantis de um Centro Acadêmico.

use uuid::Uuid;

use crate::modules::ca::{
    application::{
        error::PollApplicationError,
        ports::{AcademicCenterRepository, PollRepository},
    },
    domain::entities::{PollResults, StudentPoll},
};

/// Dados de entrada para listagem de consultas.
#[derive(Debug, Clone)]
pub struct ListPollsQuery {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
}

/// Executa o caso de uso de listagem de consultas com apuração.
pub async fn execute_list_polls<CA, PR>(
    ca_repo: &CA,
    poll_repo: &PR,
    query: ListPollsQuery,
) -> Result<Vec<(StudentPoll, PollResults)>, PollApplicationError>
where
    CA: AcademicCenterRepository,
    PR: PollRepository,
{
    let _ca = ca_repo.find_by_id(query.academic_center_id).await?.ok_or(
        PollApplicationError::AcademicCenterNotFound(query.academic_center_id),
    )?;

    let polls = poll_repo
        .find_all_by_ca_id(query.academic_center_id)
        .await?;

    let mut list = Vec::with_capacity(polls.len());
    for poll in polls {
        let vote_counts = poll_repo.count_votes_by_option(poll.id()).await?;
        let results = poll.tally_results(&vote_counts);
        list.push((poll, results));
    }

    Ok(list)
}
