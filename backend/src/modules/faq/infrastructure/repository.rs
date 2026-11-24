//! Adaptadores de persistência em memória e PostgreSQL para entradas de FAQ.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::modules::faq::{
    application::{
        error::FaqRepositoryError,
        ports::{CaManagementAccess, FaqRepository},
    },
    domain::entities::faq_entry::{FaqEntry, FaqError},
};

/// Linha de banco de dados para mapeamento relacional de entradas de FAQ.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FaqEntryRow {
    /// Identificador primário da entrada de FAQ.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico vinculado.
    pub academic_center_id: Uuid,
    /// Pergunta cadastrada.
    pub question: String,
    /// Resposta cadastrada.
    pub answer: String,
    /// Categoria opcional.
    pub category: Option<String>,
    /// Identificador do autor do cadastro.
    pub created_by: Uuid,
    /// Identificador do autor da última alteração.
    pub updated_by: Uuid,
    /// Instante de criação no banco.
    pub created_at: DateTime<Utc>,
    /// Instante da última modificação no banco.
    pub updated_at: DateTime<Utc>,
}

impl FaqEntryRow {
    /// Converte a linha de banco de dados para a entidade de domínio pura, validando as invariantes.
    pub fn try_into_domain(self) -> Result<FaqEntry, FaqError> {
        FaqEntry::reconstitute(
            self.id,
            self.academic_center_id,
            self.question,
            self.answer,
            self.category,
            self.created_by,
            self.updated_by,
            self.created_at,
            self.updated_at,
        )
    }
}

/// Implementação PostgreSQL do repositório de FAQ.
#[derive(Debug, Clone)]
pub struct PostgresFaqRepository {
    pool: PgPool,
}

impl PostgresFaqRepository {
    /// Cria uma nova instância do repositório PostgreSQL com o pool compartilhado.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FaqRepository for PostgresFaqRepository {
    async fn save(&self, entry: &FaqEntry) -> Result<(), FaqRepositoryError> {
        sqlx::query(
            "INSERT INTO faq_entry (id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(entry.id())
        .bind(entry.academic_center_id())
        .bind(entry.question())
        .bind(entry.answer())
        .bind(entry.category())
        .bind(entry.created_by())
        .bind(entry.updated_by())
        .bind(entry.created_at())
        .bind(entry.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, "Falha ao salvar entrada de FAQ no PostgreSQL");
            FaqRepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FaqEntry>, FaqRepositoryError> {
        let row: Option<FaqEntryRow> = sqlx::query_as(
            "SELECT id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at \
             FROM faq_entry WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, "Falha ao buscar entrada de FAQ por id");
            FaqRepositoryError::Database(e.to_string())
        })?;

        match row {
            Some(r) => {
                let domain = r.try_into_domain().map_err(|e| {
                    tracing::error!(%e, "Invariante corrompida ao carregar FAQ do banco");
                    FaqRepositoryError::Database(e.to_string())
                })?;
                Ok(Some(domain))
            }
            None => Ok(None),
        }
    }

    async fn find_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<FaqEntry>, FaqRepositoryError> {
        let rows: Vec<FaqEntryRow> = sqlx::query_as(
            "SELECT id, academic_center_id, question, answer, category, created_by, updated_by, created_at, updated_at \
             FROM faq_entry WHERE academic_center_id = $1 ORDER BY created_at DESC"
        )
        .bind(academic_center_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, "Falha ao listar entradas de FAQ por Centro Acadêmico");
            FaqRepositoryError::Database(e.to_string())
        })?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let domain = row.try_into_domain().map_err(|e| {
                tracing::error!(%e, "Invariante corrompida ao carregar FAQ do banco");
                FaqRepositoryError::Database(e.to_string())
            })?;
            entries.push(domain);
        }

        Ok(entries)
    }

    async fn check_ca_management_access(
        &self,
        academic_center_id: Uuid,
        user_id: Uuid,
    ) -> Result<CaManagementAccess, FaqRepositoryError> {
        let ca_row = sqlx::query(
            "SELECT id, status, created_by_id \
             FROM academic_center WHERE id = $1",
        )
        .bind(academic_center_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, "Falha ao consultar Centro Acadêmico para validação de acesso");
            FaqRepositoryError::Database(e.to_string())
        })?;

        let Some(ca) = ca_row else {
            return Ok(CaManagementAccess::new(false, false, false));
        };

        let status: String = ca.get("status");
        let created_by_id: Option<Uuid> = ca.get("created_by_id");

        let ca_exists = true;
        let ca_active = status == "active";
        let is_ca_manager = created_by_id.is_some_and(|creator| creator == user_id);

        Ok(CaManagementAccess::new(ca_exists, ca_active, is_ca_manager))
    }
}

