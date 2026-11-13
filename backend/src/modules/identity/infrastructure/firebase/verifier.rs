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
//! Essa implementação pertence ao módulo privado [`super::jwks`], também usado
//! pelo App Check. Cada verificador possui um cache independente e sua própria
//! política: este adaptador exige max-age explícito; o App Check admite fallback
//! de até seis horas. Cabeçalhos repetidos usam o menor prazo e kids duplicados
//! são rejeitados. Leitores sem chave aguardam a atualização em curso antes de
//! avaliar a contenção; uma busca inicial bem-sucedida atende todos os leitores
//! concorrentes enquanto as chaves forem reutilizáveis.
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
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, Validation, errors::ErrorKind};
use reqwest::Client;
use serde::Deserialize;

use super::jwks::{CachePolicy, JwksCache, JwksError};
use crate::modules::identity::application::{
    error::AuthenticationError,
    ports::{IdentityTokenVerifier, VerifiedIdentity},
};

/// Origem fixa e confiável das chaves de assinatura do Firebase Auth.
const JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

/// Este adaptador só reutiliza chaves quando o provedor informa prazo HTTP.
const CACHE_POLICY: CachePolicy = CachePolicy {
    fallback_ttl: Duration::ZERO,
    maximum_ttl: None,
};

/// Adaptador Firebase; clones compartilham somente o cache deste emissor.
#[derive(Clone)]
pub struct FirebaseTokenVerifier {
    keys: JwksCache,
    validation: Validation,
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

    #[serde(default)]
    email: Option<String>,
}

/// Identidades vinculadas, informadas em claim reservada do Firebase.
#[derive(Deserialize)]
struct FirebaseClaims {
    #[serde(default)]
    sign_in_provider: Option<String>,

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

        Ok(Self {
            keys: JwksCache::new(http, JWKS_URL.to_owned(), CACHE_POLICY),
            validation,
        })
    }

    /// Constrói um verificador com JWKS local exclusivamente para testes.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn new_with_endpoint(
        project_id: String,
        http: Client,
        jwks_url: String,
    ) -> anyhow::Result<Self> {
        let mut verifier = Self::new(project_id, http.clone())?;

        verifier.keys = JwksCache::new(http, jwks_url, CACHE_POLICY);

        Ok(verifier)
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

        let key = self.keys.key_for(&kid).await.map_err(|error| match error {
            JwksError::UnknownKey => AuthenticationError::InvalidToken,
            JwksError::Unavailable => AuthenticationError::ProviderUnavailable,
        })?;

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
            email: claims.email,
            sign_in_provider: claims.firebase.sign_in_provider,
        })
    }
}

#[cfg(test)]
#[path = "verifier_tests.rs"]
mod tests;
