//! Invariantes e regras de negócio do módulo de notícias.
//!
//! Centraliza as definições de erros de domínio e restrições sobre
//! artigos, ciclo de vida e operações editoriais.

use uuid::Uuid;

use crate::modules::news::domain::entities::NewsStatus;

/// Erros decorrentes da violação de regras e invariantes de domínio de notícias.
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum NewsDomainError {
    /// A transição de estado solicitada não é válida para o ciclo editorial.
    #[error("Transição de status inválida de '{current}' para '{target}'.")]
    InvalidStatusTransition {
        /// Estado atual da notícia.
        current: NewsStatus,
        /// Estado de destino pretendido.
        target: NewsStatus,
    },

    /// A notícia já se encontra arquivada e não admite novas transições ou arquivamento repetido.
    #[error("A notícia com ID '{id}' já se encontra arquivada.")]
    ArticleAlreadyArchived {
        /// Identificador da notícia.
        id: Uuid,
    },

    /// Tentativa de modificar o conteúdo de um artigo arquivado.
    #[error("Não é permitido alterar o conteúdo de uma notícia já arquivada.")]
    ArchivedArticleCannotBeModified,

    /// O título fornecido é vazio ou composto apenas de espaços em branco.
    #[error("O título da notícia não pode ser vazio.")]
    EmptyTitle,

    /// O corpo fornecido é vazio ou composto apenas de espaços em branco.
    #[error("O corpo da notícia não pode ser vazio.")]
    EmptyBody,

    /// Representação textual de status não reconhecida pelo sistema.
    #[error("O valor '{0}' não corresponde a um status de notícia válido.")]
    UnknownStatus(String),
}
