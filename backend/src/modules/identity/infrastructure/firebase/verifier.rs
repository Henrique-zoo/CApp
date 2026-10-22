//! # Verificação de Firebase ID Tokens
//!
//! Este módulo implementa a autenticação de identidades Firebase por meio
//! da validação local de JSON Web Tokens (JWTs) assinados com RS256.
//!
//! ## Funcionamento
//!
//! O Firebase Authentication emite ID Tokens assinados digitalmente com
//! uma chave privada RSA. Cada token contém, em seu cabeçalho, um
//! identificador de chave (`kid`) que permite determinar qual chave
//! pública deve ser utilizada para verificar sua assinatura.
//!
//! O Firebase disponibiliza suas chaves públicas em um endpoint JWKS
//! (JSON Web Key Set), acessível por HTTPS.
//!
//! A verificação segue as seguintes etapas:
//!
//! 1. Extrai o `kid` do cabeçalho do JWT.
//! 2. Obtém a chave pública correspondente, utilizando o cache local ou,
//!    quando necessário, consultando o endpoint JWKS.
//! 3. Verifica a assinatura digital do token com a chave pública obtida.
//! 4. Valida as claims do token, incluindo emissor (`iss`), audiência
//!    (`aud`), expiração (`exp`) e identificador do usuário (`sub`).
//! 5. Extrai a identidade Firebase e o identificador Microsoft vinculado,
//!    quando disponível.
//!
//! ## Cache de chaves públicas
//!
//! Para evitar uma requisição HTTP a cada autenticação, as chaves públicas
//! são armazenadas em um cache compartilhado entre as instâncias clonadas
//! do verificador.
//!
//! Enquanto o cache estiver válido e contiver a chave solicitada, a
//! verificação é realizada inteiramente de forma local, sem comunicação
//! com o Firebase.
//!
//! A validade do cache é determinada a partir das diretivas HTTP de cache
//! retornadas pelo provedor. As atualizações são limitadas por um intervalo
//! mínimo para impedir que tokens com `kid` desconhecidos provoquem
//! requisições excessivas.
//!
//! A sincronização utiliza:
//!
//! - Um `RwLock` para permitir leituras concorrentes das chaves armazenadas.
//! - Um `Mutex` para serializar as atualizações do conjunto de chaves.
//!
//! As requisições HTTP são realizadas sem manter o bloqueio de leitura ou
//! escrita do cache, permitindo que outras tarefas continuem utilizando
//! chaves conhecidas enquanto estiverem válidas.
//!
//! ## Limitações
//!
//! A verificação local comprova a autenticidade e a validade do token,
//! mas não consulta o estado atual da conta nem verifica revogações.
//! Consequentemente, um token revogado pode continuar sendo aceito até
//! sua expiração.
//!
//! Este módulo também não verifica o vínculo institucional do usuário,
//! sua matrícula ou suas permissões no CApp. Essas responsabilidades
//! pertencem às demais etapas de autorização da aplicação.

use std::{
    collections::HashMap,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::http::HeaderMap;
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation,
    errors::ErrorKind,
    jwk::{AlgorithmParameters, JwkSet, KeyAlgorithm, PublicKeyUse},
};
use reqwest::{
    Client,
    header::{AGE, CACHE_CONTROL},
};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

use crate::modules::identity::application::{
    error::AuthenticationError,
    ports::{IdentityTokenVerifier, VerifiedIdentity},
};

/// Origem fixa e confiável das chaves de assinatura do Firebase Auth.
const JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

/// Intervalo de contenção para tentativas por falha ou `kid` desconhecido.
///
/// Quando o TTL é positivo e menor, a próxima tentativa é liberada ao expirar
/// o cache; com TTL zero, aplica-se o intervalo completo para evitar abuso.
const REFRESH_INTERVAL: Duration = Duration::from_secs(30);

/// Chaves RSA já preparadas para verificar assinaturas.
type Keys = HashMap<String, Arc<DecodingKey>>;

