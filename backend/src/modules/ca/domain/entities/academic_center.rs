//! Entidade de domínio representando o Centro Acadêmico (CA).
//!
//! O Centro Acadêmico é a unidade organizacional de representação estudantil,
<<<<<<< HEAD
//! vinculada a uma instituição de ensino superior e responsável por representar
//! um ou mais cursos de graduação.

use std::{error::Error, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::ca::domain::rules::{
    validate_academic_center, validate_acronym, validate_description, validate_name,
    validate_status_transition, validate_timestamps,
};

/// Erros decorrentes de violações de regras de negócio ou validação de Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademicCenterError {
    /// O nome oficial do Centro Acadêmico não pode ser vazio.
    EmptyName,

    /// O nome oficial do Centro Acadêmico ultrapassou o comprimento máximo permitido.
=======
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
>>>>>>> origin/pr/331
    NameTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

<<<<<<< HEAD
    /// A sigla do Centro Acadêmico não pode ser vazia quando informada.
    EmptyAcronym,

    /// A sigla do Centro Acadêmico ultrapassou o comprimento máximo permitido.
=======
    /// A sigla do Centro Acadêmico não pode ser vazia.
    #[error("a sigla do Centro Acadêmico não pode ser vazia")]
    EmptyAcronym,

    /// A sigla do Centro Acadêmico ultrapassou o comprimento máximo permitido.
    #[error("a sigla do Centro Acadêmico excede o limite de {max} caracteres")]
>>>>>>> origin/pr/331
    AcronymTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// A sigla contém caracteres incompatíveis com o formato aceito.
<<<<<<< HEAD
    InvalidAcronym(String),

    /// A descrição institucional ultrapassou o comprimento máximo permitido.
=======
    #[error("a sigla do Centro Acadêmico contém caracteres inválidos: '{0}'")]
    InvalidAcronym(String),

    /// A descrição institucional ultrapassou o comprimento máximo permitido.
    #[error("a descrição institucional do Centro Acadêmico excede o limite de {max} caracteres")]
>>>>>>> origin/pr/331
    DescriptionTooLong {
        /// Limite máximo permitido.
        max: usize,
    },

    /// O status informado não é reconhecido pelo domínio.
<<<<<<< HEAD
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada é inválida.
=======
    #[error("status de Centro Acadêmico inválido ou desconhecido: '{0}'")]
    InvalidStatus(String),

    /// A transição de ciclo de vida solicitada é inválida.
    #[error("transição de status inválida de '{from}' para '{to}'")]
>>>>>>> origin/pr/331
    InvalidStatusTransition {
        /// Status de origem.
        from: AcademicCenterStatus,
        /// Status de destino pretendido.
        to: AcademicCenterStatus,
    },

    /// Os timestamps de criação e atualização violam a coerência temporal.
<<<<<<< HEAD
=======
    #[error(
        "data de atualização ({updated_at}) não pode ser anterior à data de criação ({created_at})"
    )]
>>>>>>> origin/pr/331
    InconsistentTimestamps {
        /// Instante de criação.
        created_at: DateTime<Utc>,
        /// Instante de atualização.
        updated_at: DateTime<Utc>,
    },
}

<<<<<<< HEAD
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AcademicCenterStatus {
=======
/// Estado cadastral e operacional de um Centro Acadêmico no ciclo de vida do CApp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AcademicCenterStatus {
    /// Cadastro em elaboração, incompleto e indisponível para fluxos normais.
    #[default]
    Draft,
>>>>>>> origin/pr/331
    /// CA regularizado, ativo e disponível no ecossistema da aplicação.
    Active,
    /// CA retirado de novos fluxos, preservando o histórico e dados pregressos.
    Archived,
}

impl AcademicCenterStatus {
    /// Retorna a representação textual canônica do status.
    pub const fn as_str(&self) -> &'static str {
        match self {
<<<<<<< HEAD
=======
            Self::Draft => "draft",
>>>>>>> origin/pr/331
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
<<<<<<< HEAD
=======
            "draft" => Ok(Self::Draft),
>>>>>>> origin/pr/331
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            other => Err(AcademicCenterError::InvalidStatus(other.to_string())),
        }
    }
}

