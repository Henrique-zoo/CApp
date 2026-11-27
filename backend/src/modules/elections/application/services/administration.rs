//! Valida alterações de período e cancelamento, mantendo o controle por comissão.

use super::super::{
    administration::{BoardCancellation, UpdatedPeriod},
    board::BoardError,
    ports::{ElectionAdministrationRepository, ElectionAdministrationTransaction},
};
use crate::modules::elections::domain::{
    entities::election::ElectionPeriod,
    identifiers::{AccountId, ElectionId},
};

fn reason_text(value: &str) -> Result<&str, BoardError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 2000 {
        return Err(BoardError::InvalidAdministration(
            "informe um motivo de até 2.000 caracteres",
        ));
    }
    Ok(trimmed)
}

/// Adia a abertura da eleição ou prorroga o prazo sem modificar o manifesto.
pub async fn reschedule(
    repository: &impl ElectionAdministrationRepository,
    election: ElectionId,
    actor: AccountId,
    period: ElectionPeriod,
    reason: &str,
) -> Result<UpdatedPeriod, BoardError> {
    let reason = reason_text(reason)?;
    if period.starts_at().timestamp_subsec_nanos() != 0
        || period.ends_at().timestamp_subsec_nanos() != 0
    {
        return Err(BoardError::InvalidAdministration(
            "informe os horários com precisão de segundos",
        ));
    }
    let (state, transaction) = repository.lock_administration(election, actor).await?;
    if state.cancellation.is_some() {
        return Err(BoardError::Cancelled);
    }
    if state.closed || state.now >= *state.period.ends_at() {
        return Err(BoardError::Closed);
    }
    let opened_at = (state.published && state.now >= *state.period.starts_at())
        .then_some(*state.period.starts_at());
    state
        .period
        .validate_revision(&period, opened_at, state.now)
        .map_err(|_| {
            BoardError::InvalidAdministration(
                "só é permitido adiar a abertura futura e prorrogar o fim",
            )
        })?;

    transaction.reschedule(&state, &period, reason).await?;
    Ok(UpdatedPeriod {
        starts_at: *period.starts_at(),
        ends_at: *period.ends_at(),
    })
}

/// Cancela uma eleição sem publicar apuração parcial.
pub async fn cancel(
    repository: &impl ElectionAdministrationRepository,
    election: ElectionId,
    actor: AccountId,
    reason: &str,
) -> Result<BoardCancellation, BoardError> {
    let reason = reason_text(reason)?;
    let (state, transaction) = repository.lock_administration(election, actor).await?;
    if let Some(previous) = state.cancellation {
        return Ok(previous);
    }
    if state.closed || state.now >= *state.period.ends_at() {
        return Err(BoardError::Closed);
    }
    let cancellation = BoardCancellation {
        cancelled_at: state.now,
        cancelled_by: actor.into_uuid(),
        reason: reason.to_owned(),
    };
    transaction.cancel(&cancellation).await?;
    Ok(cancellation)
}
