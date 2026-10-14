//! Entidade de domínio representando o Centro Acadêmico (CA).
//!
//! O Centro Acadêmico é a unidade organizacional de representação estudantil,
//! vinculada à Universidade de Brasília (UnB) e responsável por representar
//! um ou mais cursos de graduação.

use std::{error::Error, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::ca::domain::rules::validate_academic_center;

/// Erros decorrentes de violações de regras de negócio ou validação de Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademicCenterError {
    /// O nome oficial do Centro Acadêmico não pode ser vazio.
    EmptyName,

    /// O nome oficial do Centro Acadêmico ultrapassou o comprimento máximo permitido.
    NameTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// A sigla do Centro Acadêmico não pode ser vazia.
    EmptyAcronym,

    /// A sigla do Centro Acadêmico ultrapassou o comprimento máximo permitido.
    AcronymTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// A sigla contém caracteres incompatíveis com o formato aceito.
    InvalidAcronym(String),

    /// A descrição institucional ultrapassou o comprimento máximo permitido.
    DescriptionTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O status informado não é reconhecido pelo domínio.
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada é inválida.
    InvalidStatusTransition {
        /// Status de origem.
        from: AcademicCenterStatus,
        /// Status de destino pretendido.
        to: AcademicCenterStatus,
    },

    /// Os timestamps de criação e atualização violam a coerência temporal.
    InconsistentTimestamps {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de atualização.
        updated_at: DateTime<Utc>,
    },
}

impl fmt::Display for AcademicCenterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "o nome oficial do Centro Acadêmico não pode ser vazio"),
            Self::NameTooLong { max } => {
                write!(
                    f,
                    "o nome oficial do Centro Acadêmico excede o limite de {max} caracteres"
                )
            }
            Self::EmptyAcronym => write!(f, "a sigla do Centro Acadêmico não pode ser vazia"),
            Self::AcronymTooLong { max } => {
                write!(
                    f,
                    "a sigla do Centro Acadêmico excede o limite de {max} caracteres"
                )
            }
            Self::InvalidAcronym(acronym) => {
                write!(
                    f,
                    "a sigla do Centro Acadêmico contém caracteres inválidos: '{acronym}'"
                )
            }
            Self::DescriptionTooLong { max } => {
                write!(
                    f,
                    "a descrição institucional do Centro Acadêmico excede o limite de {max} caracteres"
                )
            }
            Self::InvalidStatus(status) => {
                write!(
                    f,
                    "status de Centro Acadêmico inválido ou desconhecido: '{status}'"
                )
            }
            Self::InvalidStatusTransition { from, to } => {
                write!(f, "transição de status inválida de '{from}' para '{to}'")
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
        }
    }
}

impl Error for AcademicCenterError {}

/// Estado cadastral e operacional de um Centro Acadêmico no ciclo de vida do CApp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AcademicCenterStatus {
    /// Cadastro em elaboração, incompleto e indisponível para fluxos normais.
    #[default]
    Draft,
    /// CA regularizado, ativo e disponível no ecossistema da aplicação.
    Active,
    /// CA retirado de novos fluxos, preservando o histórico e dados pregressos.
    Archived,
}

impl AcademicCenterStatus {
    /// Retorna a representação textual canônica do status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl fmt::Display for AcademicCenterStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AcademicCenterStatus {
    type Err = AcademicCenterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            other => Err(AcademicCenterError::InvalidStatus(other.to_string())),
        }
    }
}

/// Entidade de domínio que representa um Centro Acadêmico no CApp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicCenter {
    /// Identificador estável do Centro Acadêmico.
    pub id: Uuid,
    /// Identificador do usuário que realizou o cadastro inicial do CA.
    pub created_by_id: Option<Uuid>,
    /// Nome oficial do Centro Acadêmico.
    pub name: String,
    /// Sigla oficial de apresentação do Centro Acadêmico.
    pub acronym: String,
    /// Descrição e apresentação institucional do CA.
    pub description: String,
    /// Situação cadastral e operacional do Centro Acadêmico.
    pub status: AcademicCenterStatus,
    /// Instante em que o registro foi criado no sistema.
    pub created_at: DateTime<Utc>,
    /// Instante da última modificação do registro.
    pub updated_at: DateTime<Utc>,
}