<<<<<<< HEAD
/// Entidade de domínio que representa um Centro Acadêmico no CApp.
///
/// Todos os campos são privados e mantêm as invariantes de negócio protegidas
/// contra mutações ou bypass de validação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicCenter {
    /// Identificador estável do Centro Acadêmico.
    id: Uuid,
    /// Identificador da instituição à qual o Centro Acadêmico está vinculado.
    institution_id: Uuid,
    /// Identificador do usuário que realizou o cadastro inicial do CA, se houver.
    created_by_id: Option<Uuid>,
    /// Nome oficial do Centro Acadêmico.
    name: String,
    /// Sigla oficial de apresentação do Centro Acadêmico (opcional).
    acronym: Option<String>,
    /// Descrição e apresentação institucional do CA.
    description: String,
    /// Situação cadastral e operacional do Centro Acadêmico.
    status: AcademicCenterStatus,
    /// Instante em que o registro foi criado no sistema.
    created_at: DateTime<Utc>,
    /// Instante da última modificação do registro.
    updated_at: DateTime<Utc>,
}

impl AcademicCenter {
    /// Cria uma nova instância de Centro Acadêmico para um cadastro aprovado.
    ///
    /// Valida todas as invariantes e inicia a entidade com status [`AcademicCenterStatus::Active`].
    /// Os campos de texto têm espaços periféricos sanitizados (`trim`).
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso nome, sigla ou descrição violem as regras estruturais.
    pub fn new(
        id: Uuid,
        institution_id: Uuid,
        created_by_id: Option<Uuid>,
        name: String,
        acronym: Option<String>,
        description: String,
        created_at: DateTime<Utc>,
    ) -> Result<Self, AcademicCenterError> {
        let trimmed_name = name.trim();
        validate_name(trimmed_name)?;

        let trimmed_acronym = match acronym {
            Some(ref a) => {
                let t = a.trim();
                validate_acronym(t)?;
                Some(t.to_string())
            }
            None => None,
        };

        let trimmed_desc = description.trim();
        validate_description(trimmed_desc)?;

        Ok(Self {
            id,
            institution_id,
            created_by_id,
            name: trimmed_name.to_string(),
            acronym: trimmed_acronym,
            description: trimmed_desc.to_string(),
            status: AcademicCenterStatus::Active,
            created_at,
            updated_at: created_at,
        })
    }

    /// Reconstitui uma entidade persistida a partir de seus dados de armazenamento.
    ///
    /// Valida todas as invariantes de negócio, limites e coerência temporal entre
    /// `created_at` e `updated_at`.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso qualquer invariante seja violada.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        institution_id: Uuid,
        created_by_id: Option<Uuid>,
        name: String,
        acronym: Option<String>,
        description: String,
=======
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
>>>>>>> origin/pr/331
        status: AcademicCenterStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, AcademicCenterError> {
<<<<<<< HEAD
        let trimmed_name = name.trim();
        validate_name(trimmed_name)?;

        let trimmed_acronym = match acronym {
            Some(ref a) => {
                let t = a.trim();
                validate_acronym(t)?;
                Some(t.to_string())
            }
            None => None,
        };

        let trimmed_desc = description.trim();
        validate_description(trimmed_desc)?;

        validate_timestamps(created_at, updated_at)?;

        Ok(Self {
            id,
            institution_id,
            created_by_id,
            name: trimmed_name.to_string(),
            acronym: trimmed_acronym,
            description: trimmed_desc.to_string(),
            status,
            created_at,
            updated_at,
        })
    }

    /// Retorna o identificador estável do Centro Acadêmico.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador da instituição à qual o CA está vinculado.
    pub fn institution_id(&self) -> Uuid {
        self.institution_id
    }

    /// Retorna o identificador do usuário responsável pelo cadastro inicial, se houver.
    pub fn created_by_id(&self) -> Option<Uuid> {
        self.created_by_id
    }

    /// Retorna o nome oficial do Centro Acadêmico.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retorna a sigla oficial de apresentação do CA, se informada.
    pub fn acronym(&self) -> Option<&str> {
        self.acronym.as_deref()
    }

    /// Retorna a descrição e apresentação institucional do CA.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Retorna a situação cadastral e operacional do Centro Acadêmico.
    pub fn status(&self) -> AcademicCenterStatus {
        self.status
    }

    /// Retorna o instante de criação do registro no sistema.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Retorna o instante da última modificação do registro.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Valida todas as invariantes da entidade.
    pub fn validate(&self) -> Result<(), AcademicCenterError> {
        validate_academic_center(self)
    }

