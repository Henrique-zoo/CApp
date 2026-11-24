//! Dados e falhas das operações do quadro eleitoral público.
//!
//! Os contratos de leitura e persistência em [`super::ports`] usam estes tipos
//! para receber a origem assinada, confirmar o aceite e exportar as evidências.

use serde::Serialize;

use super::administration::BoardCancellation;
use serde_json::Value;

use chrono::{DateTime, Utc};

use super::super::domain::{
    Fingerprint,
    cryptography::elgamal::ElGamalCiphertext,
    entities::election::ElectionPeriod,
    identifiers::{AcademicCenterId, ElectionId},
};
use uuid::Uuid;

/// Manifesto assinado e configuração da eleição que iniciam um quadro.
///
/// Os bytes originais permitem recusar JSON ambíguo antes da canonicalização.
/// O manifesto passa pela verificação criptográfica. O período vem da eleição.
pub struct FrozenBoard {
    /// Eleição da qual o serviço recebeu o período.
    pub election_id: ElectionId,
    /// Envelope assinado do manifesto público.
    pub manifest: Vec<u8>,
    /// Período de votação definido na entidade eleitoral.
    pub period: ElectionPeriod,
    /// Exigência de cadastro escolhida na criação e congelada na publicação.
    pub requires_voter_registration: bool,
}

/// Configuração verificada a persistir junto aos documentos assinados.
pub struct BoardPublication {
    /// Eleição declarada no manifesto.
    pub election_id: ElectionId,
    /// CA vinculado pelo manifesto.
    pub academic_center_id: AcademicCenterId,
    /// Envelope assinado do manifesto.
    pub manifest: Value,
    /// Hash do manifesto.
    pub manifest_hash: String,
    /// Início do período de votação.
    pub starts_at: DateTime<Utc>,
    /// Fim do período de votação.
    pub ends_at: DateTime<Utc>,
    /// Escolhas na ordem do manifesto, incluindo Nulo.
    pub option_ids: Vec<Uuid>,
    /// Política de cadastro prévio declarada na abertura.
    pub requires_registration: bool,
}

/// Dados lidos sob o bloqueio do quadro para preparar o encerramento.
pub struct BoardClosing {
    /// Eleição cujo quadro está sendo encerrado.
    pub election_id: ElectionId,
    /// Término vigente do período.
    pub ends_at: DateTime<Utc>,
    /// Instante lido depois do bloqueio.
    pub now: DateTime<Utc>,
    /// Chave de assinatura fixada no manifesto.
    pub server_key: String,
    /// Hash do manifesto publicado.
    pub manifest_hash: String,
    /// Hash que encerra a cadeia do quadro.
    pub head_hash: String,
    /// Número de cédulas aceitas.
    pub voter_count: i32,
    /// Escolhas na ordem do manifesto.
    pub option_ids: Vec<Uuid>,
    /// Produto final das cifras.
    pub accumulator: Vec<ElGamalCiphertext>,
}

