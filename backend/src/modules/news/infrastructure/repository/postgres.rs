//! Repositório de notícias baseado em PostgreSQL via SQLx.

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::modules::news::application::ports::NewsRepository;
use crate::modules::news::domain::{NewsArticle, NewsError, NewsStatus};

/// Implementação de [`NewsRepository`] que persiste notícias na tabela `news_article`.
#[derive(Debug, Clone)]
pub struct PgNewsRepository {
    pool: PgPool,
}

impl PgNewsRepository {
    /// Constrói o repositório PostgreSQL a partir de um pool SQLx.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NewsRepository for PgNewsRepository {
    async fn save(&self, article: &NewsArticle) -> Result<(), NewsError> {
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
        .bind(article.status.as_str())
        .bind(article.revision as i32)
        .bind(article.created_at)
        .bind(article.published_at)
        .bind(article.archived_at)
        .execute(&self.pool)
        .await
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewsArticle>, NewsError> {
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
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

        row.map(map_row_to_article).transpose()
    }

    async fn update(&self, article: &NewsArticle) -> Result<(), NewsError> {
        let result = sqlx::query(
            r#"
            UPDATE news_article
            SET title = $1, body = $2, status = $3, revision = $4, published_at = $5, archived_at = $6
            WHERE id = $7
            "#,
        )
        .bind(&article.title)
        .bind(&article.body)
        .bind(article.status.as_str())
        .bind(article.revision as i32)
        .bind(article.published_at)
        .bind(article.archived_at)
        .bind(article.id)
        .execute(&self.pool)
        .await
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(NewsError::ArticleNotFound { id: article.id });
        }

        Ok(())
    }

    async fn find_by_ca_id(&self, ca_id: Uuid) -> Result<Vec<NewsArticle>, NewsError> {
        let rows = sqlx::query(
            r#"
            SELECT id, ca_id, author_id, title, body, status, revision, created_at, published_at, archived_at
            FROM news_article
            WHERE ca_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(ca_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

        rows.into_iter().map(map_row_to_article).collect()
    }
}

fn map_row_to_article(row: sqlx::postgres::PgRow) -> Result<NewsArticle, NewsError> {
    let id: Uuid = row
        .try_get("id")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let ca_id: Uuid = row
        .try_get("ca_id")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let author_id: Uuid = row
        .try_get("author_id")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let title: String = row
        .try_get("title")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let body: String = row
        .try_get("body")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let status_str: String = row
        .try_get("status")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let revision: i32 = row
        .try_get("revision")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let created_at: DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let published_at: Option<DateTime<Utc>> = row
        .try_get("published_at")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
    let archived_at: Option<DateTime<Utc>> = row
        .try_get("archived_at")
        .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

    let status = NewsStatus::from_str(&status_str)?;

    Ok(NewsArticle::reconstruct(
        id,
        ca_id,
        author_id,
        title,
        body,
        status,
        revision as u32,
        created_at,
        published_at,
        archived_at,
    ))
}
