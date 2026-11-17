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

/// Parâmetros de consulta permitidos para URLs de conexão PostgreSQL.
const ALLOWED_DATABASE_QUERY_PARAMS: &[&str] = &[
    "sslmode",
    "sslrootcert",
    "sslcert",
    "sslkey",
    "application_name",
    "statement_cache_capacity",
    "channel_binding",
    "connect_timeout",
    "options",
    "search_path",
];

/// Modos de SSL aceitos pelo driver PostgreSQL.
const VALID_SSL_MODES: &[&str] = &[
    "disable",
    "allow",
    "prefer",
    "require",
    "verify-ca",
    "verify-full",
];

/// Valida a estrutura da URL confidencial de conexão PostgreSQL.
///
/// Garante que:
/// - O esquema seja estritamente `postgres://` ou `postgresql://`.
/// - A estrutura geral da URL seja válida e contenha host.
/// - Parâmetros de consulta pertençam exclusivamente à lista de parâmetros permitidos,
///   rejeitando parâmetros desconhecidos ou malformados sem ecoar nomes ou valores de parâmetros.
/// - O valor de `sslmode`, se informado, seja um dos modos válidos do PostgreSQL, sem ecoar o valor informado.
/// - Quando em ambiente de produção (`is_production == true`), exige explicitamente `sslmode=verify-full`.
fn validate_database_url(raw_url: &str, is_production: bool) -> Result<(), ConfigError> {
    if !raw_url.starts_with("postgres://") && !raw_url.starts_with("postgresql://") {
        return Err(ConfigError::InvalidSecretFormat {
            name: "DATABASE_URL",
            message: "deve iniciar com o esquema 'postgres://' ou 'postgresql://'",
        });
    }

    let parsed = url::Url::parse(raw_url).map_err(|_| ConfigError::InvalidSecretFormat {
        name: "DATABASE_URL",
        message: "estrutura da URL de conexão inválida ou malformada",
    })?;

    let host = parsed.host_str().unwrap_or("").trim();
    if host.is_empty() {
        return Err(ConfigError::InvalidSecretFormat {
            name: "DATABASE_URL",
            message: "a URL de conexão deve conter um host válido",
        });
    }

    let mut found_sslmode: Option<String> = None;

    if let Some(query) = parsed.query().filter(|q| !q.is_empty()) {
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            if key.is_empty() || !ALLOWED_DATABASE_QUERY_PARAMS.contains(&key) {
                return Err(ConfigError::InvalidSecretFormat {
                    name: "DATABASE_URL",
                    message: "a URL de conexão contém parâmetros de consulta não permitidos ou desconhecidos",
                });
            }

            if key == "sslmode" {
                if !VALID_SSL_MODES.contains(&value) {
                    return Err(ConfigError::InvalidSecretFormat {
                        name: "DATABASE_URL",
                        message: "o parâmetro 'sslmode' na DATABASE_URL contém valor inválido (esperado: disable, allow, prefer, require, verify-ca ou verify-full)",
                    });
                }
                found_sslmode = Some(value.to_string());
            }
        }
    }

    if is_production && found_sslmode.as_deref() != Some("verify-full") {
        return Err(ConfigError::InvalidSecretFormat {
            name: "DATABASE_URL",
            message: "o ambiente de produção ('production') exige que a DATABASE_URL configure explicitamente 'sslmode=verify-full'",
        });
    }

    Ok(())
}

