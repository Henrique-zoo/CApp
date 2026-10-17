//! Validação de Firebase ID tokens para o fluxo de identidade.
//!
//! O adaptador verifica a assinatura e as claims configuradas para o projeto
//! Firebase e exige que o token declare `microsoft.com` como provedor de login.
//! Essa claim não comprova, isoladamente, que a conta pertence ao tenant
//! institucional da UnB; essa restrição depende da configuração institucional
//! do Firebase Authentication e do Microsoft Entra ID.

use std::{
    collections::HashMap,
    env,
    sync::OnceLock,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::modules::identity::application::queries::current_identity::{
    AuthenticatedIdentity, IdentityTokenVerifier, IdentityVerificationError,
};

const CERTIFICATES_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, thiserror::Error)]
pub(crate) enum FirebaseTokenError {
    #[error("Token de autenticação inválido")]
    InvalidToken,

    #[error("FIREBASE_PROJECT_ID não configurado")]
    MissingProjectId,

    #[error("Não foi possível obter as chaves públicas do Firebase")]
    SigningKeysUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FirebaseIdentity {
    pub(crate) uid: String,
    pub(crate) email: Option<String>,
    pub(crate) email_verified: bool,
}

#[derive(Debug, Deserialize)]
struct FirebaseClaims {
    aud: String,
    iss: String,
    sub: String,
    exp: u64,
    iat: u64,
    auth_time: u64,

    #[serde(default)]
    email: Option<String>,

    #[serde(default)]
    email_verified: bool,

    firebase: FirebaseMetadata,
}

#[derive(Debug, Deserialize)]
struct FirebaseMetadata {
    sign_in_provider: String,
}

struct CachedCertificates {
    certificates: HashMap<String, String>,
    expires_at: Instant,
}

fn certificate_cache() -> &'static RwLock<Option<CachedCertificates>> {
    static CACHE: OnceLock<RwLock<Option<CachedCertificates>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(None))
}

fn certificate_cache_ttl(response: &reqwest::Response) -> Duration {
    response
        .headers()
        .get(reqwest::header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value.split(',').find_map(|directive| {
                directive
                    .trim()
                    .strip_prefix("max-age=")
                    .and_then(|seconds| seconds.parse::<u64>().ok())
            })
        })
        .map(|seconds| Duration::from_secs(seconds.min(86_400)))
        .unwrap_or(DEFAULT_CACHE_TTL)
}

async fn fetch_certificates(
    force_refresh: bool,
) -> Result<HashMap<String, String>, FirebaseTokenError> {
    let cache = certificate_cache();

    if !force_refresh {
        let guard = cache.read().await;

        if let Some(cached) = guard.as_ref()
            && cached.expires_at > Instant::now()
        {
            return Ok(cached.certificates.clone());
        }
    }

    let mut guard = cache.write().await;

    if !force_refresh
        && let Some(cached) = guard.as_ref()
        && cached.expires_at > Instant::now()
    {
        return Ok(cached.certificates.clone());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| FirebaseTokenError::SigningKeysUnavailable)?;

    let response = client
        .get(CERTIFICATES_URL)
        .send()
        .await
        .map_err(|_| FirebaseTokenError::SigningKeysUnavailable)?;

    if !response.status().is_success() {
        return Err(FirebaseTokenError::SigningKeysUnavailable);
    }

    let ttl = certificate_cache_ttl(&response);

    let certificates = response
        .json::<HashMap<String, String>>()
        .await
        .map_err(|_| FirebaseTokenError::SigningKeysUnavailable)?;

    if certificates.is_empty() {
        return Err(FirebaseTokenError::SigningKeysUnavailable);
    }

    *guard = Some(CachedCertificates {
        certificates: certificates.clone(),
        expires_at: Instant::now() + ttl,
    });

    Ok(certificates)
}

