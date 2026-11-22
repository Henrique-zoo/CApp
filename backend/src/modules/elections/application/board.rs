//! Porta de persistência do quadro eleitoral público.
//!
//! Identidade, elegibilidade e autorização administrativa vêm de casos de uso
//! autenticados. Esta porta não aceita uma conta declarada no JSON da cédula.
//! A implementação serializa aceite e encerramento e só devolve evidências de
//! aceite depois do commit. Não implementa revisões de calendário da #132.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// Artefatos assinados e imutáveis que iniciam um quadro.
///
/// Os bytes originais permitem recusar JSON ambíguo antes da canonicalização.
/// O adaptador verifica assinaturas, manifesto e referência ao calendário.
pub struct FrozenBoard {
    /// Envelope assinado do manifesto público.
    pub manifest: Vec<u8>,
    /// Envelope assinado do calendário inicial referenciado pelo manifesto.
    pub calendar_origin: Vec<u8>,
}

/// Decisão confiável de identidade/elegibilidade, produzida pelo consumidor #137.
///
/// Nunca construir com campos de identidade ou chave extraídos apenas do corpo
/// da requisição. No fluxo acadêmico, `registered_public_key` vem do cadastro
/// anterior à abertura. O repositório compara a chave da cédula com essa chave.
/// A validade é reavaliada pelo relógio do banco depois da aquisição do lock.
pub struct EligibleVoter {
    /// Conta interna autenticada; não integra a exportação pública.
    pub account_id: Uuid,
    /// Identificador público opaco, estável para a conta nesta eleição.
    pub voter_id: Uuid,
    /// Chave pública confiável do eleitor, em DER SPKI/base64url sem padding.
    pub registered_public_key: String,
    /// Limite exclusivo de validade da evidência institucional.
    pub eligible_until: DateTime<Utc>,
}

/// Registro confirmado e recibo original, preservados na recuperação.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedBallot {
    /// Envelope assinado completo do registro encadeado.
    pub entry: Value,
    /// Envelope assinado original do recibo.
    pub receipt: Value,
    /// Indica recuperação sem nova participação ou alteração do acumulador.
    pub recovered: bool,
}

/// Fotografia pública consistente; não contém contas, tokens ou segredos.
///
/// O wrapper não é assinado. Cada artefato preserva seu envelope do contrato.
/// `checkpoint` é o último recibo: seu índice/hash ancoram a cabeça do quadro.
/// Ele só se torna âncora externa quando o observador o conserva separadamente.
/// O agregado somente é publicado após o encerramento.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardSnapshot {
    /// Manifesto assinado oficial da eleição.
    pub manifest: Value,
    /// Calendário inicial assinado e imutável neste adaptador.
    pub calendar_origin: Value,
    /// Registros assinados em ordem crescente, começando em um.
    pub entries: Vec<Value>,
    /// Último recibo assinado; ausente quando o quadro está vazio.
    pub checkpoint: Option<Value>,
    /// Termo assinado do encerramento, quando confirmado.
    pub closure: Option<Value>,
    /// Agregado assinado vinculado ao termo de encerramento.
    pub aggregate: Option<Value>,
}

/// Recusas distinguíveis de falhas técnicas do quadro.
#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    /// A eleição solicitada não possui quadro publicado.
    #[error("Quadro eleitoral inexistente")]
    NotFound,
    /// Já existe outro manifesto/calendário para a eleição.
    #[error("Quadro eleitoral já publicado com outros artefatos")]
    AlreadyExists,
    /// Novo voto após o encerramento confirmado.
    #[error("Eleição encerrada")]
    Closed,
    /// O período ainda não começou.
    #[error("Período de votação ainda não iniciado")]
    VotingNotStarted,
    /// O período terminou, mesmo se o encerramento ainda não foi publicado.
    #[error("Período de votação terminado")]
    VotingEnded,
    /// Evidência institucional vencida ou contexto inválido.
    #[error("Eleitor sem elegibilidade vigente")]
    NotEligible,
    /// Outra submissão já consumiu a participação da conta.
    #[error("A conta já participou desta eleição")]
    AlreadyParticipated,
    /// A mesma operação foi reapresentada com outro envelope.
    #[error("Conteúdo diferente para a mesma submissão")]
    SubmissionConflict,
    /// Identificador opaco já associado a outra conta.
    #[error("Identificador eleitoral incompatível com a conta")]
    VoterMismatch,
    /// Limite público de participantes atingido.
    #[error("Limite de participantes atingido")]
    LimitExceeded,
    /// Não existe encerramento válido antes do fim do período.
    #[error("Não é permitido encerrar antes do prazo")]
    NotYetEnded,
    /// Artefato ou estado armazenado estruturalmente inconsistente.
    #[error("Artefato eleitoral inválido: {0}")]
    InvalidArtifact(&'static str),
    /// O envelope, a assinatura ou a prova recebida não passou pela verificação.
    #[error("Evidência eleitoral inválida")]
    InvalidEvidence(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// Falha ao emitir uma evidência assinada pelo servidor.
    #[error("Assinatura do servidor indisponível")]
    SigningUnavailable,
    /// Falha técnica; não deve ser tratada como confirmação de voto.
    #[error("Falha no armazenamento eleitoral")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Operações atômicas consumidas pela aplicação eleitoral autenticada.
#[async_trait::async_trait]
pub trait ElectionBoard: Send + Sync {
    /// Publica uma única origem congelada; repetição idêntica é idempotente.
    async fn open(&self, board: FrozenBoard) -> Result<BoardSnapshot, BoardError>;

    /// Verifica, acumula e confirma um voto ou recupera o mesmo envelope aceito.
    async fn record(
        &self,
        election_id: Uuid,
        voter: EligibleVoter,
        signed_ballot: &[u8],
    ) -> Result<RecordedBallot, BoardError>;

    /// Recupera evidências da própria conta autenticada, sem nova elegibilidade.
    async fn recover(
        &self,
        election_id: Uuid,
        authenticated_account_id: Uuid,
    ) -> Result<Option<RecordedBallot>, BoardError>;

    /// Exporta artefatos públicos da mesma fotografia do banco.
    async fn snapshot(&self, election_id: Uuid) -> Result<BoardSnapshot, BoardError>;

    /// Congela cabeça, contagem e acumulador após o prazo, pelo relógio do banco.
    async fn close(&self, election_id: Uuid) -> Result<BoardSnapshot, BoardError>;
}
