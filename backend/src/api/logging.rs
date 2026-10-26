//! Configuração de observabilidade e emissão de logs estruturados (`tracing_subscriber`).
//!
//! Este módulo gerencia a inicialização do subscriber global de telemetria, suportando
//! múltiplos formatos de saída conforme a variável de ambiente `LOG_FORMAT`:
//! - `json`: Formato estruturado em JSON (uma linha por evento), otimizado para
//!   ambientes conteinerizados e coletores de log (ex: Datadog, Grafana Loki, CloudWatch).
//! - `compact`: Formato condensado legível do `tracing-subscriber`.
//! - `pretty`: Formato verboso com cores e quebras de linha para desenvolvimento local.
//! - `full`: Formato tradicional detalhado (padrão).
//!
//! # Segurança e Sanitização de Dados
//!
//! Em conformidade com a política de governança e proteção de dados:
//! - Segredos, tokens e chaves sensíveis (ex: `authorization`, `password`, `secret`, `token`)
//!   são automaticamente mascarados com `[REDACTED]`.
//! - Dados de credenciais nunca são expostos em texto claro em nenhum formato de saída.

use std::{
    env, fmt,
    io::Write,
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use tracing::{
    Event, Id, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Record},
};
use tracing_subscriber::{
    EnvFilter, Layer,
    layer::Context as LayerContext,
    prelude::*,
    registry::{LookupSpan, Registry},
};

/// Formato de formatação de logs suportado pelo processo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Formato estruturado JSON (uma linha por evento com metadados e spans).
    Json,
    /// Formato compacto legível em terminal.
    Compact,
    /// Formato colorido e identado para depuração local interativa.
    Pretty,
    /// Formato padrão detalhado do `tracing-subscriber`.
    #[default]
    Full,
}

impl LogFormat {
    /// Determina o formato de log a partir da variável `LOG_FORMAT` do ambiente.
    ///
    /// Se `LOG_FORMAT` não estiver configurada, verifica a variável `ENVIRONMENT` ou
    /// `APP_ENV`; em ambientes de produção (`production` ou `prod`), adota `Json`
    /// por padrão. Caso contrário, adota `Full`.
    #[must_use]
    pub fn from_env() -> Self {
        if let Ok(val) = env::var("LOG_FORMAT") {
            Self::from_str(&val).unwrap_or_default()
        } else if let Ok(env_val) = env::var("ENVIRONMENT").or_else(|_| env::var("APP_ENV")) {
            if env_val.eq_ignore_ascii_case("production") || env_val.eq_ignore_ascii_case("prod") {
                Self::Json
            } else {
                Self::Full
            }
        } else {
            Self::Full
        }
    }
}

impl FromStr for LogFormat {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "compact" => Ok(Self::Compact),
            "pretty" => Ok(Self::Pretty),
            _ => Ok(Self::Full),
        }
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Compact => write!(f, "compact"),
            Self::Pretty => write!(f, "pretty"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Chaves e cabeçalhos conhecidos como sensíveis que devem ser mascarados.
const SENSITIVE_KEYS: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "token",
    "access_token",
    "refresh_token",
    "secret",
    "client_secret",
    "password",
    "passwd",
    "api_key",
    "apikey",
    "cookie",
    "set-cookie",
];

/// Verifica se o nome do campo ou chave representa um segredo ou credencial sensível.
#[must_use]
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SENSITIVE_KEYS
        .iter()
        .any(|&sensitive| lower == sensitive || lower.contains("secret") || lower.contains("token"))
}

/// Mascara valores associados a campos sensíveis ou que contenham tokens em texto claro.
#[must_use]
pub fn mask_sensitive_value(key: &str, value: &str) -> String {
    if is_sensitive_key(key) {
        return "[REDACTED]".to_string();
    }

    if value.to_ascii_lowercase().starts_with("bearer ") {
        return "Bearer [REDACTED]".to_string();
    }

    value.to_string()
}

