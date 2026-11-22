//! Adaptadores de persistência do módulo de demandas.
//!
//! Contém as implementações concreta ([`PostgresDemandRepository`]) e em memória
//! ([`InMemoryDemandRepository`] e [`InMemoryAcademicAffiliationPort`]).

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::modules::demands::{
    application::{
        error::{AffiliationError, DemandRepositoryError},
        ports::{
            AcademicAffiliation, AcademicAffiliationPort, DemandFilter,
            DemandNotificationRepository, DemandRepository,
        },
    },
    domain::entities::{
        Demand, DemandForum, DemandNotification, DemandNotificationEventType, DemandStatus,
        DemandStatusHistoryEntry,
    },
};

/// Estrutura de linha dedicada para mapeamento relacional da tabela `demand`.
#[derive(Debug, sqlx::FromRow)]
struct DemandRow {
    id: Uuid,
    academic_center_id: Uuid,
    author_id: Uuid,
    forum: String,
    title: Option<String>,
    description: String,
    category: Option<String>,
    course_name: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Estrutura de linha dedicada para mapeamento relacional da tabela `demand_status_history`.
#[derive(Debug, sqlx::FromRow)]
struct DemandStatusHistoryRow {
    id: Uuid,
    demand_id: Uuid,
    from_status: Option<String>,
    to_status: String,
    changed_by_id: Uuid,
    comment: Option<String>,
    created_at: DateTime<Utc>,
}

/// Repositório de demandas em memória para testes unitários determinísticos.
#[derive(Debug, Clone, Default)]
pub struct InMemoryDemandRepository {
    demands: Arc<RwLock<HashMap<Uuid, Demand>>>,
}

impl InMemoryDemandRepository {
    /// Cria uma nova instância vazia do repositório em memória.
    pub fn new() -> Self {
        Self {
            demands: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Retorna todas as demandas armazenadas em memória.
    pub fn all(&self) -> Vec<Demand> {
        let guard = self.demands.read().unwrap();
        guard.values().cloned().collect()
    }

    /// Quantidade de demandas armazenadas.
    pub fn count(&self) -> usize {
        let guard = self.demands.read().unwrap();
        guard.len()
    }

    /// Limpa o repositório em memória.
    pub fn clear(&self) {
        let mut guard = self.demands.write().unwrap();
        guard.clear();
    }
}

#[async_trait::async_trait]
impl DemandRepository for InMemoryDemandRepository {
    async fn create(&self, demand: &Demand) -> Result<(), DemandRepositoryError> {
        let mut guard = self.demands.write().unwrap();
        guard.insert(demand.id(), demand.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Demand>, DemandRepositoryError> {
        let guard = self.demands.read().unwrap();
        Ok(guard.get(&id).cloned())
    }

    async fn update(&self, demand: &Demand) -> Result<(), DemandRepositoryError> {
        let mut guard = self.demands.write().unwrap();
        guard.insert(demand.id(), demand.clone());
        Ok(())
    }

    async fn list(&self, filter: &DemandFilter) -> Result<Vec<Demand>, DemandRepositoryError> {
        let guard = self.demands.read().unwrap();
        let mut filtered: Vec<Demand> = guard
            .values()
            .filter(|d| {
                if d.academic_center_id() != filter.academic_center_id {
                    return false;
                }
                if filter.forum.is_some_and(|forum| d.forum() != forum) {
                    return false;
                }
                if filter.status.is_some_and(|status| d.status() != status) {
                    return false;
                }
                if filter
                    .author_id
                    .is_some_and(|author_id| d.author_id() != author_id)
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        // Ordenação determinística: created_at DESC, id DESC
        filtered.sort_by(|a, b| {
            b.created_at()
                .cmp(&a.created_at())
                .then_with(|| b.id().cmp(&a.id()))
        });

        let offset = filter.offset.unwrap_or(0);
        let iter = filtered.into_iter().skip(offset);
        let result = if let Some(limit) = filter.limit {
            iter.take(limit).collect()
        } else {
            iter.collect()
        };

        Ok(result)
    }
}

/// Estrutura de linha dedicada para mapeamento relacional da tabela `demand_notification`.
#[derive(Debug, sqlx::FromRow)]
struct DemandNotificationRow {
    id: Uuid,
    demand_id: Uuid,
    recipient_user_id: Uuid,
    academic_center_id: Uuid,
    event_type: String,
    title: String,
    message: String,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

/// Repositório de notificações de demandas em memória para testes unitários determinísticos.
#[derive(Debug, Clone, Default)]
pub struct InMemoryDemandNotificationRepository {
    notifications: Arc<RwLock<HashMap<Uuid, DemandNotification>>>,
}

impl InMemoryDemandNotificationRepository {
    /// Cria uma nova instância vazia do repositório de notificações em memória.
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Retorna todas as notificações armazenadas.
    pub fn all(&self) -> Vec<DemandNotification> {
        let guard = self.notifications.read().unwrap();
        guard.values().cloned().collect()
    }

    /// Quantidade de notificações armazenadas.
    pub fn count(&self) -> usize {
        let guard = self.notifications.read().unwrap();
        guard.len()
    }

    /// Limpa todas as notificações armazenadas.
    pub fn clear(&self) {
        let mut guard = self.notifications.write().unwrap();
        guard.clear();
    }
}

#[async_trait::async_trait]
impl DemandNotificationRepository for InMemoryDemandNotificationRepository {
    async fn create(&self, notification: &DemandNotification) -> Result<(), DemandRepositoryError> {
        let mut guard = self.notifications.write().unwrap();
        guard.insert(notification.id(), notification.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DemandNotification>, DemandRepositoryError> {
        let guard = self.notifications.read().unwrap();
        Ok(guard.get(&id).cloned())
    }

    async fn list_by_recipient(
        &self,
        recipient_user_id: Uuid,
        unread_only: bool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<DemandNotification>, DemandRepositoryError> {
        let guard = self.notifications.read().unwrap();
        let mut filtered: Vec<DemandNotification> = guard
            .values()
            .filter(|n| {
                if n.recipient_user_id() != recipient_user_id {
                    return false;
                }
                if unread_only && n.is_read() {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        // Ordenação determinística: created_at DESC, id DESC
        filtered.sort_by(|a, b| {
            b.created_at()
                .cmp(&a.created_at())
                .then_with(|| b.id().cmp(&a.id()))
        });

        let offset = offset.unwrap_or(0);
        let iter = filtered.into_iter().skip(offset);
        let result = if let Some(limit) = limit {
            iter.take(limit).collect()
        } else {
            iter.collect()
        };

        Ok(result)
    }

    async fn update(&self, notification: &DemandNotification) -> Result<(), DemandRepositoryError> {
        let mut guard = self.notifications.write().unwrap();
        guard.insert(notification.id(), notification.clone());
        Ok(())
    }
}

/// Adaptador em memória para consulta de vínculo acadêmico e autorização de CA em testes unitários.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAcademicAffiliationPort {
    affiliations: Arc<RwLock<HashMap<(Uuid, Uuid), AcademicAffiliation>>>,
    default_affiliation: Arc<RwLock<Option<AcademicAffiliation>>>,
}

impl InMemoryAcademicAffiliationPort {
    /// Cria uma nova instância sem configurações padrão.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cria uma instância com uma filiação padrão retornada quando não houver registro específico.
    pub fn with_default(default: AcademicAffiliation) -> Self {
        Self {
            affiliations: Arc::new(RwLock::new(HashMap::new())),
            default_affiliation: Arc::new(RwLock::new(Some(default))),
        }
    }

    /// Define a filiação específica de um autor em determinado Centro Acadêmico.
    pub fn set_affiliation(
        &self,
        author_id: Uuid,
        academic_center_id: Uuid,
        affiliation: AcademicAffiliation,
    ) {
        let mut guard = self.affiliations.write().unwrap();
        guard.insert((author_id, academic_center_id), affiliation);
    }
}

#[async_trait::async_trait]
impl AcademicAffiliationPort for InMemoryAcademicAffiliationPort {
    async fn get_affiliation(
        &self,
        author_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<AcademicAffiliation, AffiliationError> {
        let guard = self.affiliations.read().unwrap();
        if let Some(aff) = guard.get(&(author_id, academic_center_id)) {
            return Ok(aff.clone());
        }

        let def_guard = self.default_affiliation.read().unwrap();
        if let Some(ref def) = *def_guard {
            return Ok(def.clone());
        }

        // Padrão seguro: CA não encontrado
        Ok(AcademicAffiliation {
            academic_center_exists: false,
            academic_center_active: false,
            has_active_student_membership: false,
            student_course: None,
            ca_represented_courses: Vec::new(),
            is_ca_manager: false,
        })
    }
}

/// Implementação do repositório de demandas sobre o PostgreSQL.
#[derive(Debug, Clone)]
pub struct PostgresDemandRepository {
    pool: PgPool,
}

impl PostgresDemandRepository {
    /// Cria um novo repositório com o pool PostgreSQL fornecido.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Retorna uma referência ao pool de conexões PostgreSQL subjacente.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl DemandRepository for PostgresDemandRepository {
    async fn create(&self, demand: &Demand) -> Result<(), DemandRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO demand (
                id, academic_center_id, author_id, forum, title, description,
                category, course_name, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(demand.id())
        .bind(demand.academic_center_id())
        .bind(demand.author_id())
        .bind(demand.forum().as_str())
        .bind(demand.title())
        .bind(demand.description())
        .bind(demand.category())
        .bind(demand.course_name())
        .bind(demand.status().as_str())
        .bind(demand.created_at())
        .bind(demand.updated_at())
        .execute(&mut *tx)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let history_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO demand_status_history (
                id, demand_id, from_status, to_status, changed_by_id, comment, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(history_id)
        .bind(demand.id())
        .bind(None::<String>)
        .bind(demand.status().as_str())
        .bind(demand.author_id())
        .bind(None::<String>)
        .bind(demand.created_at())
        .execute(&mut *tx)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Demand>, DemandRepositoryError> {
        let demand_row = sqlx::query_as::<_, DemandRow>(
            "SELECT id, academic_center_id, author_id, forum, title, description,
                    category, course_name, status, created_at, updated_at
             FROM demand WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let Some(row) = demand_row else {
            return Ok(None);
        };

        let forum: DemandForum = row
            .forum
            .parse()
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;
        let status: DemandStatus = row
            .status
            .parse()
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

        let history_rows = sqlx::query_as::<_, DemandStatusHistoryRow>(
            "SELECT id, demand_id, from_status, to_status, changed_by_id, comment, created_at
             FROM demand_status_history
             WHERE demand_id = $1
             ORDER BY created_at ASC",
        )
        .bind(row.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let mut history_entries = Vec::with_capacity(history_rows.len());
        for h_row in history_rows {
            let from_status = match h_row.from_status {
                Some(s) => Some(
                    s.parse::<DemandStatus>()
                        .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?,
                ),
                None => None,
            };
            let to_status: DemandStatus = h_row
                .to_status
                .parse()
                .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            let entry = DemandStatusHistoryEntry::new(
                h_row.id,
                h_row.demand_id,
                from_status,
                to_status,
                h_row.changed_by_id,
                h_row.comment,
                h_row.created_at,
            )
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            history_entries.push(entry);
        }

        let demand = Demand::reconstitute(
            row.id,
            row.academic_center_id,
            row.author_id,
            forum,
            row.title,
            row.description,
            row.category,
            row.course_name,
            status,
            row.created_at,
            row.updated_at,
            history_entries,
        )
        .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

        Ok(Some(demand))
    }

    async fn list(&self, filter: &DemandFilter) -> Result<Vec<Demand>, DemandRepositoryError> {
        let mut builder = sqlx::QueryBuilder::new(
            "SELECT id, academic_center_id, author_id, forum, title, description, \
                    category, course_name, status, created_at, updated_at \
             FROM demand \
             WHERE academic_center_id = ",
        );
        builder.push_bind(filter.academic_center_id);

        if let Some(forum) = filter.forum {
            builder.push(" AND forum = ");
            builder.push_bind(forum.as_str());
        }

        if let Some(status) = filter.status {
            builder.push(" AND status = ");
            builder.push_bind(status.as_str());
        }

        if let Some(author_id) = filter.author_id {
            builder.push(" AND author_id = ");
            builder.push_bind(author_id);
        }

        builder.push(" ORDER BY created_at DESC, id DESC");

        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }

        let rows = builder
            .build_query_as::<DemandRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let mut demands = Vec::with_capacity(rows.len());
        for row in rows {
            let forum: DemandForum = row
                .forum
                .parse()
                .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;
            let status: DemandStatus = row
                .status
                .parse()
                .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            let demand = Demand::reconstitute(
                row.id,
                row.academic_center_id,
                row.author_id,
                forum,
                row.title,
                row.description,
                row.category,
                row.course_name,
                status,
                row.created_at,
                row.updated_at,
                Vec::new(),
            )
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            demands.push(demand);
        }

        Ok(demands)
    }

    async fn update(&self, demand: &Demand) -> Result<(), DemandRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE demand
             SET status = $1, updated_at = $2
             WHERE id = $3 AND academic_center_id = $4",
        )
        .bind(demand.status().as_str())
        .bind(demand.updated_at())
        .bind(demand.id())
        .bind(demand.academic_center_id())
        .execute(&mut *tx)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        if let Some(entry) = demand.status_history().last() {
            sqlx::query(
                "INSERT INTO demand_status_history (
                    id, demand_id, from_status, to_status, changed_by_id, comment, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO NOTHING",
            )
            .bind(entry.id())
            .bind(entry.demand_id())
            .bind(entry.from_status().map(|s| s.as_str().to_string()))
            .bind(entry.to_status().as_str())
            .bind(entry.changed_by_id())
            .bind(entry.comment())
            .bind(entry.created_at())
            .execute(&mut *tx)
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl AcademicAffiliationPort for PostgresDemandRepository {
    async fn get_affiliation(
        &self,
        author_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<AcademicAffiliation, AffiliationError> {
        let ca_row = sqlx::query(
            "SELECT id, name, acronym, status, created_by_id
             FROM academic_center
             WHERE id = $1",
        )
        .bind(academic_center_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AffiliationError::Internal(e.to_string()))?;

        let Some(ca) = ca_row else {
            return Ok(AcademicAffiliation {
                academic_center_exists: false,
                academic_center_active: false,
                has_active_student_membership: false,
                student_course: None,
                ca_represented_courses: Vec::new(),
                is_ca_manager: false,
            });
        };

        let ca_name: String = ca.get("name");
        let ca_acronym: Option<String> = ca.get("acronym");
        let ca_status: String = ca.get("status");
        let ca_created_by: Option<Uuid> = ca.get("created_by_id");

        let academic_center_exists = true;
        let academic_center_active = ca_status == "active";

        // Mapeamento de cursos representados pelo Centro Acadêmico
        let mut represented_courses = Vec::new();
        represented_courses.push(ca_name.clone());

        // Extrai curso a partir do nome padrão (ex: "Centro Acadêmico de Ciência da Computação")
        if let Some(course) = ca_name.strip_prefix("Centro Acadêmico de ") {
            represented_courses.push(course.trim().to_string());
        } else if let Some(course) = ca_name.strip_prefix("Centro Acadêmico do ") {
            represented_courses.push(course.trim().to_string());
        } else if let Some(course) = ca_name.strip_prefix("Centro Acadêmico da ") {
            represented_courses.push(course.trim().to_string());
        } else if let Some(course) = ca_name.strip_prefix("CA de ") {
            represented_courses.push(course.trim().to_string());
        } else if let Some(course) = ca_name.strip_prefix("CA do ") {
            represented_courses.push(course.trim().to_string());
        } else if let Some(course) = ca_name.strip_prefix("CA da ") {
            represented_courses.push(course.trim().to_string());
        }

        // Casos didáticos e siglas conhecidas
        if let Some(ref acr) = ca_acronym {
            let acr_upper = acr.trim().to_uppercase();
            if acr_upper == "CAER" {
                represented_courses.push("Engenharia de Redes".to_string());
                represented_courses.push("Engenharia de Redes de Comunicação".to_string());
            } else if acr_upper == "CACC" || acr_upper == "CACOMP" {
                represented_courses.push("Ciência da Computação".to_string());
                represented_courses.push("Computação".to_string());
                represented_courses.push("Licenciatura em Computação".to_string());
            }
        }

        // Consulta usuário e evidência institucional
        let user_row = sqlx::query(
            "SELECT u.id, u.status, v.student_membership_confirmed, v.course_name, v.expires_at
             FROM app_user u
             LEFT JOIN institutional_verification v ON v.user_id = u.id
             WHERE u.id = $1",
        )
        .bind(author_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AffiliationError::Internal(e.to_string()))?;

        let Some(user) = user_row else {
            return Ok(AcademicAffiliation {
                academic_center_exists,
                academic_center_active,
                has_active_student_membership: false,
                student_course: None,
                ca_represented_courses: represented_courses,
                is_ca_manager: false,
            });
        };

        let student_confirmed: Option<bool> = user.get("student_membership_confirmed");
        let student_course: Option<String> = user.get("course_name");
        let expires_at: Option<DateTime<Utc>> = user.get("expires_at");

        let has_active_student_membership =
            student_confirmed.unwrap_or(false) && expires_at.is_some_and(|exp| exp > Utc::now());

        let is_ca_manager = ca_created_by.is_some() && ca_created_by == Some(author_id);

        Ok(AcademicAffiliation {
            academic_center_exists,
            academic_center_active,
            has_active_student_membership,
            student_course,
            ca_represented_courses: represented_courses,
            is_ca_manager,
        })
    }
}

/// Implementação do repositório de notificações de demandas sobre o PostgreSQL.
#[derive(Debug, Clone)]
pub struct PostgresDemandNotificationRepository {
    pool: PgPool,
}

impl PostgresDemandNotificationRepository {
    /// Cria um novo repositório de notificações com o pool PostgreSQL fornecido.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Retorna uma referência ao pool de conexões PostgreSQL subjacente.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl DemandNotificationRepository for PostgresDemandNotificationRepository {
    async fn create(&self, notification: &DemandNotification) -> Result<(), DemandRepositoryError> {
        sqlx::query(
            "INSERT INTO demand_notification (
                id, demand_id, recipient_user_id, academic_center_id, event_type,
                title, message, read_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(notification.id())
        .bind(notification.demand_id())
        .bind(notification.recipient_user_id())
        .bind(notification.academic_center_id())
        .bind(notification.event_type().as_str())
        .bind(notification.title())
        .bind(notification.message())
        .bind(notification.read_at())
        .bind(notification.created_at())
        .execute(&self.pool)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DemandNotification>, DemandRepositoryError> {
        let row = sqlx::query_as::<_, DemandNotificationRow>(
            "SELECT id, demand_id, recipient_user_id, academic_center_id, event_type,
                    title, message, read_at, created_at
             FROM demand_notification
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let event_type: DemandNotificationEventType = row
            .event_type
            .parse()
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

        let notification = DemandNotification::reconstitute(
            row.id,
            row.demand_id,
            row.recipient_user_id,
            row.academic_center_id,
            event_type,
            row.title,
            row.message,
            row.read_at,
            row.created_at,
        )
        .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

        Ok(Some(notification))
    }

    async fn list_by_recipient(
        &self,
        recipient_user_id: Uuid,
        unread_only: bool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<DemandNotification>, DemandRepositoryError> {
        let mut builder = sqlx::QueryBuilder::new(
            "SELECT id, demand_id, recipient_user_id, academic_center_id, event_type, \
                    title, message, read_at, created_at \
             FROM demand_notification \
             WHERE recipient_user_id = ",
        );
        builder.push_bind(recipient_user_id);

        if unread_only {
            builder.push(" AND read_at IS NULL");
        }

        builder.push(" ORDER BY created_at DESC, id DESC");

        if let Some(limit) = limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }

        if let Some(offset) = offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }

        let rows = builder
            .build_query_as::<DemandNotificationRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let mut notifications = Vec::with_capacity(rows.len());
        for row in rows {
            let event_type: DemandNotificationEventType = row
                .event_type
                .parse()
                .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            let notification = DemandNotification::reconstitute(
                row.id,
                row.demand_id,
                row.recipient_user_id,
                row.academic_center_id,
                event_type,
                row.title,
                row.message,
                row.read_at,
                row.created_at,
            )
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            notifications.push(notification);
        }

        Ok(notifications)
    }

    async fn update(&self, notification: &DemandNotification) -> Result<(), DemandRepositoryError> {
        sqlx::query(
            "UPDATE demand_notification
             SET read_at = $1
             WHERE id = $2",
        )
        .bind(notification.read_at())
        .bind(notification.id())
        .execute(&self.pool)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        Ok(())
    }
}