#[derive(Debug, Clone, Copy)]
struct InMemoryCaInfo {
    active: bool,
    creator_id: Option<Uuid>,
}

/// Implementação em memória de [`FaqRepository`] para testes rápidos e isolados.
#[derive(Debug, Clone, Default)]
pub struct InMemoryFaqRepository {
    entries: Arc<RwLock<HashMap<Uuid, FaqEntry>>>,
    cas: Arc<RwLock<HashMap<Uuid, InMemoryCaInfo>>>,
    managers: Arc<RwLock<HashSet<(Uuid, Uuid)>>>,
}

impl InMemoryFaqRepository {
    /// Cria uma nova instância de repositório em memória.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            cas: Arc::new(RwLock::new(HashMap::new())),
            managers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Registra um Centro Acadêmico em memória para validação de existência e status.
    pub fn add_ca(&self, id: Uuid, active: bool, creator_id: Option<Uuid>) {
        if let Ok(mut map) = self.cas.write() {
            map.insert(id, InMemoryCaInfo { active, creator_id });
        }
    }

    /// Concede permissão de gestão no Centro Acadêmico para um usuário.
    pub fn grant_manager(&self, ca_id: Uuid, user_id: Uuid) {
        if let Ok(mut set) = self.managers.write() {
            set.insert((ca_id, user_id));
        }
    }
}

#[async_trait::async_trait]
impl FaqRepository for InMemoryFaqRepository {
    async fn save(&self, entry: &FaqEntry) -> Result<(), FaqRepositoryError> {
        let mut map = self
            .entries
            .write()
            .map_err(|e| FaqRepositoryError::Database(e.to_string()))?;
        map.insert(entry.id(), entry.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FaqEntry>, FaqRepositoryError> {
        let map = self
            .entries
            .read()
            .map_err(|e| FaqRepositoryError::Database(e.to_string()))?;
        Ok(map.get(&id).cloned())
    }

    async fn find_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<FaqEntry>, FaqRepositoryError> {
        let map = self
            .entries
            .read()
            .map_err(|e| FaqRepositoryError::Database(e.to_string()))?;
        let mut list: Vec<FaqEntry> = map
            .values()
            .filter(|e| e.academic_center_id() == academic_center_id)
            .cloned()
            .collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.created_at()));
        Ok(list)
    }

    async fn check_ca_management_access(
        &self,
        academic_center_id: Uuid,
        user_id: Uuid,
    ) -> Result<CaManagementAccess, FaqRepositoryError> {
        let cas_map = self
            .cas
            .read()
            .map_err(|e| FaqRepositoryError::Database(e.to_string()))?;

        let Some(&InMemoryCaInfo { active, creator_id }) = cas_map.get(&academic_center_id) else {
            return Ok(CaManagementAccess::new(false, false, false));
        };

        let managers_set = self
            .managers
            .read()
            .map_err(|e| FaqRepositoryError::Database(e.to_string()))?;

        let is_ca_manager =
            creator_id == Some(user_id) || managers_set.contains(&(academic_center_id, user_id));

        Ok(CaManagementAccess::new(true, active, is_ca_manager))
    }
}