    /// Atualiza os dados básicos do Centro Acadêmico, validando as invariantes.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso nome, sigla, descrição ou timestamps violem os limites.
    pub fn update_details(
        &mut self,
        name: String,
        acronym: Option<String>,
        description: String,
        updated_at: DateTime<Utc>,
    ) -> Result<(), AcademicCenterError> {
        let trimmed_name = name.trim();
        validate_name(trimmed_name)?;

        let trimmed_acronym = match acronym {
            Some(ref a) => {
                let t = a.trim();
                validate_acronym(t)?;
                Some(t.to_string())
            }
            None => None,
        };

        let trimmed_desc = description.trim();
        validate_description(trimmed_desc)?;

        validate_timestamps(self.created_at, updated_at)?;

        self.name = trimmed_name.to_string();
        self.acronym = trimmed_acronym;
        self.description = trimmed_desc.to_string();
        self.updated_at = updated_at;
        Ok(())
    }

    /// Arquiva o Centro Acadêmico, retirando-o dos fluxos operacionais ativos.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso o CA já esteja arquivado ou o timestamp seja inconsistente.
    pub fn archive(&mut self, archived_at: DateTime<Utc>) -> Result<(), AcademicCenterError> {
        self.transition_to(AcademicCenterStatus::Archived, archived_at)
    }

    /// Realiza a transição de status validando a matriz de ciclo de vida e coerência temporal.
    ///
    /// # Erros
    ///
    /// Retorna [`AcademicCenterError`] caso a transição seja proibida ou o timestamp seja inconsistente.
    pub fn transition_to(
        &mut self,
        new_status: AcademicCenterStatus,
        updated_at: DateTime<Utc>,
    ) -> Result<(), AcademicCenterError> {
        validate_status_transition(self.status, new_status)?;
        validate_timestamps(self.created_at, updated_at)?;
        self.status = new_status;
        self.updated_at = updated_at;
=======
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
>>>>>>> origin/pr/331
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< HEAD
    use crate::modules::ca::domain::rules::{
        MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
    };

    const INSTITUTION_DIDACTIC_ID: Uuid =
        Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0000);
    const CAER_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);

    fn sample_valid_ca() -> AcademicCenter {
        let now = Utc::now();
        AcademicCenter::new(
            CAER_DIDACTIC_ID,
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico de Engenharia de Redes".to_string(),
            Some("CAER".to_string()),
            "Centro Acadêmico de Engenharia de Redes da Universidade de Brasília".to_string(),
            now,
        )
        .expect("should create valid academic center")
    }

    #[test]
    fn should_create_valid_academic_center_with_acronym() {
        let ca = sample_valid_ca();
        assert_eq!(ca.id(), CAER_DIDACTIC_ID);
        assert_eq!(ca.institution_id(), INSTITUTION_DIDACTIC_ID);
        assert_eq!(ca.created_by_id(), None);
        assert_eq!(ca.name(), "Centro Acadêmico de Engenharia de Redes");
        assert_eq!(ca.acronym(), Some("CAER"));
        assert_eq!(
            ca.description(),
            "Centro Acadêmico de Engenharia de Redes da Universidade de Brasília"
        );
        assert_eq!(ca.status(), AcademicCenterStatus::Active);
        assert_eq!(ca.created_at(), ca.updated_at());
        assert!(ca.validate().is_ok());
    }

    #[test]
    fn should_create_valid_academic_center_without_acronym() {
        let now = Utc::now();
        let ca = AcademicCenter::new(
            CACC_DIDACTIC_ID,
            INSTITUTION_DIDACTIC_ID,
            Some(Uuid::new_v4()),
            "Centro Acadêmico de Ciência da Computação".to_string(),
            None,
            "Descrição do CACC".to_string(),
            now,
        )
        .expect("should create valid academic center without acronym");

        assert_eq!(ca.acronym(), None);
        assert_eq!(ca.status(), AcademicCenterStatus::Active);
        assert!(ca.validate().is_ok());
    }

    #[test]
    fn should_trim_whitespace_on_creation() {
        let now = Utc::now();
        let ca = AcademicCenter::new(
            CACC_DIDACTIC_ID,
            INSTITUTION_DIDACTIC_ID,
            None,
            "  Centro Acadêmico de Ciência da Computação  ".to_string(),
            Some("  CACC  ".to_string()),
            "  Descrição do CACC  ".to_string(),
            now,
        )
        .expect("should create and trim fields");

        assert_eq!(ca.name(), "Centro Acadêmico de Ciência da Computação");
        assert_eq!(ca.acronym(), Some("CACC"));
        assert_eq!(ca.description(), "Descrição do CACC");
=======

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
>>>>>>> origin/pr/331
    }

    #[test]
    fn should_fail_when_name_is_empty_or_whitespace() {
<<<<<<< HEAD
        let now = Utc::now();
        let err_empty = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "".to_string(),
            Some("CAER".to_string()),
            "Desc".to_string(),
            now,
        )
        .unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyName);

        let err_whitespace = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "   ".to_string(),
            Some("CAER".to_string()),
            "Desc".to_string(),
            now,
        )
        .unwrap_err();
