//! Contratos dos casos de uso eleitorais com os adaptadores de infraestrutura.
//!
//! A identidade autenticada é associada à eleição antes do novo aceite. A
//! verificação confere a evidência pública, e o adaptador de persistência
//! confirma o voto e suas evidências em uma operação atômica. Uma repetição
//! recupera o aceite existente. As consultas do quadro expõem as evidências
//! persistidas, inclusive o recibo da conta autenticada.

use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio_stream::Stream;

use super::{
    administration::{AdministrationState, BoardCancellation},
    board::{
        BoardClosing, BoardClosure, BoardError, BoardPublication, BoardSnapshot, FrozenBoard,
        NewVote, RecordedBallot, SignedVoteEvidence, VoteState,
    },
    error::{BallotVerificationError, RegistrationError},
    preparation::{
        AuthorityAssignment, AuthorityRegistrationProof, Preparation, PreparedPublication,
    },
    registration::{
        RegistrationProof, RegistrationStatus, RegistrationWindow, VerifiedRegistrationKey,
        VoterIdentity,
    },
    tally::{PendingServerShare, ShareSubmission, StoredShare, TallyEvidence, TallyState},
    verification::BallotVerificationContext,
};

use crate::modules::elections::domain::{
    entities::{
        ballot::{SignedBallot, VerifiedBallot},
        election::ElectionPeriod,
        vote::VoteKey,
    },
    identifiers::{AccountId, AuthorityId, ElectionId},
};

/// Sequência de fragmentos de uma única exportação JSON do quadro público.
/// Cada erro indica que a exportação foi interrompida antes de terminar.
pub type BoardSnapshotStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, BoardError>> + Send>>;

/// Persiste o quadro eleitoral e confirma suas evidências de forma atômica.
///
/// O chamador fornece a identidade autenticada. O adaptador serializa o aceite
/// de votos e o encerramento e só devolve evidências de aceite após o commit.
#[async_trait::async_trait]
pub trait ElectionBoardRepository: Send + Sync {
    /// Transação que conserva o bloqueio até gravar o encerramento.
    type Transaction: BoardClosureTransaction;

    /// Persiste a origem já verificada. Uma origem idêntica recupera o quadro.
    async fn publish(&self, board: BoardPublication) -> Result<BoardSnapshot, BoardError>;

    /// Exporta os artefatos públicos de uma única leitura consistente do banco.
    async fn snapshot(&self, election_id: ElectionId) -> Result<BoardSnapshot, BoardError>;

    /// Lista eleições publicadas cujo prazo terminou e cujo quadro segue aberto.
    async fn due_for_closure(&self) -> Result<Vec<ElectionId>, BoardError>;

    /// Bloqueia a eleição antes de consultar a contagem, o prazo e o acumulador.
    /// Um quadro já encerrado devolve uma transação com contexto ausente.
    async fn lock_closure(
        &self,
        election_id: ElectionId,
    ) -> Result<(Option<BoardClosing>, Self::Transaction), BoardError>;
}

/// Escrita das evidências calculadas pelo serviço de encerramento.
#[async_trait::async_trait]
pub trait BoardClosureTransaction: Send {
    /// Persiste as novas evidências ou recupera o encerramento já publicado.
    async fn finish(self, evidence: Option<BoardClosure>) -> Result<BoardSnapshot, BoardError>;
}

/// Verifica a origem do quadro e assina as evidências de encerramento.
pub trait BoardEvidenceSigner: VoteEvidenceSigner {
    /// Confere o manifesto com a chave confiável.
    fn verify_origin(&self, board: FrozenBoard) -> Result<BoardPublication, BoardError>;

    /// Assina o termo e associa o agregado ao hash desse termo.
    fn sign_closure(&self, closing: BoardClosing) -> Result<BoardClosure, BoardError>;
}

/// Lê as evidências do quadro publicadas e o recibo da conta autenticada.
#[async_trait::async_trait]
pub trait ElectionBoardReader: Send + Sync {
    /// Inicia uma leitura consistente. Devolve [`BoardError::NotFound`] antes
    /// de abrir o fluxo quando a eleição não tem quadro publicado.
    async fn stream_snapshot(
        &self,
        election_id: ElectionId,
    ) -> Result<BoardSnapshotStream, BoardError>;

    /// Obtém o último recibo assinado ou o manifesto quando o quadro está vazio.
    async fn read_checkpoint(&self, election_id: ElectionId) -> Result<Value, BoardError>;

    /// Busca o recibo original da conta autenticada. `Ok(None)` indica que
    /// não há recibo para essa conta na eleição solicitada.
    async fn find_receipt(&self, key: VoteKey) -> Result<Option<Value>, BoardError>;
}

