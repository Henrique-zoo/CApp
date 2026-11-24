//! Entidades de domínio do módulo de demandas.
//!
//! Encapsula o modelo de demanda acadêmica, histórico de alterações de status,
//! fóruns de atuação (Institucional e Espaço Físico do CA) e suas invariantes.

use std::{error::Error, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::demands::domain::rules::{
    validate_description, validate_optional_category, validate_optional_comment,
    validate_optional_course_name, validate_optional_title, validate_status_transition,
    validate_timestamps,
};

/// Erros decorrentes de violações de regras de negócio ou validação de demandas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemandError {
    /// A descrição ou conteúdo da demanda não pode ser vazio.
    EmptyDescription,

    /// A descrição da demanda ultrapassou o comprimento máximo permitido.
    DescriptionTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O título da demanda não pode ser vazio quando informado.
    EmptyTitle,

    /// O título da demanda ultrapassou o comprimento máximo permitido.
    TitleTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// A categoria da demanda não pode ser vazia quando informada.
    EmptyCategory,

    /// A categoria da demanda ultrapassou o comprimento máximo permitido.
    CategoryTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O nome do curso não pode ser vazio quando informado.
    EmptyCourseName,

    /// O nome do curso ultrapassou o comprimento máximo permitido.
    CourseNameTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O comentário de alteração de status não pode ser vazio quando informado.
    EmptyComment,

    /// O comentário de alteração de status ultrapassou o comprimento máximo permitido.
    CommentTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O fórum informado não é reconhecido pelo domínio.
    InvalidForum(String),

    /// O status informado não é reconhecido pelo domínio.
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada é inválida para o fórum da demanda.
    InvalidStatusTransition {
        /// Status de origem.
        from: DemandStatus,
        /// Status de destino pretendido.
        to: DemandStatus,
        /// Fórum da demanda.
        forum: DemandForum,
    },

    /// O encaminhamento à coordenação é exclusivo do fórum institucional e proibido no espaço físico.
    CoordinationForwardingNotAllowed,

    /// Demandas de espaço físico não podem ser vinculadas a um curso específico.
    CourseNotAllowedForPhysicalSpace,

    /// Os timestamps de criação e atualização violam a coerência temporal.
    InconsistentTimestamps {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de atualização.
        updated_at: DateTime<Utc>,
    },

    /// Entrada do histórico associada a um identificador de demanda divergente.
    HistoryDemandIdMismatch {
        /// Identificador esperado da demanda.
        expected: Uuid,
        /// Identificador encontrado na entrada do histórico.
        actual: Uuid,
    },
}

impl fmt::Display for DemandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDescription => {
                write!(f, "a descrição da demanda não pode ser vazia")
            }
            Self::DescriptionTooLong { max } => {
                write!(
                    f,
                    "a descrição da demanda excede o limite de {max} caracteres"
                )
            }
            Self::EmptyTitle => {
                write!(f, "o título da demanda não pode ser vazio")
            }
            Self::TitleTooLong { max } => {
                write!(f, "o título da demanda excede o limite de {max} caracteres")
            }
            Self::EmptyCategory => {
                write!(f, "a categoria da demanda não pode ser vazia")
            }
            Self::CategoryTooLong { max } => {
                write!(
                    f,
                    "a categoria da demanda excede o limite de {max} caracteres"
                )
            }
            Self::EmptyCourseName => {
                write!(f, "o nome do curso não pode ser vazio")
            }
            Self::CourseNameTooLong { max } => {
                write!(f, "o nome do curso excede o limite de {max} caracteres")
            }
            Self::EmptyComment => {
                write!(f, "o comentário de alteração de status não pode ser vazio")
            }
            Self::CommentTooLong { max } => {
                write!(
                    f,
                    "o comentário de alteração de status excede o limite de {max} caracteres"
                )
            }
            Self::InvalidForum(forum) => {
                write!(f, "fórum de demanda inválido ou desconhecido: '{forum}'")
            }
            Self::InvalidStatus(status) => {
                write!(f, "status de demanda inválido ou desconhecido: '{status}'")
            }
            Self::InvalidStatusTransition { from, to, forum } => {
                write!(
                    f,
                    "transição de status inválida de '{from}' para '{to}' no fórum '{forum}'"
                )
            }
            Self::CoordinationForwardingNotAllowed => {
                write!(
                    f,
                    "demandas do espaço físico não podem ser encaminhadas à coordenação do curso"
                )
            }
            Self::CourseNotAllowedForPhysicalSpace => {
                write!(
                    f,
                    "demandas sobre o espaço físico do CA não admitem vínculo com curso específico"
                )
            }
            Self::InconsistentTimestamps {
                created_at,
                updated_at,
            } => {
                write!(
                    f,
                    "data de atualização ({updated_at}) não pode ser anterior à data de criação ({created_at})"
                )
            }
            Self::HistoryDemandIdMismatch { expected, actual } => {
                write!(
                    f,
                    "registro de histórico pertence à demanda '{actual}', divergindo da demanda '{expected}'"
                )
            }
        }
    }
}

