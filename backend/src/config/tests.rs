use std::time::Duration;

use super::*;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(
        config.connect_timeout(),
        Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS)
    );
    assert_eq!(
        config.request_timeout(),
        Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS)
    );
    assert_eq!(config.max_retries(), DEFAULT_MAX_RETRIES);
    assert_eq!(
        config.http.min_retry_delay,
        Duration::from_millis(DEFAULT_RETRY_INITIAL_DELAY_MS)
    );
    assert_eq!(
        config.http.max_retry_delay,
        Duration::from_millis(DEFAULT_RETRY_MAX_DELAY_MS)
    );
}

#[test]
fn test_from_lookup_defaults_when_empty() {
    let config = Config::from_lookup(|_| None).expect("deve carregar com valores padrão");
    assert_eq!(config, Config::default());
}

#[test]
fn test_from_lookup_custom_valid() {
    let config = Config::from_lookup(|key| match key {
        "HTTP_CONNECT_TIMEOUT_SECS" => Some("5".to_string()),
        "HTTP_REQUEST_TIMEOUT_SECS" => Some("15".to_string()),
        "HTTP_MAX_RETRIES" => Some("3".to_string()),
        "HTTP_RETRY_INITIAL_DELAY_MS" => Some("50".to_string()),
        "HTTP_RETRY_MAX_DELAY_MS" => Some("1000".to_string()),
        _ => None,
    })
    .expect("deve aceitar valores válidos");

    assert_eq!(config.connect_timeout(), Duration::from_secs(5));
    assert_eq!(config.request_timeout(), Duration::from_secs(15));
    assert_eq!(config.max_retries(), 3);
    assert_eq!(config.http.min_retry_delay, Duration::from_millis(50));
    assert_eq!(config.http.max_retry_delay, Duration::from_millis(1000));
}

#[test]
fn test_from_lookup_boundary_values() {
    let min_config = Config::from_lookup(|key| match key {
        "HTTP_CONNECT_TIMEOUT_SECS" => Some(MIN_TIMEOUT_SECS.to_string()),
        "HTTP_REQUEST_TIMEOUT_SECS" => Some(MIN_TIMEOUT_SECS.to_string()),
        "HTTP_MAX_RETRIES" => Some("0".to_string()),
        "HTTP_RETRY_INITIAL_DELAY_MS" => Some(MIN_RETRY_DELAY_MS.to_string()),
        "HTTP_RETRY_MAX_DELAY_MS" => Some(MIN_RETRY_DELAY_MS.to_string()),
        _ => None,
    })
    .expect("limites mínimos devem ser aceitos");

    assert_eq!(
        min_config.connect_timeout(),
        Duration::from_secs(MIN_TIMEOUT_SECS)
    );
    assert_eq!(
        min_config.request_timeout(),
        Duration::from_secs(MIN_TIMEOUT_SECS)
    );
    assert_eq!(min_config.max_retries(), 0);

    let max_config = Config::from_lookup(|key| match key {
        "HTTP_CONNECT_TIMEOUT_SECS" => Some(MAX_CONNECT_TIMEOUT_SECS.to_string()),
        "HTTP_REQUEST_TIMEOUT_SECS" => Some(MAX_REQUEST_TIMEOUT_SECS.to_string()),
        "HTTP_MAX_RETRIES" => Some(MAX_ALLOWED_RETRIES.to_string()),
        "HTTP_RETRY_INITIAL_DELAY_MS" => Some(MAX_RETRY_DELAY_MS.to_string()),
        "HTTP_RETRY_MAX_DELAY_MS" => Some(MAX_RETRY_DELAY_MS.to_string()),
        _ => None,
    })
    .expect("limites máximos devem ser aceitos");

    assert_eq!(
        max_config.connect_timeout(),
        Duration::from_secs(MAX_CONNECT_TIMEOUT_SECS)
    );
    assert_eq!(
        max_config.request_timeout(),
        Duration::from_secs(MAX_REQUEST_TIMEOUT_SECS)
    );
    assert_eq!(max_config.max_retries(), MAX_ALLOWED_RETRIES);
}

