//! Portas de entrada e saída da camada de aplicação de notícias.
//!
//! Define contratos abstratos para persistência de dados e controle
//! de autorização institucional baseada em cargos do Centro Acadêmico.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::news::domain::{NewsArticle, NewsStatus};

/// Falhas ocorridas na persistência ou recuperação de dados de notícias.
#[derive(Debug, thiserror::Error)]
pub enum NewsRepositoryError {
    /// O registro com o identificador informado não foi localizado.
    #[error("Notícia com ID '{0}' não encontrada.")]
    NotFound(Uuid),

    /// Conflito de concorrência ou transição inválida no repositório.
    #[error(
        "Conflito de concorrência ao atualizar a notícia '{id}': status esperado era '{expected}', mas divergiu no armazenamento."
    )]
    OptimisticLockConflict {
        /// Identificador da notícia.
        id: Uuid,
        /// Estado esperado no momento da operação.
        expected: NewsStatus,
    },

    /// Falha técnica no driver ou mecanismo de banco de dados.
    #[error("Falha técnica no repositório de notícias: {0}")]
    DatabaseError(String),
}

/// Falhas ocorridas durante a avaliação de autorizações e permissões de acesso.
#[derive(Debug, thiserror::Error)]
pub enum NewsAuthorizationError {
    /// Falha ao consultar o serviço ou repositório de governança/cargos.
    #[error("Falha técnica ao verificar permissão do usuário '{user_id}': {reason}")]
    TechnicalFailure {
        /// Identificador do usuário.
        user_id: Uuid,
        /// Descrição do motivo técnico da falha.
        reason: String,
    },
}

/// Contrato para operações de persistência e consulta do ciclo editorial de notícias.
#[async_trait]
pub trait NewsRepositoryPort: Send + Sync {
    /// Salva um novo artigo de notícia no armazenamento.
    async fn save(&self, article: &NewsArticle) -> Result<(), NewsRepositoryError>;

    /// Localiza um artigo de notícia pelo seu identificador primário.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewsArticle>, NewsRepositoryError>;

    /// Atualiza integralmente os dados de um artigo já existente.
    async fn update(&self, article: &NewsArticle) -> Result<(), NewsRepositoryError>;

    /// Executa uma transição atômica de status garantindo consistência sob concorrência.
    ///
    /// Atualiza o status e os timestamps de publicação ou arquivamento caso
    /// o registro atualmente armazenado ainda possua o status esperado.
    ///
    /// # Retorno
    ///
    /// Retorna o artigo atualizado com os novos dados e timestamps.
    ///
    /// # Erros
    ///
    /// - [`NewsRepositoryError::NotFound`] se o artigo não existir.
    /// - [`NewsRepositoryError::OptimisticLockConflict`] se o status atual no banco divergir de `expected_current_status`.
    /// - [`NewsRepositoryError::DatabaseError`] em caso de falha de conexão ou execução.
    async fn update_status_atomic(
        &self,
        id: Uuid,
        expected_current_status: NewsStatus,
        new_status: NewsStatus,
        published_at: Option<DateTime<Utc>>,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<NewsArticle, NewsRepositoryError>;
}

/// Contrato para validações de autorização institucional nos Centros Acadêmicos.
///
/// Assegura que apenas usuários com papéis ativos na gestão do CA correspondente
/// possam aprovar/publicar, arquivar ou submeter conteúdos institucionais.
#[async_trait]
pub trait NewsAuthorizationPort: Send + Sync {
    /// Verifica se o usuário tem autorização para homologar e publicar notícias no CA informado.
    async fn can_publish_news(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError>;

    /// Verifica se o usuário tem autorização para arquivar notícias no CA informado.
    async fn can_archive_news(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError>;

    /// Verifica se o usuário tem autorização para propor ou submeter notícias para revisão.
    ///
    /// Por padrão, o próprio autor do rascunho ou qualquer membro da gestão do CA tem permissão.
    async fn can_submit_news(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        author_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError>;
}