impl Error for DemandError {}

/// Fórum de atuação e escopo de visibilidade de uma demanda acadêmica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemandForum {
    /// Demanda acadêmica/institucional do curso, restrita aos discentes do curso e à gestão do CA.
    Institutional,
    /// Demanda sobre a infraestrutura e instalações do espaço físico do CA, aberta a todos os usuários.
    PhysicalSpace,
}

impl DemandForum {
    /// Retorna a representação textual canônica do fórum.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Institutional => "institutional",
            Self::PhysicalSpace => "physical_space",
        }
    }
}

impl fmt::Display for DemandForum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DemandForum {
    type Err = DemandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "institutional" | "institucional" => Ok(Self::Institutional),
            "physical_space" | "physical-space" | "espaco_fisico" | "espaço físico"
            | "espaco-fisico" | "espacofisico" => Ok(Self::PhysicalSpace),
            other => Err(DemandError::InvalidForum(other.to_string())),
        }
    }
}

/// Estado de andamento no ciclo de vida de uma demanda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemandStatus {
    /// Demanda recém-criada aguardando triagem ou atendimento inicial.
    Open,
    /// Demanda em análise pela gestão do Centro Acadêmico.
    InAnalysis,
    /// Demanda encaminhada à coordenação do curso (exclusivo do fórum institucional).
    ForwardedToCoordination,
    /// Demanda concluída ou com solução finalizada.
    Completed,
}

impl DemandStatus {
    /// Retorna a representação textual canônica do status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InAnalysis => "in_analysis",
            Self::ForwardedToCoordination => "forwarded_to_coordination",
            Self::Completed => "completed",
        }
    }
}

impl fmt::Display for DemandStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DemandStatus {
    type Err = DemandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "open" | "aberta" | "aberto" => Ok(Self::Open),
            "in_analysis" | "in-analysis" | "em_analise" | "em análise" | "em analise" => {
                Ok(Self::InAnalysis)
            }
            "forwarded_to_coordination"
            | "forwarded-to-coordination"
            | "enviada_a_coordenacao"
            | "enviada à coordenação"
            | "enviada a coordenacao" => Ok(Self::ForwardedToCoordination),
            "completed" | "concluida" | "concluída" | "resolvida" => Ok(Self::Completed),
            other => Err(DemandError::InvalidStatus(other.to_string())),
        }
    }
}

/// Registro imutável de uma alteração de status no histórico da demanda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandStatusHistoryEntry {
    /// Identificador do registro de histórico.
    id: Uuid,
    /// Identificador da demanda a que este histórico se refere.
    demand_id: Uuid,
    /// Status anterior à alteração, se houver.
    from_status: Option<DemandStatus>,
    /// Novo status assumido pela demanda.
    to_status: DemandStatus,
    /// Identificador do usuário que realizou a alteração.
    changed_by_id: Uuid,
    /// Justificativa ou comentário explicativo da transição, se informado.
    comment: Option<String>,
    /// Instante da alteração no tempo.
    created_at: DateTime<Utc>,
}

