//! Entidade de domínio representando o Usuário da aplicação (`app_user`).
//!
//! O usuário representa a conta interna no ecossistema CApp, provisionada
//! a partir da autenticação institucional.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Identificador estável do usuário veterano principal para cenários didáticos.
pub const VETERANO_1_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

/// Identificador estável do segundo usuário veterano para cenários didáticos.
pub const VETERANO_2_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0012);

/// Identificador estável do usuário calouro para cenários didáticos.
pub const CALOURO_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0013);

/// Matrícula estritamente proibida em cenários e cadastros do CApp.
pub const PROHIBITED_MATRICULA: &str = "242003979";

/// Limite máximo em caracteres para o nome de exibição do usuário.
pub const MAX_DISPLAY_NAME_LENGTH: usize = 255;

/// Limite máximo em caracteres para o e-mail institucional.
pub const MAX_EMAIL_LENGTH: usize = 255;

/// Erros decorrentes de violações de validação ou regras de usuário.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppUserError {
    /// O nome de exibição não pode ser vazio.
    #[error("o nome de exibição do usuário não pode ser vazio")]
    EmptyDisplayName,

    /// O nome de exibição excede o limite máximo permitido.
    #[error("o nome de exibição excede o limite de {max} caracteres")]
    DisplayNameTooLong {
        /// Limite máximo.
        max: usize,
    },

    /// O e-mail institucional não pode ser vazio.
    #[error("o e-mail institucional não pode ser vazio")]
    EmptyInstitutionalEmail,

    /// O e-mail institucional excede o limite máximo permitido.
    #[error("o e-mail institucional excede o limite de {max} caracteres")]
    EmailTooLong {
        /// Limite máximo.
        max: usize,
    },

    /// O e-mail não atende ao formato discente institucional esperado.
    #[error("o e-mail institucional '{0}' não possui formato válido")]
    InvalidEmailFormat(String),

    /// O cadastro utilizou a matrícula restrita por diretriz de domínio.
    #[error("o uso da matrícula restrita '{0}' é estritamente proibido")]
    ProhibitedRegistration(String),

    /// O status informado não é reconhecido pelo domínio.
    #[error("status de usuário inválido: '{0}'")]
    InvalidStatus(String),

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

/// Estado cadastral do usuário no CApp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppUserStatus {
    /// Conta habilitada e apta para operações.
    #[default]
    Active,
    /// Conta desativada.
    Inactive,
}

