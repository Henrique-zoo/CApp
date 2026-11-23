//! Implementação PostgreSQL do repositório de eventos usando SQLx.

use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

use crate::modules::events::{
    application::{EventApplicationError, EventRepository},
    domain::{Event, EventPublicationStatus},
};

/// Adaptador PostgreSQL para persistência de eventos na tabela `event`.
#[derive(Debug, Clone)]
pub struct PostgresEventRepository {
    pool: PgPool,
}

impl PostgresEventRepository {
    /// Cria uma nova instância do repositório PostgreSQL para o módulo de eventos.
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn create(&self, event: &Event) -> Result<Event, EventApplicationError> {
        let row = sqlx::query_as::<_, Event>(
            "INSERT INTO event (
                id,
                ca_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                capacity,
                status,
                revision,
                registration_policy
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING
                id,
                ca_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                capacity,
                status,
                revision,
                registration_policy",
        )
        .bind(event.id())
        .bind(event.ca_id())
        .bind(event.author_id())
        .bind(event.title())
        .bind(event.description())
        .bind(event.location())
        .bind(event.starts_at())
        .bind(event.ends_at())
        .bind(event.capacity())
        .bind(event.status())
        .bind(event.revision())
        .bind(Json(event.registration().clone()))
        .fetch_one(&self.pool)
        .await
        .map_err(|error| EventApplicationError::Repository(error.to_string()))?;

        Ok(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, EventApplicationError> {
        sqlx::query_as::<_, Event>(
            "SELECT
                id,
                ca_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                capacity,
                status,
                revision,
                registration_policy
             FROM event
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| EventApplicationError::Repository(error.to_string()))
    }

    async fn list_by_academic_center(
        &self,
        academic_center_id: Uuid,
        only_published: bool,
    ) -> Result<Vec<Event>, EventApplicationError> {
        if only_published {
            sqlx::query_as::<_, Event>(
                "SELECT
                    id,
                    ca_id,
                    author_id,
                    title,
                    description,
                    location,
                    starts_at,
                    ends_at,
                    capacity,
                    status,
                    revision,
                    registration_policy
                 FROM event
                 WHERE ca_id = $1 AND status = $2
                 ORDER BY starts_at ASC",
            )
            .bind(academic_center_id)
            .bind(EventPublicationStatus::Published)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| EventApplicationError::Repository(error.to_string()))
        } else {
            sqlx::query_as::<_, Event>(
                "SELECT
                    id,
                    ca_id,
                    author_id,
                    title,
                    description,
                    location,
                    starts_at,
                    ends_at,
                    capacity,
                    status,
                    revision,
                    registration_policy
                 FROM event
                 WHERE ca_id = $1
                 ORDER BY starts_at ASC",
            )
            .bind(academic_center_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| EventApplicationError::Repository(error.to_string()))
        }
    }

    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError> {
        let row = sqlx::query_as::<_, Event>(
            "UPDATE event SET
                 title = $2,
                 description = $3,
                 location = $4,
                 starts_at = $5,
                 ends_at = $6,
                 capacity = $7,
                 status = $8,
                 revision = $9,
                 registration_policy = $10
             WHERE id = $1
             RETURNING
                id,
                ca_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                capacity,
                status,
                revision,
                registration_policy",
        )
        .bind(event.id())
        .bind(event.title())
        .bind(event.description())
        .bind(event.location())
        .bind(event.starts_at())
        .bind(event.ends_at())
        .bind(event.capacity())
        .bind(event.status())
        .bind(event.revision())
        .bind(Json(event.registration().clone()))
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| EventApplicationError::Repository(error.to_string()))?
        .ok_or(EventApplicationError::NotFound(event.id()))?;

        Ok(row)
    }
}
