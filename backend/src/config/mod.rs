//! Configurações do servidor e parâmetros operacionais do backend CApp.
//!
//! Este módulo centraliza a leitura, a validação e os valores padrão das
//! configurações externas da aplicação, como parâmetros de rede e prazos de
//! encerramento controlado (*graceful shutdown*).
//!
//! Variáveis de infraestrutura como `DATABASE_URL`, `FIREBASE_PROJECT_ID`,
//! `FIREBASE_PROJECT_NUMBER`, `FIREBASE_APP_CHECK_APP_IDS` e `RUST_LOG` são lidas
//! na inicialização do executável em `main.rs`.
//!
//! # Variáveis de Ambiente do Módulo
//!
//! - `SHUTDOWN_TIMEOUT_SECS`: tempo limite em segundos para drenagem de requisições
//!   ativas durante o encerramento gracioso (padrão: 15 segundos).
//! - `BACKEND_BIND_ADDRESS`: endereço IP e porta para escuta HTTP (padrão: `0.0.0.0:3000`).

use std::{env, time::Duration};
use thiserror::Error;

/// Tempo padrão de tolerância para drenagem de conexões ativas durante o encerramento (15 segundos).
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(15);

/// Endereço padrão de escuta do servidor HTTP.
pub const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:3000";

/// Limite mínimo permitido para o prazo de encerramento em segundos.
pub const MIN_SHUTDOWN_TIMEOUT_SECS: u64 = 1;

/// Limite máximo permitido para o prazo de encerramento em segundos (5 minutos).
pub const MAX_SHUTDOWN_TIMEOUT_SECS: u64 = 300;

/// Erros decorrentes do carregamento ou da validação das variáveis de configuração.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// Variável de ambiente contém valor numérico inválido ou fora dos limites aceitáveis.
    #[error("O valor informado para a variável de ambiente '{var}' é inválido: {details}")]
    InvalidValue {
        /// Nome da variável de ambiente com erro.
        var: &'static str,
        /// Descrição detalhada do erro de validação.
        details: String,
    },
}

/// Parâmetros operacionais e de ciclo de vida do servidor HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// Endereço IP e porta para abertura do socket TCP (ex: `"0.0.0.0:3000"`).
    pub bind_address: String,
    /// Prazo máximo concedido para drenagem de requisições ativas antes do encerramento forçado.
    pub shutdown_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: DEFAULT_BIND_ADDRESS.to_string(),
            shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
        }
    }
}

impl ServerConfig {
    /// Constrói a configuração informando explicitamente endereço de bind e prazo de encerramento.
    ///
    /// # Parâmetros
    ///
    /// - `bind_address`: string com IP e porta onde o socket TCP será vinculado.
    /// - `shutdown_timeout`: duração máxima da janela de drenagem durante o shutdown.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use std::time::Duration;
    /// use backend::config::ServerConfig;
    ///
    /// let config = ServerConfig::new("127.0.0.1:8080", Duration::from_secs(20));
    /// assert_eq!(config.bind_address, "127.0.0.1:8080");
    /// assert_eq!(config.shutdown_timeout, Duration::from_secs(20));
    /// ```
    pub fn new(bind_address: impl Into<String>, shutdown_timeout: Duration) -> Self {
        Self {
            bind_address: bind_address.into(),
            shutdown_timeout,
        }
    }

