//! Adaptadores de persistência do módulo de Centros Acadêmicos.
//!
//! Contém implementações PostgreSQL e em memória para Centros Acadêmicos
//! e Mandatos de gestão, garantindo isolamento entre dados relacionais e domínio.

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    infrastructure::database::PgDatabase,
    modules::ca::{
        application::ports::{
            AcademicCenterRepository, AcademicCenterRepositoryError, MandateRepository,
            MandateRepositoryError,
        },
        domain::entities::{AcademicCenter, AcademicCenterStatus, Mandate, MandateStatus},
    },
};

/// Estrutura de mapeamento relacional (Row) da tabela `academic_center`.
#[derive(Debug, sqlx::FromRow)]
pub struct AcademicCenterRow {
    /// Identificador do Centro Acadêmico.
    pub id: Uuid,
    /// Identificador da instituição associada.
    pub institution_id: Uuid,
    /// Identificador do usuário criador / responsável inicial, se houver.
    pub created_by_id: Option<Uuid>,
    /// Nome oficial do Centro Acadêmico.
    pub name: String,
    /// Sigla do Centro Acadêmico (opcional).
    pub acronym: Option<String>,
    /// Descrição institucional.
    pub description: String,
    /// Status cadastral ('active' ou 'archived').
    pub status: String,
    /// Instante UTC de criação.
    pub created_at: DateTime<Utc>,
    /// Instante UTC da última modificação.
    pub updated_at: DateTime<Utc>,
}

impl AcademicCenterRow {
    /// Converte o registro SQL para a entidade de domínio [`AcademicCenter`].
    pub fn into_entity(self) -> Result<AcademicCenter, AcademicCenterRepositoryError> {
        let status = AcademicCenterStatus::from_str(&self.status)
            .map_err(|e| AcademicCenterRepositoryError::Internal(e.to_string()))?;

        AcademicCenter::reconstitute(
            self.id,
            self.institution_id,
            self.created_by_id,
            self.name,
            self.acronym,
            self.description,
            status,
            self.created_at,
            self.updated_at,
        )
        .map_err(|e| AcademicCenterRepositoryError::Internal(e.to_string()))
    }
}

/// Estrutura de mapeamento relacional (Row) da tabela `academic_center_mandate`.
#[derive(Debug, sqlx::FromRow)]
pub struct MandateRow {
    /// Identificador único do mandato.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico ao qual o mandato pertence.
    pub academic_center_id: Uuid,
    /// Identificador do usuário que registrou a configuração inicial do mandato.
    pub created_by_id: Uuid,
    /// Título opcional da chapa ou gestão.
    pub title: Option<String>,
    /// Data de início do período do mandato.
    pub start_date: NaiveDate,
    /// Data de término do período do mandato.
    pub end_date: NaiveDate,
    /// Situação do mandato ('active', 'concluded' ou 'archived').
    pub status: String,
    /// Instante UTC de registro da configuração do mandato.
    pub created_at: DateTime<Utc>,
    /// Instante UTC da última modificação do registro.
    pub updated_at: DateTime<Utc>,
}

impl MandateRow {
    /// Converte o registro SQL para a entidade de domínio [`Mandate`].
    pub fn into_entity(self) -> Result<Mandate, MandateRepositoryError> {
        let status = MandateStatus::from_str(&self.status)
            .map_err(|e| MandateRepositoryError::Internal(e.to_string()))?;

        Mandate::reconstitute(
            self.id,
            self.academic_center_id,
            self.created_by_id,
            self.title,
            self.start_date,
            self.end_date,
            status,
            self.created_at,
            self.updated_at,
        )
        .map_err(|e| MandateRepositoryError::Internal(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Implementação PostgreSQL via PgDatabase
// ---------------------------------------------------------------------------

#[async_trait]
impl AcademicCenterRepository for PgDatabase {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<AcademicCenter>, AcademicCenterRepositoryError> {
        let row = sqlx::query_as::<_, AcademicCenterRow>(
            "SELECT id, institution_id, created_by_id, name, acronym, description, status, created_at, updated_at \
             FROM academic_center \
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, %id, "Falha ao consultar Centro Acadêmico");
            AcademicCenterRepositoryError::Internal(e.to_string())
        })?;

        row.map(AcademicCenterRow::into_entity).transpose()
    }
}

#[async_trait]
impl MandateRepository for PgDatabase {
    async fn save(&self, mandate: &Mandate) -> Result<(), MandateRepositoryError> {
        let result = sqlx::query(
            "INSERT INTO academic_center_mandate ( \
                 id, academic_center_id, created_by_id, title, start_date, end_date, status, created_at, updated_at \
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (id) DO UPDATE SET \
                 title = EXCLUDED.title, \
                 start_date = EXCLUDED.start_date, \
                 end_date = EXCLUDED.end_date, \
                 status = EXCLUDED.status, \
                 updated_at = EXCLUDED.updated_at",
        )
        .bind(mandate.id())
        .bind(mandate.academic_center_id())
        .bind(mandate.created_by_id())
        .bind(mandate.title())
        .bind(mandate.start_date())
        .bind(mandate.end_date())
        .bind(mandate.status().as_str())
        .bind(mandate.created_at())
        .bind(mandate.updated_at())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(MandateRepositoryError::Conflict(
                    "já existe um mandato de gestão ativo para este Centro Acadêmico".into(),
                ))
            }
            Err(e) => {
                tracing::error!(%e, id = %mandate.id(), "Falha ao persistir mandato");
                Err(MandateRepositoryError::Internal(e.to_string()))
            }
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, MandateRepositoryError> {
        let row = sqlx::query_as::<_, MandateRow>(
            "SELECT id, academic_center_id, created_by_id, title, start_date, end_date, status, created_at, updated_at \
             FROM academic_center_mandate \
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, %id, "Falha ao consultar mandato por id");
            MandateRepositoryError::Internal(e.to_string())
        })?;

        row.map(MandateRow::into_entity).transpose()
    }

    async fn find_active_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Option<Mandate>, MandateRepositoryError> {
        let row = sqlx::query_as::<_, MandateRow>(
            "SELECT id, academic_center_id, created_by_id, title, start_date, end_date, status, created_at, updated_at \
             FROM academic_center_mandate \
             WHERE academic_center_id = $1 AND status = 'active' \
             ORDER BY created_at DESC \
             LIMIT 1",
        )
        .bind(academic_center_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, %academic_center_id, "Falha ao consultar mandato ativo");
            MandateRepositoryError::Internal(e.to_string())
        })?;

        row.map(MandateRow::into_entity).transpose()
    }

