//! Dados acadêmicos associados ao perfil interno do usuário.
//!
//! Representa as informações institucionais recuperadas (curso, situação
//! acadêmica/vínculo estudantil e instituição de ensino) que orientam o contexto
//! discente e as regras de acesso do aplicativo, em conformidade com § 1.3 do
//! Relatório de Features e a especificação BDD de associação de dados acadêmicos.

use std::{error::Error, fmt, str::FromStr};
use uuid::Uuid;

use crate::modules::identity::domain::rules::academic_data::{
    allows_student_access, determine_academic_standing, is_academic_context_complete,
};

/// Erro de validação ou conversão de situação acadêmica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademicStandingError {
    /// Situação acadêmica desconhecida ou inválida.
    InvalidStanding(String),
}

impl fmt::Display for AcademicStandingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStanding(val) => {
                write!(f, "situação acadêmica inválida ou desconhecida: '{val}'")
            }
        }
    }
}

impl Error for AcademicStandingError {}

/// Situação acadêmica do usuário identificada junto à instituição de ensino.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AcademicStanding {
    /// Discente com vínculo regular e ativo confirmado pela instituição.
    ActiveStudent,
    /// Usuário sem vínculo discente confirmado (não estudante ou vínculo inativo).
    #[default]
    NonStudent,
}

impl AcademicStanding {
    /// Retorna a representação textual canônica da situação acadêmica.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ActiveStudent => "active_student",
            Self::NonStudent => "non_student",
        }
    }
}

impl fmt::Display for AcademicStanding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActiveStudent => write!(f, "Discente Ativo"),
            Self::NonStudent => write!(f, "Não Discente"),
        }
    }
}

impl FromStr for AcademicStanding {
    type Err = AcademicStandingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "active_student" | "student" | "discente" | "ativo" => Ok(Self::ActiveStudent),
            "non_student" | "nao_discente" | "inativo" => Ok(Self::NonStudent),
            other => Err(AcademicStandingError::InvalidStanding(other.to_string())),
        }
    }
}

/// Dados acadêmicos associados ao estudante no domínio da aplicação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicData {
    user_id: Uuid,
    institution_id: Uuid,
    course_name: Option<String>,
    standing: AcademicStanding,
}

impl AcademicData {
    /// Constrói uma nova instância de dados acadêmicos com validação e normalização.
    pub fn new(
        user_id: Uuid,
        institution_id: Uuid,
        course_name: Option<String>,
        standing: AcademicStanding,
    ) -> Self {
        let course_name = course_name
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        Self {
            user_id,
            institution_id,
            course_name,
            standing,
        }
    }

    /// Cria dados acadêmicos a partir do resultado de consulta ao diretório institucional.
    pub fn from_institutional_source(
        user_id: Uuid,
        institution_id: Uuid,
        is_student_member: bool,
        course_name: Option<String>,
    ) -> Self {
        let standing = determine_academic_standing(is_student_member, course_name.as_deref());
        let course_name = if standing == AcademicStanding::ActiveStudent {
            course_name
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
        } else {
            None
        };

        Self {
            user_id,
            institution_id,
            course_name,
            standing,
        }
    }

    /// Identificador do usuário interno ao qual os dados acadêmicos estão associados.
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Identificador da instituição de ensino associada.
    pub fn institution_id(&self) -> Uuid {
        self.institution_id
    }

    /// Nome do curso identificado pela instituição, quando confirmado.
    pub fn course_name(&self) -> Option<&str> {
        self.course_name.as_deref()
    }

    /// Situação acadêmica associada ao perfil.
    pub fn standing(&self) -> AcademicStanding {
        self.standing
    }

    /// Indica se o perfil possui curso e situação regular de discente ativo.
    pub fn is_student(&self) -> bool {
        self.standing == AcademicStanding::ActiveStudent && self.course_name.is_some()
    }

    /// Verifica se o contexto acadêmico está completo.
    pub fn is_complete(&self) -> bool {
        is_academic_context_complete(self.course_name.as_deref(), self.standing)
    }

    /// Verifica se as informações acadêmicas conferem acesso a regras estudantis.
    pub fn allows_academic_access(&self) -> bool {
        allows_student_access(self.standing, self.course_name.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_academic_data() {
        let user_id = Uuid::new_v4();
        let inst_id = Uuid::new_v4();
        let data = AcademicData::new(
            user_id,
            inst_id,
            Some("  Engenharia de Redes  ".into()),
            AcademicStanding::ActiveStudent,
        );

        assert_eq!(data.user_id(), user_id);
        assert_eq!(data.institution_id(), inst_id);
        assert_eq!(data.course_name(), Some("Engenharia de Redes"));
        assert_eq!(data.standing(), AcademicStanding::ActiveStudent);
        assert!(data.is_student());
        assert!(data.is_complete());
        assert!(data.allows_academic_access());
    }

    #[test]
    fn should_handle_non_student_standing_correctly() {
        let user_id = Uuid::new_v4();
        let inst_id = Uuid::new_v4();
        let data = AcademicData::new(
            user_id,
            inst_id,
            Some("Administração".into()),
            AcademicStanding::NonStudent,
        );

        assert_eq!(data.standing(), AcademicStanding::NonStudent);
        assert!(!data.is_student());
        assert!(!data.is_complete());
        assert!(!data.allows_academic_access());
    }

    #[test]
    fn should_build_from_institutional_source() {
        let user_id = Uuid::new_v4();
        let inst_id = Uuid::new_v4();

        // Membro discente com curso
        let data_student = AcademicData::from_institutional_source(
            user_id,
            inst_id,
            true,
            Some("Ciência da Computação".into()),
        );
        assert!(data_student.is_student());
        assert_eq!(data_student.course_name(), Some("Ciência da Computação"));

        // Membro discente sem curso (deve virar NonStudent e sem curso)
        let data_no_course = AcademicData::from_institutional_source(user_id, inst_id, true, None);
        assert!(!data_no_course.is_student());
        assert_eq!(data_no_course.course_name(), None);

        // Não membro discente com curso (deve virar NonStudent e sem curso)
        let data_non_member = AcademicData::from_institutional_source(
            user_id,
            inst_id,
            false,
            Some("Ciência da Computação".into()),
        );
        assert!(!data_non_member.is_student());
        assert_eq!(data_non_member.course_name(), None);
    }

    #[test]
    fn should_parse_standing_from_str() {
        assert_eq!(
            AcademicStanding::from_str("active_student").unwrap(),
            AcademicStanding::ActiveStudent
        );
        assert_eq!(
            AcademicStanding::from_str("student").unwrap(),
            AcademicStanding::ActiveStudent
        );
        assert_eq!(
            AcademicStanding::from_str("non_student").unwrap(),
            AcademicStanding::NonStudent
        );
        assert!(AcademicStanding::from_str("desconhecido").is_err());
    }
}
