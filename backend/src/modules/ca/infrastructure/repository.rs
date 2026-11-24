//! Adaptadores de persistência do módulo de Centros Acadêmicos.
//!
//! Contém implementações PostgreSQL e em memória para Centros Acadêmicos
//! e Mandatos de gestão, garantindo isolamento entre dados relacionais e domínio.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use uuid::Uuid;
use crate::infrastructure::database::PgDatabase;
use crate::modules::ca::application::ports::AcademicCenterRepository;
use crate::modules::ca::application::ports::AcademicCenterRepositoryError;
use crate::modules::ca::application::ports::MandateRepository;
use crate::modules::ca::application::ports::MandateRepositoryError;
use crate::modules::ca::domain::entities::AcademicCenter;
use crate::modules::ca::domain::entities::AcademicCenterStatus;
use crate::modules::ca::domain::entities::Mandate;
use crate::modules::ca::domain::entities::MandateMember;
use crate::modules::ca::domain::entities::MandateStatus;
use crate::modules::ca::application::ports::PollRepository;
use crate::modules::ca::application::ports::PollRepositoryError;
use crate::modules::ca::domain::entities::PollOption;
use crate::modules::ca::domain::entities::PollStatus;
use crate::modules::ca::domain::entities::PollVote;
use crate::modules::ca::domain::entities::StudentPoll;
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
    /// Instante UTC de encerramento do mandato, se houver.
    pub concluded_at: Option<DateTime<Utc>>,
    /// Instante UTC de registro da configuração do mandato.
    pub created_at: DateTime<Utc>,
    /// Instante UTC da última modificação do registro.
    pub updated_at: DateTime<Utc>,
}
impl MandateRow {
    /// Converte o registro SQL para a entidade de domínio [`Mandate`].
    pub fn into_entity(
        self,
        members: Vec<MandateMember>,
    ) -> Result<Mandate, MandateRepositoryError> {
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
            self.concluded_at,
            members,
            self.created_at,
            self.updated_at,
        )
        .map_err(|e| MandateRepositoryError::Internal(e.to_string()))
    }
}
/// Estrutura de mapeamento relacional (Row) da tabela `academic_center_mandate_member`.
#[derive(Debug, sqlx::FromRow)]
pub struct MandateMemberRow {
    /// Identificador único do vínculo do integrante.
    pub id: Uuid,
    /// Identificador do mandato correspondente.
    pub mandate_id: Uuid,
    /// Identificador opcional do usuário associado.
    pub user_id: Option<Uuid>,
    /// Nome do integrante.
    pub name: String,
    /// Cargo ocupado.
    pub role: String,
    /// Se o vínculo ativo permanece ativo.
    pub is_active: bool,
    /// Instante UTC de início da vinculação.
    pub joined_at: DateTime<Utc>,
    /// Instante UTC de desvinculação, se aplicável.
    pub left_at: Option<DateTime<Utc>>,
}
impl MandateMemberRow {
    /// Converte o registro SQL para a entidade de domínio [`MandateMember`].
    pub fn into_entity(self) -> Result<MandateMember, MandateRepositoryError> {
        MandateMember::reconstitute(
            self.id,
            self.user_id,
            self.name,
            self.role,
            self.is_active,
            self.joined_at,
            self.left_at,
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
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!(%e, "Falha ao iniciar transação para persistir mandato");
            MandateRepositoryError::Internal(e.to_string())
        })?;

        let result = sqlx::query(
            "INSERT INTO academic_center_mandate ( \
                 id, academic_center_id, created_by_id, title, start_date, end_date, status, concluded_at, created_at, updated_at \
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET \
                 title = EXCLUDED.title, \
                 start_date = EXCLUDED.start_date, \
                 end_date = EXCLUDED.end_date, \
                 status = EXCLUDED.status, \
                 concluded_at = EXCLUDED.concluded_at, \
                 updated_at = EXCLUDED.updated_at",
        )
        .bind(mandate.id())
        .bind(mandate.academic_center_id())
        .bind(mandate.created_by_id())
        .bind(mandate.title())
        .bind(mandate.start_date())
        .bind(mandate.end_date())
        .bind(mandate.status().as_str())
        .bind(mandate.concluded_at())
        .bind(mandate.created_at())
        .bind(mandate.updated_at())
        .execute(&mut *tx)
        .await;

