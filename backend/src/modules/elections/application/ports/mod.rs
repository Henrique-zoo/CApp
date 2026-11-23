//! Contratos dos casos de uso eleitorais com os adaptadores de infraestrutura.
//!
//! A identidade autenticada é associada à eleição antes do novo aceite. A
//! verificação confere a evidência pública, e o adaptador de persistência
//! confirma o voto e suas evidências em uma operação atômica. Uma repetição
//! recupera o aceite existente.

use crate::modules::elections::{
    application::error::{BallotVerificationError, RecordBallotError},
    domain::{
        cryptography::elgamal::ElGamalCiphertext,
        entities::{
            ballot::{SignedBallot, VerifiedBallot},
            vote::{Receipt, VoteKey},
        },
    },
};

/// Verifica a assinatura da cédula antes do aceite.
#[async_trait::async_trait]
pub trait BallotVerifier: Send + Sync {
    /// Usa a chave pública da cédula para verificar a assinatura e calcula o
    /// digest do payload, que inclui a chave pública. A cédula confidencial já
    /// contém as cifras validadas durante sua construção.
    async fn verify_ballot(
        &self,
        ballot: SignedBallot,
    ) -> Result<VerifiedBallot, BallotVerificationError>;
}

/// Persiste votos e permite recuperar recibos confidenciais.
#[async_trait::async_trait]
pub trait BallotStore: Send + Sync {
    /// Confirma um voto da conta autenticada e retorna o recibo na modalidade
    /// confidencial ou `Ok(None)` na identificada.
    /// O chamador monta `key` a partir da conta CApp autenticada e da eleição
    /// solicitada.
    ///
    /// Em uma repetição, `Vote::retry` confere a chave, o identificador de envio
    /// e o digest antes das regras de novo aceite. O repositório devolve o
    /// mesmo recibo persistido e preserva a contagem e o acumulador.
    ///
    /// Em um novo aceite, compara o curso da evidência institucional vigente
    /// da conta com o curso do CA da eleição. Curso ausente ou diferente
    /// resulta em [`RecordBallotError::NotEligible`]. Também confere estado e
    /// prazo com `Vote::try_accept`. Na modalidade confidencial,
    /// `updated_accumulator` contém as cifras na ordem do manifesto, calculadas
    /// a partir da revisão `expected_version`. Na identificada, o chamador
    /// fornece `None` no acumulador.
    ///
    /// A linha do voto guarda a cédula e sua chave pública de assinatura para
    /// o par de eleição e conta em `key`. A mesma transação atualiza a contagem
    /// e grava, quando aplicável, o registro encadeado, o recibo e o acumulador.
    /// Só devolve o recibo após o commit. Conflitos de revisão e recusas de voto
    /// retornam erro.
    async fn record_ballot(
        &self,
        key: VoteKey,
        ballot: VerifiedBallot,
        expected_version: u64,
        updated_accumulator: Option<Vec<ElGamalCiphertext>>,
    ) -> Result<Option<Receipt>, RecordBallotError>;

    /// Recupera o recibo original pela chave da conta autenticada, inclusive
    /// em outro dispositivo. `Ok(None)` indica que a conta não tem recibo
    /// nessa eleição.
    async fn find_receipt(&self, key: VoteKey) -> Result<Option<Receipt>, RecordBallotError>;
}
