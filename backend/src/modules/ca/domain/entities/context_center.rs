//! Entidade de domínio representando o Centro Acadêmico (CA).
//!
//! O Centro Acadêmico é a unidade organizacional de representação estudantil,
//! vinculada institucionalmente a uma universidade (por padrão a UnB) e capaz
//! de representar um ou mais cursos de graduação.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Identificador estável do Centro Acadêmico de Engenharia de Redes (CAER) para cenários didáticos.
pub const CAER_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);

/// Identificador estável do Centro Acadêmico de Ciência da Computação (CACC) para cenários didáticos.
pub const CACC_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);

/// Limite máximo em caracteres para o nome oficial do Centro Acadêmico.
pub const MAX_NAME_LENGTH: usize = 255;

/// Limite máximo em caracteres para a sigla do Centro Acadêmico.
pub const MAX_ACRONYM_LENGTH: usize = 30;

/// Limite máximo em caracteres para a descrição institucional do Centro Acadêmico.
pub const MAX_DESCRIPTION_LENGTH: usize = 4000;

/// Erros decorrentes de violações de regras de negócio ou validação de Centro Acadêmico.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AcademicCenterError {
    /// O nome oficial do Centro Acadêmico não pode ser vazio.
    #[error("o nome oficial do Centro Acadêmico não pode ser vazio")]
    EmptyName,

    /// O nome oficial do Centro Acadêmico ultrapassou o comprimento máximo permitido.
    #[error("o nome oficial do Centro Acadêmico excede o limite de {max} caracteres")]
    NameTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// A sigla do Centro Acadêmico não pode ser vazia.
    #[error("a sigla do Centro Acadêmico não pode ser vazia")]
    EmptyAcronym,

    /// A sigla do Centro Acadêmico ultrapassou o comprimento máximo permitido.
    #[error("a sigla do Centro Acadêmico excede o limite de {max} caracteres")]
    AcronymTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// A sigla contém caracteres incompatíveis com o formato aceito.
    #[error("a sigla do Centro Acadêmico contém caracteres inválidos: '{0}'")]
    InvalidAcronym(String),

    /// A descrição institucional ultrapassou o comprimento máximo permitido.
    #[error("a descrição institucional do Centro Acadêmico excede o limite de {max} caracteres")]
    DescriptionTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O status informado não é reconhecido pelo domínio.
    #[error("status de Centro Acadêmico inválido ou desconhecido: '{0}'")]
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada é inválida.
    #[error("transição de status inválida de '{from}' para '{to}'")]
    InvalidStatusTransition {
        /// Status de origem.
        from: AcademicCenterStatus,
        /// Status de destino pretendido.
        to: AcademicCenterStatus,
    },

    /// Os timestamps de criação e atualização violam a coerência temporal.
    #[error(
        "data de atualização ({updated_at}) não pode ser anterior à data de criação ({created_at})"
    )]
    InconsistentTimestamps {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de atualização.
        updated_at: DateTime<Utc>,
    },
}

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

impl sqlx::Type<sqlx::Postgres> for AcademicCenterStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AcademicCenterStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let text = <&'r str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        text.parse()
            .map_err(|err: AcademicCenterError| Box::new(err) as _)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AcademicCenterStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + 'static + Send + Sync>> {
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