        match result {
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                return Err(MandateRepositoryError::Conflict(
                    "já existe um mandato de gestão ativo para este Centro Acadêmico".into(),
                ));
            }
            Err(e) => {
                tracing::error!(%e, id = %mandate.id(), "Falha ao persistir mandato");
                return Err(MandateRepositoryError::Internal(e.to_string()));
            }
            Ok(_) => {}
        }

        for member in mandate.members() {
            sqlx::query(
                "INSERT INTO academic_center_mandate_member ( \
                     id, mandate_id, user_id, name, role, is_active, joined_at, left_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
                 ON CONFLICT (id) DO UPDATE SET \
                     user_id = EXCLUDED.user_id, \
                     name = EXCLUDED.name, \
                     role = EXCLUDED.role, \
                     is_active = EXCLUDED.is_active, \
                     joined_at = EXCLUDED.joined_at, \
                     left_at = EXCLUDED.left_at, \
                     updated_at = EXCLUDED.updated_at",
            )
            .bind(member.id())
            .bind(mandate.id())
            .bind(member.user_id())
            .bind(member.name())
            .bind(member.role())
            .bind(member.is_active())
            .bind(member.joined_at())
            .bind(member.left_at())
            .bind(member.joined_at())
            .bind(mandate.updated_at())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(%e, member_id = %member.id(), "Falha ao persistir integrante de mandato");
                MandateRepositoryError::Internal(e.to_string())
            })?;
        }

        tx.commit().await.map_err(|e| {
            tracing::error!(%e, "Falha ao comitar transação de mandato");
            MandateRepositoryError::Internal(e.to_string())
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, MandateRepositoryError> {
        let row = sqlx::query_as::<_, MandateRow>(
            "SELECT id, academic_center_id, created_by_id, title, start_date, end_date, status, concluded_at, created_at, updated_at \
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

        if let Some(row) = row {
            let member_rows = sqlx::query_as::<_, MandateMemberRow>(
                "SELECT id, mandate_id, user_id, name, role, is_active, joined_at, left_at, created_at, updated_at \
                 FROM academic_center_mandate_member \
                 WHERE mandate_id = $1 \
                 ORDER BY joined_at ASC",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(%e, %id, "Falha ao consultar integrantes de mandato");
                MandateRepositoryError::Internal(e.to_string())
            })?;

            let members = member_rows
                .into_iter()
                .map(MandateMemberRow::into_entity)
                .collect::<Result<Vec<_>, _>>()?;

            row.into_entity(members).map(Some)
        } else {
            Ok(None)
        }
    }

    async fn find_active_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Option<Mandate>, MandateRepositoryError> {
        let row = sqlx::query_as::<_, MandateRow>(
            "SELECT id, academic_center_id, created_by_id, title, start_date, end_date, status, concluded_at, created_at, updated_at \
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

        if let Some(row) = row {
            let member_rows = sqlx::query_as::<_, MandateMemberRow>(
                "SELECT id, mandate_id, user_id, name, role, is_active, joined_at, left_at, created_at, updated_at \
                 FROM academic_center_mandate_member \
                 WHERE mandate_id = $1 \
                 ORDER BY joined_at ASC",
            )
            .bind(row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(%e, mandate_id = %row.id, "Falha ao consultar integrantes de mandato ativo");
                MandateRepositoryError::Internal(e.to_string())
            })?;

            let members = member_rows
                .into_iter()
                .map(MandateMemberRow::into_entity)
                .collect::<Result<Vec<_>, _>>()?;

            row.into_entity(members).map(Some)
        } else {
            Ok(None)
        }
    }

    async fn find_all_by_academic_center_id(
        &self,
        academic_center_id: Uuid,
    ) -> Result<Vec<Mandate>, MandateRepositoryError> {
        let rows = sqlx::query_as::<_, MandateRow>(
            "SELECT id, academic_center_id, created_by_id, title, start_date, end_date, status, concluded_at, created_at, updated_at \
             FROM academic_center_mandate \
             WHERE academic_center_id = $1 \
             ORDER BY start_date DESC, created_at DESC",
        )
        .bind(academic_center_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(%e, %academic_center_id, "Falha ao listar mandatos do Centro Acadêmico");
            MandateRepositoryError::Internal(e.to_string())
        })?;

        let mut mandates = Vec::with_capacity(rows.len());
        for row in rows {
            let member_rows = sqlx::query_as::<_, MandateMemberRow>(
                "SELECT id, mandate_id, user_id, name, role, is_active, joined_at, left_at, created_at, updated_at \
                 FROM academic_center_mandate_member \
                 WHERE mandate_id = $1 \
                 ORDER BY joined_at ASC",
            )
            .bind(row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(%e, mandate_id = %row.id, "Falha ao listar integrantes do mandato");
                MandateRepositoryError::Internal(e.to_string())
            })?;

            let members = member_rows
                .into_iter()
                .map(MandateMemberRow::into_entity)
                .collect::<Result<Vec<_>, _>>()?;

            mandates.push(row.into_entity(members)?);
        }

        Ok(mandates)
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

        // Verifica unicidade de mandato ativo por Centro Acadêmico
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
        let mut list: Vec<Mandate> = self
            .mandates
            .read()
            .unwrap()
            .iter()
            .filter(|m| m.academic_center_id() == academic_center_id)
            .cloned()
            .collect();

        // Ordenação cronológica estrita: start_date DESC, created_at DESC
        list.sort_by(|a, b| {
            b.start_date()
                .cmp(&a.start_date())
                .then_with(|| b.created_at().cmp(&a.created_at()))
        });

        Ok(list)
    }
}
/// Estrutura de mapeamento relacional (Row) da tabela `student_poll`.
#[derive(Debug, sqlx::FromRow)]
pub struct StudentPollRow {
    /// Identificador único da consulta.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Identificador do gestor criador da consulta.
    pub created_by_id: Uuid,
    /// Título da consulta.
    pub title: String,
    /// Descrição ou proposta.
    pub description: String,
    /// Público-alvo / critério de elegibilidade.
    pub target_audience: String,
    /// Modalidade de votação ('identified' ou 'secret').
    pub vote_modality: String,
    /// Início do período de votação.
    pub starts_at: DateTime<Utc>,
    /// Término do período de votação (opcional).
    pub ends_at: Option<DateTime<Utc>>,
    /// Status ('open' ou 'closed').
    pub status: String,
    /// Instante de criação.
    pub created_at: DateTime<Utc>,
    /// Instante de atualização.
    pub updated_at: DateTime<Utc>,
    /// Instante de encerramento manual (se houver).
    pub closed_at: Option<DateTime<Utc>>,
    /// Gestor que encerrou a consulta (se houver).
    pub closed_by_id: Option<Uuid>,
}
/// Estrutura de mapeamento relacional (Row) da tabela `student_poll_option`.
#[derive(Debug, sqlx::FromRow)]
pub struct PollOptionRow {
    /// Identificador da opção.
    pub id: Uuid,
    /// Identificador da consulta.
    pub poll_id: Uuid,
    /// Título da opção.
    pub title: String,
    /// Ordem de apresentação.
    pub option_order: i32,
    /// Instante de criação.
    pub created_at: DateTime<Utc>,
}
impl PollOptionRow {
    /// Converte a linha SQL na entidade [`PollOption`].
    pub fn into_entity(self) -> Result<PollOption, PollRepositoryError> {
        PollOption::reconstitute(
            self.id,
            self.poll_id,
            self.title,
            self.option_order,
            self.created_at,
        )
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))
    }
}
/// Estrutura de mapeamento relacional (Row) da tabela `student_poll_vote`.
#[derive(Debug, sqlx::FromRow)]
pub struct PollVoteRow {
    /// Identificador do voto.
    pub id: Uuid,
    /// Identificador da consulta.
    pub poll_id: Uuid,
    /// Identificador da opção votada.
    pub option_id: Uuid,
    /// Identificador do estudante votante.
    pub user_id: Uuid,
    /// Instante do voto.
    pub voted_at: DateTime<Utc>,
}
impl PollVoteRow {
    /// Converte a linha SQL na entidade [`PollVote`].
    pub fn into_entity(self) -> PollVote {
        PollVote::reconstitute(
            self.id,
            self.poll_id,
            self.option_id,
            self.user_id,
            self.voted_at,
        )
    }
}
#[async_trait]
impl PollRepository for PgDatabase {
    async fn save(&self, poll: &StudentPoll) -> Result<(), PollRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO student_poll (
                id, academic_center_id, created_by_id, title, description,
                target_audience, vote_modality, starts_at, ends_at, status,
                created_at, updated_at, closed_at, closed_by_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(poll.id())
        .bind(poll.academic_center_id())
        .bind(poll.created_by_id())
        .bind(poll.title())
        .bind(poll.description().unwrap_or(""))
        .bind(poll.target_audience())
        .bind(poll.vote_modality())
        .bind(poll.starts_at())
        .bind(poll.ends_at())
        .bind(poll.status().to_string())
        .bind(poll.created_at())
        .bind(poll.updated_at())
        .bind(poll.closed_at())
        .bind(poll.closed_by_id())
        .execute(&mut *tx)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        for opt in poll.options() {
            sqlx::query(
                "INSERT INTO student_poll_option (id, poll_id, title, option_order, created_at) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(opt.id())
            .bind(opt.poll_id())
            .bind(opt.title())
            .bind(opt.order())
            .bind(opt.created_at())
            .execute(&mut *tx)
            .await
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<StudentPoll>, PollRepositoryError> {
        let poll_row: Option<StudentPollRow> = sqlx::query_as(
            "SELECT id, academic_center_id, created_by_id, title, description,
                    target_audience, vote_modality, starts_at, ends_at, status,
                    created_at, updated_at, closed_at, closed_by_id
             FROM student_poll WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        let poll_row = match poll_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let option_rows: Vec<PollOptionRow> = sqlx::query_as(
            "SELECT id, poll_id, title, option_order, created_at \
             FROM student_poll_option WHERE poll_id = $1 ORDER BY option_order ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        let mut options = Vec::with_capacity(option_rows.len());
        for row in option_rows {
            options.push(row.into_entity()?);
        }

        let status = PollStatus::from_str(&poll_row.status)
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        let description = if poll_row.description.is_empty() {
            None
        } else {
            Some(poll_row.description)
        };

        let entity = StudentPoll::reconstitute(
            poll_row.id,
            poll_row.academic_center_id,
            poll_row.created_by_id,
            poll_row.title,
            description,
            poll_row.target_audience,
            poll_row.vote_modality,
            poll_row.starts_at,
            poll_row.ends_at,
            status,
            poll_row.created_at,
            poll_row.updated_at,
            poll_row.closed_at,
            poll_row.closed_by_id,
            options,
        )
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        Ok(Some(entity))
    }

    async fn find_all_by_ca_id(
        &self,
        ca_id: Uuid,
    ) -> Result<Vec<StudentPoll>, PollRepositoryError> {
        let poll_rows: Vec<StudentPollRow> = sqlx::query_as(
            "SELECT id, academic_center_id, created_by_id, title, description,
                    target_audience, vote_modality, starts_at, ends_at, status,
                    created_at, updated_at, closed_at, closed_by_id
             FROM student_poll WHERE academic_center_id = $1 ORDER BY created_at DESC",
        )
        .bind(ca_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        let mut polls = Vec::with_capacity(poll_rows.len());

        for poll_row in poll_rows {
            let option_rows: Vec<PollOptionRow> = sqlx::query_as(
                "SELECT id, poll_id, title, option_order, created_at \
                 FROM student_poll_option WHERE poll_id = $1 ORDER BY option_order ASC",
            )
            .bind(poll_row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

            let mut options = Vec::with_capacity(option_rows.len());
            for row in option_rows {
                options.push(row.into_entity()?);
            }

            let status = PollStatus::from_str(&poll_row.status)
                .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

            let description = if poll_row.description.is_empty() {
                None
            } else {
                Some(poll_row.description)
            };

            let entity = StudentPoll::reconstitute(
                poll_row.id,
                poll_row.academic_center_id,
                poll_row.created_by_id,
                poll_row.title,
                description,
                poll_row.target_audience,
                poll_row.vote_modality,
                poll_row.starts_at,
                poll_row.ends_at,
                status,
                poll_row.created_at,
                poll_row.updated_at,
                poll_row.closed_at,
                poll_row.closed_by_id,
                options,
            )
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

            polls.push(entity);
        }

        Ok(polls)
    }

    async fn save_vote(&self, vote: &PollVote) -> Result<(), PollRepositoryError> {
        let result = sqlx::query(
            "INSERT INTO student_poll_vote (id, poll_id, option_id, user_id, voted_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(vote.id())
        .bind(vote.poll_id())
        .bind(vote.option_id())
        .bind(vote.user_id())
        .bind(vote.voted_at())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(PollRepositoryError::Conflict("duplicate vote".to_string()))
            }
            Err(e) => Err(PollRepositoryError::Internal(e.to_string())),
        }
    }

    async fn has_user_voted(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, PollRepositoryError> {
        let exists: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM student_poll_vote WHERE poll_id = $1 AND user_id = $2)",
        )
        .bind(poll_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        Ok(exists.unwrap_or(false))
    }

    async fn count_votes_by_option(
        &self,
        poll_id: Uuid,
    ) -> Result<HashMap<Uuid, i64>, PollRepositoryError> {
        let rows: Vec<(Uuid, i64)> = sqlx::query_as(
            "SELECT option_id, COUNT(*)::bigint AS count \
             FROM student_poll_vote WHERE poll_id = $1 GROUP BY option_id",
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        let mut counts = HashMap::new();
        for (option_id, count) in rows {
            counts.insert(option_id, count);
        }

        Ok(counts)
    }

    async fn find_votes_by_poll_id(
        &self,
        poll_id: Uuid,
    ) -> Result<Vec<PollVote>, PollRepositoryError> {
        let rows: Vec<PollVoteRow> = sqlx::query_as(
            "SELECT id, poll_id, option_id, user_id, voted_at \
             FROM student_poll_vote WHERE poll_id = $1 ORDER BY voted_at ASC",
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_entity()).collect())
    }

    async fn close_poll(
        &self,
        poll_id: Uuid,
        closed_by_id: Uuid,
        closed_at: DateTime<Utc>,
    ) -> Result<StudentPoll, PollRepositoryError> {
        sqlx::query(
            "UPDATE student_poll SET status = 'closed', closed_at = GREATEST(created_at, $2), closed_by_id = $3, updated_at = GREATEST(created_at, $2) \
             WHERE id = $1",
        )
        .bind(poll_id)
        .bind(closed_at)
        .bind(closed_by_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        PollRepository::find_by_id(self, poll_id)
            .await?
            .ok_or_else(|| PollRepositoryError::NotFound(poll_id.to_string()))
    }
}
/// Adaptador em memória para Consultas e Enquetes Estudantis.
#[derive(Debug, Default, Clone)]
pub struct InMemoryPollRepository {
    polls: Arc<RwLock<HashMap<Uuid, StudentPoll>>>,
    votes: Arc<RwLock<Vec<PollVote>>>,
}
impl InMemoryPollRepository {
    /// Cria um novo repositório de enquetes em memória.
    pub fn new() -> Self {
        Self::default()
    }
}
#[async_trait]
impl PollRepository for InMemoryPollRepository {
    async fn save(&self, poll: &StudentPoll) -> Result<(), PollRepositoryError> {
        let mut polls = self.polls.write().unwrap();
        polls.insert(poll.id(), poll.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<StudentPoll>, PollRepositoryError> {
        let polls = self.polls.read().unwrap();
        Ok(polls.get(&id).cloned())
    }

    async fn find_all_by_ca_id(
        &self,
        ca_id: Uuid,
    ) -> Result<Vec<StudentPoll>, PollRepositoryError> {
        let polls = self.polls.read().unwrap();
        let mut list: Vec<StudentPoll> = polls
            .values()
            .filter(|p| p.academic_center_id() == ca_id)
            .cloned()
            .collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.created_at()));
        Ok(list)
    }

    async fn save_vote(&self, vote: &PollVote) -> Result<(), PollRepositoryError> {
        let mut votes = self.votes.write().unwrap();
        if votes
            .iter()
            .any(|v| v.poll_id() == vote.poll_id() && v.user_id() == vote.user_id())
        {
            return Err(PollRepositoryError::Conflict("duplicate vote".to_string()));
        }
        votes.push(vote.clone());
        Ok(())
    }

    async fn has_user_voted(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, PollRepositoryError> {
        let votes = self.votes.read().unwrap();
        Ok(votes
            .iter()
            .any(|v| v.poll_id() == poll_id && v.user_id() == user_id))
    }

    async fn count_votes_by_option(
        &self,
        poll_id: Uuid,
    ) -> Result<HashMap<Uuid, i64>, PollRepositoryError> {
        let votes = self.votes.read().unwrap();
        let mut map = HashMap::new();
        for vote in votes.iter().filter(|v| v.poll_id() == poll_id) {
            *map.entry(vote.option_id()).or_insert(0) += 1;
        }
        Ok(map)
    }

    async fn find_votes_by_poll_id(
        &self,
        poll_id: Uuid,
    ) -> Result<Vec<PollVote>, PollRepositoryError> {
        let votes = self.votes.read().unwrap();
        Ok(votes
            .iter()
            .filter(|v| v.poll_id() == poll_id)
            .cloned()
            .collect())
    }

    async fn close_poll(
        &self,
        poll_id: Uuid,
        closed_by_id: Uuid,
        closed_at: DateTime<Utc>,
    ) -> Result<StudentPoll, PollRepositoryError> {
        let mut polls = self.polls.write().unwrap();
        let poll = polls
            .get_mut(&poll_id)
            .ok_or_else(|| PollRepositoryError::NotFound(poll_id.to_string()))?;

        poll.close(closed_by_id, closed_at)
            .map_err(|e| PollRepositoryError::Internal(e.to_string()))?;

        Ok(poll.clone())
    }
}
