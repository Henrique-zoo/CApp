//! Erros compartilhados pelas entidades e regras do domínio de materiais.

/// Erros que podem ocorrer durante a construção ou interpretação de um material.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StudyMaterialError {
    /// O título não pode ser vazio ou composto somente por espaços.
    #[error("o título do material não pode ser vazio")]
    EmptyTitle,

    /// O endereço externo não pode ser vazio ou composto somente por espaços.
    #[error("o endereço do link externo não pode ser vazio")]
    EmptyExternalLink,

    /// O tipo de conteúdo textual não corresponde a um valor conhecido.
    #[error("tipo de conteúdo de material inválido: '{0}'")]
    InvalidKind(String),

    /// O estado textual não corresponde a um valor conhecido.
    #[error("estado de material inválido: '{0}'")]
    InvalidStatus(String),
}
