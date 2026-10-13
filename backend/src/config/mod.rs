//! Centralização do carregamento, validação e mascaramento de configurações e segredos.
//!
//! Este módulo fornece tipos estruturados para carregar variáveis de ambiente com
//! validação em tempo de inicialização (*fail-fast*), garantindo diagnóstico seguro
//! em caso de ausência ou configuração incorreta, sem vazar valores sensíveis em
//! mensagens de erro, logs ou saídas de depuração.
//!
//! # Invariantes de Segurança
//!
//! - **Sem vazamento de segredos:** Valores sensíveis são armazenados em [`SecretString`],
//!   cujas implementações de [`std::fmt::Debug`] e [`std::fmt::Display`] produzem `[REDACTED]`.
//! - **Diagnóstico seguro:** Se uma credencial obrigatória estiver ausente ou possuir
//!   formato inválido, o erro reporta apenas o nome da variável e o motivo estrutural,
//!   nunca ecoando o valor do segredo.
//! - **Isolamento de ambiente:** A aplicação identifica se está em desenvolvimento,
//!   homologação ou produção via [`Environment`].

use std::env::VarError;
use std::fmt;
use std::str::FromStr;

/// Envoltório seguro para sequências de caracteres sensíveis (senhas, chaves, URLs).
///
/// Impede que o conteúdo do segredo seja acidentalmente registrado em logs ou
/// exposto por formatadores padrão de depuração. Para acessar o conteúdo real,
/// é necessário invocar explicitamente [`SecretString::expose_secret`].
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    /// Cria um novo segredo a partir de uma sequência de texto.
    ///
    /// # Parâmetros
    ///
    /// - `secret`: valor confidencial a ser protegido.
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// Revela explicitamente a sequência confidencial interna.
    ///
    /// Deve ser utilizada apenas nos pontos estritamente necessários para
    /// conexões criptografadas ou provedores de infraestrutura (por exemplo, SQLx).
    ///
    /// # Retorno
    ///
    /// Referência de fatia de texto com o segredo puro.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString([REDACTED])")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Identificação do ambiente de execução do sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    /// Ambiente de desenvolvimento local (padrão).
    #[default]
    Development,
    /// Ambiente de homologação e testes de integração.
    Staging,
    /// Ambiente de produção com isolamento e criptografia estritos.
    Production,
}

impl Environment {
    /// Indica se o ambiente atual corresponde à produção.
    ///
    /// # Retorno
    ///
    /// `true` se for [`Environment::Production`], `false` caso contrário.
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Retorna a representação textual canônica do ambiente.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "dev" | "development" | "desenvolvimento" => Ok(Self::Development),
            "stage" | "staging" | "homolog" | "homologacao" | "homologação" => Ok(Self::Staging),
            "prod" | "production" | "producao" | "produção" => Ok(Self::Production),
            _ => Err(ConfigError::InvalidValue {
                name: "APP_ENV",
                reason: "ambiente desconhecido (esperado: development, staging ou production)"
                    .to_string(),
            }),
        }
    }
}

/// Erros decorrentes do carregamento e validação de configurações e segredos.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    /// Variável de ambiente obrigatória não encontrada ou vazia.
    #[error("A variável de ambiente obrigatória '{0}' não foi definida")]
    MissingRequiredVariable(&'static str),

    /// Variável de ambiente contém sequência de bytes não decodificável como UTF-8.
    #[error("A variável de ambiente '{0}' contém caracteres UTF-8 inválidos")]
    InvalidUtf8(&'static str),

    /// Variável de ambiente possui valor inválido para o tipo esperado.
    #[error("A variável de ambiente '{name}' contém um valor inválido: {reason}")]
    InvalidValue {
        /// Nome da variável de ambiente com erro.
        name: &'static str,
        /// Descrição sanitizada do erro (sem expor segredos).
        reason: String,
    },

    /// Segredo possui formato inválido, com diagnóstico protegido sem ecoar dados sensíveis.
    #[error("A variável de ambiente '{name}' contém formato inválido: {message}")]
    InvalidSecretFormat {
        /// Nome da variável sensível.
        name: &'static str,
        /// Diagnóstico de validação sanitizado.
        message: &'static str,
    },
}

/// Configuração unificada do backend CApp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// URL de conexão confidencial ao banco PostgreSQL.
    database_url: SecretString,
    /// Filtro de log e telemetria para o tracing.
    rust_log: String,
    /// Endereço IP ou hostname do listener HTTP.
    server_host: String,
    /// Porta TCP em que o listener HTTP aceitará conexões.
    server_port: u16,
    /// Ambiente de execução da aplicação.
    environment: Environment,
}

