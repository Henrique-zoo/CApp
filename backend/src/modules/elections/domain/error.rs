//! Falhas de domínio da preparação, votação e apuração de eleições.
//!
//! As entidades e regras usam tipos específicos para indicar qual invariante
//! foi violada. A aplicação decide como tratar cada falha no fluxo eleitoral,
//! e a camada API decide como comunicá-la ao cliente.

use chrono::{DateTime, Utc};

/// Identificador inválido para o tipo de domínio correspondente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Identificador inválido: {kind}")]
pub struct InvalidIdentifier {
    kind: &'static str,
}

impl InvalidIdentifier {
    /// Cria o erro para o tipo de identificador informado.
    pub(crate) const fn new(kind: &'static str) -> Self {
        Self { kind }
    }

    /// Retorna o nome do tipo de identificador inválido.
    pub const fn kind(&self) -> &'static str {
        self.kind
    }
}

/// Falhas das operações criptográficas do protocolo eleitoral.
#[derive(Debug, thiserror::Error)]
pub enum CryptographyError {
    /// A fonte de aleatoriedade do sistema falhou ao preencher o buffer.
    ///
    /// A causa é preservada para diagnóstico. A operação deve ser interrompida
    /// porque o buffer pode ter sido preenchido apenas em parte.
    #[error("Não foi possível obter bytes aleatórios para a operação criptográfica")]
    RandomnessUnavailable(#[from] getrandom::Error),

    /// O número de escolhas viola o perfil ou difere da quantidade de cifras.
    #[error("Quantidade de cifras incompatível com as escolhas esperadas")]
    InvalidCiphertexts,

    /// O identificador do grupo não é o fixado pelo perfil eleitoral.
    #[error("Grupo ElGamal não suportado pelo perfil eleitoral")]
    UnsupportedGroup,

    /// Um valor viola as regras do subgrupo ou de identidade.
    #[error("Elemento ElGamal inválido")]
    InvalidGroupElement,

    /// A contagem de cédulas aceitas excede o limite do perfil eleitoral.
    #[error("Limite de cédulas excedido")]
    TooManyVoters,

    /// O conjunto de contribuições está incompleto ou é incompatível.
    #[error("Contribuições incompatíveis com o agregado")]
    InvalidShares,

    /// Um elemento recuperado não representa nenhum total em `[0, n]`.
    #[error("Total fora do intervalo de cédulas aceitas")]
    TotalOutOfRange,
}

/// Motivo pelo qual o nome de uma chapa foi recusado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionOption {
    /// Nome vazio ou composto somente por espaços.
    #[error("O nome da opção não pode ser vazio")]
    EmptyName,

    /// Nome maior que o limite do contrato.
    #[error("O nome da opção excede 200 caracteres")]
    NameTooLong,
}

/// Violação estrutural da coleção de opções eleitorais.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionOptions {
    /// Exige uma a cinquenta chapas, além de Nulo.
    #[error("A eleição deve possuir entre 2 e 51 opções")]
    InvalidCount,

    /// IDs de opções são únicos na eleição.
    #[error("Os identificadores das opções devem ser distintos")]
    DuplicateId,

    /// Nulo é obrigatório e único.
    #[error("A eleição deve possuir exatamente uma opção Nulo")]
    InvalidNullCount,
}

/// Erro ao construir um período eleitoral inválido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionPeriod {
    /// O início deve ser estritamente anterior ao fim.
    #[error("O início do período deve ser anterior ao fim")]
    StartMustPrecedeEnd,

    /// O contrato eleitoral não permite frações de segundo.
    #[error("Os instantes do período devem ter precisão exata de segundos")]
    SubsecondPrecision,
}

/// Erros encontrados ao construir uma autoridade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidAuthority {
    /// Os valores públicos zero e um são proibidos pelo perfil eleitoral.
    #[error("O valor público da autoridade não pode ser zero ou um")]
    InvalidPublicValue,

    /// O compromisso da prova de conhecimento não pode ser zero ou um.
    #[error("O compromisso da prova de conhecimento não pode ser zero ou um")]
    InvalidKeyProofCommitment,
}

