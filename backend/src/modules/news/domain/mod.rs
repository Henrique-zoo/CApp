//! Camada de domínio do módulo de notícias.
//!
//! Contém as entidades, regras de validação, eventos de domínio e os erros
//! de negócio associados à criação, edição e ciclo editorial de notícias.

use thiserror::Error;
use uuid::Uuid;

pub mod entities;
pub mod events;
pub mod rules;

pub use entities::{NewsArticle, NewsStatus};
pub use events::{
    NewsArticleArchived, NewsArticleCreated, NewsArticlePublished, NewsArticleUpdated,
};

/// Erros de domínio e de regras de negócio do módulo de notícias.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NewsError {
    /// O título da notícia não pode ser vazio.
    #[error("O título da notícia não pode ser vazio.")]
    EmptyTitle,

    /// O título da notícia é menor que o limite mínimo exigido.
    #[error("O título da notícia é muito curto: mínimo de {min} caracteres, recebido {actual}.")]
    TitleTooShort {
        /// Limite mínimo exigido.
        min: usize,
        /// Quantidade de caracteres informada.
        actual: usize,
    },

    /// O título da notícia excede o limite máximo permitido.
    #[error("O título da notícia excede o limite de {max} caracteres: recebido {actual}.")]
    TitleTooLong {
        /// Limite máximo permitido.
        max: usize,
        /// Quantidade de caracteres informada.
        actual: usize,
    },

    /// O corpo da notícia não pode ser vazio.
    #[error("O corpo da notícia não pode ser vazio.")]
    EmptyBody,

    /// O corpo da notícia excede o limite máximo permitido.
    #[error("O corpo da notícia excede o limite de {max} caracteres: recebido {actual}.")]
    BodyTooLong {
        /// Limite máximo permitido.
        max: usize,
        /// Quantidade de caracteres informada.
        actual: usize,
    },

    /// Tentativa de editar uma notícia que já se encontra arquivada.
    #[error("Não é possível editar uma notícia arquivada.")]
    CannotEditArchived,

    /// Notícia não encontrada pelo identificador informado.
    #[error("Notícia {id} não encontrada.")]
    ArticleNotFound {
        /// Identificador da notícia procurada.
        id: Uuid,
    },

    /// Usuário não possui permissão de gestão no Centro Acadêmico especificado.
    #[error("Usuário {author_id} não possui permissão de gestão no Centro Acadêmico {ca_id}.")]
    UnauthorizedAuthor {
        /// Identificador do usuário.
        author_id: Uuid,
        /// Identificador do Centro Acadêmico.
        ca_id: Uuid,
    },

    /// Usuário não possui permissão para publicação direta de notícias.
    #[error(
        "Usuário {author_id} não possui permissão de publicação direta no Centro Acadêmico {ca_id}."
    )]
    UnauthorizedPublish {
        /// Identificador do usuário.
        author_id: Uuid,
        /// Identificador do Centro Acadêmico.
        ca_id: Uuid,
    },

    /// Transição de status inválida no ciclo editorial.
    #[error("Transição de status inválida de {from} para {to}.")]
    InvalidStatusTransition {
        /// Estado de origem.
        from: NewsStatus,
        /// Estado de destino.
        to: NewsStatus,
    },

    /// String de status desconhecida ou não reconhecida.
    #[error("Status de notícia desconhecido: {0}.")]
    InvalidStatusString(String),

    /// Falha na camada de persistência de dados.
    #[error("Erro na persistência de notícias: {0}.")]
    RepositoryError(String),
}
