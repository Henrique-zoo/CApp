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
        ports::{AcademicAffiliation, AcademicAffiliationPort, DemandRepository},
    },
    domain::entities::{Demand, DemandForum, DemandStatus, DemandStatusHistoryEntry},
};

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
        let demand_row = sqlx::query(
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

        let demand_id: Uuid = row.get("id");
        let academic_center_id: Uuid = row.get("academic_center_id");
        let author_id: Uuid = row.get("author_id");
        let forum_str: String = row.get("forum");
        let title: Option<String> = row.get("title");
        let description: String = row.get("description");
        let category: Option<String> = row.get("category");
        let course_name: Option<String> = row.get("course_name");
        let status_str: String = row.get("status");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        let forum: DemandForum = forum_str
            .parse()
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;
        let status: DemandStatus = status_str
            .parse()
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

        let history_rows = sqlx::query(
            "SELECT id, demand_id, from_status, to_status, changed_by_id, comment, created_at
             FROM demand_status_history
             WHERE demand_id = $1
             ORDER BY created_at ASC",
        )
        .bind(demand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DemandRepositoryError::Internal(e.to_string()))?;

        let mut history_entries = Vec::with_capacity(history_rows.len());
        for h_row in history_rows {
            let h_id: Uuid = h_row.get("id");
            let h_demand_id: Uuid = h_row.get("demand_id");
            let from_status_str: Option<String> = h_row.get("from_status");
            let to_status_str: String = h_row.get("to_status");
            let changed_by_id: Uuid = h_row.get("changed_by_id");
            let comment: Option<String> = h_row.get("comment");
            let h_created_at: DateTime<Utc> = h_row.get("created_at");

            let from_status = match from_status_str {
                Some(s) => Some(
                    s.parse::<DemandStatus>()
                        .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?,
                ),
                None => None,
            };
            let to_status: DemandStatus = to_status_str
                .parse()
                .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            let entry = DemandStatusHistoryEntry::new(
                h_id,
                h_demand_id,
                from_status,
                to_status,
                changed_by_id,
                comment,
                h_created_at,
            )
            .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

            history_entries.push(entry);
        }

        let demand = Demand::reconstitute(
            demand_id,
            academic_center_id,
            author_id,
            forum,
            title,
            description,
            category,
            course_name,
            status,
            created_at,
            updated_at,
            history_entries,
        )
        .map_err(|e| DemandRepositoryError::Internal(format!("{e}")))?;

        Ok(Some(demand))
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
