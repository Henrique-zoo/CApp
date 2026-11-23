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