/// Erros encontrados ao construir um manifesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidManifest {
    /// Quantidade de opções fora dos limites do protocolo.
    #[error("O manifesto deve possuir entre 2 e 51 opções")]
    InvalidOptionCount,

    /// Quantidade de autoridades fora dos limites do protocolo.
    #[error("O manifesto deve possuir entre 3 e 52 autoridades")]
    InvalidAuthorityCount,

    /// O manifesto precisa conter exatamente uma opção Nulo.
    #[error("O manifesto deve possuir exatamente uma opção Nulo")]
    InvalidNullOptionCount,

    /// A opção Nulo deve ser a última opção.
    #[error("A opção Nulo deve ocupar a última posição")]
    NullOptionMustBeLast,

    /// Duas opções não podem compartilhar o mesmo identificador.
    #[error("Os identificadores das opções devem ser distintos")]
    DuplicateOptionId,

    /// Duas autoridades não podem compartilhar o mesmo identificador.
    #[error("Os identificadores das autoridades devem ser distintos")]
    DuplicateAuthorityId,

    /// Duas autoridades não podem compartilhar a mesma chave de assinatura.
    #[error("As chaves de assinatura das autoridades devem ser distintas")]
    DuplicateAuthoritySigningKey,

    /// As contribuições públicas das autoridades devem ser distintas.
    #[error("Os valores públicos das autoridades devem ser distintos")]
    DuplicateAuthorityPublicValue,

    /// Deve existir exatamente uma autoridade do servidor.
    #[error("O manifesto deve possuir exatamente uma autoridade Server")]
    InvalidServerAuthorityCount,

    /// Deve existir exatamente uma autoridade da comissão.
    #[error("O manifesto deve possuir exatamente uma autoridade da comissão")]
    InvalidCommissionAuthorityCount,

    /// Deve existir uma autoridade para cada chapa.
    #[error("Deve existir exatamente uma autoridade para cada chapa")]
    SlateAuthorityMismatch,

    /// A chave pública combinada não pode ser zero ou um.
    #[error("A chave pública combinada não pode ser zero ou um")]
    InvalidCombinedPublicKey,
}

/// Recusas ao avançar o estado da eleição ou alterar seu calendário vigente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionTransitionError {
    /// A operação não é permitida no estado operacional atual.
    #[error("A operação não é permitida no estado atual da eleição")]
    InvalidState,

    /// A configuração ou a apuração é incompatível com a modalidade de votação.
    #[error("Operação incompatível com a modalidade eleitoral")]
    ModalityMismatch,

    /// O manifesto pronto pertence a outra eleição ou a outro CA.
    #[error("O manifesto não pertence a esta eleição e ao seu Centro Acadêmico")]
    InvalidConfiguration,

    /// A tentativa de abertura ocorreu fora do período `[starts_at, ends_at)`.
    #[error("Tentativa de abertura em {attempted_at} fora do período [{starts_at}, {ends_at})")]
    OpeningOutsidePeriod {
        /// Instante em que o backend tentou abrir a votação.
        attempted_at: DateTime<Utc>,
        /// Início previsto da votação.
        starts_at: DateTime<Utc>,
        /// Encerramento previsto da votação.
        ends_at: DateTime<Utc>,
    },

    /// O encerramento não pode ocorrer antes do prazo vigente.
    #[error("A eleição não pode ser encerrada antes do prazo vigente")]
    ClosingBeforeDeadline,

    /// As contribuições verificadas divergem das autoridades exigidas.
    #[error("As contribuições verificadas não correspondem às autoridades da eleição")]
    ContributionsMismatch,

    /// Quantidades ou opções incompatíveis com o conjunto final de votos.
    #[error(transparent)]
    InvalidTally(#[from] InvalidCountConsistency),

    /// O motivo do cancelamento não possui tamanho permitido.
    #[error("O motivo do cancelamento deve possuir entre 1 e 2000 caracteres")]
    InvalidCancellationReasonLength,

    /// O UUID de quem solicitou o cancelamento é zerado.
    #[error("O responsável pelo cancelamento deve ter um UUID diferente de zero")]
    InvalidCancellationRequester,

    /// O cancelamento foi registrado antes da abertura da votação.
    #[error("O cancelamento não pode anteceder a abertura da votação")]
    CancellationBeforeOpening,

    /// O cancelamento ocorreu antes do encerramento já registrado.
    #[error("O cancelamento não pode anteceder o encerramento da eleição")]
    CancellationBeforeClosure,

    /// A abertura não pode ser antecipada por revisão de calendário.
    #[error("Uma revisão não pode antecipar a abertura")]
    StartCannotMoveEarlier,

    /// O novo encerramento não é posterior ao encerramento atual.
    #[error("O novo encerramento deve ser posterior ao encerramento atual")]
    EndMustBeExtended,

    /// A prorrogação tentou modificar o início da votação.
    #[error("O início da votação não pode ser alterado após sua abertura")]
    StartCannotChange,

    /// A revisão foi registrada antes da abertura da votação.
    #[error("A revisão não pode anteceder a abertura da votação")]
    ScheduleChangeBeforeOpening,

    /// A prorrogação ocorreu depois do prazo vigente.
    #[error("A votação só pode ser prorrogada antes do prazo vigente")]
    ExtensionAfterDeadline,
}