=======
        let err_empty = AcademicCenter::draft("", "CAER", "Desc").unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyName);

        let err_whitespace = AcademicCenter::draft("   ", "CAER", "Desc").unwrap_err();
>>>>>>> origin/pr/331
        assert_eq!(err_whitespace, AcademicCenterError::EmptyName);
    }

    #[test]
    fn should_fail_when_name_exceeds_max_length() {
<<<<<<< HEAD
        let now = Utc::now();
        let long_name = "A".repeat(MAX_NAME_LENGTH + 1);
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            long_name,
            Some("CAER".to_string()),
            "Desc".to_string(),
            now,
        )
        .unwrap_err();
=======
        let long_name = "A".repeat(MAX_NAME_LENGTH + 1);
        let err = AcademicCenter::draft(long_name, "CAER", "Desc").unwrap_err();
>>>>>>> origin/pr/331
        assert_eq!(
            err,
            AcademicCenterError::NameTooLong {
                max: MAX_NAME_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_acronym_is_empty_or_whitespace() {
<<<<<<< HEAD
        let now = Utc::now();
        let err_empty = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("".to_string()),
            "Desc".to_string(),
            now,
        )
        .unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyAcronym);

        let err_whitespace = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("   ".to_string()),
            "Desc".to_string(),
            now,
        )
        .unwrap_err();
=======
        let err_empty = AcademicCenter::draft("Centro Acadêmico", "", "Desc").unwrap_err();
        assert_eq!(err_empty, AcademicCenterError::EmptyAcronym);

        let err_whitespace = AcademicCenter::draft("Centro Acadêmico", "   ", "Desc").unwrap_err();
>>>>>>> origin/pr/331
        assert_eq!(err_whitespace, AcademicCenterError::EmptyAcronym);
    }

    #[test]
    fn should_fail_when_acronym_exceeds_max_length() {
<<<<<<< HEAD
        let now = Utc::now();
        let long_acronym = "A".repeat(MAX_ACRONYM_LENGTH + 1);
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some(long_acronym),
            "Desc".to_string(),
            now,
        )
        .unwrap_err();
=======
        let long_acronym = "A".repeat(MAX_ACRONYM_LENGTH + 1);
        let err = AcademicCenter::draft("Centro Acadêmico", long_acronym, "Desc").unwrap_err();
>>>>>>> origin/pr/331
        assert_eq!(
            err,
            AcademicCenterError::AcronymTooLong {
                max: MAX_ACRONYM_LENGTH
            }
        );
    }

    #[test]
    fn should_fail_when_acronym_contains_invalid_characters() {
<<<<<<< HEAD
        let now = Utc::now();
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("CA ER!".to_string()),
            "Desc".to_string(),
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
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("CA-REDES".to_string()),
            "Desc".to_string(),
            now,
        )
        .expect("should allow hyphen in acronym");
        assert_eq!(ca1.acronym(), Some("CA-REDES"));

        let ca2 = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("C.A.E.R".to_string()),
            "Desc".to_string(),
            now,
        )
        .expect("should allow dot in acronym");
        assert_eq!(ca2.acronym(), Some("C.A.E.R"));

        let ca3 = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("CA_COMP".to_string()),
            "Desc".to_string(),
            now,
        )
        .expect("should allow underscore in acronym");
        assert_eq!(ca3.acronym(), Some("CA_COMP"));
=======
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
>>>>>>> origin/pr/331
    }

    #[test]
    fn should_fail_when_description_exceeds_max_length() {
<<<<<<< HEAD
        let now = Utc::now();
        let long_desc = "D".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let err = AcademicCenter::new(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("CA".to_string()),
            long_desc,
            now,
        )
        .unwrap_err();
=======
        let long_desc = "D".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let err = AcademicCenter::draft("Centro Acadêmico", "CA", long_desc).unwrap_err();
>>>>>>> origin/pr/331
        assert_eq!(
            err,
            AcademicCenterError::DescriptionTooLong {
                max: MAX_DESCRIPTION_LENGTH
            }
        );
    }

    #[test]
