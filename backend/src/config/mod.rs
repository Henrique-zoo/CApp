//! Configurações da aplicação e integrações externas do backend CApp.
//!
//! Este módulo centraliza a leitura, validação estrita e valores padrão para
//! parâmetros operacionais do processo, incluindo timeouts e políticas de
//! retentativa segura para clientes HTTP externos.
//!
//! # Invariantes de Segurança
//!
//! - **Sem vazamento de segredos:** Erros de validação reportam o nome da variável
//!   e o motivo estrutural da falha, sem nunca ecoar valores confidenciais.
//! - **Fail-fast:** Configurações inválidas ou timeouts nulos/zerados são rejeitados
//!   imediatamente na inicialização do serviço.
//! - **Timeouts limitados:** Timeouts possuem limites inferiores e superiores para
//!   evitar espera indefinida ou esgotamento prematuro de recursos.
//!
//! # Variáveis de Ambiente
//!
//! - `HTTP_CONNECT_TIMEOUT_SECS`: timeout de conexão em segundos para integrações
//!   HTTP externas (padrão: 3, limites: 1 a 30).
//! - `HTTP_REQUEST_TIMEOUT_SECS`: timeout total por requisição em segundos para
//!   integrações HTTP externas (padrão: 10, limites: 1 a 60).
//! - `HTTP_MAX_RETRIES`: quantidade máxima de retentativas para falhas transitórias
//!   (padrão: 2, limites: 0 a 5).
//! - `HTTP_RETRY_INITIAL_DELAY_MS`: espera inicial do backoff em milissegundos
//!   (padrão: 100, limites: 1 a 10000).
//! - `HTTP_RETRY_MAX_DELAY_MS`: espera máxima do backoff em milissegundos
//!   (padrão: 2000, limites: 1 a 30000).

use std::{env, time::Duration};

/// Timeout de conexão padrão em segundos (3 segundos).
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 3;

/// Timeout de requisição total padrão em segundos (10 segundos).
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 10;

/// Quantidade padrão de retentativas para falhas transitórias (2 retentativas).
pub const DEFAULT_MAX_RETRIES: u32 = 2;

/// Atraso inicial padrão de retentativa em milissegundos (100 ms).
pub const DEFAULT_RETRY_INITIAL_DELAY_MS: u64 = 100;

/// Atraso máximo padrão de retentativa em milissegundos (2000 ms = 2 segundos).
pub const DEFAULT_RETRY_MAX_DELAY_MS: u64 = 2000;

/// Limite mínimo permitido para timeouts em segundos (1 segundo).
pub const MIN_TIMEOUT_SECS: u64 = 1;

/// Limite máximo permitido para timeout de conexão em segundos (30 segundos).
pub const MAX_CONNECT_TIMEOUT_SECS: u64 = 30;

/// Limite máximo permitido para timeout total de requisição em segundos (60 segundos).
pub const MAX_REQUEST_TIMEOUT_SECS: u64 = 60;

/// Limite máximo permitido para a quantidade de retentativas (5 retentativas).
pub const MAX_ALLOWED_RETRIES: u32 = 5;

/// Limite mínimo permitido para o atraso de retentativa em milissegundos (1 ms).
pub const MIN_RETRY_DELAY_MS: u64 = 1;

/// Limite máximo permitido para o atraso de retentativa em milissegundos (30 segundos).
pub const MAX_RETRY_DELAY_MS: u64 = 30_000;

/// Erros de carregamento e validação de configurações externas.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    /// Variável de ambiente contém valor inválido ou fora dos limites permitidos.
    #[error("O valor informado para a variável de ambiente '{var}' é inválido: {details}")]
    InvalidValue {
        /// Nome da variável de ambiente que falhou na validação.
        var: &'static str,
        /// Detalhes estruturais do erro sem expor dados confidenciais.
        details: String,
    },
}