#[test]
fn test_from_lookup_zero_connect_timeout_fails() {
    let err = Config::from_lookup(|key| match key {
        "HTTP_CONNECT_TIMEOUT_SECS" => Some("0".to_string()),
        _ => None,
    })
    .expect_err("deve rejeitar connect timeout zero");

    match err {
        ConfigError::InvalidValue { var, details } => {
            assert_eq!(var, "HTTP_CONNECT_TIMEOUT_SECS");
            assert!(details.contains("não pode ser zero"));
        }
    }
}

#[test]
fn test_from_lookup_zero_request_timeout_fails() {
    let err = Config::from_lookup(|key| match key {
        "HTTP_REQUEST_TIMEOUT_SECS" => Some("0".to_string()),
        _ => None,
    })
    .expect_err("deve rejeitar request timeout zero");

    match err {
        ConfigError::InvalidValue { var, details } => {
            assert_eq!(var, "HTTP_REQUEST_TIMEOUT_SECS");
            assert!(details.contains("não pode ser zero"));
        }
    }
}

#[test]
fn test_from_lookup_connect_greater_than_request_timeout_fails() {
    let err = Config::from_lookup(|key| match key {
        "HTTP_CONNECT_TIMEOUT_SECS" => Some("15".to_string()),
        "HTTP_REQUEST_TIMEOUT_SECS" => Some("5".to_string()),
        _ => None,
    })
    .expect_err("deve rejeitar connect timeout maior que request timeout");

    match err {
        ConfigError::InvalidValue { var, details } => {
            assert_eq!(var, "HTTP_CONNECT_TIMEOUT_SECS");
            assert!(details.contains("não pode ser maior"));
        }
    }
}

#[test]
fn test_from_lookup_invalid_strings_fail_without_leak() {
    let secret_payload = "SENHA_SUPER_SECRETA_12345";
    let err = Config::from_lookup(|key| match key {
        "HTTP_CONNECT_TIMEOUT_SECS" => Some(secret_payload.to_string()),
        _ => None,
    })
    .expect_err("deve falhar com string não numérica");

    let err_str = err.to_string();
    assert!(!err_str.contains(secret_payload));
    assert!(err_str.contains("HTTP_CONNECT_TIMEOUT_SECS"));
}

#[test]
fn test_from_lookup_retries_exceeds_max_fails() {
    let err = Config::from_lookup(|key| match key {
        "HTTP_MAX_RETRIES" => Some("6".to_string()),
        _ => None,
    })
    .expect_err("deve rejeitar retentativas acima de 5");

    match err {
        ConfigError::InvalidValue { var, details } => {
            assert_eq!(var, "HTTP_MAX_RETRIES");
            assert!(details.contains("não pode exceder 5"));
        }
    }
}

#[test]
fn test_from_lookup_min_retry_delay_greater_than_max_fails() {
    let err = Config::from_lookup(|key| match key {
        "HTTP_RETRY_INITIAL_DELAY_MS" => Some("5000".to_string()),
        "HTTP_RETRY_MAX_DELAY_MS" => Some("1000".to_string()),
        _ => None,
    })
    .expect_err("deve rejeitar atraso inicial maior que máximo");

    match err {
        ConfigError::InvalidValue { var, details } => {
            assert_eq!(var, "HTTP_RETRY_INITIAL_DELAY_MS");
            assert!(details.contains("não pode ser superior"));
        }
    }
}