/// Valida o identificador do projeto Firebase configurado.
///
/// Valores ausentes ou compostos apenas por espaços são considerados
/// configuração ausente. Espaços acidentais nas extremidades são removidos.
fn validate_project_id(project_id: &str) -> Result<String, FirebaseTokenError> {
    let project_id = project_id.trim();

    if project_id.is_empty() {
        return Err(FirebaseTokenError::MissingProjectId);
    }

    Ok(project_id.to_owned())
}

/// Verifica um Firebase ID token e recupera a identidade autenticada.
pub(crate) async fn verify_id_token(token: &str) -> Result<FirebaseIdentity, FirebaseTokenError> {
    let header = decode_header(token).map_err(|_| FirebaseTokenError::InvalidToken)?;

    if header.alg != Algorithm::RS256 {
        return Err(FirebaseTokenError::InvalidToken);
    }

    let kid = header
        .kid
        .filter(|kid| !kid.trim().is_empty())
        .ok_or(FirebaseTokenError::InvalidToken)?;

    let raw_project_id =
        env::var("FIREBASE_PROJECT_ID").map_err(|_| FirebaseTokenError::MissingProjectId)?;
    let project_id = validate_project_id(&raw_project_id)?;

    let certificates = fetch_certificates(false).await?;

    let certificate = match certificates.get(&kid) {
        Some(certificate) => certificate.clone(),
        None => {
            let refreshed = fetch_certificates(true).await?;

            refreshed
                .get(&kid)
                .cloned()
                .ok_or(FirebaseTokenError::InvalidToken)?
        }
    };

    validate_signed_id_token(token, &certificate, &project_id)
}

/// Valida a assinatura e as reivindicações de um token usando um certificado confiável.
///
/// A busca e a rotação dos certificados ficam em [`verify_id_token`]. Separar
/// essas operações permite testar a validação criptográfica sem acesso à rede.
fn validate_signed_id_token(
    token: &str,
    certificate: &str,
    project_id: &str,
) -> Result<FirebaseIdentity, FirebaseTokenError> {
    let header = decode_header(token).map_err(|_| FirebaseTokenError::InvalidToken)?;

    if header.alg != Algorithm::RS256
        || header
            .kid
            .as_deref()
            .is_none_or(|kid| kid.trim().is_empty())
    {
        return Err(FirebaseTokenError::InvalidToken);
    }

    let decoding_key = DecodingKey::from_rsa_pem(certificate.as_bytes())
        .map_err(|_| FirebaseTokenError::SigningKeysUnavailable)?;

    let expected_issuer = format!("https://securetoken.google.com/{project_id}");

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[project_id]);
    validation.set_issuer(&[expected_issuer.as_str()]);
    validation.leeway = 0;
    validation.required_spec_claims = ["aud", "iss", "sub", "exp", "iat", "auth_time", "firebase"]
        .iter()
        .map(|claim| (*claim).to_owned())
        .collect();

    let decoded = decode::<FirebaseClaims>(token, &decoding_key, &validation)
        .map_err(|_| FirebaseTokenError::InvalidToken)?;

    let claims = decoded.claims;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| FirebaseTokenError::InvalidToken)?
        .as_secs();

    if claims.aud != project_id
        || claims.iss != expected_issuer
        || claims.sub.trim().is_empty()
        || claims.sub.len() > 128
        || claims.exp <= now
        || claims.iat > now
        || claims.auth_time > now
        || claims.firebase.sign_in_provider != "microsoft.com"
    {
        return Err(FirebaseTokenError::InvalidToken);
    }

    Ok(FirebaseIdentity {
        uid: claims.sub,
        email: claims.email,
        email_verified: claims.email_verified,
    })
}

/// Adaptador de infraestrutura para Firebase Authentication.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FirebaseIdentityTokenVerifier;