<<<<<<< HEAD
    fn should_reconstitute_persisted_academic_center() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(24);
        let ca = AcademicCenter::reconstitute(
            CAER_DIDACTIC_ID,
            INSTITUTION_DIDACTIC_ID,
            Some(Uuid::new_v4()),
            "Centro Acadêmico de Engenharia de Redes".to_string(),
            Some("CAER".to_string()),
            "Descrição histórica".to_string(),
            AcademicCenterStatus::Archived,
            past,
            now,
        )
        .expect("should reconstitute academic center");

        assert_eq!(ca.status(), AcademicCenterStatus::Archived);
        assert_eq!(ca.created_at(), past);
        assert_eq!(ca.updated_at(), now);
    }

    #[test]
    fn should_fail_reconstitution_when_updated_at_before_created_at() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        let err = AcademicCenter::reconstitute(
            Uuid::new_v4(),
            INSTITUTION_DIDACTIC_ID,
            None,
            "Centro Acadêmico".to_string(),
            Some("CA".to_string()),
            "Desc".to_string(),
            AcademicCenterStatus::Active,
=======
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
>>>>>>> origin/pr/331
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
<<<<<<< HEAD
    fn should_update_details_and_validate_invariants() {
        let mut ca = sample_valid_ca();
        let now = Utc::now() + chrono::Duration::minutes(5);

        ca.update_details(
            "Novo Nome Oficial do CA".to_string(),
            Some("NOVO".to_string()),
            "Nova descrição".to_string(),
            now,
        )
        .expect("should update details");

        assert_eq!(ca.name(), "Novo Nome Oficial do CA");
        assert_eq!(ca.acronym(), Some("NOVO"));
        assert_eq!(ca.description(), "Nova descrição");
        assert_eq!(ca.updated_at(), now);

        // Atualização inválida (nome vazio)
        let err = ca.update_details("".to_string(), None, "".to_string(), now);
        assert_eq!(err.unwrap_err(), AcademicCenterError::EmptyName);
    }

    #[test]
    fn should_archive_academic_center_and_prevent_unarchiving() {
        let mut ca = sample_valid_ca();
        let archive_time = Utc::now() + chrono::Duration::minutes(10);

        ca.archive(archive_time).expect("should archive ca");
        assert_eq!(ca.status(), AcademicCenterStatus::Archived);
        assert_eq!(ca.updated_at(), archive_time);

        // Tentar reverter arquivamento deve falhar
        let later = archive_time + chrono::Duration::minutes(5);
        let err = ca.transition_to(AcademicCenterStatus::Active, later);
        assert_eq!(
            err.unwrap_err(),
=======
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
>>>>>>> origin/pr/331
            AcademicCenterError::InvalidStatusTransition {
                from: AcademicCenterStatus::Archived,
                to: AcademicCenterStatus::Active,
            }
        );
    }

    #[test]
    fn should_parse_status_from_string() {
        assert_eq!(
<<<<<<< HEAD
=======
            "draft".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Draft
        );
        assert_eq!(
>>>>>>> origin/pr/331
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
<<<<<<< HEAD
        assert_eq!(
            "  Active  ".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Active
        );

        let err = "draft".parse::<AcademicCenterStatus>().unwrap_err();
        assert_eq!(err, AcademicCenterError::InvalidStatus("draft".to_string()));

        let err_unknown = "desconhecido".parse::<AcademicCenterStatus>().unwrap_err();
        assert_eq!(
            err_unknown,
=======

        let err = "desconhecido".parse::<AcademicCenterStatus>().unwrap_err();
        assert_eq!(
            err,
>>>>>>> origin/pr/331
            AcademicCenterError::InvalidStatus("desconhecido".to_string())
        );
    }

    #[test]
    fn should_format_status_as_expected() {
<<<<<<< HEAD
        assert_eq!(AcademicCenterStatus::Active.as_str(), "active");
        assert_eq!(AcademicCenterStatus::Archived.as_str(), "archived");

=======
        assert_eq!(AcademicCenterStatus::Draft.to_string(), "draft");
>>>>>>> origin/pr/331
        assert_eq!(AcademicCenterStatus::Active.to_string(), "active");
        assert_eq!(AcademicCenterStatus::Archived.to_string(), "archived");
    }

    #[test]
<<<<<<< HEAD
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
=======
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
>>>>>>> origin/pr/331
    }
}