impl DemandStatusHistoryEntry {
    /// Constrói uma nova entrada de histórico com validação de invariantes.
    pub fn new(
        id: Uuid,
        demand_id: Uuid,
        from_status: Option<DemandStatus>,
        to_status: DemandStatus,
        changed_by_id: Uuid,
        comment: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DemandError> {
        let trimmed_comment = match comment {
            Some(ref c) => {
                let t = c.trim();
                validate_optional_comment(Some(t))?;
                Some(t.to_string())
            }
            None => None,
        };

        Ok(Self {
            id,
            demand_id,
            from_status,
            to_status,
            changed_by_id,
            comment: trimmed_comment,
            created_at,
        })
    }

    /// Identificador do registro de histórico.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador da demanda associada.
    pub fn demand_id(&self) -> Uuid {
        self.demand_id
    }

    /// Status anterior à alteração.
    pub fn from_status(&self) -> Option<DemandStatus> {
        self.from_status
    }

    /// Status para o qual a demanda transitou.
    pub fn to_status(&self) -> DemandStatus {
        self.to_status
    }

    /// Usuário responsável pela alteração.
    pub fn changed_by_id(&self) -> Uuid {
        self.changed_by_id
    }

    /// Comentário explicativo opcional.
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    /// Data e hora da alteração.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// Entidade de domínio representando uma demanda acadêmica ou sobre o espaço físico.
///
/// Todos os campos são privados e o acesso se dá via métodos observáveis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Demand {
    /// Identificador estável da demanda.
    id: Uuid,
    /// Centro Acadêmico responsável pelo tratamento da solicitação.
    academic_center_id: Uuid,
    /// Estudante solicitante que registrou a demanda.
    author_id: Uuid,
    /// Fórum ao qual a demanda pertence (Institucional ou Espaço Físico).
    forum: DemandForum,
    /// Título ou resumo identificador da demanda (opcional).
    title: Option<String>,
    /// Descrição detalhada do problema ou necessidade comunicada.
    description: String,
    /// Categoria temática para classificação e triagem (opcional).
    category: Option<String>,
    /// Curso associado, aplicável exclusivamente quando o fórum for institucional.
    course_name: Option<String>,
    /// Status atual do ciclo de vida da demanda.
    status: DemandStatus,
    /// Instante de registro da demanda.
    created_at: DateTime<Utc>,
    /// Instante da última modificação de estado.
    updated_at: DateTime<Utc>,
    /// Histórico ordenado de alterações de status.
    status_history: Vec<DemandStatusHistoryEntry>,
}

impl Demand {
    /// Cria uma nova demanda com status inicial [`DemandStatus::Open`].
    ///
    /// Valida as regras estruturais e o alinhamento com o fórum de destino.
    ///
    /// # Erros
    ///
    /// Retorna [`DemandError`] caso a descrição seja vazia, o curso seja informado
    /// para o fórum de espaço físico, ou limites de tamanho sejam violados.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        academic_center_id: Uuid,
        author_id: Uuid,
        forum: DemandForum,
        title: Option<String>,
        description: String,
        category: Option<String>,
        course_name: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DemandError> {
        let trimmed_desc = description.trim();
        validate_description(trimmed_desc)?;

        let trimmed_title = match title {
            Some(ref t) => {
                let s = t.trim();
                validate_optional_title(Some(s))?;
                Some(s.to_string())
            }
            None => None,
        };

        let trimmed_cat = match category {
            Some(ref c) => {
                let s = c.trim();
                validate_optional_category(Some(s))?;
                Some(s.to_string())
            }
            None => None,
        };

        let trimmed_course = match course_name {
            Some(ref c) => {
                let s = c.trim();
                validate_optional_course_name(Some(s), forum)?;
                Some(s.to_string())
            }
            None => None,
        };

        Ok(Self {
            id,
            academic_center_id,
            author_id,
            forum,
            title: trimmed_title,
            description: trimmed_desc.to_string(),
            category: trimmed_cat,
            course_name: trimmed_course,
            status: DemandStatus::Open,
            created_at,
            updated_at: created_at,
            status_history: Vec::new(),
        })
    }

