//! Configurações do servidor e parâmetros operacionais do backend CApp.
//!
//! Este módulo centraliza a leitura, a validação e os valores padrão das
//! configurações externas da aplicação, como parâmetros de rede e prazos de
//! encerramento controlado (*graceful shutdown*).
//!
//! # Variáveis de Ambiente
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
        let bind_address =
            env::var("BACKEND_BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_string());

        let shutdown_timeout = match env::var("SHUTDOWN_TIMEOUT_SECS") {
            Ok(val) => {
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
            Err(_) => DEFAULT_SHUTDOWN_TIMEOUT,
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
    fn test_server_config_from_env_defaults_when_unset() {
        unsafe {
            env::remove_var("SHUTDOWN_TIMEOUT_SECS");
            env::remove_var("BACKEND_BIND_ADDRESS");
        }
        let config = ServerConfig::from_env().expect("configuração padrão deve ser válida");
        assert_eq!(config.bind_address, DEFAULT_BIND_ADDRESS);
        assert_eq!(config.shutdown_timeout, DEFAULT_SHUTDOWN_TIMEOUT);
    }

    #[test]
    fn test_server_config_from_env_custom_valid() {
        unsafe {
            env::set_var("SHUTDOWN_TIMEOUT_SECS", "25");
            env::set_var("BACKEND_BIND_ADDRESS", "127.0.0.1:4500");
        }
        let config = ServerConfig::from_env().expect("deve carregar variáveis personalizadas");
        assert_eq!(config.bind_address, "127.0.0.1:4500");
        assert_eq!(config.shutdown_timeout, Duration::from_secs(25));
        unsafe {
            env::remove_var("SHUTDOWN_TIMEOUT_SECS");
            env::remove_var("BACKEND_BIND_ADDRESS");
        }
    }

    #[test]
    fn test_server_config_from_env_invalid_string() {
        unsafe {
            env::set_var("SHUTDOWN_TIMEOUT_SECS", "texto_invalido");
        }
        let err = ServerConfig::from_env().expect_err("deve falhar com string não numérica");
        match err {
            ConfigError::InvalidValue { var, details } => {
                assert_eq!(var, "SHUTDOWN_TIMEOUT_SECS");
                assert!(details.contains("esperado número inteiro"));
            }
        }
        unsafe {
            env::remove_var("SHUTDOWN_TIMEOUT_SECS");
        }
    }

    #[test]
    fn test_server_config_from_env_zero_is_invalid() {
        unsafe {
            env::set_var("SHUTDOWN_TIMEOUT_SECS", "0");
        }
        let err = ServerConfig::from_env().expect_err("deve falhar com zero");
        match err {
            ConfigError::InvalidValue { var, details } => {
                assert_eq!(var, "SHUTDOWN_TIMEOUT_SECS");
                assert!(details.contains("o prazo deve estar entre"));
            }
        }
        unsafe {
            env::remove_var("SHUTDOWN_TIMEOUT_SECS");
        }
    }

    #[test]
    fn test_server_config_from_env_exceeds_max() {
        unsafe {
            env::set_var("SHUTDOWN_TIMEOUT_SECS", "301");
        }
        let err = ServerConfig::from_env().expect_err("deve falhar acima do limite máximo");
        match err {
            ConfigError::InvalidValue { var, details } => {
                assert_eq!(var, "SHUTDOWN_TIMEOUT_SECS");
                assert!(details.contains("o prazo deve estar entre"));
            }
        }
        unsafe {
            env::remove_var("SHUTDOWN_TIMEOUT_SECS");
        }
    }
}