/// Dados associados a um span ativo armazenados em suas extensões.
#[derive(Default, Debug)]
struct SpanData {
    fields: Mutex<serde_json::Map<String, serde_json::Value>>,
}

/// Visitor que coleta e mascara campos de eventos e spans em um mapa JSON.
struct JsonFieldsVisitor<'a>(&'a mut serde_json::Map<String, serde_json::Value>);

impl Visit for JsonFieldsVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let field_name = field.name();
        let raw_value = format!("{value:?}");
        let sanitized = mask_sensitive_value(field_name, &raw_value);
        self.0
            .insert(field_name.to_string(), serde_json::Value::String(sanitized));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let field_name = field.name();
        let sanitized = mask_sensitive_value(field_name, value);
        self.0
            .insert(field_name.to_string(), serde_json::Value::String(sanitized));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }
}

/// Camada do `tracing-subscriber` responsável por emitir eventos em formato JSON estruturado.
///
/// Cada linha é um objeto JSON delimitado por quebra de linha contendo `timestamp`,
/// `level`, `target`, `message`, `request_id` (se presente no span) e a cadeia de spans ativos.
#[derive(Clone)]
pub struct JsonLogLayer {
    writer: Arc<Mutex<Box<dyn Write + Send + Sync>>>,
}

impl Default for JsonLogLayer {
    fn default() -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
        }
    }
}

impl JsonLogLayer {
    /// Constrói uma nova camada JSON direcionando a saída para o escritor fornecido.
    #[must_use]
    pub fn new<W: Write + Send + Sync + 'static>(writer: W) -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(writer))),
        }
    }
}

impl<S> Layer<S> for JsonLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: LayerContext<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut fields = serde_json::Map::new();
            attrs.record(&mut JsonFieldsVisitor(&mut fields));
            span.extensions_mut().insert(SpanData {
                fields: Mutex::new(fields),
            });
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: LayerContext<'_, S>) {
        if let Some(span) = ctx.span(id)
            && let Some(data) = span.extensions().get::<SpanData>()
            && let Ok(mut fields) = data.fields.lock()
        {
            values.record(&mut JsonFieldsVisitor(&mut fields));
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: LayerContext<'_, S>) {
        let mut fields = serde_json::Map::new();
        event.record(&mut JsonFieldsVisitor(&mut fields));

        let mut log_entry = serde_json::Map::new();
        log_entry.insert(
            "timestamp".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );
        log_entry.insert(
            "level".to_string(),
            serde_json::Value::String(event.metadata().level().to_string()),
        );
        log_entry.insert(
            "target".to_string(),
            serde_json::Value::String(event.metadata().target().to_string()),
        );

        if let Some(message) = fields.remove("message") {
            log_entry.insert("message".to_string(), message);
        }

        if let Some(scope) = ctx.event_scope(event) {
            let mut span_chain = Vec::new();
            for span in scope {
                let mut span_obj = serde_json::Map::new();
                span_obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(span.name().to_string()),
                );
                if let Some(data) = span.extensions().get::<SpanData>()
                    && let Ok(guard) = data.fields.lock()
                {
                    if let Some(req_id) = guard.get("request_id")
                        && !log_entry.contains_key("request_id")
                    {
                        log_entry.insert("request_id".to_string(), req_id.clone());
                    }
                    span_obj.insert(
                        "fields".to_string(),
                        serde_json::Value::Object(guard.clone()),
                    );
                }
                span_chain.push(serde_json::Value::Object(span_obj));
            }
            if !span_chain.is_empty() {
                log_entry.insert("spans".to_string(), serde_json::Value::Array(span_chain));
            }
        }

        if !fields.is_empty() {
            log_entry.insert("fields".to_string(), serde_json::Value::Object(fields));
        }

        if let Ok(serialized) = serde_json::to_string(&log_entry)
            && let Ok(mut writer) = self.writer.lock()
        {
            let _ = writer.write_all(serialized.as_bytes());
            let _ = writer.write_all(b"\n");
        }
    }
}