/// Parâmetros de timeout e retentativa segura para clientes HTTP externos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpConfig {
    /// Timeout para o estabelecimento da conexão TCP/TLS.
    pub connect_timeout: Duration,
    /// Timeout total para a conclusão de cada requisição HTTP.
    pub request_timeout: Duration,
    /// Quantidade máxima de retentativas para falhas transitórias.
    pub max_retries: u32,
    /// Atraso base inicial para cálculo do backoff exponencial.
    pub min_retry_delay: Duration,
    /// Teto máximo de espera entre retentativas.
    pub max_retry_delay: Duration,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
            min_retry_delay: Duration::from_millis(DEFAULT_RETRY_INITIAL_DELAY_MS),
            max_retry_delay: Duration::from_millis(DEFAULT_RETRY_MAX_DELAY_MS),
        }
    }
}

impl HttpConfig {
    /// Cria uma configuração rápida adequada para testes unitários determinísticos.
    pub fn default_for_test() -> Self {
        Self {
            connect_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(2),
            max_retries: DEFAULT_MAX_RETRIES,
            min_retry_delay: Duration::from_millis(1),
            max_retry_delay: Duration::from_millis(5),
        }
    }

    /// Carrega as configurações HTTP a partir das variáveis de ambiente do processo.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError::InvalidValue`] se alguma variável de timeout ou retentativa
    /// contiver valores inválidos, zero ou fora dos limites estabelecidos.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    /// Carrega e valida as configurações HTTP a partir de uma função arbitrária de busca.
    ///
    /// Permite testar a leitura de variáveis sem concorrência global no ambiente do processo.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError::InvalidValue`] se algum valor for inválido, zero ou fora dos limites.
    pub fn from_lookup<F>(lookup: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let connect_timeout_secs = match lookup("HTTP_CONNECT_TIMEOUT_SECS")
            .or_else(|| lookup("EXTERNAL_HTTP_CONNECT_TIMEOUT_SECS"))
        {
            Some(raw) => {
                let trimmed = raw.trim();
                let val: u64 = trimmed.parse().map_err(|_| ConfigError::InvalidValue {
                    var: "HTTP_CONNECT_TIMEOUT_SECS",
                    details: "esperado número inteiro positivo em segundos".to_string(),
                })?;
                if !(MIN_TIMEOUT_SECS..=MAX_CONNECT_TIMEOUT_SECS).contains(&val) {
                    return Err(ConfigError::InvalidValue {
                        var: "HTTP_CONNECT_TIMEOUT_SECS",
                        details: format!(
                            "o timeout deve estar entre {MIN_TIMEOUT_SECS} e {MAX_CONNECT_TIMEOUT_SECS} segundos (não pode ser zero)"
                        ),
                    });
                }
                val
            }
            None => DEFAULT_CONNECT_TIMEOUT_SECS,
        };

        let request_timeout_secs = match lookup("HTTP_REQUEST_TIMEOUT_SECS")
            .or_else(|| lookup("HTTP_TIMEOUT_SECS"))
            .or_else(|| lookup("EXTERNAL_HTTP_REQUEST_TIMEOUT_SECS"))
            .or_else(|| lookup("EXTERNAL_HTTP_TIMEOUT_SECS"))
        {
            Some(raw) => {
                let trimmed = raw.trim();
                let val: u64 = trimmed.parse().map_err(|_| ConfigError::InvalidValue {
                    var: "HTTP_REQUEST_TIMEOUT_SECS",
                    details: "esperado número inteiro positivo em segundos".to_string(),
                })?;
                if !(MIN_TIMEOUT_SECS..=MAX_REQUEST_TIMEOUT_SECS).contains(&val) {
                    return Err(ConfigError::InvalidValue {
                        var: "HTTP_REQUEST_TIMEOUT_SECS",
                        details: format!(
                            "o timeout deve estar entre {MIN_TIMEOUT_SECS} e {MAX_REQUEST_TIMEOUT_SECS} segundos (não pode ser zero)"
                        ),
                    });
                }
                val
            }
            None => DEFAULT_REQUEST_TIMEOUT_SECS,
        };

        if connect_timeout_secs > request_timeout_secs {
            return Err(ConfigError::InvalidValue {
                var: "HTTP_CONNECT_TIMEOUT_SECS",
                details:
                    "o timeout de conexão não pode ser maior que o timeout total da requisição"
                        .to_string(),
            });
        }

        let max_retries =
            match lookup("HTTP_MAX_RETRIES").or_else(|| lookup("EXTERNAL_HTTP_MAX_RETRIES")) {
                Some(raw) => {
                    let trimmed = raw.trim();
                    let val: u32 = trimmed.parse().map_err(|_| ConfigError::InvalidValue {
                        var: "HTTP_MAX_RETRIES",
                        details: format!("esperado número inteiro entre 0 e {MAX_ALLOWED_RETRIES}"),
                    })?;
                    if val > MAX_ALLOWED_RETRIES {
                        return Err(ConfigError::InvalidValue {
                            var: "HTTP_MAX_RETRIES",
                            details: format!(
                                "o limite de retentativas não pode exceder {MAX_ALLOWED_RETRIES}"
                            ),
                        });
                    }
                    val
                }
                None => DEFAULT_MAX_RETRIES,
            };

        let min_retry_delay_ms = match lookup("HTTP_RETRY_INITIAL_DELAY_MS") {
            Some(raw) => {
                let trimmed = raw.trim();
                let val: u64 = trimmed.parse().map_err(|_| ConfigError::InvalidValue {
                    var: "HTTP_RETRY_INITIAL_DELAY_MS",
                    details: format!(
                        "esperado número inteiro positivo em milissegundos (entre {MIN_RETRY_DELAY_MS} e {MAX_RETRY_DELAY_MS})"
                    ),
                })?;
                if !(MIN_RETRY_DELAY_MS..=MAX_RETRY_DELAY_MS).contains(&val) {
                    return Err(ConfigError::InvalidValue {
                        var: "HTTP_RETRY_INITIAL_DELAY_MS",
                        details: format!(
                            "o atraso inicial deve estar entre {MIN_RETRY_DELAY_MS} e {MAX_RETRY_DELAY_MS} milissegundos"
                        ),
                    });
                }
                val
            }
            None => DEFAULT_RETRY_INITIAL_DELAY_MS,
        };

        let max_retry_delay_ms = match lookup("HTTP_RETRY_MAX_DELAY_MS") {
            Some(raw) => {
                let trimmed = raw.trim();
                let val: u64 = trimmed.parse().map_err(|_| ConfigError::InvalidValue {
                    var: "HTTP_RETRY_MAX_DELAY_MS",
                    details: format!(
                        "esperado número inteiro positivo em milissegundos (entre {MIN_RETRY_DELAY_MS} e {MAX_RETRY_DELAY_MS})"
                    ),
                })?;
                if !(MIN_RETRY_DELAY_MS..=MAX_RETRY_DELAY_MS).contains(&val) {
                    return Err(ConfigError::InvalidValue {
                        var: "HTTP_RETRY_MAX_DELAY_MS",
                        details: format!(
                            "o atraso máximo deve estar entre {MIN_RETRY_DELAY_MS} e {MAX_RETRY_DELAY_MS} milissegundos"
                        ),
                    });
                }
                val
            }
            None => DEFAULT_RETRY_MAX_DELAY_MS,
        };

        if min_retry_delay_ms > max_retry_delay_ms {
            return Err(ConfigError::InvalidValue {
                var: "HTTP_RETRY_INITIAL_DELAY_MS",
                details: "o atraso inicial não pode ser superior ao atraso máximo de retentativa"
                    .to_string(),
            });
        }

        Ok(Self {
            connect_timeout: Duration::from_secs(connect_timeout_secs),
            request_timeout: Duration::from_secs(request_timeout_secs),
            max_retries,
            min_retry_delay: Duration::from_millis(min_retry_delay_ms),
            max_retry_delay: Duration::from_millis(max_retry_delay_ms),
        })
    }

