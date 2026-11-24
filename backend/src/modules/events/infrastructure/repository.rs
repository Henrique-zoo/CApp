//! Implementação PostgreSQL do repositório de eventos usando SQLx.

use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;
use crate::modules::events::application::EventApplicationError;
use crate::modules::events::application::EventRepository;
use crate::modules::events::domain::Event;
use crate::modules::events::domain::EventPublicationStatus;
use crate::modules::events::application::EventAuthorizationPort;
use crate::modules::events::domain::DIRECT_EVENT_MANAGEMENT_PERMISSION;
use crate::modules::events::domain::EventRegistration;
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

    async fn find_registration(
        &self,
        event_id: Uuid,
        participant_id: Uuid,
    ) -> Result<Option<EventRegistration>, EventApplicationError> {
        let row: Option<EventRegistration> = sqlx::query_as(
            "SELECT
                id,
                event_id,
                participant_id,
                status,
                registered_at,
                cancelled_at
             FROM event_registrations
             WHERE event_id = $1 AND participant_id = $2",
        )
        .bind(event_id)
        .bind(participant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        Ok(row)
    }

    async fn save_registration(
        &self,
        registration: &EventRegistration,
    ) -> Result<EventRegistration, EventApplicationError> {
        let row: EventRegistration = sqlx::query_as(
            "INSERT INTO event_registrations (
                id,
                event_id,
                participant_id,
                status,
                registered_at,
                cancelled_at
             )
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (event_id, participant_id)
             DO UPDATE SET
                status = EXCLUDED.status,
                registered_at = EXCLUDED.registered_at,
                cancelled_at = EXCLUDED.cancelled_at
             RETURNING
                id,
                event_id,
                participant_id,
                status,
                registered_at,
                cancelled_at",
        )
        .bind(registration.id)
        .bind(registration.event_id)
        .bind(registration.participant_id)
        .bind(registration.status)
        .bind(registration.registered_at)
        .bind(registration.cancelled_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        Ok(row)
    }

    async fn list_registrations_by_event(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<EventRegistration>, EventApplicationError> {
        let rows: Vec<EventRegistration> = sqlx::query_as(
            "SELECT
                id,
                event_id,
                participant_id,
                status,
                registered_at,
                cancelled_at
             FROM event_registrations
             WHERE event_id = $1
             ORDER BY registered_at ASC",
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        Ok(rows)
    }
}
/// Adaptador PostgreSQL de controle de acesso e autorização administrativa sobre eventos do CA.
#[derive(Debug, Clone)]
pub struct PostgresEventAuthorizationService {
    pool: PgPool,
}
impl PostgresEventAuthorizationService {
    /// Constrói o serviço PostgreSQL de autorização de eventos baseado em [`PgPool`].
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Concede a permissão [`DIRECT_EVENT_MANAGEMENT_PERMISSION`] a `user_id` na gestão de `academic_center_id`.
    pub async fn grant_direct_event_permission(
        &self,
        academic_center_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), EventApplicationError> {
        sqlx::query(
            "INSERT INTO ca_event_permissions (academic_center_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (academic_center_id, user_id, permission) DO NOTHING",
        )
        .bind(academic_center_id)
        .bind(user_id)
        .bind(DIRECT_EVENT_MANAGEMENT_PERMISSION)
        .execute(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;
        Ok(())
    }

    /// Revoga a permissão [`DIRECT_EVENT_MANAGEMENT_PERMISSION`] de `user_id` em `academic_center_id`.
    pub async fn revoke_direct_event_permission(
        &self,
        academic_center_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), EventApplicationError> {
        sqlx::query(
            "DELETE FROM ca_event_permissions
             WHERE academic_center_id = $1 AND user_id = $2 AND permission = $3",
        )
        .bind(academic_center_id)
        .bind(user_id)
        .bind(DIRECT_EVENT_MANAGEMENT_PERMISSION)
        .execute(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;
        Ok(())
    }
}
#[async_trait]
impl EventAuthorizationPort for PostgresEventAuthorizationService {
    async fn can_manage_and_publish_directly(
        &self,
        user_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<bool, EventApplicationError> {
        let has_permission: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM ca_event_permissions
                WHERE academic_center_id = $1
                  AND user_id = $2
                  AND permission = $3
            )",
        )
        .bind(academic_center_id)
        .bind(user_id)
        .bind(DIRECT_EVENT_MANAGEMENT_PERMISSION)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| EventApplicationError::Repository(err.to_string()))?;

        Ok(has_permission)
    }
}