/// Termo e agregado assinados para publicação atômica.
pub struct BoardClosure {
    /// Termo de encerramento, com contagem e hash final.
    pub closure: Value,
    /// Agregado referenciado pelo encerramento.
    pub aggregate: Value,
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

/// Dados do voto já aceito para comparar com uma cédula recém-verificada.
pub struct AcceptedVote {
    /// Hash do conteúdo assinado, independente dos bytes da assinatura.
    pub ballot_hash: Fingerprint,
    /// Registro e recibo emitidos no primeiro aceite.
    pub record: RecordedBallot,
}

/// Dados lidos sob o bloqueio da eleição para decidir um novo aceite.
pub struct VoteWindow {
    /// Indica que o encerramento já foi gravado.
    pub closed: bool,
    /// A comissão registrou o cancelamento.
    pub cancelled: bool,
    /// Primeiro instante em que o voto pode ser aceito.
    pub starts_at: DateTime<Utc>,
    /// Primeiro instante em que o voto deve ser recusado.
    pub ends_at: DateTime<Utc>,
    /// Validade da evidência de elegibilidade, quando localizada.
    pub eligible_until: Option<DateTime<Utc>>,
    /// Instante lido após os bloqueios necessários.
    pub now: DateTime<Utc>,
    /// Chave de assinatura do servidor fixada com o manifesto.
    pub server_key: String,
    /// Indica se a conta precisava cadastrar a chave antes da abertura.
    pub requires_registration: bool,
    /// Chave que a conta cadastrou nesta eleição.
    pub registered_key: Option<Vec<u8>>,
    /// Hash da última entrada ou do manifesto, quando o quadro está vazio.
    pub head_hash: String,
    /// Número de votos já confirmados.
    pub voter_count: u32,
    /// Produto das cifras, na ordem do manifesto.
    pub accumulator: Vec<ElGamalCiphertext>,
}

/// O voto da conta já foi aceito ou está apto a ser avaliado pelo serviço.
pub enum VoteState {
    /// Registro localizado pela chave eleição/conta.
    Accepted(AcceptedVote),
    /// Estado do quadro e da conta para avaliar um novo voto.
    Pending(VoteWindow),
}

/// Escritas decididas pelo serviço enquanto a eleição está bloqueada.
pub struct NewVote {
    /// Hash da cédula recém-verificada.
    pub ballot_hash: Fingerprint,
    /// Chave de assinatura da conta nesta eleição, em DER SPKI.
    pub public_key: Vec<u8>,
    /// Instante de aceite consultado na transação.
    pub accepted_at: DateTime<Utc>,
    /// Índice da entrada no quadro público.
    pub index: u32,
    /// Hash da entrada recém-assinada.
    pub entry_hash: String,
    /// Novo produto das cifras, na ordem do manifesto.
    pub accumulator: Vec<ElGamalCiphertext>,
    /// Registro e recibo assinados para persistir no mesmo commit.
    pub record: RecordedBallot,
}

/// Resultado da assinatura do registro e do recibo de um voto.
pub struct SignedVoteEvidence {
    /// Hash da entrada pública.
    pub entry_hash: String,
    /// Entrada e recibo assinados.
    pub record: RecordedBallot,
}

/// Exportação consistente das evidências públicas do quadro.
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
    /// Início do período informado pela eleição.
    pub starts_at: DateTime<Utc>,
    /// Fim do período informado pela eleição.
    pub ends_at: DateTime<Utc>,
    /// Registros assinados em ordem crescente, começando em um.
    pub entries: Vec<Value>,
    /// Cancelamento administrativo, quando publicado pela comissão.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<BoardCancellation>,
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
    /// Já existe outra configuração publicada para a eleição.
    #[error("Quadro eleitoral já publicado com outros artefatos")]
    AlreadyExists,
    /// A comissão cancelou a eleição.
    #[error("Eleição cancelada")]
    Cancelled,
    /// Calendário ou comando administrativo incompatível com a fase da eleição.
    #[error("Alteração eleitoral inválida: {0}")]
    InvalidAdministration(&'static str),
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
    /// A eleição exige cadastro anterior à abertura e a conta não possui chave.
    #[error("Cadastre a chave eleitoral antes da abertura")]
    RegistrationRequired,
    /// O voto usa uma chave diferente da cadastrada pela própria conta.
    #[error("A chave da cédula não corresponde à chave cadastrada")]
    RegisteredKeyMismatch,
    /// A conta já confirmou outra cédula nesta eleição.
    #[error("A conta já participou desta eleição")]
    AlreadyParticipated,
    /// Identificador do voto incompatível com a conta.
    #[error("Identificador eleitoral incompatível com a conta")]
    VoterMismatch,
    /// Limite público de participantes atingido.
    #[error("Limite de participantes atingido")]
    LimitExceeded,
    /// Faltam credenciais verificadas para publicar o manifesto.
    #[error("Faltam chaves das autoridades designadas")]
    PreparationIncomplete,
    /// A conta não foi designada para enviar a contribuição desta autoridade.
    #[error("Conta sem atribuição para esta autoridade eleitoral")]
    AuthorityNotAssigned,
    /// A autoridade já publicou uma contribuição diferente.
    #[error("A autoridade já publicou sua contribuição")]
    ShareAlreadyAccepted,
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