/// Adaptador Firebase, reutilizável entre requisições.
///
/// Clones compartilham o cache e o controle de atualização. Leituras de
/// chaves válidas podem ocorrer em paralelo, inclusive durante uma busca HTTP.
#[derive(Clone)]
pub struct FirebaseTokenVerifier {
    /// Cliente HTTP utilizado para buscar as chaves públicas do Firebase.
    http: Client,
    /// Endpoint público fixo em produção; testes podem usar um servidor local.
    jwks_url: String,
    /// Regras de validação dos Firebase ID tokens.
    validation: Validation,
    /// Estado do cache; permite leitores concorrentes e escrita exclusiva.
    cache: Arc<RwLock<KeyCache>>,
    /// Serializa atualizações HTTP, sem bloquear leitores de chaves válidas.
    refresh_lock: Arc<Mutex<()>>,
}

/// Estado do cache e do controle de novas tentativas.
struct KeyCache {
    /// Chaves públicas de verificação, indexadas pelo `kid` do JWT.
    keys: Keys,
    /// Instante em que as chaves deixam de ser consideradas válidas.
    valid_until: Instant,
    /// Instante a partir do qual uma nova atualização pode ser iniciada.
    refresh_after: Instant,
    /// Indica que a última atualização falhou ou ainda não foi concluída.
    last_refresh_failed: bool,
}

impl KeyCache {
    /// Consulta a chave e o estado do cache sem efetuar I/O.
    ///
    /// - `Ok(Some(key))`: chave disponível em cache válido;
    /// - `Ok(None)`: é permitido tentar atualizar o conjunto;
    /// - `Err(_)`: atualização temporariamente bloqueada.
    ///
    /// # Erros
    ///
    /// Durante a contenção, retorna `InvalidToken` para chave desconhecida com
    /// cache válido e último refresh bem-sucedido. Se o cache venceu ou houve
    /// falha de refresh, retorna `ProviderUnavailable`. Não altera o estado.
    fn lookup(
        &self,
        kid: &str,
        now: Instant,
    ) -> Result<Option<Arc<DecodingKey>>, AuthenticationError> {
        let fresh = now < self.valid_until;

        if fresh {
            if let Some(key) = self.keys.get(kid) {
                return Ok(Some(Arc::clone(key)));
            }
        }

        if now < self.refresh_after {
            return Err(if self.last_refresh_failed || !fresh {
                AuthenticationError::ProviderUnavailable
            } else {
                AuthenticationError::InvalidToken
            });
        }

        Ok(None)
    }
}

/// Claims necessárias à validação e ao vínculo de identidades.
///
/// A presença e os valores de `aud` e `iss` são verificados pelo
/// `jsonwebtoken`, usando a configuração de `Validation`.
#[derive(Deserialize)]
struct Claims {
    /// Identificador único do usuário no projeto Firebase (Firebase UID).
    sub: String,

    /// Data e hora de expiração do token, em segundos desde a época Unix.
    exp: u64,

    /// Data e hora de emissão do token, em segundos desde a época Unix.
    iat: u64,

    /// Data e hora da última autenticação do usuário, em segundos
    /// desde a época Unix.
    ///
    /// Pode ser anterior a `iat`, pois o Firebase pode emitir novos
    /// ID Tokens sem exigir que o usuário se autentique novamente.
    auth_time: u64,

    /// Informações específicas do Firebase, incluindo as identidades
    /// associadas aos provedores de autenticação.
    firebase: FirebaseClaims,
}

/// Identidades vinculadas, informadas em claim reservada do Firebase.
#[derive(Deserialize)]
struct FirebaseClaims {
    /// Identificadores externos associados ao usuário, agrupados por provedor.
    ///
    /// Cada chave identifica um provedor de autenticação (por exemplo,
    /// `microsoft.com`), e seu valor contém os identificadores vinculados
    /// àquele provedor.
    identities: HashMap<String, Vec<String>>,
}