impl AppUserStatus {
    /// Retorna a representação textual canônica do status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl fmt::Display for AppUserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AppUserStatus {
    type Err = AppUserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            other => Err(AppUserError::InvalidStatus(other.to_string())),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for AppUserStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AppUserStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let text = <&'r str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        text.parse().map_err(|err: AppUserError| Box::new(err) as _)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AppUserStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + 'static + Send + Sync>> {
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

/// Entidade de domínio que representa o usuário da aplicação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct AppUser {
    /// Identificador estável do usuário.
    pub id: Uuid,
    /// Nome de exibição na interface do aplicativo.
    pub display_name: String,
    /// E-mail institucional conhecido do estudante.
    pub institutional_email: String,
    /// Situação cadastral do usuário.
    pub status: AppUserStatus,
    /// Instante em que o registro foi criado no sistema.
    pub created_at: DateTime<Utc>,
    /// Instante da última modificação do registro.
    pub updated_at: DateTime<Utc>,
}

impl AppUser {
    /// Cria uma nova instância de usuário validando suas invariantes de domínio.
    ///
    /// # Erros
    ///
    /// Retorna [`AppUserError`] caso nome, e-mail ou timestamps sejam inválidos
    /// ou contenham matrícula restrita.
    pub fn new(
        id: Uuid,
        display_name: impl Into<String>,
        institutional_email: impl Into<String>,
        status: AppUserStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, AppUserError> {
        let instance = Self {
            id,
            display_name: display_name.into().trim().to_string(),
            institutional_email: institutional_email.into().trim().to_lowercase(),
            status,
            created_at,
            updated_at,
        };
        instance.validate()?;
        Ok(instance)
    }

    /// Cria uma instância do estudante veterano didático da UnB.
    pub fn veterano_didactic() -> Self {
        let reference_time = DateTime::from_timestamp(1_767_225_600, 0).unwrap_or_default();
        Self {
            id: VETERANO_1_DIDACTIC_ID,
            display_name: "Estudante Veterano UnB".to_string(),
            institutional_email: "231012345@aluno.unb.br".to_string(),
            status: AppUserStatus::Active,
            created_at: reference_time,
            updated_at: reference_time,
        }
    }

    /// Cria uma instância do estudante calouro didático da UnB.
    pub fn calouro_didactic() -> Self {
        let reference_time = DateTime::from_timestamp(1_767_225_600, 0).unwrap_or_default();
        Self {
            id: CALOURO_DIDACTIC_ID,
            display_name: "Estudante Calouro UnB".to_string(),
            institutional_email: "262001234@aluno.unb.br".to_string(),
            status: AppUserStatus::Active,
            created_at: reference_time,
            updated_at: reference_time,
        }
    }

    /// Valida as invariantes de negócio da entidade de usuário.
    pub fn validate(&self) -> Result<(), AppUserError> {
        if self.display_name.is_empty() {
            return Err(AppUserError::EmptyDisplayName);
        }
        if self.display_name.chars().count() > MAX_DISPLAY_NAME_LENGTH {
            return Err(AppUserError::DisplayNameTooLong {
                max: MAX_DISPLAY_NAME_LENGTH,
            });
        }

        if self.institutional_email.is_empty() {
            return Err(AppUserError::EmptyInstitutionalEmail);
        }
        if self.institutional_email.chars().count() > MAX_EMAIL_LENGTH {
            return Err(AppUserError::EmailTooLong {
                max: MAX_EMAIL_LENGTH,
            });
        }

        if self.institutional_email.contains(PROHIBITED_MATRICULA) {
            return Err(AppUserError::ProhibitedRegistration(
                PROHIBITED_MATRICULA.to_string(),
            ));
        }

        if !self.institutional_email.contains('@') {
            return Err(AppUserError::InvalidEmailFormat(
                self.institutional_email.clone(),
            ));
        }

        if self.updated_at < self.created_at {
            return Err(AppUserError::InconsistentTimestamps {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_user() {
        let now = Utc::now();
        let user = AppUser::new(
            Uuid::new_v4(),
            "Maria Silva",
            "231012345@aluno.unb.br",
            AppUserStatus::Active,
            now,
            now,
        )
        .expect("should create valid user");

        assert_eq!(user.display_name, "Maria Silva");
        assert_eq!(user.institutional_email, "231012345@aluno.unb.br");
        assert_eq!(user.status, AppUserStatus::Active);
    }

    #[test]
    fn should_instantiate_didactic_users_correctly() {
        let veterano = AppUser::veterano_didactic();
        assert_eq!(veterano.id, VETERANO_1_DIDACTIC_ID);
        assert_eq!(veterano.institutional_email, "231012345@aluno.unb.br");
        veterano
            .validate()
            .expect("veterano didactic must be valid");

        let calouro = AppUser::calouro_didactic();
        assert_eq!(calouro.id, CALOURO_DIDACTIC_ID);
        assert_eq!(calouro.institutional_email, "262001234@aluno.unb.br");
        calouro.validate().expect("calouro didactic must be valid");
    }

    #[test]
    fn should_reject_prohibited_registration() {
        let now = Utc::now();
        let email = format!("{PROHIBITED_MATRICULA}@aluno.unb.br");
        let err = AppUser::new(
            Uuid::new_v4(),
            "Usuario Proibido",
            email,
            AppUserStatus::Active,
            now,
            now,
        )
        .unwrap_err();

        assert_eq!(
            err,
            AppUserError::ProhibitedRegistration(PROHIBITED_MATRICULA.to_string())
        );
    }

    #[test]
    fn should_reject_empty_display_name() {
        let now = Utc::now();
        let err = AppUser::new(
            Uuid::new_v4(),
            "   ",
            "231012345@aluno.unb.br",
            AppUserStatus::Active,
            now,
            now,
        )
        .unwrap_err();

        assert_eq!(err, AppUserError::EmptyDisplayName);
    }

    #[test]
    fn should_parse_status() {
        assert_eq!(
            "active".parse::<AppUserStatus>().unwrap(),
            AppUserStatus::Active
        );
        assert_eq!(
            "inactive".parse::<AppUserStatus>().unwrap(),
            AppUserStatus::Inactive
        );
    }
}