    /// Carrega as configurações operacionais a partir das variáveis de ambiente do processo.
    ///
    /// Lê `BACKEND_BIND_ADDRESS` (ou usa [`DEFAULT_BIND_ADDRESS`]) e `SHUTDOWN_TIMEOUT_SECS`
    /// (ou usa [`DEFAULT_SHUTDOWN_TIMEOUT`]). Valores numéricos devem ser inteiros positivos
    /// entre [`MIN_SHUTDOWN_TIMEOUT_SECS`] e [`MAX_SHUTDOWN_TIMEOUT_SECS`] segundos.
    ///
    /// # Retorno
    ///
    /// Retorna [`ServerConfig`] com as propriedades lidas ou seus padrões.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError::InvalidValue`] se `SHUTDOWN_TIMEOUT_SECS` contiver um valor não numérico,
    /// zero ou superior a 300 segundos.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use backend::config::ServerConfig;
    ///
    /// let config = ServerConfig::from_env().expect("configuração padrão válida");
    /// assert!(!config.bind_address.is_empty());
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    /// Carrega e valida as configurações a partir de uma função arbitrária de busca de variáveis.
    ///
    /// Permite testar o carregamento de configurações de forma determinística e isolada,
    /// sem mutação concorrente de variáveis globais de ambiente no processo.
    ///
    /// # Parâmetros
    ///
    /// - `lookup`: função ou fechamento que consulta o valor opcional de uma variável.
    ///
    /// # Retorno
    ///
    /// Retorna [`ServerConfig`] com as propriedades resolvidas ou seus padrões.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError::InvalidValue`] se `SHUTDOWN_TIMEOUT_SECS` contiver um valor não numérico,
    /// zero ou superior a 300 segundos.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use std::time::Duration;
    /// use backend::config::ServerConfig;
    ///
    /// let config = ServerConfig::from_lookup(|key| match key {
    ///     "SHUTDOWN_TIMEOUT_SECS" => Some("20".to_string()),
    ///     "BACKEND_BIND_ADDRESS" => Some("127.0.0.1:8080".to_string()),
    ///     _ => None,
    /// }).expect("configuração válida");
    /// assert_eq!(config.shutdown_timeout, Duration::from_secs(20));
    /// assert_eq!(config.bind_address, "127.0.0.1:8080");
    /// ```
    pub fn from_lookup<F>(lookup: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let bind_address = lookup("BACKEND_BIND_ADDRESS")
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BIND_ADDRESS.to_string());

        let shutdown_timeout = match lookup("SHUTDOWN_TIMEOUT_SECS") {
            Some(val) => {
                let secs: u64 = val.parse().map_err(|_| ConfigError::InvalidValue {
                    var: "SHUTDOWN_TIMEOUT_SECS",
                    details: format!(
                        "esperado número inteiro positivo em segundos, obtido '{val}'"
                    ),
                })?;

                if !(MIN_SHUTDOWN_TIMEOUT_SECS..=MAX_SHUTDOWN_TIMEOUT_SECS).contains(&secs) {
                    return Err(ConfigError::InvalidValue {
                        var: "SHUTDOWN_TIMEOUT_SECS",
                        details: format!(
                            "o prazo deve estar entre {MIN_SHUTDOWN_TIMEOUT_SECS} e {MAX_SHUTDOWN_TIMEOUT_SECS} segundos, obtido '{secs}'"
                        ),
                    });
                }

                Duration::from_secs(secs)
            }
            None => DEFAULT_SHUTDOWN_TIMEOUT,
        };