#[test]
fn test_is_retryable_status() {
    use reqwest::StatusCode;

    // Deve retentar apenas falhas transitórias seguras
    assert!(is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
    assert!(is_retryable_status(StatusCode::BAD_GATEWAY));
    assert!(is_retryable_status(StatusCode::SERVICE_UNAVAILABLE));
    assert!(is_retryable_status(StatusCode::GATEWAY_TIMEOUT));

    // NUNCA deve retentar erros de cliente, sucesso ou erros internos não transitórios
    assert!(!is_retryable_status(StatusCode::OK));
    assert!(!is_retryable_status(StatusCode::BAD_REQUEST));
    assert!(!is_retryable_status(StatusCode::UNAUTHORIZED));
    assert!(!is_retryable_status(StatusCode::FORBIDDEN));
    assert!(!is_retryable_status(StatusCode::NOT_FOUND));
    assert!(!is_retryable_status(StatusCode::INTERNAL_SERVER_ERROR));
}

#[test]
fn test_is_permanent_client_error() {
    use reqwest::StatusCode;

    assert!(is_permanent_client_error(StatusCode::BAD_REQUEST));
    assert!(is_permanent_client_error(StatusCode::UNAUTHORIZED));
    assert!(is_permanent_client_error(StatusCode::FORBIDDEN));
    assert!(is_permanent_client_error(StatusCode::NOT_FOUND));

    assert!(!is_permanent_client_error(StatusCode::OK));
    assert!(!is_permanent_client_error(StatusCode::TOO_MANY_REQUESTS));
    assert!(!is_permanent_client_error(StatusCode::SERVICE_UNAVAILABLE));
    assert!(!is_permanent_client_error(
        StatusCode::INTERNAL_SERVER_ERROR
    ));
}

#[test]
fn test_parse_retry_after() {
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

    let mut headers = HeaderMap::new();
    assert_eq!(parse_retry_after(&headers), None);

    headers.insert(RETRY_AFTER, HeaderValue::from_static("15"));
    assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(15)));

    headers.insert(RETRY_AFTER, HeaderValue::from_static("invalido"));
    assert_eq!(parse_retry_after(&headers), None);
}

#[test]
fn test_compute_backoff_exponential_and_retry_after() {
    let config = HttpConfig {
        connect_timeout: Duration::from_secs(3),
        request_timeout: Duration::from_secs(10),
        max_retries: 3,
        min_retry_delay: Duration::from_millis(100),
        max_retry_delay: Duration::from_millis(1000),
    };

    // Retry-After fornecido tem precedência e é limitado pelo max_delay
    let with_ra = config.compute_backoff(1, Some(Duration::from_millis(500)));
    assert_eq!(with_ra, Duration::from_millis(500));

    let with_ra_exceeded = config.compute_backoff(1, Some(Duration::from_millis(5000)));
    assert_eq!(with_ra_exceeded, Duration::from_millis(1000));

    // Sem Retry-After: escala exponencial com jitter delimitado
    let attempt1 = config.compute_backoff(1, None);
    assert!(
        attempt1 >= Duration::from_millis(100) && attempt1 <= Duration::from_millis(130),
        "attempt 1: {:?}",
        attempt1
    );

    let attempt2 = config.compute_backoff(2, None);
    assert!(
        attempt2 >= Duration::from_millis(200) && attempt2 <= Duration::from_millis(260),
        "attempt 2: {:?}",
        attempt2
    );

    let attempt3 = config.compute_backoff(3, None);
    assert!(
        attempt3 >= Duration::from_millis(400) && attempt3 <= Duration::from_millis(520),
        "attempt 3: {:?}",
        attempt3
    );

    // Tentativas altas são limitadas pelo max_retry_delay (1000 ms)
    let attempt10 = config.compute_backoff(10, None);
    assert_eq!(attempt10, Duration::from_millis(1000));
}

#[test]
fn test_build_client_succeeds() {
    let config = HttpConfig::default();
    let client = config.build_client();
    assert!(client.is_ok());
}

#[test]
fn helper_subprocess_env() {
    if std::env::var("__CAPP_SUBPROCESS_CONFIG_TEST").as_deref() == Ok("1") {
        let config = Config::from_env().expect("deve carregar do ambiente do subprocesso");
        assert_eq!(config.connect_timeout(), Duration::from_secs(4));
        assert_eq!(config.request_timeout(), Duration::from_secs(12));
        assert_eq!(config.max_retries(), 1);
    }
}

#[test]
fn test_config_from_env_in_isolated_subprocess() {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return,
    };
    let status = std::process::Command::new(exe)
        .arg("--exact")
        .arg("config::tests::helper_subprocess_env")
        .arg("--nocapture")
        .env("__CAPP_SUBPROCESS_CONFIG_TEST", "1")
        .env("HTTP_CONNECT_TIMEOUT_SECS", "4")
        .env("HTTP_REQUEST_TIMEOUT_SECS", "12")
        .env("HTTP_MAX_RETRIES", "1")
        .status()
        .expect("deve executar subprocesso de teste");
    assert!(status.success(), "subprocesso de teste falhou");
}
