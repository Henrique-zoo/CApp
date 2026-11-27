//! Persistência SQLx e resolução do acesso aos Centros Acadêmicos.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    infrastructure::database::PgDatabase,
    modules::materials::{
        application::{
            ports::{
                MaterialAccount, MaterialCenter, MaterialDiscipline, MaterialFile,
                MaterialRepository, Publication, PublishedMaterial,
            },
            services::MaterialError,
        },
        domain::entities::{StudyMaterial, StudyMaterialContent},
    },
};

const ELIGIBLE_CENTERS: &str = "
    FROM academic_center c
    JOIN institution i ON i.id = c.institution_id
    JOIN institutional_verification v ON v.institution_id = i.id AND v.user_id = $1
    JOIN app_user u ON u.id = v.user_id";

const ELIGIBLE_ACCOUNT: &str = "
    WHERE c.status = 'active' AND i.status = 'active' AND u.status <> 'inactive'
      AND v.student_membership_confirmed AND v.microsoft_object_id = $2
      AND v.microsoft_tenant_id = i.microsoft_tenant_id
      AND v.microsoft_student_group_id = i.microsoft_student_group_id
      AND v.checked_at <= clock_timestamp() AND v.expires_at > clock_timestamp()";

#[derive(sqlx::FromRow)]
struct MaterialRow {
    id: Uuid,
    ca_id: Uuid,
    author_id: Uuid,
    discipline_id: Uuid,
    title: String,
    description: String,
    kind: String,
    external_url: Option<String>,
    file_name: Option<String>,
    file_size: Option<i32>,
    file_sha256: Option<Vec<u8>>,
    status: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<MaterialRow> for PublishedMaterial {
    type Error = MaterialError;

    fn try_from(row: MaterialRow) -> Result<Self, Self::Error> {
        let invalid = || MaterialError::Persistence;
        let (content, file) = match row.kind.as_str() {
            "external_link" => (
                StudyMaterialContent::external_link(row.external_url.ok_or_else(invalid)?)
                    .map_err(|_| invalid())?,
                None,
            ),
            "file" => (
                StudyMaterialContent::file(row.id),
                Some(MaterialFile {
                    name: row.file_name.ok_or_else(invalid)?,
                    size: row.file_size.ok_or_else(invalid)?,
                    sha256: row
                        .file_sha256
                        .ok_or_else(invalid)?
                        .try_into()
                        .map_err(|_| invalid())?,
                }),
            ),
            _ => return Err(invalid()),
        };
        let material = StudyMaterial::new(
            row.id,
            row.ca_id,
            row.discipline_id,
            row.author_id,
            row.title,
            row.description,
            content,
            row.status.parse().map_err(|_| invalid())?,
            row.created_at,
        )
        .map_err(|_| invalid())?;
        Ok(Self { material, file })
    }
}

impl PgDatabase {
    async fn eligible_material_centers(
        &self,
        account: MaterialAccount,
        ca_id: Option<Uuid>,
    ) -> Result<Vec<MaterialCenter>, MaterialError> {
        let query = format!(
            "SELECT c.id, c.name, p.ca_id IS NOT NULL {ELIGIBLE_CENTERS}
             LEFT JOIN academic_center_material_publisher p ON p.ca_id = c.id AND p.account_id = u.id
             {ELIGIBLE_ACCOUNT} AND ($3::uuid IS NULL OR c.id = $3) ORDER BY c.name, c.id"
        );
        let rows: Vec<(Uuid, String, bool)> = sqlx::query_as(&query)
            .bind(account.id)
            .bind(account.microsoft_object_id)
            .bind(ca_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(id, name, can_publish)| MaterialCenter {
                id,
                name,
                can_publish,
            })
            .collect())
    }
}

#[async_trait]
impl MaterialRepository for PgDatabase {
    async fn centers(
        &self,
        account: MaterialAccount,
    ) -> Result<Vec<MaterialCenter>, MaterialError> {
        self.eligible_material_centers(account, None).await
    }

    async fn center(
        &self,
        account: MaterialAccount,
        ca_id: Uuid,
    ) -> Result<MaterialCenter, MaterialError> {
        self.eligible_material_centers(account, Some(ca_id))
            .await?
            .into_iter()
            .next()
            .ok_or(MaterialError::Forbidden)
    }

