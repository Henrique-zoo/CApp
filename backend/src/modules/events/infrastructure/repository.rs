//! Implementação PostgreSQL do repositório de eventos usando SQLx.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::modules::events::{
    application::{EventApplicationError, EventRepository},
    domain::{Event, EventPublicationStatus, EventRegistrationInfo},
};

/// Representação de uma linha da tabela `events` no PostgreSQL.
#[derive(Debug, FromRow)]
struct EventRow {
    id: Uuid,
    academic_center_id: Uuid,
    author_id: Uuid,
    title: String,
    description: String,
    location: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: EventPublicationStatus,
    requires_registration: bool,
    max_capacity: Option<i32>,
    confirmed_registrations: i32,
    registration_deadline: Option<DateTime<Utc>>,
    published_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<EventRow> for Event {
    fn from(row: EventRow) -> Self {
        Self {
            id: row.id,
            academic_center_id: row.academic_center_id,
            author_id: row.author_id,
            title: row.title,
            description: row.description,
            location: row.location,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            status: row.status,
            registration: EventRegistrationInfo {
                requires_registration: row.requires_registration,
                max_capacity: row.max_capacity.and_then(|value| u32::try_from(value).ok()),
                confirmed_registrations: u32::try_from(row.confirmed_registrations).unwrap_or(0),
                registration_deadline: row.registration_deadline,
            },
            published_at: row.published_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Repositório PostgreSQL de eventos baseado em [`PgPool`].
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
        let max_capacity = event
            .registration
            .max_capacity
            .map(|value| i32::try_from(value).unwrap_or(i32::MAX));
        let confirmed_registrations =
            i32::try_from(event.registration.confirmed_registrations).unwrap_or(i32::MAX);

        let row: EventRow = sqlx::query_as(
            "INSERT INTO events (
                id,
                academic_center_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                status,
                requires_registration,
                max_capacity,
                confirmed_registrations,
                registration_deadline,
                published_at,
                created_at,
                updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             RETURNING
                id,
                academic_center_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                status,
                requires_registration,
                max_capacity,
                confirmed_registrations,
                registration_deadline,
                published_at,
                created_at,
                updated_at",
        )
        .bind(event.id)
        .bind(event.academic_center_id)
        .bind(event.author_id)
        .bind(&event.title)
        .bind(&event.description)
        .bind(&event.location)
        .bind(event.starts_at)
        .bind(event.ends_at)
        .bind(event.status)
        .bind(event.registration.requires_registration)
        .bind(max_capacity)
        .bind(confirmed_registrations)
        .bind(event.registration.registration_deadline)
        .bind(event.published_at)
        .bind(event.created_at)
        .bind(event.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        Ok(Event::from(row))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, EventApplicationError> {
        let row: Option<EventRow> = sqlx::query_as(
            "SELECT
                id,
                academic_center_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                status,
                requires_registration,
                max_capacity,
                confirmed_registrations,
                registration_deadline,
                published_at,
                created_at,
                updated_at
             FROM events
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        Ok(row.map(Event::from))
    }

    async fn list_by_academic_center(
        &self,
        academic_center_id: Uuid,
        only_published: bool,
    ) -> Result<Vec<Event>, EventApplicationError> {
        let rows: Vec<EventRow> = if only_published {
            sqlx::query_as(
                "SELECT
                    id,
                    academic_center_id,
                    author_id,
                    title,
                    description,
                    location,
                    starts_at,
                    ends_at,
                    status,
                    requires_registration,
                    max_capacity,
                    confirmed_registrations,
                    registration_deadline,
                    published_at,
                    created_at,
                    updated_at
                 FROM events
                 WHERE academic_center_id = $1 AND status IN ('published', 'publicado')
                 ORDER BY starts_at ASC, created_at ASC",
            )
            .bind(academic_center_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| EventApplicationError::Repository(err.to_string()))?
        } else {
            sqlx::query_as(
                "SELECT
                    id,
                    academic_center_id,
                    author_id,
                    title,
                    description,
                    location,
                    starts_at,
                    ends_at,
                    status,
                    requires_registration,
                    max_capacity,
                    confirmed_registrations,
                    registration_deadline,
                    published_at,
                    created_at,
                    updated_at
                 FROM events
                 WHERE academic_center_id = $1
                 ORDER BY starts_at ASC, created_at ASC",
            )
            .bind(academic_center_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| EventApplicationError::Repository(err.to_string()))?
        };

        Ok(rows.into_iter().map(Event::from).collect())
    }

    async fn update(&self, event: &Event) -> Result<Event, EventApplicationError> {
        let max_capacity = event
            .registration
            .max_capacity
            .map(|value| i32::try_from(value).unwrap_or(i32::MAX));
        let confirmed_registrations =
            i32::try_from(event.registration.confirmed_registrations).unwrap_or(i32::MAX);

        let row: Option<EventRow> = sqlx::query_as(
            "UPDATE events
             SET title = $2,
                 description = $3,
                 location = $4,
                 starts_at = $5,
                 ends_at = $6,
                 status = $7,
                 requires_registration = $8,
                 max_capacity = $9,
                 confirmed_registrations = $10,
                 registration_deadline = $11,
                 published_at = $12,
                 updated_at = $13
             WHERE id = $1
             RETURNING
                id,
                academic_center_id,
                author_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                status,
                requires_registration,
                max_capacity,
                confirmed_registrations,
                registration_deadline,
                published_at,
                created_at,
                updated_at",
        )
        .bind(event.id)
        .bind(&event.title)
        .bind(&event.description)
        .bind(&event.location)
        .bind(event.starts_at)
        .bind(event.ends_at)
        .bind(event.status)
        .bind(event.registration.requires_registration)
        .bind(max_capacity)
        .bind(confirmed_registrations)
        .bind(event.registration.registration_deadline)
        .bind(event.published_at)
        .bind(event.updated_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        row.map(Event::from)
            .ok_or(EventApplicationError::NotFound(event.id))
    }
}
