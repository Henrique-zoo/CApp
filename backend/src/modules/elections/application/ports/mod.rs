//! Contratos dos casos de uso eleitorais com os adaptadores de infraestrutura.
//!
//! A identidade autenticada é associada à eleição antes do novo aceite. A
//! verificação confere a evidência pública, e o adaptador de persistência
//! confirma o voto e suas evidências em uma operação atômica. Uma repetição
//! recupera o aceite existente. As consultas do quadro expõem as evidências
//! persistidas, inclusive o recibo da conta autenticada.

use std::pin::Pin;

use serde_json::Value;
use tokio_stream::Stream;
use uuid::Uuid;

use super::{
    board::BoardError,
    error::RegistrationError,
    registration::{RegistrationStatus, VoterIdentity},
};

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

/// Sequência de fragmentos de uma única exportação JSON do quadro público.
/// Cada erro indica que a exportação foi interrompida antes de terminar.
pub type BoardSnapshotStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, BoardError>> + Send>>;

/// Lê as evidências do quadro publicadas e o recibo da conta autenticada.
#[async_trait::async_trait]
pub trait ElectionBoardReader: Send + Sync {
    /// Inicia uma leitura consistente. Devolve [`BoardError::NotFound`] antes
    /// de abrir o fluxo quando a eleição não tem quadro publicado.
    async fn stream_snapshot(&self, election_id: Uuid) -> Result<BoardSnapshotStream, BoardError>;

    /// Obtém o último recibo assinado ou o manifesto quando o quadro está vazio.
    async fn read_checkpoint(&self, election_id: Uuid) -> Result<Value, BoardError>;

    /// Busca o recibo original da conta autenticada. `Ok(None)` indica que
    /// não há recibo para essa conta na eleição solicitada.
    async fn find_receipt(
        &self,
        election_id: Uuid,
        authenticated_account_id: Uuid,
    ) -> Result<Option<Value>, BoardError>;
}

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

/// Verifica a prova de posse e retorna a chave SPKI em DER.
pub trait VoterKeyProofVerifier: Send + Sync {
    /// Recebe os bytes originais para detectar JSON ambíguo antes da verificação.
    fn verify(
        &self,
        proof: &[u8],
        election_id: Uuid,
        account_id: Uuid,
    ) -> Result<Vec<u8>, RegistrationError>;
}

/// Consulta e persiste o cadastro privado antes da abertura.
#[async_trait::async_trait]
pub trait VoterRegistry: Send + Sync {
    /// Consulta o cadastro ou voto aceito da própria conta.
    async fn status(
        &self,
        election_id: Uuid,
        account_id: Uuid,
    ) -> Result<RegistrationStatus, RegistrationError>;

    /// Confere prazo, elegibilidade e chave existente sob lock antes de inserir
    /// a chave pública já verificada. Só confirma o novo cadastro após o commit.
    async fn register_verified_key(
        &self,
        election_id: Uuid,
        identity: VoterIdentity,
        public_key: Vec<u8>,
    ) -> Result<RegistrationStatus, RegistrationError>;
}