    /// Reconstitui uma demanda a partir dos dados persistidos.
    ///
    /// Valida coerência temporal, compatibilidade do fórum com status e curso,
    /// e integridade do histórico associado.
    ///
    /// # Erros
    ///
    /// Retorna [`DemandError`] caso qualquer invariante de domínio seja violada.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        academic_center_id: Uuid,
        author_id: Uuid,
        forum: DemandForum,
        title: Option<String>,
        description: String,
        category: Option<String>,
        course_name: Option<String>,
        status: DemandStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        status_history: Vec<DemandStatusHistoryEntry>,
    ) -> Result<Self, DemandError> {
        validate_timestamps(created_at, updated_at)?;

        let trimmed_desc = description.trim();
        validate_description(trimmed_desc)?;

        let trimmed_title = match title {
            Some(ref t) => {
                let s = t.trim();
                validate_optional_title(Some(s))?;
                Some(s.to_string())
            }
            None => None,
        };

        let trimmed_cat = match category {
            Some(ref c) => {
                let s = c.trim();
                validate_optional_category(Some(s))?;
                Some(s.to_string())
            }
            None => None,
        };

        let trimmed_course = match course_name {
            Some(ref c) => {
                let s = c.trim();
                validate_optional_course_name(Some(s), forum)?;
                Some(s.to_string())
            }
            None => None,
        };

        if forum == DemandForum::PhysicalSpace && status == DemandStatus::ForwardedToCoordination {
            return Err(DemandError::CoordinationForwardingNotAllowed);
        }

        for entry in &status_history {
            if entry.demand_id != id {
                return Err(DemandError::HistoryDemandIdMismatch {
                    expected: id,
                    actual: entry.demand_id,
                });
            }
        }

        Ok(Self {
            id,
            academic_center_id,
            author_id,
            forum,
            title: trimmed_title,
            description: trimmed_desc.to_string(),
            category: trimmed_cat,
            course_name: trimmed_course,
            status,
            created_at,
            updated_at,
            status_history,
        })
    }

    /// Executa uma transição de status validando as regras do fórum e registrando o histórico.
    ///
    /// # Erros
    ///
    /// Retorna [`DemandError`] se a transição for inválida, se `at` for anterior a `updated_at`,
    /// ou se o comentário informado for inválido.
    pub fn transition_to(
        &mut self,
        new_status: DemandStatus,
        changed_by_id: Uuid,
        comment: Option<String>,
        at: DateTime<Utc>,
    ) -> Result<(), DemandError> {
        if at < self.updated_at {
            return Err(DemandError::InconsistentTimestamps {
                created_at: self.created_at,
                updated_at: at,
            });
        }

        validate_status_transition(self.forum, self.status, new_status)?;

        let history_entry = DemandStatusHistoryEntry::new(
            Uuid::new_v4(),
            self.id,
            Some(self.status),
            new_status,
            changed_by_id,
            comment,
            at,
        )?;

        self.status = new_status;
        self.updated_at = at;
        self.status_history.push(history_entry);

        Ok(())
    }

    /// Transição de conveniência: inicia a análise da demanda pela gestão.
    pub fn start_analysis(
        &mut self,
        changed_by_id: Uuid,
        comment: Option<String>,
        at: DateTime<Utc>,
    ) -> Result<(), DemandError> {
        self.transition_to(DemandStatus::InAnalysis, changed_by_id, comment, at)
    }

    /// Transição de conveniência: encaminha a demanda à coordenação do curso (fórum institucional).
    pub fn forward_to_coordination(
        &mut self,
        changed_by_id: Uuid,
        comment: Option<String>,
        at: DateTime<Utc>,
    ) -> Result<(), DemandError> {
        self.transition_to(
            DemandStatus::ForwardedToCoordination,
            changed_by_id,
            comment,
            at,
        )
    }

    /// Transição de conveniência: conclui a demanda.
    pub fn complete(
        &mut self,
        changed_by_id: Uuid,
        comment: Option<String>,
        at: DateTime<Utc>,
    ) -> Result<(), DemandError> {
        self.transition_to(DemandStatus::Completed, changed_by_id, comment, at)
    }

    /// Transição de conveniência: reabre a demanda.
    pub fn reopen(
        &mut self,
        changed_by_id: Uuid,
        comment: Option<String>,
        at: DateTime<Utc>,
    ) -> Result<(), DemandError> {
        self.transition_to(DemandStatus::Open, changed_by_id, comment, at)
    }

    /// Identificador estável da demanda.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Identificador do Centro Acadêmico responsável.
    pub fn academic_center_id(&self) -> Uuid {
        self.academic_center_id
    }

    /// Identificador do estudante solicitante/autor.
    pub fn author_id(&self) -> Uuid {
        self.author_id
    }

    /// Fórum ao qual a demanda pertence.
    pub fn forum(&self) -> DemandForum {
        self.forum
    }

    /// Título ou resumo identificador da demanda, se houver.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Descrição detalhada do problema ou necessidade comunicada.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Conteúdo registrado da demanda (alias semântico para a descrição).
    pub fn content(&self) -> &str {
        self.description()
    }

    /// Categoria temática opcional.
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    /// Nome do curso associado (presente apenas em demandas institucionais).
    pub fn course_name(&self) -> Option<&str> {
        self.course_name.as_deref()
    }

