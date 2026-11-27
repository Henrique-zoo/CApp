//! Obtenção e cache de chaves públicas para os adaptadores Firebase de identidade.
//!
//! ID tokens de usuários e atestados App Check usam emissores e conjuntos de
//! chaves distintos, mas precisam da mesma coordenação de downloads. Este módulo
//! privado concentra HTTP, interpretação de JWKS e cache, sem receber JWTs,
//! interpretar claims ou depender dos erros dos contratos de autenticação.
//! Cada adaptador fixa seu endpoint e traduz [`JwksError`] para seu contrato.
//!
//! # Concorrência e validade
//!
//! Clones de um cache compartilham chaves e um mutex de atualização. Caches
//! construídos separadamente permanecem isolados, inclusive quando os provedores
//! usam o mesmo `kid`. Uma chave conhecida e válida pode ser lida durante HTTP;
//! os demais leitores aguardam a atualização em curso antes de consultar a
//! contenção. Assim, o carregamento inicial não produz indisponibilidade artificial.
//!
//! O prazo vem de Cache-Control/Age, com fallback e teto explicitados pelo
//! adaptador. Diretivas conflitantes usam o menor max-age; no-cache, no-store ou
//! prazo inválido impedem reutilização. O tempo de processamento também é
//! descontado. TTL zero atende apenas à busca atual. Falha ou cancelamento
//! conserva uma reserva de trinta segundos, evitando um download por requisição.
//!
//! Somente chaves RSA de assinatura compatíveis com RS256 e `kid` não vazio são
//! utilizáveis. Duplicatas de `kid` invalidam a resposta inteira. Uma busca que
//! falha não substitui chaves anteriores ainda válidas. Logs se limitam a falhas
//! de obtenção das chaves públicas; este componente nunca recebe credenciais.

use std::{
    collections::HashMap,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use jsonwebtoken::{
    DecodingKey,
    jwk::{AlgorithmParameters, JwkSet, KeyAlgorithm, PublicKeyUse},
};
use reqwest::{
    Client,
    header::{AGE, CACHE_CONTROL, HeaderMap},
};
use tokio::sync::{Mutex, RwLock};

const REFRESH_INTERVAL: Duration = Duration::from_secs(30);

type Keys = HashMap<String, Arc<DecodingKey>>;

/// Política explícita para respostas sem prazo e para o limite do provedor.
#[derive(Clone, Copy)]
pub(super) struct CachePolicy {
    /// Prazo usado somente quando nenhuma diretiva max-age foi informada.
    pub(super) fallback_ttl: Duration,

    /// Teto aplicado ao prazo informado ou ao fallback, antes de descontar Age.
    pub(super) maximum_ttl: Option<Duration>,
}

/// Resultado técnico independente de usuário, aplicativo e protocolo HTTP da API.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(super) enum JwksError {
    /// O conjunto válido não possui a chave solicitada.
    #[error("Chave de assinatura desconhecida")]
    UnknownKey,

    /// Não há chave válida e uma busca falhou ou está em contenção.
    #[error("Chaves de assinatura temporariamente indisponíveis")]
    Unavailable,
}

/// Cache de um único endpoint; construir não acessa a rede.
#[derive(Clone)]
pub(super) struct JwksCache {
    http: Client,
    endpoint: String,
    policy: CachePolicy,

    cache: Arc<RwLock<KeyCache>>,
    refresh_lock: Arc<Mutex<()>>,
}

struct KeyCache {
    keys: Keys,
    valid_until: Instant,

    refresh_after: Instant,
    last_refresh_failed: bool,
}

impl KeyCache {
    /// Consulta somente após adquirir o mutex de atualização ou no teste direto.
    fn lookup(&self, kid: &str, now: Instant) -> Result<Option<Arc<DecodingKey>>, JwksError> {
        let fresh = now < self.valid_until;

        if fresh && let Some(key) = self.keys.get(kid) {
            return Ok(Some(Arc::clone(key)));
        }

        if now < self.refresh_after {
            return Err(if self.last_refresh_failed || !fresh {
                JwksError::Unavailable
            } else {
                JwksError::UnknownKey
            });
        }

        Ok(None)
    }
}