impl AcademicCenter {
    /// Cria uma nova instância da entidade validando todas as suas invariantes de domínio.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso nome, sigla, descrição ou timestamps
    /// violem os critérios de integridade estrutural definidos nas regras de domínio.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        created_by_id: Option<Uuid>,
        name: String,
        acronym: String,
        description: String,
        status: AcademicCenterStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, AcademicCenterError> {
        let instance = Self {
            id,
            created_by_id,
            name: name.trim().to_string(),
            acronym: acronym.trim().to_string(),
            description: description.trim().to_string(),
            status,
            created_at,
            updated_at,
        };
        instance.validate()?;
        Ok(instance)
    }

    /// Valida as invariantes de negócio e a integridade estrutural da entidade.
    ///
    /// Delega a verificação das regras estruturais para o módulo de regras.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso qualquer invariante seja violada.
    pub fn validate(&self) -> Result<(), AcademicCenterError> {
        validate_academic_center(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ca::domain::rules::{
        MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
    };

    const CAER_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);

    fn sample_valid_ca() -> AcademicCenter {
        let now = Utc::now();
        AcademicCenter::new(
            CAER_DIDACTIC_ID,
            None,
            "Centro Acadêmico de Engenharia de Redes".to_string(),
            "CAER".to_string(),
            "Centro Acadêmico de Engenharia de Redes da Universidade de Brasília".to_string(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .expect("should create valid academic center")
    }

    #[test]
    fn should_create_valid_academic_center() {
        let ca = sample_valid_ca();
        assert_eq!(ca.id, CAER_DIDACTIC_ID);
        assert_eq!(ca.name, "Centro Acadêmico de Engenharia de Redes");
        assert_eq!(ca.acronym, "CAER");
        assert_eq!(ca.status, AcademicCenterStatus::Active);
        assert!(ca.created_at <= ca.updated_at);
    }

    #[test]
    fn should_trim_whitespace_on_creation() {
        let now = Utc::now();
        let ca = AcademicCenter::new(
            CACC_DIDACTIC_ID,
            None,
            "  Centro Acadêmico de Ciência da Computação  ".to_string(),
            "  CACC  ".to_string(),
            "  Descrição do CACC  ".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .expect("should create and trim fields");

        assert_eq!(ca.name, "Centro Acadêmico de Ciência da Computação");
        assert_eq!(ca.acronym, "CACC");
        assert_eq!(ca.description, "Descrição do CACC");
    }

    #[test]
    fn should_fail_when_name_is_empty_or_whitespace() {
        let now = Utc::now();
        let err_empty = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "".to_string(),
            "CAER".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyName);

        let err_whitespace = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "   ".to_string(),
            "CAER".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(err_whitespace, AcademicCenterError::EmptyName);
    }

    #[test]
    fn should_fail_when_name_exceeds_max_length() {
        let now = Utc::now();
        let long_name = "A".repeat(MAX_NAME_LENGTH + 1);
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            long_name,
            "CAER".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_acronym_is_empty_or_whitespace() {
        let now = Utc::now();
        let err_empty = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyAcronym);

        let err_whitespace = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "   ".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(err_whitespace, AcademicCenterError::EmptyAcronym);
    }

    #[test]
    fn should_fail_when_acronym_exceeds_max_length() {
        let now = Utc::now();
        let long_acronym = "A".repeat(MAX_ACRONYM_LENGTH + 1);
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            long_acronym,
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_acronym_contains_invalid_characters() {
        let now = Utc::now();
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "CA ER!".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::InvalidAcronym("CA ER!".to_string())
        );
    }

    #[test]
    fn should_allow_acronyms_with_hyphens_dots_and_underscores() {
        let now = Utc::now();
        let ca1 = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "CA-REDES".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .expect("should allow hyphen in acronym");
        assert_eq!(ca1.acronym, "CA-REDES");

        let ca2 = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "C.A.E.R".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .expect("should allow dot in acronym");
        assert_eq!(ca2.acronym, "C.A.E.R");

        let ca3 = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "CA_COMP".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .expect("should allow underscore in acronym");
        assert_eq!(ca3.acronym, "CA_COMP");
    }

    #[test]
    fn should_fail_when_description_exceeds_max_length() {
        let now = Utc::now();
        let long_desc = "D".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "CA".to_string(),
            long_desc,
            AcademicCenterStatus::Draft,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_timestamps_are_inconsistent() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);

        let err = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "Centro Acadêmico".to_string(),
            "CA".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            past,
        )
        .unwrap_err();

        assert_eq!(
            err,
            AcademicCenterError::InconsistentTimestamps {
                created_at: now,
                updated_at: past,
            }
        );
    }

    #[test]
    fn should_parse_status_from_string() {
        assert_eq!(
            "draft".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Draft
        );
        assert_eq!(
            "active".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Active
        );
        assert_eq!(
            "archived".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Archived
        );
        assert_eq!(
            "ACTIVE".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Active
        );
        assert_eq!(
            "  Draft  ".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Draft
        );

        let err = "desconhecido".parse::<AcademicCenterStatus>().unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::InvalidStatus("desconhecido".to_string())
        );
    }