impl FirebaseTokenVerifier {
    /// Configura verificação RS256 e um cache de chaves inicialmente vazio.
    ///
    /// # Parâmetros
    ///
    /// - `project_id`: audiência esperada e sufixo do emissor
    ///   `https://securetoken.google.com/<project_id>`; não é uma credencial secreta.
    /// - `http`: cliente reutilizável. A composição atual desabilita redirecionamentos;
    ///   a busca de chaves limita cada requisição a cinco segundos.
    ///
    /// # Retorno
    ///
    /// Verificador com `exp`, `iss`, `aud` e `sub` obrigatórios, sem tolerância temporal
    /// (`leeway = 0`) e com validação de `nbf` quando presente. As demais claims são
    /// validadas por `verify`. Não faz descoberta de projeto nem consulta de rede.
    ///
    /// # Erros
    ///
    /// Retorna erro se `project_id` estiver vazio ou contiver espaços em branco.
    /// Essa validação não comprova a existência ou a configuração externa do projeto.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use backend::modules::identity::infrastructure::firebase::FirebaseTokenVerifier;
    /// let http = reqwest::Client::builder()
    ///     .redirect(reqwest::redirect::Policy::none())
    ///     .build()?;
    /// let verifier = FirebaseTokenVerifier::new("capp-example".into(), http)?;
    /// let _shared_cache = verifier.clone();
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(project_id: String, http: Client) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !project_id.is_empty() && !project_id.chars().any(char::is_whitespace),
            "FIREBASE_PROJECT_ID deve conter um ID de projeto válido"
        );

        let mut validation = Validation::new(Algorithm::RS256);

        validation.set_audience(&[project_id.as_str()]);
        validation.set_issuer(&[format!("https://securetoken.google.com/{project_id}")]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

        // Exige validade temporal sem a tolerância padrão da biblioteca.
        validation.leeway = 0;
        validation.validate_nbf = true;

        let now = Instant::now();

        Ok(Self {
            http,
            jwks_url: JWKS_URL.to_owned(),
            validation,
            cache: Arc::new(RwLock::new(KeyCache {
                keys: HashMap::new(),
                valid_until: now,
                refresh_after: now,
                last_refresh_failed: false,
            })),
            refresh_lock: Arc::new(Mutex::new(())),
        })
    }

    /// Constrói um verificador com JWKS local exclusivamente para testes.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn new_with_endpoint(
        project_id: String,
        http: Client,
        jwks_url: String,
    ) -> anyhow::Result<Self> {
        let mut verifier = Self::new(project_id, http)?;
        verifier.jwks_url = jwks_url;
        Ok(verifier)
    }

    /// Obtém uma chave pública com leituras concorrentes e atualização única.
    ///
    /// As chaves são lidas sob `RwLock` e clonadas via `Arc`; a validação do
    /// JWT ocorre fora de qualquer lock. A atualização HTTP é serializada
    /// por `refresh_lock` e não bloqueia leitores de chaves ainda válidas.
    /// Leituras posteriores não reutilizam chaves de cache vencido. O resultado
    /// recém-baixado pode atender à chamada que o buscou mesmo com TTL zero;
    /// nesse caso não há prazo positivo para reutilização por outras chamadas.
    ///
    /// # Erros
    ///
    /// `InvalidToken` para `kid` ausente do conjunto obtido ou desconhecido durante
    /// a contenção com cache válido; `ProviderUnavailable` para falha de obtenção
    /// ou contenção com cache vencido. Cancelamento mantém a reserva de tentativa.
    async fn key_for(&self, kid: &str) -> Result<Arc<DecodingKey>, AuthenticationError> {
        // Caminho comum: somente leitura e sem requisição HTTP.
        let cache = self.cache.read().await;
        if let Some(key) = cache.lookup(kid, Instant::now())? {
            return Ok(key);
        }
        drop(cache);

        // Uma única tarefa pode atualizar o conjunto de chaves por vez.
        // Enquanto ela aguarda HTTP, outras tarefas ainda leem chaves válidas.
        let _refresh_guard = self.refresh_lock.lock().await;

        // Double-check: outra tarefa pode ter atualizado o cache enquanto
        // esta aguardava o mutex de atualização. Nesse caso, não será feita
        // uma segunda atualização redundante.
        let cache = self.cache.read().await;
        if let Some(key) = cache.lookup(kid, Instant::now())? {
            return Ok(key);
        }
        drop(cache);

        // Reserva a próxima tentativa. Em caso de falha (ou cancelamento),
        // o estado continua marcado e novas tentativas respeitam o intervalo.
        // O lock de escrita é liberado ANTES de iniciar o HTTP.
        let mut cache = self.cache.write().await;
        cache.refresh_after = Instant::now() + REFRESH_INTERVAL;
        cache.last_refresh_failed = true;
        drop(cache);

        let (keys, ttl) = self.fetch_keys().await?;
        let fetched_at = Instant::now();

        // Publica o novo conjunto e seus metadados em uma única escrita.
        // Desconta o tempo gasto aguardando o lock, para não prolongar o TTL.
        // A chave é clonada antes de liberar o lock.
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        let ttl = ttl.saturating_sub(fetched_at.elapsed());

        cache.keys = keys;
        cache.valid_until = now.checked_add(ttl).unwrap_or(now);

        // Para TTL positivo, não adiar o próximo refresh além da validade.
        // Para TTL zero, aplicar backoff para evitar uma busca por requisição.
        let retry_delay = if ttl.is_zero() {
            REFRESH_INTERVAL
        } else {
            REFRESH_INTERVAL.min(ttl)
        };
        cache.refresh_after = now + retry_delay;
        cache.last_refresh_failed = false;

        cache
            .keys
            .get(kid)
            .cloned()
            .ok_or(AuthenticationError::InvalidToken)
    }

    /// Baixa o JWKS público e prepara as chaves RSA aceitas para RS256.
    ///
    /// Desconsidera entradas de algoritmo/uso incompatível e sem `kid` utilizável.
    /// Calcula o TTL de `Cache-Control`, descontando `Age` e o tempo de processamento.
    /// O cliente HTTP limita a requisição a cinco segundos.
    ///
    /// # Retorno
    ///
    /// Mapa de chaves e duração restante, possivelmente zero. Não altera o cache;
    /// a publicação do conjunto é feita por `key_for` sob lock de escrita.
    ///
    /// # Erros
    ///
    /// Falha de HTTP, JSON, preparação de chave ou conjunto sem chaves utilizáveis é
    /// registrada e convertida em `ProviderUnavailable`.
    async fn fetch_keys(&self) -> Result<(Keys, Duration), AuthenticationError> {
        let response = self
            .http
            .get(&self.jwks_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(provider_unavailable)?
            .error_for_status()
            .map_err(provider_unavailable)?;

        let received_at = Instant::now();
        let ttl = cache_ttl(response.headers());

        let jwks: JwkSet = response.json().await.map_err(provider_unavailable)?;

        let mut keys = HashMap::new();

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

            let key = DecodingKey::from_jwk(&jwk).map_err(provider_unavailable)?;

            keys.insert(kid.clone(), Arc::new(key));
        }

        if keys.is_empty() {
            return Err(provider_unavailable("Resposta sem chaves RSA utilizáveis"));
        }

        Ok((keys, ttl.saturating_sub(received_at.elapsed())))
    }
}

