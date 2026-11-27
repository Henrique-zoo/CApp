//! Coordena a entrega de chaves e o congelamento do manifesto confidencial.

use super::super::{
    board::BoardError,
    ports::{PreparationRepository, PreparationSigner, PreparationTransaction},
    preparation::{AuthorityAssignment, AuthorityRegistrationProof},
};
use crate::modules::elections::domain::{
    entities::authority::AuthorityRole,
    identifiers::{AccountId, ElectionId},
};

/// Consulta a atribuição pertencente à sessão atual.
pub async fn assignment(
    repository: &impl PreparationRepository,
    election: ElectionId,
    account: AccountId,
) -> Result<AuthorityAssignment, BoardError> {
    repository.assignment(election, account).await
}

/// Recebe a credencial assinada de uma conta explicitamente designada.
pub async fn register_authority(
    repository: &impl PreparationRepository,
    signer: &impl PreparationSigner,
    election: ElectionId,
    account: AccountId,
    proof: &AuthorityRegistrationProof,
) -> Result<AuthorityAssignment, BoardError> {
    let (preparation, transaction) = repository.lock_preparation(election, account).await?;
    if preparation.now >= *preparation.period.starts_at() {
        return Err(BoardError::VotingEnded);
    }
    let authority = preparation
        .authorities
        .iter()
        .find(|authority| authority.account == account)
        .ok_or(BoardError::AuthorityNotAssigned)?;
    if proof.election != election
        || proof.authority.id() != authority.id
        || proof.authority.role() != authority.role
        || authority.role == AuthorityRole::Server
    {
        return Err(BoardError::AuthorityNotAssigned);
    }
    let credential = signer.verify_credential(proof)?;

    if let Some(previous) = &authority.credential {
        if previous != &credential {
            return Err(BoardError::ShareAlreadyAccepted);
        }
        drop(transaction);
    } else {
        transaction
            .save_credential(authority.id, &credential)
            .await?;
    }
    repository.assignment(election, account).await
}

/// A comissão designada publica o manifesto quando todas as provas estão prontas.
pub async fn publish_manifest(
    repository: &impl PreparationRepository,
    signer: &impl PreparationSigner,
    election: ElectionId,
    account: AccountId,
) -> Result<AuthorityAssignment, BoardError> {
    let current = repository.assignment(election, account).await?;
    if current.role != "commission" {
        return Err(BoardError::AuthorityNotAssigned);
    }
    if current.manifest_hash.is_some() {
        return Ok(current);
    }
    let locked = repository.lock_preparation(election, account).await;
    let (preparation, transaction) = match locked {
        Err(BoardError::AlreadyExists) => return repository.assignment(election, account).await,
        other => other?,
    };
    if !preparation.authorities.iter().any(|authority| {
        authority.role == AuthorityRole::Commission && authority.account == account
    }) {
        return Err(BoardError::AuthorityNotAssigned);
    }
    if preparation.now >= *preparation.period.starts_at() {
        return Err(BoardError::VotingEnded);
    }
    if preparation
        .authorities
        .iter()
        .any(|authority| authority.credential.is_none())
    {
        return Err(BoardError::PreparationIncomplete);
    }

    let publication = signer.prepare_publication(&preparation)?;
    transaction.publish(&publication).await?;
    repository.assignment(election, account).await
}