/// Verifica assinaturas de cédulas e produz a evidência usada no aceite.
///
/// A implementação não consulta cadastro ou manifesto. A aplicação e o
/// domínio conferem essas informações antes de confirmar o aceite.
pub trait BallotVerifier: Send + Sync {
    /// Usa a chave pública da cédula para verificar a assinatura e calcula o
    /// digest do payload, que inclui a chave pública. A cédula confidencial já
    /// contém as cifras validadas durante sua construção.
    fn verify_ballot(
        &self,
        ballot: SignedBallot,
    ) -> Result<VerifiedBallot, BallotVerificationError>;
}

/// Emite evidências públicas de uma cédula autenticada.
pub trait VoteEvidenceSigner: Send + Sync {
    /// Chave pública SPKI no formato publicado no manifesto.
    fn public_key(&self) -> Result<String, BoardError>;

    /// Encadeia a entrada e assina o recibo usando o índice reservado pelo serviço.
    fn sign_vote(
        &self,
        ballot: &VerifiedBallot,
        index: u32,
        previous_hash: &str,
    ) -> Result<SignedVoteEvidence, BoardError>;
}

/// Consulta o contexto do aceite e persiste a participação eleitoral.
#[async_trait::async_trait]
pub trait VoteRepository: Send + Sync {
    /// Transação que conserva o bloqueio até gravar ou descartar a operação.
    type Transaction: VoteTransaction;

    /// Lê a configuração imutável, antes de verificar a assinatura do eleitor.
    async fn ballot_context(
        &self,
        election_id: ElectionId,
    ) -> Result<BallotVerificationContext, BoardError>;

    /// Bloqueia a eleição e lê o voto anterior ou os dados de um novo aceite.
    async fn lock_vote(
        &self,
        election_id: ElectionId,
        voter: VoterIdentity,
    ) -> Result<(VoteState, Self::Transaction), BoardError>;
}

/// Escrita atômica do voto, do registro, do recibo e do acumulador.
#[async_trait::async_trait]
pub trait VoteTransaction: Send {
    /// Grava a decisão do serviço e só devolve o recibo depois do commit.
    async fn save(self, vote: NewVote) -> Result<RecordedBallot, BoardError>;
}

/// Verifica uma prova de posse tipada, sem interpretar HTTP ou JSON.
pub trait VoterKeyProofVerifier: Send + Sync {
    /// Verifica a prova de posse da chave privada correspondente à chave
    /// pública apresentada, vinculando-a à eleição e à conta autenticada.
    ///
    /// Retorna a chave pública verificada, sem decidir a elegibilidade
    /// do eleitor nem efetuar seu cadastro.
    fn verify(
        &self,
        proof: RegistrationProof,
        election_id: ElectionId,
        account_id: AccountId,
    ) -> Result<VerifiedRegistrationKey, RegistrationError>;
}

/// Consulta e persiste o cadastro privado antes da abertura.
#[async_trait::async_trait]
pub trait VoterRegistrationRepository: Send + Sync {
    /// Transação que mantém o estado do cadastro estável.
    type Transaction: RegistrationTransaction;

    /// Consulta o cadastro ou voto aceito da própria conta.
    async fn status(
        &self,
        election_id: ElectionId,
        account_id: AccountId,
    ) -> Result<RegistrationStatus, RegistrationError>;

    /// Obtém os dados necessários ao cadastro sob o bloqueio da eleição.
    async fn lock_registration(
        &self,
        election_id: ElectionId,
        identity: VoterIdentity,
    ) -> Result<(RegistrationStatus, RegistrationWindow, Self::Transaction), RegistrationError>;
}

/// Escrita do cadastro decidido pelo serviço.
#[async_trait::async_trait]
pub trait RegistrationTransaction: Send {
    /// Persiste a chave pela dupla eleição/conta e confirma a transação.
    async fn save(
        self,
        public_key: &[u8],
        registered_at: DateTime<Utc>,
    ) -> Result<(), RegistrationError>;
}

/// Verifica as evidências que autorizam uma apuração confidencial e assina seu resultado.
pub trait TallyVerifier: Send + Sync {
    /// Agregado associado ao manifesto e a um termo de encerramento verificado.
    type Context: Send + Sync;
    /// Contribuição autenticada para o contexto.
    type Share: Send + Sync;

    /// Verifica a cadeia completa e recompõe o agregado a partir das cédulas.
    fn verify_tally(&self, board: &BoardSnapshot) -> Result<Self::Context, BoardError>;

    /// Autentica uma contribuição tipada e produz sua evidência pública.
    fn verify_submission(
        &self,
        context: &Self::Context,
        submission: &ShareSubmission,
    ) -> Result<(StoredShare, Self::Share), BoardError>;

    /// Verifica um artefato público já produzido ou recuperado do quadro.
    /// Não é uma interface de entrada HTTP.
    fn verify_share(
        &self,
        context: &Self::Context,
        envelope: &Value,
    ) -> Result<(AuthorityId, String, Self::Share), BoardError>;

