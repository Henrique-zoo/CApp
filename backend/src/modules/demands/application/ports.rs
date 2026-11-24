//! Portas de persistência e integração do módulo de demandas.
//!
//! Define os contratos abstratos utilizados pelos casos de uso para persistir
//! demandas e consultar vínculos acadêmicos e representatividade do CA.

use uuid::Uuid;

use crate::modules::demands::{
    application::error::{AffiliationError, DemandRepositoryError},
    domain::entities::{Demand, DemandComment, DemandForum, DemandStatus},
};

/// Critérios de filtragem para listagem de demandas no repositório.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandFilter {
    /// Centro Acadêmico obrigatório (garante isolamento estrito entre CAs).
    pub academic_center_id: Uuid,
    /// Fórum específico opcional (ex: apenas `physical_space` ou `institutional`).
    pub forum: Option<DemandForum>,
    /// Status específico opcional (ex: apenas `open`).
    pub status: Option<DemandStatus>,
    /// Autor específico opcional (utilizado para filtragem de demandas próprias).
    pub author_id: Option<Uuid>,
    /// Limite máximo de registros retornados.
    pub limit: Option<usize>,
    /// Deslocamento inicial de registros.
    pub offset: Option<usize>,
}

/// Contrato de persistência para entidades e histórico de demandas.
#[async_trait::async_trait]
pub trait DemandRepository: Send + Sync {
    /// Cria e persiste uma nova demanda e seu registro inicial no histórico de status de forma transacional.
    async fn create(&self, demand: &Demand) -> Result<(), DemandRepositoryError>;

    /// Localiza uma demanda por seu identificador único estável.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Demand>, DemandRepositoryError>;

    /// Lista e filtra demandas persistidas de acordo com os critérios fornecidos.
    async fn list(&self, filter: &DemandFilter) -> Result<Vec<Demand>, DemandRepositoryError>;

    /// Atualiza o status, data de modificação e histórico de uma demanda existente.
    async fn update(&self, demand: &Demand) -> Result<(), DemandRepositoryError>;
}

#[async_trait::async_trait]
impl<T: DemandRepository + ?Sized> DemandRepository for &T {
    async fn create(&self, demand: &Demand) -> Result<(), DemandRepositoryError> {
        (**self).create(demand).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Demand>, DemandRepositoryError> {
        (**self).find_by_id(id).await
    }

    async fn list(&self, filter: &DemandFilter) -> Result<Vec<Demand>, DemandRepositoryError> {
        (**self).list(filter).await
    }

    async fn update(&self, demand: &Demand) -> Result<(), DemandRepositoryError> {
        (**self).update(demand).await
    }
}

/// Contrato de persistência para comentários e interações em demandas.
#[async_trait::async_trait]
pub trait DemandCommentRepository: Send + Sync {
    /// Cria e persiste um novo comentário em uma demanda.
    async fn create(&self, comment: &DemandComment) -> Result<(), DemandRepositoryError>;

    /// Lista todos os comentários de uma demanda em ordem cronológica ascendente (`created_at ASC, id ASC`).
    async fn list_by_demand_id(
        &self,
        demand_id: Uuid,
    ) -> Result<Vec<DemandComment>, DemandRepositoryError>;
}

#[async_trait::async_trait]
impl<T: DemandCommentRepository + ?Sized> DemandCommentRepository for &T {
    async fn create(&self, comment: &DemandComment) -> Result<(), DemandRepositoryError> {
        (**self).create(comment).await
    }

    async fn list_by_demand_id(
        &self,
        demand_id: Uuid,
    ) -> Result<Vec<DemandComment>, DemandRepositoryError> {
        (**self).list_by_demand_id(demand_id).await
    }
}

/// Representa a fotografia de vínculo do estudante e do Centro Acadêmico consultada pelos casos de uso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicAffiliation {
    /// Indica se o Centro Acadêmico existe no cadastro.
    pub academic_center_exists: bool,

    /// Indica se o Centro Acadêmico está com status ativo.
    pub academic_center_active: bool,

    /// Indica se o autor possui vínculo estudantil ativo na instituição de ensino.
    pub has_active_student_membership: bool,

    /// Curso do estudante informado institucionalmente, se houver.
    pub student_course: Option<String>,

    /// Cursos oficialmente representados pelo Centro Acadêmico.
    pub ca_represented_courses: Vec<String>,

    /// Indica se o autor integra a diretoria/gestão do Centro Acadêmico em questão.
    pub is_ca_manager: bool,
}

impl AcademicAffiliation {
    /// Cria uma nova estrutura de vínculo acadêmico para o contexto do CA.
    pub fn new(
        academic_center_exists: bool,
        academic_center_active: bool,
        has_active_student_membership: bool,
        student_course: Option<String>,
        ca_represented_courses: Vec<String>,
        is_ca_manager: bool,
    ) -> Self {
        Self {
            academic_center_exists,
            academic_center_active,
            has_active_student_membership,
            student_course,
            ca_represented_courses,
            is_ca_manager,
        }
    }

    /// Verifica se o curso informado é representado pelo Centro Acadêmico.
    ///
    /// A comparação normaliza espaços e não diferencia maiúsculas e minúsculas.
    pub fn ca_represents_course(&self, course_name: &str) -> bool {
        let target = course_name.trim();
        if target.is_empty() {
            return false;
        }

        // Correspondência direta ignorando caixa
        if self
            .ca_represented_courses
            .iter()
            .any(|c| c.trim().eq_ignore_ascii_case(target))
        {
            return true;
        }

        // Correspondência por continuação semântica
        let target_lower = target.to_lowercase();
        for rep in &self.ca_represented_courses {
            let rep_lower = rep.trim().to_lowercase();
            if rep_lower.contains(&target_lower) || target_lower.contains(&rep_lower) {
                return true;
            }
        }

        false
    }
}

/// Contrato para verificação de vínculo estudantil e representação do curso pelo CA.
#[async_trait::async_trait]
pub trait AcademicAffiliationPort: Send + Sync {
    /// Consulta os dados de vínculo e autorização do autor em relação ao Centro Acadêmico alvo.
    async fn get_affiliation(
        &self,
        author_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<AcademicAffiliation, AffiliationError>;
}

#[async_trait::async_trait]
impl<T: AcademicAffiliationPort + ?Sized> AcademicAffiliationPort for &T {
    async fn get_affiliation(
        &self,
        author_id: Uuid,
        academic_center_id: Uuid,
    ) -> Result<AcademicAffiliation, AffiliationError> {
        (**self)
            .get_affiliation(author_id, academic_center_id)
            .await
    }
}