        Ok(Self {
            bind_address,
            shutdown_timeout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0:3000");
        assert_eq!(config.shutdown_timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_server_config_custom_constructor() {
        let config = ServerConfig::new("127.0.0.1:4000", Duration::from_secs(25));
        assert_eq!(config.bind_address, "127.0.0.1:4000");
        assert_eq!(config.shutdown_timeout, Duration::from_secs(25));
    }

    #[test]
    fn test_server_config_from_lookup_defaults_when_unset() {
        let config =
            ServerConfig::from_lookup(|_| None).expect("configuração padrão deve ser válida");
        assert_eq!(config.bind_address, DEFAULT_BIND_ADDRESS);
        assert_eq!(config.shutdown_timeout, DEFAULT_SHUTDOWN_TIMEOUT);
    }

    #[test]
    fn test_server_config_from_lookup_custom_valid() {
        let config = ServerConfig::from_lookup(|key| match key {
            "SHUTDOWN_TIMEOUT_SECS" => Some("25".to_string()),
            "BACKEND_BIND_ADDRESS" => Some("127.0.0.1:4500".to_string()),
            _ => None,
        })
        .expect("deve carregar variáveis personalizadas");
        assert_eq!(config.bind_address, "127.0.0.1:4500");
        assert_eq!(config.shutdown_timeout, Duration::from_secs(25));
    }

    #[test]
    fn test_server_config_from_lookup_boundary_values() {
        let min_config = ServerConfig::from_lookup(|key| match key {
            "SHUTDOWN_TIMEOUT_SECS" => Some(MIN_SHUTDOWN_TIMEOUT_SECS.to_string()),
            _ => None,
        })
        .expect("limite inferior deve ser aceito");
        assert_eq!(
            min_config.shutdown_timeout,
            Duration::from_secs(MIN_SHUTDOWN_TIMEOUT_SECS)
        );

        let max_config = ServerConfig::from_lookup(|key| match key {
            "SHUTDOWN_TIMEOUT_SECS" => Some(MAX_SHUTDOWN_TIMEOUT_SECS.to_string()),
            _ => None,
        })
        .expect("limite superior deve ser aceito");
        assert_eq!(
            max_config.shutdown_timeout,
            Duration::from_secs(MAX_SHUTDOWN_TIMEOUT_SECS)
        );
    }

    #[test]
    fn test_server_config_from_lookup_invalid_string() {
        let err = ServerConfig::from_lookup(|key| match key {
            "SHUTDOWN_TIMEOUT_SECS" => Some("texto_invalido".to_string()),
            _ => None,
        })
        .expect_err("deve falhar com string não numérica");

        match err {
            ConfigError::InvalidValue { var, details } => {
                assert_eq!(var, "SHUTDOWN_TIMEOUT_SECS");
                assert!(details.contains("esperado número inteiro"));
            }
        }
    }

    #[test]
    fn test_server_config_from_lookup_zero_is_invalid() {
        let err = ServerConfig::from_lookup(|key| match key {
            "SHUTDOWN_TIMEOUT_SECS" => Some("0".to_string()),
            _ => None,
        })
        .expect_err("deve falhar com zero");

        match err {
            ConfigError::InvalidValue { var, details } => {
                assert_eq!(var, "SHUTDOWN_TIMEOUT_SECS");
                assert!(details.contains("o prazo deve estar entre"));
            }
        }
    }

    #[test]
    fn test_server_config_from_lookup_exceeds_max() {
        let err = ServerConfig::from_lookup(|key| match key {
            "SHUTDOWN_TIMEOUT_SECS" => Some("301".to_string()),
            _ => None,
        })
        .expect_err("deve falhar acima do limite máximo");

        match err {
            ConfigError::InvalidValue { var, details } => {
                assert_eq!(var, "SHUTDOWN_TIMEOUT_SECS");
                assert!(details.contains("o prazo deve estar entre"));
            }
        }
    }

    #[test]
    fn helper_subprocess_env() {
        if std::env::var("__CAPP_SUBPROCESS_CONFIG_TEST").as_deref() == Ok("1") {
            let config =
                ServerConfig::from_env().expect("deve carregar do ambiente do subprocesso");
            assert_eq!(config.bind_address, "127.0.0.1:4500");
            assert_eq!(config.shutdown_timeout, Duration::from_secs(25));
        }
    }

    #[test]
    fn test_server_config_from_env_in_isolated_subprocess() {
        let exe = match std::env::current_exe() {
            Ok(exe) => exe,
            Err(_) => return,
        };
        let status = std::process::Command::new(exe)
            .arg("--exact")
            .arg("config::tests::helper_subprocess_env")
            .arg("--nocapture")
            .env("__CAPP_SUBPROCESS_CONFIG_TEST", "1")
            .env("BACKEND_BIND_ADDRESS", "127.0.0.1:4500")
            .env("SHUTDOWN_TIMEOUT_SECS", "25")
            .status()
            .expect("deve executar subprocesso de teste");
        assert!(status.success(), "subprocesso de teste falhou");
    }
}
