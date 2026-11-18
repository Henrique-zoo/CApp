//! Erros compartilhados pelas invariantes do domínio eleitoral.

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