/// Sanitiza diagnósticos e erros de conexão para evitar vazamento de credenciais nos logs ou mensagens.
pub fn sanitize_connection_error(err: &sqlx::Error) -> String {
    match err {
        sqlx::Error::Configuration(_) => {
            "erro de configuração nos parâmetros de conexão".to_string()
        }
        sqlx::Error::Database(db_err) => {
            format!(
                "erro retornado pelo banco de dados (código: {:?}): {}",
                db_err.code(),
                db_err.message()
            )
        }
        sqlx::Error::Io(io_err) => {
            format!("erro de rede/E-S ao conectar ao banco de dados: {}", io_err)
        }
        sqlx::Error::Tls(tls_err) => {
            format!("erro de negociação TLS com o banco de dados: {}", tls_err)
        }
        _ => "falha de conexão com o banco de dados".to_string(),
    }
}

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
        let env_candidate = match lookup("APP_ENV") {
            Ok(v) if !v.trim().is_empty() => Some(("APP_ENV", v)),
            Ok(_) | Err(VarError::NotPresent) => match lookup("ENVIRONMENT") {
                Ok(v) if !v.trim().is_empty() => Some(("ENVIRONMENT", v)),
                Ok(_) | Err(VarError::NotPresent) => None,
                Err(VarError::NotUnicode(_)) => {
                    return Err(ConfigError::InvalidUtf8("ENVIRONMENT"));
                }
            },
            Err(VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUtf8("APP_ENV"));
            }
        };

        let environment = match env_candidate {
            Some((_, val)) => val.parse::<Environment>()?,
            None => Environment::Development,
        };

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

        validate_database_url(&raw_db_url, environment.is_production())?;
        let database_url = SecretString::new(raw_db_url);

        let rust_log = match lookup("RUST_LOG") {
            Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
            Ok(_) | Err(VarError::NotPresent) => "info".to_string(),
            Err(VarError::NotUnicode(_)) => return Err(ConfigError::InvalidUtf8("RUST_LOG")),
        };

        let server_host = match lookup("SERVER_HOST") {
            Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
            Ok(_) | Err(VarError::NotPresent) => "0.0.0.0".to_string(),
            Err(VarError::NotUnicode(_)) => return Err(ConfigError::InvalidUtf8("SERVER_HOST")),
        };

        let port_candidate = match lookup("BACKEND_PORT") {
            Ok(v) if !v.trim().is_empty() => Some(("BACKEND_PORT", v)),
            Ok(_) | Err(VarError::NotPresent) => match lookup("SERVER_PORT") {
                Ok(v) if !v.trim().is_empty() => Some(("SERVER_PORT", v)),
                Ok(_) | Err(VarError::NotPresent) => None,
                Err(VarError::NotUnicode(_)) => {
                    return Err(ConfigError::InvalidUtf8("SERVER_PORT"));
                }
            },
            Err(VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUtf8("BACKEND_PORT"));
            }
        };

        let server_port = match port_candidate {
            Some((var_name, val)) => {
                let parsed = val
                    .trim()
                    .parse::<u16>()
                    .map_err(|_| ConfigError::InvalidValue {
                        name: var_name,
                        reason: "deve ser um número inteiro positivo entre 1 e 65535".to_string(),
                    })?;
                if parsed == 0 {
                    return Err(ConfigError::InvalidValue {
                        name: var_name,
                        reason: "deve ser um número inteiro positivo entre 1 e 65535 (porta 0 é inválida)".to_string(),
                    });
                }
                parsed
            }
            None => 3000,
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
            "postgres://capp_prod:secret@db:5432/capp_prod?sslmode=verify-full".to_string(),
        );
        vars.insert("APP_ENV", "production".to_string());

        let config = AppConfig::from_lookup(make_lookup(vars)).expect("Configuração válida");

        assert_eq!(config.environment(), &Environment::Production);
        assert!(config.environment().is_production());
    }

    #[test]
    fn test_production_rejects_missing_or_inadequate_sslmode() {
        let modes = [
            None,
            Some("disable"),
            Some("allow"),
            Some("prefer"),
            Some("require"),
            Some("verify-ca"),
        ];

        for mode in modes {
            let mut vars = HashMap::new();
            let url = match mode {
                Some(m) => format!("postgres://capp_prod:secret@db:5432/capp_prod?sslmode={m}"),
                None => "postgres://capp_prod:secret@db:5432/capp_prod".to_string(),
            };
            vars.insert("DATABASE_URL", url);
            vars.insert("APP_ENV", "production".to_string());

            let result = AppConfig::from_lookup(make_lookup(vars));
            assert_eq!(
                result,
                Err(ConfigError::InvalidSecretFormat {
                    name: "DATABASE_URL",
                    message: "o ambiente de produção ('production') exige que a DATABASE_URL configure explicitamente 'sslmode=verify-full'",
                })
            );
        }

        // Aceite explícito de verify-full em produção
        let mut valid_vars = HashMap::new();
        valid_vars.insert(
            "DATABASE_URL",
            "postgres://capp_prod:secret@db:5432/capp_prod?sslmode=verify-full".to_string(),
        );
        valid_vars.insert("APP_ENV", "production".to_string());
        assert!(AppConfig::from_lookup(make_lookup(valid_vars)).is_ok());
    }

    #[test]
    fn test_port_boundaries() {
        // Porta 0 rejeitada explicitamente
        let mut vars_zero = HashMap::new();
        vars_zero.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );
        vars_zero.insert("BACKEND_PORT", "0".to_string());
        assert_eq!(
            AppConfig::from_lookup(make_lookup(vars_zero)),
            Err(ConfigError::InvalidValue {
                name: "BACKEND_PORT",
                reason: "deve ser um número inteiro positivo entre 1 e 65535 (porta 0 é inválida)"
                    .to_string(),
            })
        );

        // Limite inferior válido: 1
        let mut vars_one = HashMap::new();
        vars_one.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );
        vars_one.insert("BACKEND_PORT", "1".to_string());
        let config_one = AppConfig::from_lookup(make_lookup(vars_one)).expect("Porta 1 é válida");
        assert_eq!(config_one.server_port(), 1);

        // Limite superior válido: 65535
        let mut vars_max = HashMap::new();
        vars_max.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );
        vars_max.insert("BACKEND_PORT", "65535".to_string());
        let config_max =
            AppConfig::from_lookup(make_lookup(vars_max)).expect("Porta 65535 é válida");
        assert_eq!(config_max.server_port(), 65535);

        // Limite estourado: 65536
        let mut vars_overflow = HashMap::new();
        vars_overflow.insert(
            "DATABASE_URL",
            "postgres://capp:capp@localhost:5432/capp".to_string(),
        );
        vars_overflow.insert("BACKEND_PORT", "65536".to_string());
        assert_eq!(
            AppConfig::from_lookup(make_lookup(vars_overflow)),
            Err(ConfigError::InvalidValue {
                name: "BACKEND_PORT",
                reason: "deve ser um número inteiro positivo entre 1 e 65535".to_string(),
            })
        );
    }

    #[test]
    fn test_app_env_not_unicode_fails_explicitly() {
        let result = AppConfig::from_lookup(|key| {
            if key == "APP_ENV" {
                Err(VarError::NotUnicode(std::ffi::OsString::from(
                    "invalid_env_utf8",
                )))
            } else if key == "DATABASE_URL" {
                Ok("postgres://capp:capp@localhost:5432/capp".to_string())
            } else {
                Err(VarError::NotPresent)
            }
        });

        assert_eq!(result, Err(ConfigError::InvalidUtf8("APP_ENV")));
    }

    #[test]
    fn test_database_url_unknown_query_param_does_not_leak() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgres://user:senhasecreta@localhost:5432/capp?passwrod=SENHA_FICTICIA_999"
                .to_string(),
        );

        let result = AppConfig::from_lookup(make_lookup(vars));
        assert_eq!(
            result,
            Err(ConfigError::InvalidSecretFormat {
                name: "DATABASE_URL",
                message: "a URL de conexão contém parâmetros de consulta não permitidos ou desconhecidos",
            })
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(!err_msg.contains("SENHA_FICTICIA_999"));
        assert!(!err_msg.contains("senhasecreta"));
        assert!(!err_msg.contains("passwrod"));
    }

    #[test]
    fn test_database_url_invalid_sslmode_does_not_leak() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            "postgres://user:senhasecreta@localhost:5432/capp?sslmode=SENHA_FICTICIA_888"
                .to_string(),
        );

        let result = AppConfig::from_lookup(make_lookup(vars));
        assert_eq!(
            result,
            Err(ConfigError::InvalidSecretFormat {
                name: "DATABASE_URL",
                message: "o parâmetro 'sslmode' na DATABASE_URL contém valor inválido (esperado: disable, allow, prefer, require, verify-ca ou verify-full)",
            })
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(!err_msg.contains("SENHA_FICTICIA_888"));
        assert!(!err_msg.contains("senhasecreta"));
    }

    #[test]
    fn test_sanitize_connection_error_avoids_leaks() {
        let config_err = sqlx::Error::Configuration("parametro_secreto_123".into());
        let sanitized = sanitize_connection_error(&config_err);
        assert!(!sanitized.contains("parametro_secreto_123"));
        assert_eq!(sanitized, "erro de configuração nos parâmetros de conexão");
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
