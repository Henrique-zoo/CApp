//! Camada de casos de uso e aplicação do módulo de notícias.
//!
//! Orquestra as regras de negócio, autorizações institucionais de gestão de CAs
//! e persistência dos artigos no mural administrado.

pub mod commands;
pub mod ports;
pub mod queries;
pub mod services;

use uuid::Uuid;

pub use commands::{
    ArchiveNewsArticleCommand, ArchiveNewsArticleUseCase, PublishNewsArticleCommand,
    PublishNewsArticleUseCase, SubmitForReviewNewsArticleCommand,
    SubmitForReviewNewsArticleUseCase,
};
pub use ports::{
    NewsAuthorizationError, NewsAuthorizationPort, NewsRepositoryError, NewsRepositoryPort,
};
pub use queries::{
    GetNewsArticleQuery, GetNewsArticleUseCase, GetPublishedNewsDetailQuery,
    GetPublishedNewsDetailUseCase, ListPublishedNewsByCaQuery, ListPublishedNewsByCaUseCase,
};
pub use services::NewsApplicationService;

use crate::modules::news::domain::NewsDomainError;

/// Erros originados na camada de aplicação durante o processamento de casos de uso.
#[derive(Debug, thiserror::Error)]
pub enum NewsApplicationError {
    /// O artigo solicitado não foi localizado no repositório.
    #[error("Notícia com ID '{0}' não foi encontrada.")]
    NotFound(Uuid),

    /// O usuário não tem permissão para realizar a operação no Centro Acadêmico contextual.
    #[error(
        "Usuário '{user_id}' não possui permissão para executar a operação '{operation}' no Centro Acadêmico '{ca_id}'."
    )]
    Forbidden {
        /// Usuário que tentou a operação.
        user_id: Uuid,
        /// Centro Acadêmico de contexto.
        ca_id: Uuid,
        /// Identificação da operação proibida.
        operation: &'static str,
    },

    /// A notícia já se encontra em estado arquivado e não aceita a operação solicitada.
    #[error("A notícia com ID '{id}' já se encontra arquivada.")]
    AlreadyArchived {
        /// Identificador da notícia.
        id: Uuid,
    },

    /// Transição de estado recusada pelas regras do domínio.
    #[error("Transição de estado inválida: {0}")]
    InvalidTransition(#[from] NewsDomainError),

    /// Falha nas operações de persistência do repositório.
    #[error("Erro de persistência: {0}")]
    Repository(#[from] NewsRepositoryError),

    /// Falha técnica no serviço de checagem de autorizações.
    #[error("Erro de autorização: {0}")]
    Authorization(#[from] NewsAuthorizationError),
}