/// Entidade de domínio que representa um Centro Acadêmico no CApp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct AcademicCenter {
    /// Identificador estável do Centro Acadêmico.
    pub id: Uuid,
    /// Identificador da instituição à qual o CA está vinculado.
    pub institution_id: Option<Uuid>,
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
    /// violem os critérios de integridade estrutural.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        institution_id: Option<Uuid>,
        created_by_id: Option<Uuid>,
        name: impl Into<String>,
        acronym: impl Into<String>,
        description: impl Into<String>,
        status: AcademicCenterStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, AcademicCenterError> {
        let instance = Self {
            id,
            institution_id,
            created_by_id,
            name: name.into().trim().to_string(),
            acronym: acronym.into().trim().to_string(),
            description: description.into().trim().to_string(),
            status,
            created_at,
            updated_at,
        };
        instance.validate()?;
        Ok(instance)
    }

    /// Cria um novo Centro Acadêmico em estado inicial de rascunho (`draft`).
    ///
    /// Gera um novo UUID aleatório e define `created_at` e `updated_at` com o instante atual em UTC.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] se os dados básicos não atenderem às regras estruturais.
    pub fn draft(
        name: impl Into<String>,
        acronym: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, AcademicCenterError> {
        let now = Utc::now();
        Self::new(
            Uuid::new_v4(),
            None,
            None,
            name,
            acronym,
            description,
            AcademicCenterStatus::Draft,
            now,
            now,
        )
    }

    /// Cria a instância didática do Centro Acadêmico de Engenharia de Redes (CAER).
    pub fn caer_didactic() -> Self {
        let reference_time = DateTime::from_timestamp(1_767_225_600, 0).unwrap_or_default();
        Self {
            id: CAER_DIDACTIC_ID,
            institution_id: None,
            created_by_id: None,
            name: "Centro Acadêmico de Engenharia de Redes".to_string(),
            acronym: "CAER".to_string(),
            description: "Centro Acadêmico de Engenharia de Redes da Universidade de Brasília"
                .to_string(),
            status: AcademicCenterStatus::Active,
            created_at: reference_time,
            updated_at: reference_time,
        }
    }

    /// Cria a instância didática do Centro Acadêmico de Ciência da Computação (CACC).
    pub fn cacc_didactic() -> Self {
        let reference_time = DateTime::from_timestamp(1_767_225_600, 0).unwrap_or_default();
        Self {
            id: CACC_DIDACTIC_ID,
            institution_id: None,
            created_by_id: None,
            name: "Centro Acadêmico de Ciência da Computação".to_string(),
            acronym: "CACC".to_string(),
            description: "Centro Acadêmico de Ciência da Computação da Universidade de Brasília"
                .to_string(),
            status: AcademicCenterStatus::Active,
            created_at: reference_time,
            updated_at: reference_time,
        }
    }

    /// Valida as invariantes de negócio e a integridade estrutural da entidade.
    ///
    /// # Regras verificadas
    ///
    /// - O nome não pode ser vazio ou composto exclusivamente por espaços em branco.
    /// - O nome não pode exceder [`MAX_NAME_LENGTH`].
    /// - A sigla não pode ser vazia ou composta exclusivamente por espaços em branco.
    /// - A sigla não pode exceder [`MAX_ACRONYM_LENGTH`].
    /// - A sigla deve ser composta apenas por caracteres alfanuméricos, hífens ou pontos.
    /// - A descrição não pode exceder [`MAX_DESCRIPTION_LENGTH`].
    /// - `updated_at` não pode ser estritamente anterior a `created_at`.
    pub fn validate(&self) -> Result<(), AcademicCenterError> {
        if self.name.trim().is_empty() {
            return Err(AcademicCenterError::EmptyName);
        }
        if self.name.chars().count() > MAX_NAME_LENGTH {
            return Err(AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH,
            });
        }

        if self.acronym.trim().is_empty() {
            return Err(AcademicCenterError::EmptyAcronym);
        }
        if self.acronym.chars().count() > MAX_ACRONYM_LENGTH {
            return Err(AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH,
            });
        }

        let is_valid_acronym_char =
            |c: char| c.is_alphanumeric() || c == '-' || c == '.' || c == '_';
        if !self.acronym.chars().all(is_valid_acronym_char) {
            return Err(AcademicCenterError::InvalidAcronym(self.acronym.clone()));
        }

        if self.description.chars().count() > MAX_DESCRIPTION_LENGTH {
            return Err(AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH,
            });
        }

        if self.updated_at < self.created_at {
            return Err(AcademicCenterError::InconsistentTimestamps {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }

        Ok(())
    }

    /// Ativa o Centro Acadêmico, tornando-o elegível para os fluxos regulares do aplicativo.
    ///
    /// # Erros
    ///
    /// Retorna erro se o CA estiver arquivado, pois a desativação por arquivamento
    /// encerra os fluxos ativos daquela entidade.
    pub fn activate(&mut self) -> Result<(), AcademicCenterError> {
        match self.status {
            AcademicCenterStatus::Active => Ok(()),
            AcademicCenterStatus::Draft => {
                self.status = AcademicCenterStatus::Active;
                self.updated_at = Utc::now();
                Ok(())
            }
            AcademicCenterStatus::Archived => Err(AcademicCenterError::InvalidStatusTransition {
                from: AcademicCenterStatus::Archived,
                to: AcademicCenterStatus::Active,
            }),
        }
    }

    /// Arquiva o Centro Acadêmico, retirando-o de novos fluxos e mantendo o histórico de dados.
    pub fn archive(&mut self) -> Result<(), AcademicCenterError> {
        if self.status != AcademicCenterStatus::Archived {
            self.status = AcademicCenterStatus::Archived;
            self.updated_at = Utc::now();
        }
        Ok(())
    }

    /// Atualiza o nome oficial do Centro Acadêmico após validação.
    pub fn update_name(&mut self, new_name: impl Into<String>) -> Result<(), AcademicCenterError> {
        let trimmed = new_name.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(AcademicCenterError::EmptyName);
        }
        if trimmed.chars().count() > MAX_NAME_LENGTH {
            return Err(AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH,
            });
        }
        self.name = trimmed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Atualiza a sigla do Centro Acadêmico após validação.
    pub fn update_acronym(
        &mut self,
        new_acronym: impl Into<String>,
    ) -> Result<(), AcademicCenterError> {
        let trimmed = new_acronym.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(AcademicCenterError::EmptyAcronym);
        }
        if trimmed.chars().count() > MAX_ACRONYM_LENGTH {
            return Err(AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH,
            });
        }
        let is_valid_acronym_char =
            |c: char| c.is_alphanumeric() || c == '-' || c == '.' || c == '_';
        if !trimmed.chars().all(is_valid_acronym_char) {
            return Err(AcademicCenterError::InvalidAcronym(trimmed));
        }
        self.acronym = trimmed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Atualiza a descrição institucional do Centro Acadêmico.
    pub fn update_description(
        &mut self,
        new_description: impl Into<String>,
    ) -> Result<(), AcademicCenterError> {
        let trimmed = new_description.into().trim().to_string();
        if trimmed.chars().count() > MAX_DESCRIPTION_LENGTH {
            return Err(AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH,
            });
        }
        self.description = trimmed;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_academic_center_draft() {
        let ca = AcademicCenter::draft(
            "Centro Acadêmico de Engenharia de Redes",
            "CAER",
            "Apresentação institucional do CAER",
        )
        .expect("should create valid draft");

        assert_eq!(ca.name, "Centro Acadêmico de Engenharia de Redes");
        assert_eq!(ca.acronym, "CAER");
        assert_eq!(ca.description, "Apresentação institucional do CAER");
        assert_eq!(ca.status, AcademicCenterStatus::Draft);
        assert!(ca.created_at <= ca.updated_at);
    }

    #[test]
    fn should_instantiate_didactic_cas_validly() {
        let caer = AcademicCenter::caer_didactic();
        assert_eq!(caer.id, CAER_DIDACTIC_ID);
        assert_eq!(caer.acronym, "CAER");
        assert_eq!(caer.name, "Centro Acadêmico de Engenharia de Redes");
        assert_eq!(caer.status, AcademicCenterStatus::Active);
        caer.validate().expect("CAER didactic must be valid");

        let cacc = AcademicCenter::cacc_didactic();
        assert_eq!(cacc.id, CACC_DIDACTIC_ID);
        assert_eq!(cacc.acronym, "CACC");
        assert_eq!(cacc.name, "Centro Acadêmico de Ciência da Computação");
        assert_eq!(cacc.status, AcademicCenterStatus::Active);
        cacc.validate().expect("CACC didactic must be valid");
    }

    #[test]
    fn should_fail_when_name_is_empty_or_whitespace() {
        let err_empty = AcademicCenter::draft("", "CAER", "Desc").unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyName);

        let err_whitespace = AcademicCenter::draft("   ", "CAER", "Desc").unwrap_err();
        assert_eq!(err_whitespace, AcademicCenterError::EmptyName);
    }

    #[test]
    fn should_fail_when_name_exceeds_max_length() {
        let long_name = "A".repeat(MAX_NAME_LENGTH + 1);
        let err = AcademicCenter::draft(long_name, "CAER", "Desc").unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_acronym_is_empty_or_whitespace() {
        let err_empty = AcademicCenter::draft("Centro Acadêmico", "", "Desc").unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyAcronym);

        let err_whitespace = AcademicCenter::draft("Centro Acadêmico", "   ", "Desc").unwrap_err();
        assert_eq!(err_whitespace, AcademicCenterError::EmptyAcronym);
    }

    #[test]
    fn should_fail_when_acronym_exceeds_max_length() {
        let long_acronym = "A".repeat(MAX_ACRONYM_LENGTH + 1);
        let err = AcademicCenter::draft("Centro Acadêmico", long_acronym, "Desc").unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_acronym_contains_invalid_characters() {
        let err = AcademicCenter::draft("Centro Acadêmico", "CA ER!", "Desc").unwrap_err();
        assert!(matches!(err, AcademicCenterError::InvalidAcronym(_)));
    }

    #[test]
    fn should_allow_acronyms_with_hyphens_or_dots() {
        let ca = AcademicCenter::draft("Centro Acadêmico", "CA-REDES", "Desc")
            .expect("should allow hyphen in acronym");
        assert_eq!(ca.acronym, "CA-REDES");

        let ca2 = AcademicCenter::draft("Centro Acadêmico", "C.A.E.R", "Desc")
            .expect("should allow dot in acronym");
        assert_eq!(ca2.acronym, "C.A.E.R");
    }

    #[test]
    fn should_fail_when_description_exceeds_max_length() {
        let long_desc = "D".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let err = AcademicCenter::draft("Centro Acadêmico", "CA", long_desc).unwrap_err();
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
            None,
            "Centro Acadêmico",
            "CA",
            "Desc",
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
    fn should_transition_status_correctly() {
        let mut ca = AcademicCenter::draft("Centro Acadêmico", "CA", "Desc").unwrap();
        assert_eq!(ca.status, AcademicCenterStatus::Draft);

        ca.activate().expect("should activate from draft");
        assert_eq!(ca.status, AcademicCenterStatus::Active);

        // Activating again is idempotent
        ca.activate().expect("activating active CA should succeed");
        assert_eq!(ca.status, AcademicCenterStatus::Active);

        ca.archive().expect("should archive from active");
        assert_eq!(ca.status, AcademicCenterStatus::Archived);

        // Cannot reactivate archived CA
        let err = ca.activate().unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::InvalidStatusTransition {
                from: AcademicCenterStatus::Archived,
                to: AcademicCenterStatus::Active,
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

        let err = "desconhecido".parse::<AcademicCenterStatus>().unwrap_err();
        assert_eq!(
            err,
            AcademicCenterError::InvalidStatus("desconhecido".to_string())
        );
    }

    #[test]
    fn should_format_status_as_expected() {
        assert_eq!(AcademicCenterStatus::Draft.to_string(), "draft");
        assert_eq!(AcademicCenterStatus::Active.to_string(), "active");
        assert_eq!(AcademicCenterStatus::Archived.to_string(), "archived");
    }

    #[test]
    fn should_update_attributes_with_validation() {
        let mut ca = AcademicCenter::draft("Nome Original", "SIGLA", "Descricao").unwrap();

        ca.update_name("Novo Nome").expect("should update name");
        assert_eq!(ca.name, "Novo Nome");

        ca.update_acronym("NOVASIGLA")
            .expect("should update acronym");
        assert_eq!(ca.acronym, "NOVASIGLA");

        ca.update_description("Nova Descricao")
            .expect("should update description");
        assert_eq!(ca.description, "Nova Descricao");

        assert_eq!(
            ca.update_name("   ").unwrap_err(),
            AcademicCenterError::EmptyName
        );
        assert_eq!(
            ca.update_acronym("").unwrap_err(),
            AcademicCenterError::EmptyAcronym
        );
    }

    #[test]
    fn should_serialize_and_deserialize_json() {
        let original = AcademicCenter::caer_didactic();
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: AcademicCenter = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(original, deserialized);
    }
}