impl AppConfig {
    /// Carrega as configurações a partir das variáveis de ambiente do processo.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError`] se `DATABASE_URL` estiver ausente, malformada ou se
    /// outra variável obrigatória contiver valores inválidos.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|k| std::env::var(k))
    }

    /// Carrega as configurações utilizando uma função customizada de consulta de variáveis.
    ///
    /// Permite testar deterministicamente cenários de ausência de variáveis ou
    /// formatos incorretos sem alterar o ambiente global do processo.
    ///
    /// # Parâmetros
    ///
    /// - `lookup`: função que recebe o nome da variável e retorna o valor ou erro.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError`] caso a validação falhe. Mensagens de erro nunca expõem
    /// o valor de segredos fornecidos.
    pub fn from_lookup<F>(mut lookup: F) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, VarError>,
    {
        let raw_db_url = match lookup("DATABASE_URL") {
            Ok(val) => {
                let trimmed = val.trim();
                if trimmed.is_empty() {
                    return Err(ConfigError::MissingRequiredVariable("DATABASE_URL"));
                }
                trimmed.to_string()
            }
            Err(VarError::NotPresent) => {
                return Err(ConfigError::MissingRequiredVariable("DATABASE_URL"));
            }
            Err(VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUtf8("DATABASE_URL"));
            }
        };

        if !raw_db_url.starts_with("postgres://") && !raw_db_url.starts_with("postgresql://") {
            return Err(ConfigError::InvalidSecretFormat {
                name: "DATABASE_URL",
                message: "deve iniciar com o esquema 'postgres://' ou 'postgresql://'",
            });
        }

        let database_url = SecretString::new(raw_db_url);

        let rust_log = match lookup("RUST_LOG") {
            Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
            _ => "info".to_string(),
        };

        let server_host = match lookup("SERVER_HOST") {
            Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
            _ => "0.0.0.0".to_string(),
        };

        let port_candidate = match lookup("BACKEND_PORT") {
            Ok(v) if !v.trim().is_empty() => Some(("BACKEND_PORT", v)),
            _ => match lookup("SERVER_PORT") {
                Ok(v) if !v.trim().is_empty() => Some(("SERVER_PORT", v)),
                _ => None,
            },
        };

        let server_port = match port_candidate {
            Some((var_name, val)) => {
                val.trim()
                    .parse::<u16>()
                    .map_err(|_| ConfigError::InvalidValue {
                        name: var_name,
                        reason: "deve ser um número inteiro positivo entre 1 e 65535".to_string(),
                    })?
            }
            None => 3000,
        };

        let env_candidate = match lookup("APP_ENV") {
            Ok(v) if !v.trim().is_empty() => Some(("APP_ENV", v)),
            _ => match lookup("ENVIRONMENT") {
                Ok(v) if !v.trim().is_empty() => Some(("ENVIRONMENT", v)),
                _ => None,
            },
        };

        let environment = match env_candidate {
            Some((_, val)) => val.parse::<Environment>()?,
            None => Environment::Development,
        };

        Ok(Self {
            database_url,
            rust_log,
            server_host,
            server_port,
            environment,
        })
    }

    /// Retorna a URL de conexão confidencial ao banco de dados.
    pub fn database_url(&self) -> &SecretString {
        &self.database_url
    }

    /// Retorna o filtro de log configurado.
    pub fn rust_log(&self) -> &str {
        &self.rust_log
    }

    /// Retorna o endereço de host configurado para o listener HTTP.
    pub fn server_host(&self) -> &str {
        &self.server_host
    }

    /// Retorna a porta TCP configurada para o listener HTTP.
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    /// Retorna o ambiente de execução configurado.
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_lookup(
        vars: HashMap<&'static str, String>,
    ) -> impl FnMut(&str) -> Result<String, VarError> {
        move |key: &str| vars.get(key).cloned().ok_or(VarError::NotPresent)
    }

    #[test]
    fn test_secret_string_masks_debug_and_display() {
        let sensitive = "postgres://usr:senhasecreta123@db.capp.local:5432/capp_prod";
        let secret = SecretString::new(sensitive);

        let debug_output = format!("{:?}", secret);
        let display_output = format!("{}", secret);

        assert_eq!(debug_output, "SecretString([REDACTED])");
        assert_eq!(display_output, "[REDACTED]");
        assert!(!debug_output.contains("senhasecreta123"));
        assert!(!display_output.contains("senhasecreta123"));

        assert_eq!(secret.expose_secret(), sensitive);
    }

    #[test]
    fn test_secret_string_inside_struct_debug() {
        let sensitive = "postgres://usr:senhasecreta123@db.capp.local:5432/capp_prod";
        let config = AppConfig {
            database_url: SecretString::new(sensitive),
            rust_log: "info".to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 3000,
            environment: Environment::Production,
        };

        let debug_output = format!("{:?}", config);
        assert!(!debug_output.contains("senhasecreta123"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_missing_database_url_produces_safe_diagnosis() {
        let vars = HashMap::new();
        let result = AppConfig::from_lookup(make_lookup(vars));

        assert_eq!(
            result,
            Err(ConfigError::MissingRequiredVariable("DATABASE_URL"))
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("DATABASE_URL"));
        assert!(err_msg.contains("obrigatória"));
    }

    #[test]
    fn test_empty_database_url_produces_safe_diagnosis() {
        let mut vars = HashMap::new();
        vars.insert("DATABASE_URL", "   ".to_string());

        let result = AppConfig::from_lookup(make_lookup(vars));

        assert_eq!(
            result,
            Err(ConfigError::MissingRequiredVariable("DATABASE_URL"))
        );
    }

    #[test]
    fn test_invalid_utf8_database_url_produces_safe_diagnosis() {
        let result = AppConfig::from_lookup(|key| {
            if key == "DATABASE_URL" {
                Err(VarError::NotUnicode(std::ffi::OsString::from(
                    "invalid_utf8",
                )))
            } else {
                Err(VarError::NotPresent)
            }
        });

        assert_eq!(result, Err(ConfigError::InvalidUtf8("DATABASE_URL")));
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("DATABASE_URL"));
        assert!(err_msg.contains("UTF-8 inválidos"));
        assert!(!err_msg.contains("invalid_utf8"));
    }

    #[test]
    fn test_invalid_scheme_database_url_does_not_leak_credentials() {
        let sensitive_url = "mysql://admin:senha_super_secreta_999@10.0.0.1:3306/db";
        let mut vars = HashMap::new();
        vars.insert("DATABASE_URL", sensitive_url.to_string());

        let result = AppConfig::from_lookup(make_lookup(vars));

        match &result {
            Err(ConfigError::InvalidSecretFormat { name, message }) => {
                assert_eq!(*name, "DATABASE_URL");
                assert!(message.contains("postgres://"));
            }
            _ => panic!("Esperava erro de formato inválido para esquema incorreto"),
        }

        let err_msg = result.unwrap_err().to_string();
        assert!(!err_msg.contains("senha_super_secreta_999"));
        assert!(!err_msg.contains("admin"));
        assert!(!err_msg.contains("mysql"));
    }

    #[test]
    fn test_valid_minimal_config_applies_safe_defaults() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );

        let config = AppConfig::from_lookup(make_lookup(vars)).expect("Configuração válida");

        assert_eq!(
            config.database_url().expose_secret(),
            "postgres://capp:capp@localhost:5432/capp"
        );
        assert_eq!(config.server_port(), 3000);
        assert_eq!(config.server_host(), "0.0.0.0");
        assert_eq!(config.rust_log(), "info");
        assert_eq!(config.environment(), &Environment::Development);
        assert!(!config.environment().is_production());
    }

    #[test]
    fn test_valid_custom_config_with_staging_and_ports() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgresql://capp_staging:secret@postgres:5432/capp_stg".to_string(),
        );
        vars.insert("APP_ENV", "homologacao".to_string());
        vars.insert("BACKEND_PORT", "8080".to_string());
        vars.insert("SERVER_HOST", "127.0.0.1".to_string());
        vars.insert("RUST_LOG", "debug".to_string());

        let config = AppConfig::from_lookup(make_lookup(vars)).expect("Configuração válida");

        assert_eq!(config.server_port(), 8080);
        assert_eq!(config.server_host(), "127.0.0.1");
        assert_eq!(config.rust_log(), "debug");
        assert_eq!(config.environment(), &Environment::Staging);
        assert!(!config.environment().is_production());
    }

    #[test]
    fn test_production_environment_flag() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgres://capp_prod:secret@db:5432/capp_prod".to_string(),
        );
        vars.insert("APP_ENV", "production".to_string());

        let config = AppConfig::from_lookup(make_lookup(vars)).expect("Configuração válida");

        assert_eq!(config.environment(), &Environment::Production);
        assert!(config.environment().is_production());
    }

    #[test]
    fn test_invalid_port_produces_safe_diagnosis() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );
        vars.insert("BACKEND_PORT", "porta_invalida".to_string());

        let result = AppConfig::from_lookup(make_lookup(vars));

        assert_eq!(
            result,
            Err(ConfigError::InvalidValue {
                name: "BACKEND_PORT",
                reason: "deve ser um número inteiro positivo entre 1 e 65535".to_string(),
            })
        );
    }

    #[test]
    fn test_invalid_environment_produces_safe_diagnosis() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );
        vars.insert("APP_ENV", "metaverso".to_string());

        let result = AppConfig::from_lookup(make_lookup(vars));

        assert_eq!(
            result,
            Err(ConfigError::InvalidValue {
                name: "APP_ENV",
                reason: "ambiente desconhecido (esperado: development, staging ou production)"
                    .to_string(),
            })
        );
    }
}