    async fn find_all_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<Mandate>, MandateRepositoryError> {
        let rows = sqlx::query_as::<_, MandateRow>(
            "SELECT id, academic_center_id, created_by_id, title, start_date, end_date, status, created_at, updated_at \
             FROM academic_center_mandate \
             WHERE academic_center_id = $1 \
             ORDER BY start_date DESC",
        )
        .bind(academic_center_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, %academic_center_id, "Falha ao listar mandatos do Centro Acadêmico");
            MandateRepositoryError::Internal(e.to_string())
        })?;

        rows.into_iter().map(MandateRow::into_entity).collect()
    }
}

// ---------------------------------------------------------------------------
// Implementações em memória (InMemory) para testes e isolamento rápido
// ---------------------------------------------------------------------------

/// Repositório de Centro Acadêmico em memória, thread-safe para testes.
#[derive(Debug, Default, Clone)]
pub struct InMemoryAcademicCenterRepository {
    /// Mapa em memória indexado por identificador único de Centro Acadêmico.
    pub centers: Arc<RwLock<HashMap<Uuid, AcademicCenter>>>,
}

impl InMemoryAcademicCenterRepository {
    /// Cria uma nova instância vazia.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insere ou atualiza um Centro Acadêmico em memória.
    pub fn insert(&self, ca: AcademicCenter) {
        self.centers.write().unwrap().insert(ca.id(), ca);
    }
}

#[async_trait]
impl AcademicCenterRepository for InMemoryAcademicCenterRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<AcademicCenter>, AcademicCenterRepositoryError> {
        Ok(self.centers.read().unwrap().get(&id).cloned())
    }
}

/// Repositório de mandatos em memória, thread-safe para testes.
#[derive(Debug, Default, Clone)]
pub struct InMemoryMandateRepository {
    /// Lista em memória dos mandatos persistidos.
    pub mandates: Arc<RwLock<Vec<Mandate>>>,
}

impl InMemoryMandateRepository {
    /// Cria uma nova instância vazia.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MandateRepository for InMemoryMandateRepository {
    async fn save(&self, mandate: &Mandate) -> Result<(), MandateRepositoryError> {
        let mut list = self.mandates.write().unwrap();

        // Verifica unicidade de mandato ativo
        if mandate.is_active() {
            let conflict = list.iter().any(|m| {
                m.id() != mandate.id()
                    && m.academic_center_id() == mandate.academic_center_id()
                    && m.is_active()
            });
            if conflict {
                return Err(MandateRepositoryError::Conflict(
                    "já existe mandato ativo para este CA".into(),
                ));
            }
        }

        list.retain(|m| m.id() != mandate.id());
        list.push(mandate.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, MandateRepositoryError> {
        Ok(self
            .mandates
            .read()
            .unwrap()
            .iter()
            .find(|m| m.id() == id)
            .cloned())
    }

    async fn find_active_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Option<Mandate>, MandateRepositoryError> {
        Ok(self
            .mandates
            .read()
            .unwrap()
            .iter()
            .find(|m| m.academic_center_id() == academic_center_id && m.is_active())
            .cloned())
    }

    async fn find_all_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<Mandate>, MandateRepositoryError> {
        Ok(self
            .mandates
            .read()
            .unwrap()
            .iter()
            .filter(|m| m.academic_center_id() == academic_center_id)
            .cloned()
            .collect())
    }
}
