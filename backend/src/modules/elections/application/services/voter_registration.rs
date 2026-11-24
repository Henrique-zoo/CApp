//! Executa a consulta e o cadastro da chave eleitoral da própria conta.
//!
//! A identidade vem da API autenticada, a porta criptográfica valida a prova
//! assinada e o serviço decide o aceite com dados mantidos estáveis pelo banco.

use crate::modules::elections::{
    application::{
        error::RegistrationError,
        ports::{RegistrationTransaction, VoterKeyProofVerifier, VoterRegistrationRepository},
        registration::{RegistrationProof, RegistrationStatus, VoterIdentity},
    },
    domain::{
        identifiers::{AccountId, ElectionId},
        rules::registration::{RegistrationDecision, registration_decision},
    },
};

/// Consulta o cadastro ou o voto aceito para a conta autenticada.
///
/// # Parâmetros
/// - `registry`: consulta a situação armazenada para a dupla eleição e conta.
/// - `election_id`: identifica a eleição consultada.
/// - `account_id`: UUID interno da conta obtido na autenticação da requisição.
///
/// # Retorno
/// Informa se o cadastro é exigido, qual chave está vinculada e se a conta já
/// votou. A consulta pode ser feita após o encerramento ou a expiração do
/// vínculo acadêmico. Uma eleição inexistente retorna [`RegistrationError::NotFound`].
pub async fn status(
    registry: &impl VoterRegistrationRepository,
    election_id: ElectionId,
    account_id: AccountId,
) -> Result<RegistrationStatus, RegistrationError> {
    registry.status(election_id, account_id).await
}

/// Confirma ou recupera o cadastro da chave pública do eleitor.
///
/// # Parâmetros
/// - `registry`: lê o contexto e persiste a chave verificada na mesma transação.
/// - `verifier`: confere a assinatura e o vínculo da prova com a conta e a eleição.
/// - `election_id`: identifica a eleição em que a chave será usada.
/// - `identity`: conta autenticada e objeto Microsoft usado na elegibilidade.
/// - `proof`: chave pública e assinatura tipadas, já desserializadas pela API.
///
/// # Retorno
/// Retorna a situação confirmada após a transação. Uma prova inválida retorna
/// [`RegistrationError::InvalidProof`] antes de consultar o repositório. O
/// serviço recupera um cadastro existente com a mesma chave e verifica o
/// prazo e a elegibilidade para um cadastro novo.
pub async fn register(
    registry: &impl VoterRegistrationRepository,
    verifier: &impl VoterKeyProofVerifier,
    election_id: ElectionId,
    identity: VoterIdentity,
    proof: RegistrationProof,
) -> Result<RegistrationStatus, RegistrationError> {
    let public_key = verifier.verify(proof, election_id, identity.account_id)?;

    let (status, window, transaction) = registry.lock_registration(election_id, identity).await?;
    match registration_decision(
        status.requires_registration,
        status.public_key.as_deref(),
        public_key.as_bytes(),
        window.starts_at,
        window.closed,
        window.now,
        window.eligible_until,
    )? {
        RegistrationDecision::Reuse => Ok(status),
        RegistrationDecision::Insert => {
            transaction.save(public_key.as_bytes(), window.now).await?;
            Ok(RegistrationStatus {
                requires_registration: true,
                account_id: Some(identity.account_id),
                public_key: Some(public_key.into_vec()),
                participated: false,
            })
        }
    }
}
