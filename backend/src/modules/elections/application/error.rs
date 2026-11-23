//! Erros dos casos de uso e das portas do módulo eleitoral.
//!
//! Os erros não dependem de HTTP ou de detalhes de persistência.
//! A camada API decide como apresentar as falhas ao cliente.

/// Falha ao registrar ou recuperar o aceite de uma cédula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RecordBallotError {
    /// Outra operação alterou a eleição; repetir após leitura atualizada.
    #[error("Conflito de concorrência eleitoral")]
    Conflict,

    /// Existe participação desta conta por outra operação/cédula.
    #[error("A conta já participou da eleição")]
    AlreadyParticipated,

    /// Mesmo identificador de envio com outro payload.
    #[error("Envio eleitoral conflitante")]
    SubmissionConflict,

    /// Curso verificado ausente ou diferente do curso vinculado ao CA da eleição.
    #[error("Você precisa ter vínculo estudantil verificado com o curso deste CA para votar")]
    NotEligible,

    /// Estado, prazo, configuração ou identidade recusados.
    #[error("Novo voto recusado")]
    Rejected,

    /// Falha de armazenamento; não confirma o aceite ao chamador.
    #[error("Armazenamento eleitoral indisponível")]
    Unavailable,
}

/// Falha ao verificar a assinatura ou os dados públicos de uma cédula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BallotVerificationError {
    /// Codificação, chave pública ou assinatura inválida.
    #[error("A cédula contém evidência inválida")]
    InvalidEvidence,

    /// A verificação não pôde ser concluída por falha técnica.
    #[error("Verificação de cédula indisponível")]
    Unavailable,
}

/// Recusas e falhas do caso de uso de cadastro prévio.
#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    /// A eleição solicitada não foi encontrada.
    #[error("Eleição inexistente")]
    NotFound,

    /// A eleição vincula a chave junto do primeiro voto.
    #[error("Esta eleição dispensa o cadastro prévio")]
    NotRequired,

    /// O vínculo vigente não corresponde a um curso representado pelo CA.
    #[error("Sem elegibilidade para esta eleição")]
    NotEligible,

    /// A votação já atingiu o horário de abertura ou foi encerrada.
    #[error("Período de cadastro encerrado")]
    Closed,

    /// Outra chave já foi cadastrada para a conta na mesma eleição.
    #[error("Outra chave já foi cadastrada")]
    KeyConflict,

    /// A prova é inválida para essa eleição, conta ou chave pública.
    #[error("Prova de posse inválida")]
    InvalidProof,

    /// Falha técnica sem confirmação da operação.
    #[error("Falha no armazenamento do cadastro eleitoral")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}
