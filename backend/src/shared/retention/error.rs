//! Erros associados às políticas de retenção, minimização e descarte de dados.

use thiserror::Error;

/// Erros decorrentes da validação de políticas de retenção, minimização e privacidade.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RetentionError {
    /// Violação estrita de política de privacidade decorrente do uso de matrícula proibida.
    #[error(
        "Uso proibido da matrícula real restrita '{0}'. Utilize apenas matrículas didáticas homologadas."
    )]
    ForbiddenMatricula(String),

    /// E-mail institucional discente inválido ou fora do domínio padrão da UnB.
    #[error(
        "E-mail institucional discente inválido '{0}'. Apenas contas '@aluno.unb.br' são permitidas."
    )]
    InvalidUnbEmail(String),

    /// Formato inválido de matrícula institucional (deve conter exatamente 9 dígitos numéricos).
    #[error(
        "Formato inválido de matrícula '{0}'. A matrícula da UnB deve conter exatamente 9 dígitos numéricos."
    )]
    InvalidMatriculaFormat(String),

    /// Tentativa indevida de parametrizar identificador de instituição no CApp.
    #[error(
        "O uso de 'institution_id' é terminantemente proibido. O CApp atende com exclusividade à UnB."
    )]
    InstitutionIdProhibited,

    /// Violação da garantia de sigilo eleitoral (ex.: tentativa de associar usuário à cédula de voto).
    #[error(
        "Violação de sigilo eleitoral: a cédula não pode conter identificador de eleitor ('{0}')."
    )]
    ElectoralSecretViolation(String),

    /// Bloqueio de exclusão física decorrente de obrigação estatutária ou legal de guarda.
    #[error(
        "Exclusão física bloqueada por salvaguarda de integridade referencial: {0}. Aplique anonimização irreversível."
    )]
    ReferentialIntegrityBlocked(String),

    /// Registro com período de retenção expirado.
    #[error(
        "Registro expirado para a categoria '{0}'. Período regulamentar de retenção ultrapassado."
    )]
    RecordExpired(String),
}