#[async_trait]
impl IdentityTokenVerifier for FirebaseIdentityTokenVerifier {
    async fn verify_id_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedIdentity, IdentityVerificationError> {
        match crate::infrastructure::firebase::verify_id_token(token).await {
            Ok(identity) => Ok(AuthenticatedIdentity {
                uid: identity.uid,
                email: identity.email,
                email_verified: identity.email_verified,
            }),
            Err(FirebaseTokenError::InvalidToken) => Err(IdentityVerificationError::InvalidToken),
            Err(FirebaseTokenError::MissingProjectId) => {
                Err(IdentityVerificationError::MissingConfiguration)
            }
            Err(FirebaseTokenError::SigningKeysUnavailable) => {
                Err(IdentityVerificationError::ProviderUnavailable)
            }
        }
    }
}

#[cfg(test)]
mod token_validation_tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::Serialize;

    use super::{FirebaseTokenError, validate_project_id, validate_signed_id_token};

    const PROJECT_ID: &str = "capp-test-project";

    // Chave e certificado de teste autossinados. Nunca usar em produção.
    const TEST_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAlRrUd7yR/s5zRtMBmBM++7Ix2acpAOOlnfzWEB5Arsf8Bvjp
RX4aIIgp9WC7ITbV1WLkNDALUwsD8JM9yZNZWV3yyl/VY0GFAoPqhajI3lQKr4sA
6zvpdEOLF83+130DoqCMrR8UxaueJ/rbwXex1C6ryF+nE66OEnevNbVt6MzKFKbB
5nNWjD5TgmIvaVaHw+1MYh1RumpnesFrdg5wLAhNuc/IAqhNI6G6inhUKbB+TYU0
Yj7R0lyGrUXZo9fnQzvRQfma93lZA8fEe9J+udVcRxbP6AkYHf/7cJ/+XSWgPok2
ZKunZv/SVP3HqlKuvf/soBP0NINtS4Q5BoscOQIDAQABAoIBABPxsZ94f4t2laXQ
vfhXrrI1F4Trs/OJ/70JCksVTi/y0RZ6VJ2L12EoRXssdAC72Htr+Ga1/HXLAJJn
545fIXGaPEpvP4LrddKseYIPsxqbYY3ciS7DO0q8FCQG1QKBltW50EdXOSHoVEXj
Fgz5U+CJWPH66sVvsPztub0fO7M1U7PjQXwroos0zPLSlzBrhj0oShpJJ6xfY2Eg
/rrhUHQADkTOqrcYY83DbpP5fEI1wP1GST7Pyloh76BxFK01KYNGVYPI22QOZTLM
JKX/ODbseXweCAqkwxPgk2Bw35eHZUK4mljlOkVm+v5eqCFPZuH02d2SbusOJdMB
Jxyt7r8CgYEAzNa7UY8iexNkkXiv500CtSOKFdtS4E8LKMBvbW9u/1XttnZCmsaz
MoZ6IzJox6uptcgXSu0quojBmdAGkOHUKRl2RnJIj6QWwCGK3Spvg0/i3d6FB93k
QxbuopHuB6Jb7u7DEZ3y3jOicrLcCpyHTP1JZnlqZg6uMqzDABjJV9MCgYEAuliA
K2T+eUawX13PaBo5MwzVL3jFRnhwTVQzTcnND56WnmI7NmG6i3cSWbpd1fJ7y20U
l+oQvg82+EMNnamevLbvQWGb6lzbq8mEZQJbYNKWIvI/Vg8xKL6EiB2Evd+3NVI7
JisXStWsbztxsmLN0wz5o12+00Cp8tHU8hsuYEMCgYEAhN0fgUZzWht5FvaOOlx0
TiPwND8foYkDtvojDHvEJnCJENniV9xcbYBWhq/y0Nii6tsPaLUvCvEbinYpXB5u
6/2s/wKv2CPZ8eOP2+bjj+voPntQFIszvOSKAv7ewyrnsLJD6xBlcQVS8EzddzCS
fjioPISyo+IP0aTHky+AoasCgYAYG+EhPCTMTf/9pqZx4oqp9ywRwz9KucRVzOIC
W2yDa9Iy1aSxiX1YE+S05S7EwmWP0SIL8YWPAWrsPciiMFLQOZ7Ex7sjXDYziJPv
eZmW8kfoZ40YNkj264c8EQ2efzY12Ni8PHD8Bp8TmQ5BtRTKDHCz2SoaLNCQEOPL
Gwcp/wKBgQC+zGCdz4qUrRtD0mfVO9z+ckLjRqtHELywznbRFk1D/3ZSWreIb5w9
CUZkIIWXg6JCIVlwMeW+tilV7PTSdUIHgpVBXPUD4oQ2o004OqfreUYR7dQOApnw
J/fTsFqrEKx7ne5agEanQ7optPP3AoXC2Qss5oJBQS1vL7zwzkEFtg==
-----END RSA PRIVATE KEY-----"#;

    const TEST_CERTIFICATE: &str = r#"-----BEGIN CERTIFICATE-----