    async fn disciplines(&self, ca_id: Uuid) -> Result<Vec<MaterialDiscipline>, MaterialError> {
        let rows: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT id, name FROM academic_center_discipline WHERE ca_id = $1 ORDER BY name, id",
        )
        .bind(ca_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| MaterialDiscipline { id, name })
            .collect())
    }

    async fn require_discipline(&self, ca_id: Uuid, id: Uuid) -> Result<(), MaterialError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM academic_center_discipline WHERE ca_id = $1 AND id = $2)",
        )
        .bind(ca_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        exists.then_some(()).ok_or(MaterialError::InvalidDiscipline)
    }

    async fn publish(
        &self,
        account: MaterialAccount,
        request_id: Uuid,
        value: PublishedMaterial,
    ) -> Result<Publication, MaterialError> {
        let material = &value.material;
        if account.id != material.author_id() || !material.is_published() {
            return Err(MaterialError::Invalid);
        }
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let query = format!(
            "SELECT v.expires_at {ELIGIBLE_CENTERS}
             JOIN academic_center_material_publisher p ON p.ca_id = c.id AND p.account_id = u.id
             {ELIGIBLE_ACCOUNT} AND c.id = $3 FOR SHARE OF c, i, u, v, p"
        );
        let expiry: DateTime<Utc> = sqlx::query_scalar(&query)
            .bind(account.id)
            .bind(account.microsoft_object_id)
            .bind(material.ca_id())
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .ok_or(MaterialError::Forbidden)?;
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM academic_center_discipline WHERE ca_id = $1 AND id = $2 FOR KEY SHARE",
        )
        .bind(material.ca_id())
        .bind(material.discipline_id())
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?
        .ok_or(MaterialError::InvalidDiscipline)?;
        let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
        if expiry <= now {
            return Err(MaterialError::Forbidden);
        }

        let file = value.file.as_ref();
        let created = sqlx::query_as::<_, MaterialRow>(
            "INSERT INTO study_material
                (id, ca_id, author_id, discipline_id, title, description, kind, external_url, file_name,
                 file_size, file_sha256, status, created_at, request_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'published', $12, $13)
             ON CONFLICT (ca_id, author_id, request_id) DO NOTHING RETURNING *"
        ).bind(material.id()).bind(material.ca_id()).bind(account.id).bind(material.discipline_id())
            .bind(material.title()).bind(material.description()).bind(material.kind().as_str())
            .bind(material.content().external_url()).bind(file.map(|file| &file.name))
            .bind(file.map(|file| file.size)).bind(file.map(|file| file.sha256.as_slice()))
            .bind(now).bind(request_id).fetch_optional(&mut *tx).await.map_err(db_error)?;

        let publication = if let Some(row) = created {
            Publication {
                value: row.try_into()?,
                created: true,
            }
        } else {
            let existing: PublishedMaterial = sqlx::query_as::<_, MaterialRow>(
                "SELECT * FROM study_material WHERE ca_id = $1 AND author_id = $2 AND request_id = $3"
            ).bind(material.ca_id()).bind(account.id).bind(request_id)
                .fetch_one(&mut *tx).await.map_err(db_error)?.try_into()?;
            if !existing.material.is_published()
                || existing.material.discipline_id() != material.discipline_id()
                || existing.material.title() != material.title()
                || existing.material.description() != material.description()
                || existing.material.kind() != material.kind()
                || existing.material.content().external_url() != material.content().external_url()
                || existing.file != value.file
            {
                return Err(MaterialError::Conflict);
            }
            Publication {
                value: existing,
                created: false,
            }
        };
        tx.commit().await.map_err(db_error)?;
        Ok(publication)
    }

    async fn list(
        &self,
        ca_id: Uuid,
        discipline_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PublishedMaterial>, MaterialError> {
        sqlx::query_as::<_, MaterialRow>(
            "SELECT * FROM study_material WHERE ca_id = $1 AND status = 'published'
             AND ($2::uuid IS NULL OR discipline_id = $2)
             ORDER BY created_at DESC, id DESC LIMIT $3 OFFSET $4",
        )
        .bind(ca_id)
        .bind(discipline_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
    }

    async fn find(&self, ca_id: Uuid, id: Uuid) -> Result<PublishedMaterial, MaterialError> {
        sqlx::query_as::<_, MaterialRow>(
            "SELECT * FROM study_material WHERE ca_id = $1 AND id = $2 AND status = 'published'",
        )
        .bind(ca_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?
        .ok_or(MaterialError::NotFound)?
        .try_into()
    }
}

fn db_error(error: sqlx::Error) -> MaterialError {
    tracing::error!(%error, "Falha PostgreSQL no módulo de materiais");
    match error {
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_) => {
            MaterialError::StorageUnavailable
        }
        _ => MaterialError::Persistence,
    }
}
