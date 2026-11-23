//! Verificação independente da origem do aplicativo pelo Firebase App Check.
//!
//! Um App Check token identifica o aplicativo atestado, não a pessoa autenticada.
//! O middleware HTTP usa este adaptador antes das operações do backend; Firebase
//! ID tokens e autorização institucional continuam sendo verificados separadamente.
//!
//! A validação exige RS256, typ JWT, assinatura, emissor e audiência do número do
//! projeto, expiração estrita, emissão não futura e App ID na lista permitida.
//! Chaves vêm de uma origem HTTPS fixa, respeitam Cache-Control/Age e ficam em
//! cache por no máximo seis horas. Clones compartilham cache e atualização;
//! falhas e kids desconhecidos têm contenção de trinta segundos. Chaves ainda
//! válidas continuam disponíveis durante uma atualização. Tokens nunca são logados.
//! O mecanismo está em [`super::jwks`], também utilizado por Firebase Auth. A
//! implementação é compartilhada, mas as origens e instâncias do cache permanecem
//! separadas. App Check usa fallback e teto de seis horas; o adaptador de ID tokens
//! só reutiliza chaves quando há prazo HTTP explícito. Claims e resultados deste
//! adaptador não dependem do contrato de autenticação de usuários.
//!
//! A verificação local não consome tokens de uso único nem detecta replay. A
//! ativação dos provedores e a fiscalização dos produtos Firebase são configuradas
//! no Console, independentemente da fiscalização deste servidor Rust.

use jsonwebtoken::{Algorithm, Validation};
use reqwest::Client;
use serde::Deserialize;
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::jwks::{CachePolicy, JwksCache, JwksError};

const JWKS_URL: &str = "https://firebaseappcheck.googleapis.com/v1/jwks";

const MAX_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// Resultado referente exclusivamente ao aplicativo, sem UID de usuário.
#[derive(Clone, Debug)]
pub struct VerifiedApp {
    /// App ID Firebase autenticado pela comprovação aceita pelo servidor.
    pub app_id: String,
}

/// Falhas públicas do adaptador, sem incluir tokens ou diagnósticos secretos.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppCheckError {
    /// Token, assinatura, claims ou App ID não aceitos.
    #[error("Comprovação do aplicativo inválida")]
    Invalid,

    /// Não foi possível obter chaves válidas ou consultar o relógio do processo.
    #[error("Verificação do aplicativo temporariamente indisponível")]
    Unavailable,
}

/// Verificador reutilizável de atestados dos aplicativos permitidos do projeto.
#[derive(Clone)]
pub struct AppCheckVerifier {
    keys: JwksCache,
    validation: Validation,
    app_ids: Arc<HashSet<String>>,
}

#[derive(Deserialize)]
struct Claims {
    sub: String,

    exp: u64,
    iat: u64,
}

impl AppCheckVerifier {
    /// Configura o projeto numérico e uma lista não vazia de App IDs desse projeto.
    /// Não realiza I/O; erros de configuração impedem a inicialização do servidor.
    pub fn new(project_number: String, app_ids: Vec<String>, http: Client) -> anyhow::Result<Self> {
        Self::build(project_number, app_ids, http, JWKS_URL.into())
    }

    /// Permite JWKS local exclusivamente em testes e na feature test-support.
    #[cfg(any(test, feature = "test-support"))]
    pub fn new_with_endpoint(
        project_number: String,
        app_ids: Vec<String>,
        http: Client,
        endpoint: String,
    ) -> anyhow::Result<Self> {
        Self::build(project_number, app_ids, http, endpoint)
    }

    fn build(
        project_number: String,
        app_ids: Vec<String>,
        http: Client,
        endpoint: String,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !project_number.starts_with('0')
                && project_number.bytes().all(|b| b.is_ascii_digit())
                && project_number.parse::<u64>().is_ok_and(|n| n > 0),
            "FIREBASE_PROJECT_NUMBER deve ser um número de projeto válido"
        );

        let prefix = format!("1:{project_number}:");

        anyhow::ensure!(
            !app_ids.is_empty()
                && app_ids.iter().all(|id| {
                    !id.chars().any(char::is_whitespace)
                        && id
                            .strip_prefix(&prefix)
                            .and_then(|tail| tail.split_once(':'))
                            .is_some_and(|(platform, value)| {
                                !platform.is_empty() && !value.is_empty()
                            })
                }),
            "FIREBASE_APP_CHECK_APP_IDS deve listar App IDs do projeto configurado"
        );

        let mut validation = Validation::new(Algorithm::RS256);

        validation.set_audience(&[format!("projects/{project_number}")]);
        validation.set_issuer(&[format!(
            "https://firebaseappcheck.googleapis.com/{project_number}"
        )]);

        validation.set_required_spec_claims(&["exp", "iat", "aud", "iss", "sub"]);

        validation.leeway = 0;
        validation.validate_nbf = true;

        Ok(Self {
            keys: JwksCache::new(
                http,
                endpoint,
                CachePolicy {
                    fallback_ttl: MAX_CACHE_TTL,
                    maximum_ttl: Some(MAX_CACHE_TTL),
                },
            ),
            validation,
            app_ids: Arc::new(app_ids.into_iter().collect()),
        })
    }

    /// Verifica a comprovação e retorna somente o App ID autenticado.
    /// Indisponibilidade de chaves não é convertida em aceitação ou rejeição do usuário.
    pub async fn verify(&self, token: &str) -> Result<VerifiedApp, AppCheckError> {
        if token.len() > 16_384 {
            return Err(AppCheckError::Invalid);
        }

        let header = jsonwebtoken::decode_header(token).map_err(|_| AppCheckError::Invalid)?;

        if header.alg != Algorithm::RS256 || header.typ.as_deref() != Some("JWT") {
            return Err(AppCheckError::Invalid);
        }

        let kid = header
            .kid
            .filter(|kid| !kid.is_empty())
            .ok_or(AppCheckError::Invalid)?;

        let key = self.keys.key_for(&kid).await.map_err(|error| match error {
            JwksError::UnknownKey => AppCheckError::Invalid,
            JwksError::Unavailable => AppCheckError::Unavailable,
        })?;

        let claims = jsonwebtoken::decode::<Claims>(token, key.as_ref(), &self.validation)
            .map_err(|_| AppCheckError::Invalid)?
            .claims;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppCheckError::Unavailable)?
            .as_secs();

        if claims.iat > now
            || claims.iat >= claims.exp
            || claims.exp <= now
            || !self.app_ids.contains(&claims.sub)
        {
            return Err(AppCheckError::Invalid);
        }

        Ok(VerifiedApp { app_id: claims.sub })
    }
}

#[cfg(test)]
mod tests;