impl JwksCache {
    /// Recebe uma origem confiável escolhida pelo adaptador e um cliente reutilizável.
    /// O cliente de produção deve recusar redirecionamentos. Cada busca tem timeout
    /// de cinco segundos; não aceita um endereço retirado do cabeçalho de um JWT.
    pub(super) fn new(http: Client, endpoint: String, policy: CachePolicy) -> Self {
        let now = Instant::now();

        Self {
            http,
            endpoint,
            policy,
            cache: Arc::new(RwLock::new(KeyCache {
                keys: Keys::new(),
                valid_until: now,
                refresh_after: now,
                last_refresh_failed: false,
            })),
            refresh_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Obtém uma chave utilizável sem manter locks durante a validação do JWT.
    ///
    /// Leitores de chaves válidas não aguardam HTTP. Os demais compartilham a
    /// atualização em curso e só depois avaliam backoff. Uma busca com TTL zero
    /// pode atender a chamada atual, sem autorizar reutilização posterior.
    /// UnknownKey distingue ausência em um conjunto válido de indisponibilidade.
    pub(super) async fn key_for(&self, kid: &str) -> Result<Arc<DecodingKey>, JwksError> {
        {
            let cache = self.cache.read().await;

            if Instant::now() < cache.valid_until
                && let Some(key) = cache.keys.get(kid)
            {
                return Ok(Arc::clone(key));
            }
        }

        let _refresh = self.refresh_lock.lock().await;

        if let Some(key) = self.cache.read().await.lookup(kid, Instant::now())? {
            return Ok(key);
        }

        {
            let mut cache = self.cache.write().await;

            cache.last_refresh_failed = true;
            cache.refresh_after = Instant::now() + REFRESH_INTERVAL;
        }

        let (keys, ttl) = self.fetch_keys().await?;
        let fetched_at = Instant::now();

        let mut cache = self.cache.write().await;
        let now = Instant::now();
        let ttl = ttl.saturating_sub(fetched_at.elapsed());

        cache.keys = keys;
        cache.valid_until = now.checked_add(ttl).unwrap_or(now);
        cache.refresh_after = now
            + if ttl.is_zero() {
                REFRESH_INTERVAL
            } else {
                ttl.min(REFRESH_INTERVAL)
            };
        cache.last_refresh_failed = false;

        cache.keys.get(kid).cloned().ok_or(JwksError::UnknownKey)
    }

    /// Prepara um conjunto completo antes de publicá-lo; resposta inválida não
    /// modifica o cache. Entradas incompatíveis são ignoradas e duplicatas de
    /// chaves utilizáveis são rejeitadas para evitar escolha dependente da ordem.
    async fn fetch_keys(&self) -> Result<(Keys, Duration), JwksError> {
        let response = self
            .http
            .get(&self.endpoint)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(unavailable)?
            .error_for_status()
            .map_err(unavailable)?;

        let received_at = Instant::now();
        let ttl = cache_ttl(response.headers(), self.policy);

        let jwks: JwkSet = response.json().await.map_err(unavailable)?;

        let mut keys = Keys::new();

        for jwk in jwks.keys {
            if !matches!(&jwk.algorithm, AlgorithmParameters::RSA(_))
                || jwk
                    .common
                    .key_algorithm
                    .is_some_and(|alg| alg != KeyAlgorithm::RS256)
                || jwk
                    .common
                    .public_key_use
                    .as_ref()
                    .is_some_and(|usage| usage != &PublicKeyUse::Signature)
            {
                continue;
            }

            let Some(kid) = jwk.common.key_id.as_ref().filter(|kid| !kid.is_empty()) else {
                continue;
            };

            let key = DecodingKey::from_jwk(&jwk).map_err(unavailable)?;

            if keys.insert(kid.clone(), Arc::new(key)).is_some() {
                return Err(unavailable("JWKS com kid duplicado"));
            }
        }

        if keys.is_empty() {
            return Err(unavailable("JWKS sem chaves RSA utilizáveis"));
        }

        Ok((keys, ttl.saturating_sub(received_at.elapsed())))
    }
}

/// Interpreta todos os cabeçalhos Cache-Control e escolhe o prazo mais curto.
fn cache_ttl(headers: &HeaderMap, policy: CachePolicy) -> Duration {
    let mut max_age: Option<Duration> = None;

    for value in headers.get_all(CACHE_CONTROL) {
        let Ok(value) = value.to_str() else {
            return Duration::ZERO;
        };

        for directive in value.split(',').map(str::trim) {
            let name = directive.split('=').next().unwrap_or("").trim();

            if name.eq_ignore_ascii_case("no-store") || name.eq_ignore_ascii_case("no-cache") {
                return Duration::ZERO;
            }

            if let Some((name, value)) = directive.split_once('=')
                && name.trim().eq_ignore_ascii_case("max-age")
            {
                let Ok(seconds) = value.trim().trim_matches('"').parse::<u64>() else {
                    return Duration::ZERO;
                };

                let value = Duration::from_secs(seconds);

                max_age = Some(max_age.map_or(value, |previous| previous.min(value)));
            }
        }
    }

    let ttl = max_age.unwrap_or(policy.fallback_ttl);
    let ttl = policy.maximum_ttl.map_or(ttl, |maximum| ttl.min(maximum));

    let age = headers
        .get(AGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    ttl.saturating_sub(Duration::from_secs(age))
}

/// Registra somente problemas no download público, sem tokens nem kids recebidos.
fn unavailable(error: impl Display) -> JwksError {
    tracing::warn!(%error, "Falha ao obter chaves públicas Firebase");

    JwksError::Unavailable
}

#[cfg(test)]
mod tests;
