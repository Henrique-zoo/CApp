//! Erros dos casos de uso e das portas do módulo eleitoral.
//!
//! Os erros não dependem de HTTP ou de detalhes de persistência.
//! A camada API decide como apresentar as falhas ao cliente.

/// Falha ao resolver a identidade ou a elegibilidade de um eleitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AuthenticatedElectorError {
    /// A identidade autenticada não possui uma conta interna válida.
    #[error("A identidade autenticada não possui uma conta válida")]
    IdentityNotFound,

    /// A identidade não está elegível para participar desta eleição.
    #[error("O eleitor não está elegível para esta eleição")]
    NotEligible,

    /// A identidade não pode participar por causa do estado institucional.
    #[error("A identidade não possui vínculo institucional válido")]
    InvalidInstitutionalStatus,

    /// Falha técnica ao consultar identidade ou elegibilidade.
    #[error("Falha ao consultar identidade ou elegibilidade")]
    Internal,
}

/// Falha ao consultar o agregado eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionReaderError {
    /// O registro persistido não pôde ser reconstituído.
    #[error("Registro eleitoral inválido")]
    InvalidStoredData,

    /// Falha técnica durante a consulta.
    #[error("Falha interna na consulta eleitoral")]
    Internal,
}

/// Falha ao persistir o agregado eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionStoreError {
    /// Outra operação alterou o agregado ou sua revisão esperada.
    #[error("Conflito de concorrência na eleição")]
    Conflict,

    /// O registro persistido não pôde ser reconstituído.
    #[error("Registro eleitoral inválido")]
    InvalidStoredData,

    /// Falha técnica de persistência.
    #[error("Falha interna no armazenamento eleitoral")]
    Internal,
}

/// Falha ao obter o instante confiável do backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Não foi possível obter o instante confiável do backend")]
pub struct ClockError;

/// Falha ao executar a consulta de uma eleição.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetElectionError {
    /// Não existe eleição com o identificador solicitado.
    #[error("Eleição não encontrada")]
    NotFound,

    /// A porta de leitura falhou ao consultar a eleição.
    #[error(transparent)]
    Reader(#[from] ElectionReaderError),
}