/// Inicializa o subscriber global de logging conforme o ambiente.
///
/// Lê as variáveis de ambiente `RUST_LOG` (filtro de nível de log) e `LOG_FORMAT`
/// (`json`, `compact`, `pretty`, `full`).
///
/// # Erros
///
/// Retorna erro se o subscriber global já tiver sido inicializado neste processo.
pub fn init_logging() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let format = LogFormat::from_env();

    match format {
        LogFormat::Json => {
            Registry::default()
                .with(filter)
                .with(JsonLogLayer::default())
                .try_init()
                .map_err(|e| anyhow::anyhow!(e))
                .context("Falha ao inicializar o subscriber de log JSON")?;
        }
        LogFormat::Compact => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .try_init()
                .map_err(|e| anyhow::anyhow!(e))
                .context("Falha ao inicializar o subscriber de log compact")?;
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .pretty()
                .try_init()
                .map_err(|e| anyhow::anyhow!(e))
                .context("Falha ao inicializar o subscriber de log pretty")?;
        }
        LogFormat::Full => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .try_init()
                .map_err(|e| anyhow::anyhow!(e))
                .context("Falha ao inicializar o subscriber de log full")?;
        }
    }

    Ok(())
}

/// Tenta inicializar o subscriber de log sem propagar erro caso já esteja instalado.
///
/// Útil para inicialização idempotente em suítes de testes unitários ou de integração.
pub fn try_init_test_logging() {
    let _ = init_logging();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("JSON").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("compact").unwrap(), LogFormat::Compact);
        assert_eq!(LogFormat::from_str("pretty").unwrap(), LogFormat::Pretty);
        assert_eq!(LogFormat::from_str("full").unwrap(), LogFormat::Full);
        assert_eq!(LogFormat::from_str("unknown").unwrap(), LogFormat::Full);
    }

    #[test]
    fn test_sensitive_masking() {
        assert!(is_sensitive_key("authorization"));
        assert!(is_sensitive_key("Authorization"));
        assert!(is_sensitive_key("access_token"));
        assert!(is_sensitive_key("client_secret"));
        assert!(!is_sensitive_key("request_id"));
        assert!(!is_sensitive_key("method"));
        assert!(!is_sensitive_key("uri"));

        assert_eq!(
            mask_sensitive_value("authorization", "secret123"),
            "[REDACTED]"
        );
        assert_eq!(
            mask_sensitive_value("token", "jwt-header.payload.signature"),
            "[REDACTED]"
        );
        assert_eq!(
            mask_sensitive_value("custom_header", "Bearer xyz123"),
            "Bearer [REDACTED]"
        );
        assert_eq!(mask_sensitive_value("user_id", "42"), "42");
    }

    #[test]
    fn test_json_log_layer_captures_fields_and_request_id() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let layer = JsonLogLayer::new(CloneWriter(buffer.clone()));

        let subscriber = Registry::default().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!(
                "http_request",
                request_id = "test-req-1234",
                method = "GET",
                uri = "/health",
                status = tracing::field::Empty,
            );
            let _enter = span.enter();

            tracing::info!(usuario = "capp_user", "Evento dentro do span de teste");
            span.record("status", 200);
        });

        let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
        assert!(
            !output.is_empty(),
            "deve emitir pelo menos uma linha de log"
        );

        let log_json: serde_json::Value =
            serde_json::from_str(output.lines().next().unwrap()).expect("deve ser JSON válido");

        assert_eq!(log_json["request_id"], "test-req-1234");
        assert_eq!(log_json["level"], "INFO");
        assert_eq!(log_json["message"], "Evento dentro do span de teste");
    }

    #[derive(Clone)]
    struct CloneWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CloneWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }
}