#[async_trait::async_trait]
impl IdentityTokenVerifier for FirebaseTokenVerifier {
    /// Valida um Firebase ID token e extrai UID e vínculo Microsoft.
    ///
    /// Exige cabeçalho RS256 com `kid`, assinatura válida, audiência e emissor do
    /// projeto configurado e claims temporais válidas. Verifica UID não vazio com até
    /// 128 caracteres, emissão e autenticação não futuras, `auth_time <= iat` e
    /// `iat < exp`. A expiração é conferida também após a decodificação.
    ///
    /// O mapa `firebase.identities` pode não conter `microsoft.com`, resultando em
    /// `None`; quando presente, deve conter exatamente um identificador não vazio.
    /// A interpretação desse identificador como UUID cabe à aplicação institucional.
    ///
    /// # Retorno
    ///
    /// Identidade obtida exclusivamente de claims verificadas. Não consulta Graph,
    /// revogação ou situação atual da conta, e não cria sessão ou vínculo acadêmico.
    ///
    /// # Erros
    ///
    /// Expiração reconhecida retorna `ExpiredToken`; outras falhas de token retornam
    /// `InvalidToken`. A obtenção de chaves pode retornar `ProviderUnavailable`; relógio
    /// anterior à época Unix resulta em `Internal`. Os tokens não são registrados.
    async fn verify(&self, token: &str) -> Result<VerifiedIdentity, AuthenticationError> {
        let header =
            jsonwebtoken::decode_header(token).map_err(|_| AuthenticationError::InvalidToken)?;

        if header.alg != Algorithm::RS256 {
            return Err(AuthenticationError::InvalidToken);
        }

        let kid = header
            .kid
            .filter(|kid| !kid.is_empty())
            .ok_or(AuthenticationError::InvalidToken)?;

        let key = self.key_for(&kid).await?;

        let claims = jsonwebtoken::decode::<Claims>(token, key.as_ref(), &self.validation)
            .map_err(|error| match error.kind() {
                ErrorKind::ExpiredSignature => AuthenticationError::ExpiredToken,
                _ => AuthenticationError::InvalidToken,
            })?
            .claims;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthenticationError::Internal)?
            .as_secs();