/// Falha estrutural ou contextual da cédula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBallot {
    /// Configuração, eleição, modalidade ou manifesto divergente.
    #[error("A cédula não corresponde à configuração eleitoral")]
    ConfigurationMismatch,

    /// Vetor incompleto, repetido ou fora da ordem configurada.
    #[error("As opções da cédula divergem da configuração")]
    InvalidOptions,

    /// Componente ElGamal fora da faixa ou do subgrupo, ou identidade proibida.
    #[error("A cédula contém uma cifra ElGamal inválida")]
    InvalidCiphertext,
}

/// Recusa de novo aceite, repetição conflitante ou reconstituição inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VoteError {
    /// Somente eleição aberta admite novo aceite.
    #[error("A eleição não está aberta")]
    ElectionNotOpen,

    /// Aceite fora do intervalo vigente.
    #[error("O instante do aceite é inválido")]
    InvalidAcceptanceTime,

    /// A conta ou a eleição da cédula diverge da operação autenticada.
    #[error("A identidade não corresponde à participação")]
    IdentityMismatch,

    /// Outra operação não pode criar nova participação.
    #[error("A conta já participou desta eleição")]
    AlreadyParticipated,

    /// O mesmo identificador de envio foi usado com outro payload.
    #[error("O identificador de envio já possui outro conteúdo")]
    SubmissionConflict,

    /// Contexto ou opções inválidos na cédula.
    #[error(transparent)]
    InvalidBallot(#[from] InvalidBallot),
}

/// Incompatibilidade estrutural no registro ou entre registros consecutivos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBoardEntry {
    /// A identificada não participa deste quadro criptográfico.
    #[error("O quadro confidencial exige uma cédula confidencial")]
    IdentifiedBallot,

    /// O perfil limita os índices ao intervalo inclusivo de 1 a 100000.
    #[error("Índice inválido no quadro eleitoral")]
    InvalidIndex,

    /// A primeira referência deve ser o digest do manifesto.
    #[error("Referência inicial inválida no quadro eleitoral")]
    InvalidOrigin,

    /// Contexto, índice ou referência ao registro anterior divergem.
    #[error("Os registros eleitorais não são consecutivos")]
    InvalidPredecessor,
}

/// Índice inválido no quadro, cujos registros começam em um.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("O índice do recibo deve estar entre 1 e 100000")]
pub struct InvalidReceiptIndex;

/// Referências incompatíveis com a existência do manifesto e a contagem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBoardSnapshot {
    /// Sem manifesto, não pode existir cabeça de quadro nem participação.
    #[error("Quadro ou participação sem manifesto")]
    ManifestRequired,

    /// Publicado o manifesto, a fotografia precisa identificar a cabeça do quadro.
    #[error("A fotografia confidencial exige a cabeça do quadro")]
    HeadRequired,

    /// A origem do quadro representa exatamente zero participantes.
    #[error("A cabeça do quadro é incompatível com a contagem de participantes")]
    InvalidEmptyBoard,
}

/// Erros encontrados ao construir uma quantidade de eleitores inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidVoterCount {
    /// A quantidade excede o limite definido pelo contrato.
    #[error("A quantidade de eleitores não pode exceder 100000")]
    ExceedsMaximum,
}

/// Erros encontrados ao construir uma contagem de opção inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidOptionCount {
    /// A quantidade de votos excede o limite do contrato.
    #[error("A quantidade de votos da opção excede 100000")]
    InvalidCount(#[from] InvalidVoterCount),
}

/// Erros estruturais encontrados nas contagens eleitorais.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidCountConsistency {
    /// Uma contagem excede o total de participantes fixado no encerramento.
    #[error("Uma contagem excede o número de participantes")]
    CountOutOfRange,

    /// A quantidade de contagens difere da quantidade de opções do manifesto.
    #[error("A quantidade de contagens difere da quantidade de opções do manifesto")]
    OptionCountMismatch,

    /// Os identificadores das opções não correspondem à ordem do manifesto.
    #[error("As opções contabilizadas não correspondem à ordem do manifesto")]
    OptionOrderMismatch,
}