    #[test]
    fn should_format_status_as_expected() {
        assert_eq!(AcademicCenterStatus::Draft.as_str(), "draft");
        assert_eq!(AcademicCenterStatus::Active.as_str(), "active");
        assert_eq!(AcademicCenterStatus::Archived.as_str(), "archived");

        assert_eq!(AcademicCenterStatus::Draft.to_string(), "draft");
        assert_eq!(AcademicCenterStatus::Active.to_string(), "active");
        assert_eq!(AcademicCenterStatus::Archived.to_string(), "archived");

        assert_eq!(AcademicCenterStatus::default(), AcademicCenterStatus::Draft);
    }

    #[test]
    fn should_format_all_error_messages_correctly() {
        assert_eq!(
            AcademicCenterError::EmptyName.to_string(),
            "o nome oficial do Centro Acadêmico não pode ser vazio"
        );
        assert_eq!(
            AcademicCenterError::NameTooLong { max: 255 }.to_string(),
            "o nome oficial do Centro Acadêmico excede o limite de 255 caracteres"
        );
        assert_eq!(
            AcademicCenterError::EmptyAcronym.to_string(),
            "a sigla do Centro Acadêmico não pode ser vazia"
        );
        assert_eq!(
            AcademicCenterError::AcronymTooLong { max: 30 }.to_string(),
            "a sigla do Centro Acadêmico excede o limite de 30 caracteres"
        );
        assert_eq!(
            AcademicCenterError::InvalidAcronym("CA!".to_string()).to_string(),
            "a sigla do Centro Acadêmico contém caracteres inválidos: 'CA!'"
        );
        assert_eq!(
            AcademicCenterError::DescriptionTooLong { max: 4000 }.to_string(),
            "a descrição institucional do Centro Acadêmico excede o limite de 4000 caracteres"
        );
        assert_eq!(
            AcademicCenterError::InvalidStatus("xyz".to_string()).to_string(),
            "status de Centro Acadêmico inválido ou desconhecido: 'xyz'"
        );
        assert_eq!(
            AcademicCenterError::InvalidStatusTransition {
                from: AcademicCenterStatus::Archived,
                to: AcademicCenterStatus::Active,
            }
            .to_string(),
            "transição de status inválida de 'archived' para 'active'"
        );

        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        let ts_err = AcademicCenterError::InconsistentTimestamps {
            created_at: now,
            updated_at: past,
        };
        assert!(ts_err.to_string().contains("não pode ser anterior"));
    }

    #[test]
    fn should_implement_std_error_trait() {
        let err: Box<dyn Error> = Box::new(AcademicCenterError::EmptyName);
        assert_eq!(
            err.to_string(),
            "o nome oficial do Centro Acadêmico não pode ser vazio"
        );
        assert!(err.source().is_none());
    }

    #[test]
    fn should_serialize_and_deserialize_json() {
        let original = sample_valid_ca();
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: AcademicCenter = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(original, deserialized);
    }
}