    /// Status atual da demanda.
    pub fn status(&self) -> DemandStatus {
        self.status
    }

    /// Data e hora de criação da demanda.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Data e hora da última modificação.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Histórico de alterações de status.
    pub fn status_history(&self) -> &[DemandStatusHistoryEntry] {
        &self.status_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ca_id() -> Uuid {
        Uuid::new_v4()
    }

    fn sample_author_id() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn should_create_valid_physical_space_demand_with_open_status() {
        let id = Uuid::new_v4();
        let ca_id = sample_ca_id();
        let author_id = sample_author_id();
        let now = Utc::now();

        let demand = Demand::new(
            id,
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            Some("Reparo da porta".to_string()),
            "A porta da sala do CA está danificada".to_string(),
            Some("Infraestrutura".to_string()),
            None,
            now,
        )
        .expect("demanda válida deve ser criada");

        assert_eq!(demand.id(), id);
        assert_eq!(demand.academic_center_id(), ca_id);
        assert_eq!(demand.author_id(), author_id);
        assert_eq!(demand.forum(), DemandForum::PhysicalSpace);
        assert_eq!(demand.title(), Some("Reparo da porta"));
        assert_eq!(
            demand.description(),
            "A porta da sala do CA está danificada"
        );
        assert_eq!(demand.content(), "A porta da sala do CA está danificada");
        assert_eq!(demand.category(), Some("Infraestrutura"));
        assert_eq!(demand.course_name(), None);
        assert_eq!(demand.status(), DemandStatus::Open);
        assert_eq!(demand.created_at(), now);
        assert_eq!(demand.updated_at(), now);
        assert!(demand.status_history().is_empty());
    }

    #[test]
    fn should_create_valid_demand_without_optional_fields() {
        let demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            None,
            "A lâmpada da sala precisa ser substituída".to_string(),
            None,
            None,
            Utc::now(),
        )
        .expect("demanda sem campos opcionais deve ser criada");

        assert_eq!(demand.title(), None);
        assert_eq!(demand.category(), None);
        assert_eq!(demand.course_name(), None);
        assert_eq!(
            demand.description(),
            "A lâmpada da sala precisa ser substituída"
        );
    }

    #[test]
    fn should_trim_whitespace_on_creation() {
        let demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::Institutional,
            Some("  Ajuste de grade  ".to_string()),
            "  Falta de vagas na disciplina obrigatória  ".to_string(),
            Some("  Acadêmico  ".to_string()),
            Some("  Ciência da Computação  ".to_string()),
            Utc::now(),
        )
        .expect("deve sanitizar espaços");

