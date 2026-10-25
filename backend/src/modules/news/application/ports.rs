//! Portas e contratos da camada de aplicação do módulo de notícias.

use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::news::domain::{NewsArticle, NewsError};

/// Contrato para persistência e recuperação de artigos de notícias.
#[async_trait]
pub trait NewsRepository: Send + Sync {
    /// Salva um novo artigo de notícia.
    async fn save(&self, article: &NewsArticle) -> Result<(), NewsError>;

    /// Recupera um artigo de notícia pelo seu identificador único.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewsArticle>, NewsError>;

    /// Atualiza um artigo de notícia existente.
    async fn update(&self, article: &NewsArticle) -> Result<(), NewsError>;

    /// Lista os artigos de notícia pertencentes a um Centro Acadêmico específico.
    async fn find_by_ca_id(&self, ca_id: Uuid) -> Result<Vec<NewsArticle>, NewsError>;
}

/// Contrato para verificação de permissão e autorização de gestão em notícias.
#[async_trait]
pub trait NewsAuthorizationPort: Send + Sync {
    /// Verifica se o usuário possui permissão de gestão no Centro Acadêmico.
    async fn can_manage_news(&self, user_id: Uuid, ca_id: Uuid) -> Result<bool, NewsError>;

    /// Verifica se o usuário possui permissão para publicação direta no Centro Acadêmico.
    async fn can_publish_directly(&self, user_id: Uuid, ca_id: Uuid) -> Result<bool, NewsError>;
}