    /// Constrói um cliente HTTP reutilizável [`reqwest::Client`] com os timeouts configurados
    /// e política de redirecionamento estrita (`Policy::none()`).
    ///
    /// # Erros
    ///
    /// Propaga falha retornada pelo construtor do `reqwest`.
    pub fn build_client(&self) -> Result<reqwest::Client, reqwest::Error> {
        reqwest::Client::builder()
            .connect_timeout(self.connect_timeout)
            .timeout(self.request_timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
    }

    /// Calcula a espera para a próxima retentativa com backoff exponencial e jitter.
    ///
    /// Respeita o cabeçalho `Retry-After` informado pelo provedor quando presente,
    /// limitando a espera ao teto configurado em [`Self::max_retry_delay`].
    pub fn compute_backoff(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        if let Some(ra) = retry_after {
            return ra.min(self.max_retry_delay);
        }

        let shift = attempt.saturating_sub(1).min(10);
        let base_ms = (self.min_retry_delay.as_millis() as u64).saturating_mul(1u64 << shift);
        let capped_ms = base_ms.min(self.max_retry_delay.as_millis() as u64);
        if capped_ms == 0 {
            return Duration::ZERO;
        }

        // Adiciona jitter de até 25% com base nos nanos do relógio do sistema
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let jitter_ms = (capped_ms as f64 * ((nanos % 250) as f64 / 1000.0)) as u64;
        let final_ms = capped_ms
            .saturating_add(jitter_ms)
            .min(self.max_retry_delay.as_millis() as u64);

        Duration::from_millis(final_ms)
    }
}

/// Configuração central da aplicação backend CApp.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    /// Configurações de clientes e integrações HTTP externas.
    pub http: HttpConfig,
}