        if claims.exp <= now {
            return Err(AuthenticationError::ExpiredToken);
        }

        if claims.sub.is_empty()
            || claims.sub.chars().count() > 128
            || claims.iat > now
            || claims.auth_time > now
            || claims.auth_time > claims.iat
            || claims.iat >= claims.exp
        {
            return Err(AuthenticationError::InvalidToken);
        }

        let microsoft_object_id = match claims.firebase.identities.get("microsoft.com") {
            None => None,
            Some(ids) => match ids.as_slice() {
                [id] if !id.is_empty() => Some(id.clone()),
                _ => return Err(AuthenticationError::InvalidToken),
            },
        };

        Ok(VerifiedIdentity {
            firebase_uid: claims.sub,
            microsoft_object_id,
        })
    }
}

/// Interpreta Cache-Control e desconta a idade informada pelo servidor.
///
/// Sem prazo explícito, com `no-store`/`no-cache` ou `max-age` inválido, retorna
/// duração zero. `Age` inválido equivale a zero; subtração usa saturação para
/// evitar duração negativa. Se houver vários `max-age`, prevalece o último
/// processado. Não realiza I/O nem implementa o protocolo HTTP completo de cache.
fn cache_ttl(headers: &HeaderMap) -> Duration {
    let control = headers
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let mut max_age = 0_u64;

    for directive in control.split(',').map(str::trim) {
        let name = directive.split('=').next().unwrap_or("").trim();

        if name.eq_ignore_ascii_case("no-store") || name.eq_ignore_ascii_case("no-cache") {
            return Duration::ZERO;
        }

        if let Some((name, value)) = directive.split_once('=') {
            if name.trim().eq_ignore_ascii_case("max-age") {
                max_age = value.trim().trim_matches('"').parse().unwrap_or(0);
            }
        }
    }

    let age = headers
        .get(AGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    Duration::from_secs(max_age.saturating_sub(age))
}

/// Registra somente falhas na obtenção de chaves públicas.
///
/// Não recebe nem registra tokens de usuários.
fn provider_unavailable(error: impl Display) -> AuthenticationError {
    tracing::warn!(
        %error,
        "Falha ao obter chaves públicas do Firebase"
    );

    AuthenticationError::ProviderUnavailable
}

#[cfg(test)]
#[path = "verifier_tests.rs"]
mod tests;