MIIC/TCCAeWgAwIBAgIUI0hTVXYjDJFEZYiBWI+ylZ+lKbIwDQYJKoZIhvcNAQEL
BQAwLjEsMCoGA1UEAwwjQ0FwcCBGaXJlYmFzZSB2ZXJpZmllciB0ZXN0IGZpeHR1
cmUwHhcNMjYxMDEzMjMxNjUyWhcNMzYxMDExMjMxNjUyWjAuMSwwKgYDVQQDDCND
QXBwIEZpcmViYXNlIHZlcmlmaWVyIHRlc3QgZml4dHVyZTCCASIwDQYJKoZIhvcN
AQEBBQADggEPADCCAQoCggEBAJUa1He8kf7Oc0bTAZgTPvuyMdmnKQDjpZ381hAe
QK7H/Ab46UV+GiCIKfVguyE21dVi5DQwC1MLA/CTPcmTWVld8spf1WNBhQKD6oWo
yN5UCq+LAOs76XRDixfN/td9A6KgjK0fFMWrnif628F3sdQuq8hfpxOujhJ3rzW1
bejMyhSmweZzVow+U4JiL2lWh8PtTGIdUbpqZ3rBa3YOcCwITbnPyAKoTSOhuop4
VCmwfk2FNGI+0dJchq1F2aPX50M70UH5mvd5WQPHxHvSfrnVXEcWz+gJGB3/+3Cf
/l0loD6JNmSrp2b/0lT9x6pSrr3/7KAT9DSDbUuEOQaLHDkCAwEAAaMTMBEwDwYD
VR0TAQH/BAUwAwEB/zANBgkqhkiG9w0BAQsFAAOCAQEAhW8qeNVQBd6cvGr94mG4
vFzLH2wtfWwykWDRe4zGcZ4aTdR2xhgkZ4kwJL9L9anNuFImKrRXybzBUY4oMsYd
UbFfChz0EfgcBWMDE+so8YI5/avKFeWyCAtyq6qcCjvB7xRehoFxMRlshgW8NAc4
RcSpp8aJevHHpOKIFp1By2k3b7bVnQPtDOEDfD8NWqJQuzHBfp8rchMz2MvkKZmm
TuJkqBjxISYy9DIKP15g5qHggnZlHye+GJEDK2mR2K4UtZfpWe7bv27ZsqROpu+t
3mcJtpuKuXU6k9atXEdUmAVNFnvzK9VQ5+hLNlM4/lGlRGM8ohZRdftzT8O9Pous
Ww==
-----END CERTIFICATE-----"#;

    #[derive(Serialize)]
    struct TestClaims {
        aud: String,
        iss: String,
        sub: String,
        exp: u64,
        iat: u64,
        auth_time: u64,
        email: Option<String>,
        email_verified: bool,
        firebase: TestFirebaseMetadata,
    }

    #[derive(Serialize)]
    struct TestFirebaseMetadata {
        sign_in_provider: String,
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock should be after Unix epoch")
            .as_secs()
    }

    fn valid_claims(now: u64) -> TestClaims {
        TestClaims {
            aud: PROJECT_ID.to_owned(),
            iss: format!("https://securetoken.google.com/{PROJECT_ID}"),
            sub: "firebase-test-user".to_owned(),
            exp: now + 3600,
            iat: now.saturating_sub(60),
            auth_time: now.saturating_sub(120),
            email: Some("student@example.test".to_owned()),
            email_verified: true,
            firebase: TestFirebaseMetadata {
                sign_in_provider: "microsoft.com".to_owned(),
            },
        }
    }

    fn create_token(claims: &TestClaims) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("firebase-test-key".to_owned());

        let encoding_key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes())
            .expect("test private key should be valid");

        encode(&header, claims, &encoding_key).expect("test token should be generated")
    }

    fn verify(token: &str) -> Result<super::FirebaseIdentity, FirebaseTokenError> {
        validate_signed_id_token(token, TEST_CERTIFICATE, PROJECT_ID)
    }

    #[test]
    fn rejects_empty_project_id() {
        assert!(matches!(
            validate_project_id(""),
            Err(FirebaseTokenError::MissingProjectId)
        ));
    }

    #[test]
    fn rejects_whitespace_only_project_id() {
        assert!(matches!(
            validate_project_id("   \t "),
            Err(FirebaseTokenError::MissingProjectId)
        ));
    }

    #[test]
    fn trims_project_id() {
        assert_eq!(
            validate_project_id(" capp-test-project ").expect("project ID should be valid"),
            PROJECT_ID
        );
    }

    #[test]
    fn accepts_valid_microsoft_token_signed_by_x509_certificate() {
        let token = create_token(&valid_claims(now()));
        let identity = verify(&token).expect("valid token should be accepted");

        assert_eq!(identity.uid, "firebase-test-user");
        assert_eq!(identity.email.as_deref(), Some("student@example.test"));
        assert!(identity.email_verified);
    }

    #[test]
    fn rejects_expired_token() {
        let current_time = now();
        let mut claims = valid_claims(current_time);
        claims.exp = current_time.saturating_sub(10);
        claims.iat = current_time.saturating_sub(3600);
        claims.auth_time = current_time.saturating_sub(3600);

        let token = create_token(&claims);

        assert!(matches!(
            verify(&token),
            Err(FirebaseTokenError::InvalidToken)
        ));
    }

    #[test]
    fn rejects_wrong_audience() {
        let mut claims = valid_claims(now());
        claims.aud = "another-project".to_owned();
        let token = create_token(&claims);

        assert!(matches!(
            verify(&token),
            Err(FirebaseTokenError::InvalidToken)
        ));
    }

    #[test]
    fn rejects_wrong_issuer() {
        let mut claims = valid_claims(now());
        claims.iss = "https://securetoken.google.com/another-project".to_owned();
        let token = create_token(&claims);

        assert!(matches!(
            verify(&token),
            Err(FirebaseTokenError::InvalidToken)
        ));
    }

    #[test]
    fn rejects_non_microsoft_sign_in_provider() {
        let mut claims = valid_claims(now());
        claims.firebase.sign_in_provider = "password".to_owned();
        let token = create_token(&claims);

        assert!(matches!(
            verify(&token),
            Err(FirebaseTokenError::InvalidToken)
        ));
    }

    #[test]
    fn rejects_empty_subject() {
        let mut claims = valid_claims(now());
        claims.sub = "  ".to_owned();
        let token = create_token(&claims);

        assert!(matches!(
            verify(&token),
            Err(FirebaseTokenError::InvalidToken)
        ));
    }

    #[test]
    fn rejects_tampered_signature() {
        let token = create_token(&valid_claims(now()));
        let mut parts: Vec<String> = token.split('.').map(str::to_owned).collect();
        assert_eq!(parts.len(), 3);

        let first = parts[2]
            .chars()
            .next()
            .expect("signature should not be empty");
        parts[2].replace_range(0..1, if first == 'A' { "B" } else { "A" });

        assert!(matches!(
            verify(&parts.join(".")),
            Err(FirebaseTokenError::InvalidToken)
        ));
    }
}