impl Config {
    /// Constrói uma nova instância da configuração com os parâmetros fornecidos.
    pub fn new(http: HttpConfig) -> Self {
        Self { http }
    }

    /// Carrega as configurações a partir das variáveis de ambiente do processo.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError::InvalidValue`] se alguma variável contiver valores inválidos.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    /// Carrega as configurações a partir de uma função arbitrária de busca.
    ///
    /// # Erros
    ///
    /// Retorna [`ConfigError::InvalidValue`] se alguma variável for inválida.
    pub fn from_lookup<F>(lookup: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let http = HttpConfig::from_lookup(lookup)?;
        Ok(Self { http })
    }

    /// Timeout de conexão configurado para integrações externas.
    pub fn connect_timeout(&self) -> Duration {
        self.http.connect_timeout
    }

    /// Timeout total por requisição configurado para integrações externas.
    pub fn request_timeout(&self) -> Duration {
        self.http.request_timeout
    }

    /// Quantidade máxima de retentativas para falhas transitórias.
    pub fn max_retries(&self) -> u32 {
        self.http.max_retries
    }

    /// Constrói um cliente HTTP reutilizável com os timeouts e redirecionamentos configurados.
    ///
    /// # Erros
    ///
    /// Propaga falha retornada pelo construtor do `reqwest`.
    pub fn build_http_client(&self) -> Result<reqwest::Client, reqwest::Error> {
        self.http.build_client()
    }
}

/// Identifica se um código de status HTTP representa uma falha transitória elegível para retentativa.
///
/// Retorna `true` apenas para falhas de gateway e limitação de taxa:
/// - `429 Too Many Requests`
/// - `502 Bad Gateway`
/// - `503 Service Unavailable`
/// - `504 Gateway Timeout`
pub fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::TOO_MANY_REQUESTS
            | reqwest::StatusCode::BAD_GATEWAY
            | reqwest::StatusCode::SERVICE_UNAVAILABLE
            | reqwest::StatusCode::GATEWAY_TIMEOUT
    )
}

/// Identifica se um código de status HTTP representa um erro definitivo do cliente.
///
/// **Nunca** deve ser retentado:
/// - `400 Bad Request`
/// - `401 Unauthorized`
/// - `403 Forbidden`
/// - `404 Not Found`
pub fn is_permanent_client_error(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::BAD_REQUEST
            | reqwest::StatusCode::UNAUTHORIZED
            | reqwest::StatusCode::FORBIDDEN
            | reqwest::StatusCode::NOT_FOUND
    )
}

/// Identifica se um erro do `reqwest` decorre de falha de conexão, transporte ou timeout.
pub fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

/// Extrai e valida a quantidade de segundos indicada no cabeçalho `Retry-After`.
///
/// Retorna `None` se o cabeçalho estiver ausente ou não contiver um número inteiro válido.
pub fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let header_val = headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim();
    header_val.parse::<u64>().ok().map(Duration::from_secs)
}

#[cfg(test)]
mod tests;

mod server;
pub use server::{ServerConfig, DEFAULT_BIND_ADDRESS, DEFAULT_SHUTDOWN_TIMEOUT};
