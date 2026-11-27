//! Coordena a apuração confidencial após o encerramento confirmado.
//!
//! O verificador confere a cadeia e as contribuições. O repositório conserva
//! a atribuição da autoridade e confirma a contribuição e o resultado juntos.

use super::super::{
    board::BoardError,
    ports::{ElectionBoardRepository, TallyRepository, TallyTransaction, TallyVerifier},
    tally::{PendingServerShare, ShareSubmission, StoredShare, TallyEvidence, TallyState},
};
use crate::modules::elections::domain::identifiers::{AccountId, AuthorityId, ElectionId};

/// Consulta contribuições e resultado da eleição sem acessar material privado.
pub async fn read_result(
    repository: &impl TallyRepository,
    election: ElectionId,
) -> Result<TallyEvidence, BoardError> {
    repository.read_tally(election).await
}

/// Verifica e publica a contribuição entregue pela conta designada.
///
/// A cédula e o agregado já estão protegidos contra alterações após o
/// encerramento. Um reenvio do mesmo conteúdo recupera a evidência persistida.
/// A última contribuição válida permite publicar os totais no mesmo commit.
pub async fn submit_share(
    boards: &impl ElectionBoardRepository,
    repository: &impl TallyRepository,
    verifier: &impl TallyVerifier,
    election: ElectionId,
    authority: AuthorityId,
    account: AccountId,
    submission: ShareSubmission,
) -> Result<TallyEvidence, BoardError> {
    repository
        .check_assignment(election, authority, account)
        .await?;
    let board = boards.snapshot(election).await?;
    if board.cancellation.is_some() {
        return Err(BoardError::Cancelled);
    }
    if board.closure.is_none() {
        return Err(BoardError::NotYetEnded);
    }
    let context = verifier.verify_tally(&board)?;
    let (share, contribution) = verifier.verify_submission(&context, &submission)?;
    if share.authority != authority {
        return Err(BoardError::AuthorityNotAssigned);
    }

    let (state, transaction) = repository.lock_tally(election, authority, account).await?;
    publish_share(verifier, &context, state, transaction, share, contribution).await
}

/// Publica contribuições da autoridade automática para quadros já encerrados.
pub async fn submit_pending_server_shares(
    boards: &impl ElectionBoardRepository,
    repository: &impl TallyRepository,
    verifier: &impl TallyVerifier,
) -> Result<(), BoardError> {
    for pending in repository.pending_server_shares().await? {
        if let Err(error) = submit_server_share(boards, repository, verifier, &pending).await {
            tracing::error!(%error, election_id = %pending.election.as_uuid(), "Falha na contribuição da autoridade do servidor");
        }
    }
    Ok(())
}

async fn submit_server_share(
    boards: &impl ElectionBoardRepository,
    repository: &impl TallyRepository,
    verifier: &impl TallyVerifier,
    pending: &PendingServerShare,
) -> Result<(), BoardError> {
    let board = boards.snapshot(pending.election).await?;
    let context = verifier.verify_tally(&board)?;
    let envelope = verifier.sign_server_share(&context, pending)?;
    let (authority, digest, contribution) = verifier.verify_share(&context, &envelope)?;
    let (state, transaction) = repository.lock_server_share(pending).await?;
    if state
        .shares
        .iter()
        .any(|share| share.authority == authority)
    {
        return Ok(());
    }
    publish_share(
        verifier,
        &context,
        state,
        transaction,
        StoredShare {
            authority,
            digest,
            envelope,
        },
        contribution,
    )
    .await?;
    Ok(())
}

async fn publish_share<V: TallyVerifier>(
    verifier: &V,
    context: &V::Context,
    mut state: TallyState,
    transaction: impl TallyTransaction,
    share: StoredShare,
    contribution: V::Share,
) -> Result<TallyEvidence, BoardError> {
    if let Some(previous) = state
        .shares
        .iter()
        .find(|saved| saved.authority == share.authority)
    {
        if previous.digest == share.digest {
            return Ok(state.into());
        }
        return Err(BoardError::ShareAlreadyAccepted);
    }

    let mut shares = state
        .shares
        .iter()
        .map(|saved| {
            verifier
                .verify_share(context, &saved.envelope)
                .map(|(_, _, share)| share)
        })
        .collect::<Result<Vec<_>, _>>()?;
    shares.push(contribution);
    let result = verifier.sign_result(context, &shares)?;
    transaction.save(&share, &result).await?;
    state.shares.push(share);
    state.result = Some(result);
    Ok(state.into())
}