    /// Emite a contribuição da autoridade que pertence ao servidor.
    fn sign_server_share(
        &self,
        context: &Self::Context,
        pending: &PendingServerShare,
    ) -> Result<Value, BoardError>;

    /// Assina o resultado calculado a partir das contribuições verificadas.
    fn sign_result(
        &self,
        context: &Self::Context,
        shares: &[Self::Share],
    ) -> Result<Value, BoardError>;
}

/// Lê a apuração do quadro e serializa os aceites de contribuições.
#[async_trait::async_trait]
pub trait TallyRepository: Send + Sync {
    /// Transação que mantém o conjunto de contribuições estável.
    type Transaction: TallyTransaction;

    /// Obtém a apuração pública da eleição encerrada.
    async fn read_tally(&self, election: ElectionId) -> Result<TallyEvidence, BoardError>;

    /// Lê as chaves próprias do servidor ainda pendentes após o fechamento.
    async fn pending_server_shares(&self) -> Result<Vec<PendingServerShare>, BoardError>;

    /// Bloqueia a publicação da contribuição da autoridade interna.
    async fn lock_server_share(
        &self,
        pending: &PendingServerShare,
    ) -> Result<(TallyState, Self::Transaction), BoardError>;

    /// Rejeita contas sem atribuição antes do trabalho criptográfico.
    async fn check_assignment(
        &self,
        election: ElectionId,
        authority: AuthorityId,
        account: AccountId,
    ) -> Result<(), BoardError>;

    /// Confere a atribuição da autoridade à conta e bloqueia os aceites do pleito.
    async fn lock_tally(
        &self,
        election: ElectionId,
        authority: AuthorityId,
        account: AccountId,
    ) -> Result<(TallyState, Self::Transaction), BoardError>;
}

/// Persistência atômica de uma contribuição e do resultado que ela produz.
#[async_trait::async_trait]
pub trait TallyTransaction: Send {
    /// Grava os dois artefatos e confirma a transação.
    async fn save(self, share: &StoredShare, result: &Value) -> Result<(), BoardError>;
}

/// Assinaturas e provas usadas na preparação de uma eleição confidencial.
pub trait PreparationSigner: Send + Sync {
    /// Confere que a conta designada apresentou uma chave e sua prova de posse.
    fn verify_credential(&self, proof: &AuthorityRegistrationProof) -> Result<Value, BoardError>;

    /// Gera a chave do servidor, valida o conjunto e assina o manifesto.
    fn prepare_publication(&self, setup: &Preparation) -> Result<PreparedPublication, BoardError>;
}

/// Consultas e transações de preparação do manifesto.
#[async_trait::async_trait]
pub trait PreparationRepository: Send + Sync {
    /// Transação que conserva a preparação estável até a decisão do serviço.
    type Transaction: PreparationTransaction;

    /// Lê a designação pertencente à conta autenticada.
    async fn assignment(
        &self,
        election: ElectionId,
        account: AccountId,
    ) -> Result<AuthorityAssignment, BoardError>;

    /// Lê a configuração e as credenciais sob bloqueio.
    async fn lock_preparation(
        &self,
        election: ElectionId,
        account: AccountId,
    ) -> Result<(Preparation, Self::Transaction), BoardError>;
}

/// Persiste a decisão sobre credencial ou publicação.
#[async_trait::async_trait]
pub trait PreparationTransaction: Send {
    /// Confirma a credencial da conta enquanto o quadro segue em preparação.
    async fn save_credential(
        self,
        authority: AuthorityId,
        credential: &Value,
    ) -> Result<(), BoardError>;

    /// Confirma manifesto, período, custódia do servidor e designações no mesmo commit.
    async fn publish(self, publication: &PreparedPublication) -> Result<(), BoardError>;
}

/// Bloqueia uma eleição para administração pela comissão designada.
#[async_trait::async_trait]
pub trait ElectionAdministrationRepository: Send + Sync {
    /// Transação que conserva o estado carregado.
    type Transaction: ElectionAdministrationTransaction;

    /// Verifica a designação da conta e lê o estado após o bloqueio.
    async fn lock_administration(
        &self,
        election: ElectionId,
        actor: AccountId,
    ) -> Result<(AdministrationState, Self::Transaction), BoardError>;
}

/// Grava mudanças decididas pela aplicação e mantém o histórico transacional.
#[async_trait::async_trait]
pub trait ElectionAdministrationTransaction: Send {
    /// Grava o novo período e o motivo no mesmo commit.
    async fn reschedule(
        self,
        state: &AdministrationState,
        period: &ElectionPeriod,
        reason: &str,
    ) -> Result<(), BoardError>;

    /// Registra o cancelamento conservando quadro e votos existentes.
    async fn cancel(self, cancellation: &BoardCancellation) -> Result<(), BoardError>;
}
