//! Adaptadores de persistência e autorização para o módulo de notícias.
//!
//! Fornece implementações em memória ([`InMemoryNewsRepository`]) para testes rápidos
//! e em PostgreSQL ([`PgNewsRepository`]) para persistência relacional de produção.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::news::{
    application::ports::{
        NewsAuthorizationError, NewsAuthorizationPort, NewsRepositoryError, NewsRepositoryPort,
    },
    domain::{NewsArticle, NewsStatus},
};

/// Repositório de notícias em memória baseado em `RwLock<HashMap>`.
///
/// Fornece isolamento completo sem dependência de banco de dados,
/// sendo ideal para testes unitários e de integração de serviços de aplicação.
#[derive(Debug, Default, Clone)]
pub struct InMemoryNewsRepository {
    storage: Arc<RwLock<HashMap<Uuid, NewsArticle>>>,
}

impl InMemoryNewsRepository {
    /// Inicializa um repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inicializa o repositório populado previamente com uma coleção de artigos.
    pub fn with_articles(articles: impl IntoIterator<Item = NewsArticle>) -> Self {
        let map = articles.into_iter().map(|a| (a.id, a)).collect();
        Self {
            storage: Arc::new(RwLock::new(map)),
        }
    }
}

#[async_trait]
impl NewsRepositoryPort for InMemoryNewsRepository {
    async fn save(&self, article: &NewsArticle) -> Result<(), NewsRepositoryError> {
        let mut guard = self.storage.write().await;
        guard.insert(article.id, article.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewsArticle>, NewsRepositoryError> {
        let guard = self.storage.read().await;
        Ok(guard.get(&id).cloned())
    }

    async fn update(&self, article: &NewsArticle) -> Result<(), NewsRepositoryError> {
        let mut guard = self.storage.write().await;
        if let std::collections::hash_map::Entry::Occupied(mut entry) = guard.entry(article.id) {
            entry.insert(article.clone());
            Ok(())
        } else {
            Err(NewsRepositoryError::NotFound(article.id))
        }
    }

    async fn update_status_atomic(
        &self,
        id: Uuid,
        expected_current_status: NewsStatus,
        new_status: NewsStatus,
        published_at: Option<DateTime<Utc>>,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<NewsArticle, NewsRepositoryError> {
        let mut guard = self.storage.write().await;
        let article = guard
            .get_mut(&id)
            .ok_or(NewsRepositoryError::NotFound(id))?;

        if article.status != expected_current_status {
            return Err(NewsRepositoryError::OptimisticLockConflict {
                id,
                expected: expected_current_status,
            });
        }

        article.status = new_status;
        if let Some(pub_time) = published_at {
            article.published_at = Some(pub_time);
        }
        if let Some(arc_time) = archived_at {
            article.archived_at = Some(arc_time);
        }

        Ok(article.clone())
    }
}

/// Adaptador de persistência para PostgreSQL via SQLx.
#[derive(Debug, Clone)]
pub struct PgNewsRepository {
    pool: PgPool,
}

impl PgNewsRepository {
    /// Cria uma nova instância conectada ao pool PostgreSQL fornecido.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NewsRepositoryPort for PgNewsRepository {
    async fn save(&self, article: &NewsArticle) -> Result<(), NewsRepositoryError> {
        let status_str = article.status.as_str();

        sqlx::query(
            r#"
            INSERT INTO news_article (
                id, ca_id, author_id, title, body, status, revision, created_at, published_at, archived_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(article.id)
        .bind(article.ca_id)
        .bind(article.author_id)
        .bind(&article.title)
        .bind(&article.body)
        .bind(status_str)
        .bind(article.revision)
        .bind(article.created_at)
        .bind(article.published_at)
        .bind(article.archived_at)
        .execute(&self.pool)
        .await
        .map_err(|e| NewsRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewsArticle>, NewsRepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, ca_id, author_id, title, body, status, revision, created_at, published_at, archived_at
            FROM news_article
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| NewsRepositoryError::DatabaseError(e.to_string()))?;

        row.map(|r| {
            let status_raw: String = r.get("status");
            let status: NewsStatus = status_raw.parse().map_err(|e| {
                NewsRepositoryError::DatabaseError(format!("Status inválido no banco: {e}"))
            })?;

            Ok(NewsArticle::reconstruct(
                r.get("id"),
                r.get("ca_id"),
                r.get("author_id"),
                r.get("title"),
                r.get("body"),
                status,
                r.get("revision"),
                r.get("created_at"),
                r.get("published_at"),
                r.get("archived_at"),
            ))
        })
        .transpose()
    }

    async fn update(&self, article: &NewsArticle) -> Result<(), NewsRepositoryError> {
        let status_str = article.status.as_str();

        let result = sqlx::query(
            r#"
            UPDATE news_article
            SET title = $1, body = $2, status = $3, revision = $4, published_at = $5, archived_at = $6
            WHERE id = $7
            "#,
        )
        .bind(&article.title)
        .bind(&article.body)
        .bind(status_str)
        .bind(article.revision)
        .bind(article.published_at)
        .bind(article.archived_at)
        .bind(article.id)
        .execute(&self.pool)
        .await
        .map_err(|e| NewsRepositoryError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            Err(NewsRepositoryError::NotFound(article.id))
        } else {
            Ok(())
        }
    }

    async fn update_status_atomic(
        &self,
        id: Uuid,
        expected_current_status: NewsStatus,
        new_status: NewsStatus,
        published_at: Option<DateTime<Utc>>,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<NewsArticle, NewsRepositoryError> {
        let expected_str = expected_current_status.as_str();
        let new_status_str = new_status.as_str();

        let updated_row = sqlx::query(
            r#"
            UPDATE news_article
            SET status = $1,
                published_at = COALESCE($2, published_at),
                archived_at = COALESCE($3, archived_at)
            WHERE id = $4 AND status = $5
            RETURNING id, ca_id, author_id, title, body, status, revision, created_at, published_at, archived_at
            "#,
        )
        .bind(new_status_str)
        .bind(published_at)
        .bind(archived_at)
        .bind(id)
        .bind(expected_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| NewsRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(r) = updated_row {
            let status_raw: String = r.get("status");
            let parsed_status: NewsStatus = status_raw.parse().map_err(|e| {
                NewsRepositoryError::DatabaseError(format!("Status retornado inválido: {e}"))
            })?;

            Ok(NewsArticle::reconstruct(
                r.get("id"),
                r.get("ca_id"),
                r.get("author_id"),
                r.get("title"),
                r.get("body"),
                parsed_status,
                r.get("revision"),
                r.get("created_at"),
                r.get("published_at"),
                r.get("archived_at"),
            ))
        } else {
            // Verifica se a linha não existe ou se divergiu o status esperado
            let current = self.find_by_id(id).await?;
            match current {
                Some(_) => Err(NewsRepositoryError::OptimisticLockConflict {
                    id,
                    expected: expected_current_status,
                }),
                None => Err(NewsRepositoryError::NotFound(id)),
            }
        }
    }
}

/// Adaptador de autorização configurável em memória por permissões explícitas.
///
/// Permite conceder permissões por par `(user_id, ca_id)` de forma determinística
/// em testes de regras de acesso institucionais.
#[derive(Debug, Default, Clone)]
pub struct StaticRoleNewsAuthorization {
    publishers: HashSet<(Uuid, Uuid)>,
    archivers: HashSet<(Uuid, Uuid)>,
}

impl StaticRoleNewsAuthorization {
    /// Cria uma nova instância sem permissões prévias.
    pub fn new() -> Self {
        Self::default()
    }

    /// Concede permissão de publicação para um usuário em um Centro Acadêmico específico.
    pub fn grant_publish(&mut self, user_id: Uuid, ca_id: Uuid) {
        self.publishers.insert((user_id, ca_id));
    }

    /// Concede permissão de arquivamento para um usuário em um Centro Acadêmico específico.
    pub fn grant_archive(&mut self, user_id: Uuid, ca_id: Uuid) {
        self.archivers.insert((user_id, ca_id));
    }

    /// Concede permissão integral de gestão (publicação e arquivamento) para o usuário no CA.
    pub fn grant_manage(&mut self, user_id: Uuid, ca_id: Uuid) {
        self.grant_publish(user_id, ca_id);
        self.grant_archive(user_id, ca_id);
    }
}

#[async_trait]
impl NewsAuthorizationPort for StaticRoleNewsAuthorization {
    async fn can_publish_news(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        Ok(self.publishers.contains(&(user_id, ca_id)))
    }

    async fn can_archive_news(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        Ok(self.archivers.contains(&(user_id, ca_id)))
    }

    async fn can_submit_news(
        &self,
        user_id: Uuid,
        ca_id: Uuid,
        author_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        let is_author = user_id == author_id;
        let is_publisher = self.publishers.contains(&(user_id, ca_id));
        Ok(is_author || is_publisher)
    }
}

/// Implementação permissiva de autorização para ambientes onde toda operação é permitida.
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAllNewsAuthorization;

#[async_trait]
impl NewsAuthorizationPort for AllowAllNewsAuthorization {
    async fn can_publish_news(
        &self,
        _user_id: Uuid,
        _ca_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        Ok(true)
    }

    async fn can_archive_news(
        &self,
        _user_id: Uuid,
        _ca_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        Ok(true)
    }

    async fn can_submit_news(
        &self,
        _user_id: Uuid,
        _ca_id: Uuid,
        _author_id: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deve_salvar_e_recuperar_artigo_em_memoria() {
        let repo = InMemoryNewsRepository::new();
        let id = Uuid::new_v4();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let article = NewsArticle::new(
            id,
            ca_id,
            author_id,
            "Comunicado de Eleições".to_string(),
            "Edital de convocação das eleições do Centro Acadêmico.".to_string(),
            Utc::now(),
        )
        .unwrap();

        repo.save(&article).await.unwrap();

        let encontrado = repo.find_by_id(id).await.unwrap();
        assert!(encontrado.is_some());
        assert_eq!(encontrado.unwrap().title, "Comunicado de Eleições");
    }

    #[tokio::test]
    async fn deve_atualizar_status_atomicamente_com_sucesso() {
        let repo = InMemoryNewsRepository::new();
        let id = Uuid::new_v4();
        let article = NewsArticle::new(
            id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Título".to_string(),
            "Corpo".to_string(),
            Utc::now(),
        )
        .unwrap();

        repo.save(&article).await.unwrap();

        let now = Utc::now();
        let atualizado = repo
            .update_status_atomic(
                id,
                NewsStatus::Draft,
                NewsStatus::Published,
                Some(now),
                None,
            )
            .await
            .unwrap();

        assert_eq!(atualizado.status, NewsStatus::Published);
        assert_eq!(atualizado.published_at, Some(now));
    }

    #[tokio::test]
    async fn deve_rejeitar_atualizacao_atomica_se_status_divergir() {
        let repo = InMemoryNewsRepository::new();
        let id = Uuid::new_v4();
        let article = NewsArticle::new(
            id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Título".to_string(),
            "Corpo".to_string(),
            Utc::now(),
        )
        .unwrap();

        repo.save(&article).await.unwrap();

        // Tenta atualizar esperando PendingReview, mas está em Draft
        let resultado = repo
            .update_status_atomic(
                id,
                NewsStatus::PendingReview,
                NewsStatus::Published,
                None,
                None,
            )
            .await;

        match resultado.unwrap_err() {
            NewsRepositoryError::OptimisticLockConflict {
                id: err_id,
                expected,
            } => {
                assert_eq!(err_id, id);
                assert_eq!(expected, NewsStatus::PendingReview);
            }
            outro => panic!("Esperava OptimisticLockConflict, recebeu: {outro:?}"),
        }
    }

    #[tokio::test]
    async fn deve_rejeitar_atualizacao_atomica_de_artigo_inexistente() {
        let repo = InMemoryNewsRepository::new();
        let id_inexistente = Uuid::new_v4();

        let resultado = repo
            .update_status_atomic(
                id_inexistente,
                NewsStatus::Draft,
                NewsStatus::Published,
                None,
                None,
            )
            .await;

        match resultado.unwrap_err() {
            NewsRepositoryError::NotFound(err_id) => assert_eq!(err_id, id_inexistente),
            outro => panic!("Esperava NotFound, recebeu: {outro:?}"),
        }
    }
}
