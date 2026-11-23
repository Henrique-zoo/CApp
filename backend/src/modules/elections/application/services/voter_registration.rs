//! Executa a consulta e o cadastro da chave eleitoral da própria conta.
//!
//! A identidade vem da API autenticada, a porta criptográfica valida a prova
//! assinada e o repositório aplica as condições que exigem uma transação.

use uuid::Uuid;

use super::super::{
    error::RegistrationError,
    ports::{VoterKeyProofVerifier, VoterRegistry},
    registration::{RegistrationStatus, VoterIdentity},
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
    registry: &impl VoterRegistry,
    election_id: Uuid,
    account_id: Uuid,
) -> Result<RegistrationStatus, RegistrationError> {
    registry.status(election_id, account_id).await
}

/// Confirma ou recupera o cadastro da chave pública do eleitor.
///
/// # Parâmetros
/// - `registry`: verifica as condições de cadastro e persiste a chave validada
///   na mesma transação.
/// - `verifier`: confere a assinatura e o vínculo da prova com a conta e a eleição.
/// - `election_id`: identifica a eleição em que a chave será usada.
/// - `identity`: conta autenticada e objeto Microsoft usado na elegibilidade.
/// - `proof`: bytes originais da prova de posse assinada pelo dispositivo.
///
/// # Retorno
/// Retorna a situação confirmada após a transação. Uma prova inválida retorna
/// [`RegistrationError::InvalidProof`] antes de consultar o repositório. O
/// repositório recupera um cadastro existente com a mesma chave e verifica o
/// prazo e a elegibilidade para um cadastro novo.
pub async fn register(
    registry: &impl VoterRegistry,
    verifier: &impl VoterKeyProofVerifier,
    election_id: Uuid,
    identity: VoterIdentity,
    proof: &[u8],
) -> Result<RegistrationStatus, RegistrationError> {
    let public_key = verifier.verify(proof, election_id, identity.account_id)?;

    registry
        .register_verified_key(election_id, identity, public_key)
        .await
}