        assert_eq!(demand.title(), Some("Ajuste de grade"));
        assert_eq!(
            demand.description(),
            "Falta de vagas na disciplina obrigatória"
        );
        assert_eq!(demand.category(), Some("Acadêmico"));
        assert_eq!(demand.course_name(), Some("Ciência da Computação"));
    }

    #[test]
    fn should_reject_creation_with_empty_description() {
        let result = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            Some("Título".to_string()),
            "    ".to_string(),
            None,
            None,
            Utc::now(),
        );

        assert_eq!(result, Err(DemandError::EmptyDescription));
    }

    #[test]
    fn should_reject_course_name_for_physical_space_demand() {
        let result = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            Some("Reparo".to_string()),
            "Problema na tomada".to_string(),
            None,
            Some("Engenharia de Redes".to_string()),
            Utc::now(),
        );

        assert_eq!(result, Err(DemandError::CourseNotAllowedForPhysicalSpace));
    }

    #[test]
    fn should_allow_course_name_for_institutional_demand() {
        let demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::Institutional,
            Some("Ementa".to_string()),
            "Divergência na ementa da matéria".to_string(),
            Some("Pedagógico".to_string()),
            Some("Ciência da Computação".to_string()),
            Utc::now(),
        );

        assert!(demand.is_ok());
        assert_eq!(demand.unwrap().course_name(), Some("Ciência da Computação"));
    }

    #[test]
    fn should_transition_status_and_append_to_history() {
        let mut demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            Some("Reparo da porta".to_string()),
            "Porta emperrada".to_string(),
            None,
            None,
            Utc::now(),
        )
        .unwrap();

        let gestor_a = Uuid::new_v4();
        let t1 = demand.created_at() + chrono::Duration::hours(2);

        demand
            .start_analysis(gestor_a, Some("Gestor iniciou triagem".to_string()), t1)
            .expect("deve transitar para em análise");

        assert_eq!(demand.status(), DemandStatus::InAnalysis);
        assert_eq!(demand.updated_at(), t1);
        assert_eq!(demand.status_history().len(), 1);

        let entry1 = &demand.status_history()[0];
        assert_eq!(entry1.demand_id(), demand.id());
        assert_eq!(entry1.from_status(), Some(DemandStatus::Open));
        assert_eq!(entry1.to_status(), DemandStatus::InAnalysis);
        assert_eq!(entry1.changed_by_id(), gestor_a);
        assert_eq!(entry1.comment(), Some("Gestor iniciou triagem"));
        assert_eq!(entry1.created_at(), t1);

        let gestor_b = Uuid::new_v4();
        let t2 = t1 + chrono::Duration::hours(5);

        demand
            .complete(
                gestor_b,
                Some("Reparo finalizado com sucesso".to_string()),
                t2,
            )
            .expect("deve transitar para concluída");

        assert_eq!(demand.status(), DemandStatus::Completed);
        assert_eq!(demand.updated_at(), t2);
        assert_eq!(demand.status_history().len(), 2);

        let entry2 = &demand.status_history()[1];
        assert_eq!(entry2.from_status(), Some(DemandStatus::InAnalysis));
        assert_eq!(entry2.to_status(), DemandStatus::Completed);
        assert_eq!(entry2.changed_by_id(), gestor_b);
    }

    #[test]
    fn should_strictly_forbid_forwarding_physical_space_demand_to_coordination() {
        let mut demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            Some("Troca da lâmpada".to_string()),
            "Lâmpada queimada".to_string(),
            None,
            None,
            Utc::now(),
        )
        .unwrap();

        let gestor = Uuid::new_v4();
        let t1 = demand.created_at() + chrono::Duration::minutes(10);

        let result = demand.forward_to_coordination(gestor, None, t1);
        assert_eq!(result, Err(DemandError::CoordinationForwardingNotAllowed));
        assert_eq!(demand.status(), DemandStatus::Open);
        assert!(demand.status_history().is_empty());
    }

    #[test]
    fn should_allow_forwarding_institutional_demand_to_coordination() {
        let mut demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::Institutional,
            Some("Quebra de pré-requisito".to_string()),
            "Solicitação coletiva de quebra de pré-requisito".to_string(),
            None,
            Some("Ciência da Computação".to_string()),
            Utc::now(),
        )
        .unwrap();

        let gestor = Uuid::new_v4();
        let t1 = demand.created_at() + chrono::Duration::hours(1);

        demand
            .start_analysis(gestor, None, t1)
            .expect("inicia análise");

        let t2 = t1 + chrono::Duration::hours(1);
        demand
            .forward_to_coordination(
                gestor,
                Some("Encaminhado à coordenação para parecer".to_string()),
                t2,
            )
            .expect("deve permitir encaminhamento no institucional");

        assert_eq!(demand.status(), DemandStatus::ForwardedToCoordination);
        assert_eq!(demand.status_history().len(), 2);
    }

    #[test]
    fn should_reject_transition_with_timestamp_before_last_update() {
        let mut demand = Demand::new(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            None,
            "Problema de infra".to_string(),
            None,
            None,
            Utc::now(),
        )
        .unwrap();

        let past = demand.created_at() - chrono::Duration::minutes(5);
        let result = demand.start_analysis(Uuid::new_v4(), None, past);

        assert_eq!(
            result,
            Err(DemandError::InconsistentTimestamps {
                created_at: demand.created_at(),
                updated_at: past
            })
        );
    }

    #[test]
    fn should_reconstitute_persisted_demand() {
        let id = Uuid::new_v4();
        let ca_id = sample_ca_id();
        let author_id = sample_author_id();
        let created_at = Utc::now() - chrono::Duration::days(2);
        let updated_at = created_at + chrono::Duration::days(1);

        let entry = DemandStatusHistoryEntry::new(
            Uuid::new_v4(),
            id,
            Some(DemandStatus::Open),
            DemandStatus::InAnalysis,
            Uuid::new_v4(),
            Some("Triagem realizada".to_string()),
            updated_at,
        )
        .unwrap();

        let demand = Demand::reconstitute(
            id,
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            Some("Vazamento".to_string()),
            "Torneira pingando no banheiro do CA".to_string(),
            Some("Hidráulica".to_string()),
            None,
            DemandStatus::InAnalysis,
            created_at,
            updated_at,
            vec![entry.clone()],
        )
        .expect("reconstituição válida");

        assert_eq!(demand.id(), id);
        assert_eq!(demand.status(), DemandStatus::InAnalysis);
        assert_eq!(demand.status_history().len(), 1);
        assert_eq!(demand.status_history()[0], entry);
    }

    #[test]
    fn should_fail_reconstitution_when_updated_at_before_created_at() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);

        let result = Demand::reconstitute(
            Uuid::new_v4(),
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            None,
            "Conteúdo".to_string(),
            None,
            None,
            DemandStatus::Open,
            now,
            past,
            vec![],
        );

        assert_eq!(
            result,
            Err(DemandError::InconsistentTimestamps {
                created_at: now,
                updated_at: past
            })
        );
    }

    #[test]
    fn should_fail_reconstitution_when_history_entry_belongs_to_another_demand() {
        let id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let now = Utc::now();

        let entry = DemandStatusHistoryEntry::new(
            Uuid::new_v4(),
            other_id,
            Some(DemandStatus::Open),
            DemandStatus::InAnalysis,
            Uuid::new_v4(),
            None,
            now,
        )
        .unwrap();

        let result = Demand::reconstitute(
            id,
            sample_ca_id(),
            sample_author_id(),
            DemandForum::PhysicalSpace,
            None,
            "Conteúdo".to_string(),
            None,
            None,
            DemandStatus::InAnalysis,
            now,
            now,
            vec![entry],
        );

        assert_eq!(
            result,
            Err(DemandError::HistoryDemandIdMismatch {
                expected: id,
                actual: other_id
            })
        );
    }

    #[test]
    fn should_parse_forum_and_status_from_strings() {
        assert_eq!(
            "institutional".parse::<DemandForum>().unwrap(),
            DemandForum::Institutional
        );
        assert_eq!(
            "institucional".parse::<DemandForum>().unwrap(),
            DemandForum::Institutional
        );
        assert_eq!(
            "physical_space".parse::<DemandForum>().unwrap(),
            DemandForum::PhysicalSpace
        );
        assert_eq!(
            "espaço físico".parse::<DemandForum>().unwrap(),
            DemandForum::PhysicalSpace
        );
        assert_eq!(
            "espaco_fisico".parse::<DemandForum>().unwrap(),
            DemandForum::PhysicalSpace
        );

        assert_eq!("open".parse::<DemandStatus>().unwrap(), DemandStatus::Open);
        assert_eq!(
            "aberta".parse::<DemandStatus>().unwrap(),
            DemandStatus::Open
        );
        assert_eq!(
            "in_analysis".parse::<DemandStatus>().unwrap(),
            DemandStatus::InAnalysis
        );
        assert_eq!(
            "em análise".parse::<DemandStatus>().unwrap(),
            DemandStatus::InAnalysis
        );
        assert_eq!(
            "forwarded_to_coordination".parse::<DemandStatus>().unwrap(),
            DemandStatus::ForwardedToCoordination
        );
        assert_eq!(
            "enviada à coordenação".parse::<DemandStatus>().unwrap(),
            DemandStatus::ForwardedToCoordination
        );
        assert_eq!(
            "completed".parse::<DemandStatus>().unwrap(),
            DemandStatus::Completed
        );
        assert_eq!(
            "concluída".parse::<DemandStatus>().unwrap(),
            DemandStatus::Completed
        );
    }

    #[test]
    fn should_format_error_messages_correctly() {
        assert_eq!(
            DemandError::EmptyDescription.to_string(),
            "a descrição da demanda não pode ser vazia"
        );
        assert_eq!(
            DemandError::CoordinationForwardingNotAllowed.to_string(),
            "demandas do espaço físico não podem ser encaminhadas à coordenação do curso"
        );
        assert_eq!(
            DemandError::CourseNotAllowedForPhysicalSpace.to_string(),
            "demandas sobre o espaço físico do CA não admitem vínculo com curso específico"
        );
    }

    #[test]
    fn should_implement_std_error_trait() {
        let err: Box<dyn Error> = Box::new(DemandError::EmptyDescription);
        assert_eq!(err.to_string(), "a descrição da demanda não pode ser vazia");
    }
}
