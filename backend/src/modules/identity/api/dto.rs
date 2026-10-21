//! Contratos JSON da verificação institucional.
//!
//! A entrada usa `microsoftAccessToken`; o ID token Firebase pertence ao cabeçalho
//! `Authorization`. A saída contém `profile` e `expiresAt` em camelCase, com instante
//! UTC serializado pelo Chrono em RFC 3339. Campos opcionais ausentes do perfil são
//! serializados como `null`; não há UID, senha ou credencial na resposta.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::modules::identity::application::{
    ports::InstitutionalProfile, services::authentication::InstitutionalVerificationResult,
};

/// Corpo de `POST /auth/institutional/verify`.
///
/// Campo ausente, `null` ou string vazia resulta em `None`; o handler o rejeita com
/// 400. Valores de outros tipos são rejeitados pelo Serde. A string não é aparada:
/// um valor composto só por espaços não é normalizado para ausência.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstitutionalAuthRequest {
    /// Credencial delegada do Graph, correspondente à identidade do ID token Firebase.
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub microsoft_access_token: Option<String>,
}

/// Resposta HTTP 200 da verificação institucional concluída e persistida.
///
/// # Formato JSON
///
/// ```json
/// {
///   "profile": {
///     "microsoftObjectId": "00000000-0000-0000-0000-000000000001",
///     "name": "Estudante",
///     "email": null,
///     "userPrincipalName": "estudante@example.edu",
///     "courseName": null
///   },
///   "expiresAt": "2026-10-20T21:00:00Z"
/// }
/// ```
///
/// A validade é um auxílio ao cliente; não substitui a checagem no backend e não
/// representa a expiração da sessão Firebase.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstitutionalVerificationResponse {
    /// Perfil Microsoft retornado pela verificação, sem provisionamento interno.
    pub profile: InstitutionalIdentityResponse,
    /// Prazo UTC atribuído à evidência persistida; serializado como `expiresAt`.
    pub expires_at: DateTime<Utc>,
}

/// Identidade validada pela operação de verificação institucional.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstitutionalIdentityResponse {
    /// Identificador correspondente no diretório Microsoft.
    pub microsoft_object_id: String,
    /// Nome informado pela instituição.
    pub name: Option<String>,
    /// E-mail informado pelo diretório.
    pub email: Option<String>,
    /// Nome de entrada da conta institucional.
    pub user_principal_name: Option<String>,
    /// Curso informado pela instituição, quando disponível.
    pub course_name: Option<String>,
}

impl From<InstitutionalProfile> for InstitutionalIdentityResponse {
    /// Move os atributos externos para os nomes públicos da API, sem novas consultas.
    fn from(profile: InstitutionalProfile) -> Self {
        Self {
            microsoft_object_id: profile.id,
            name: profile.name,
            email: profile.mail,
            user_principal_name: profile.user_principal_name,
            course_name: profile.course_name,
        }
    }
}

impl From<InstitutionalVerificationResult> for InstitutionalVerificationResponse {
    /// Converte perfil e validade sem recalcular o prazo ou duplicar strings.
    fn from(result: InstitutionalVerificationResult) -> Self {
        Self {
            profile: InstitutionalIdentityResponse::from(result.profile),
            expires_at: result.expires_at,
        }
    }
}

/// Desserializa `null` ou string vazia como ausência, preservando outros textos.
///
/// A ausência do campo é tratada por `serde(default)` na struct. Não aplica `trim`
/// e não valida a credencial; espaços permanecem no valor entregue ao handler.
///
/// # Erros
///
/// Propaga o erro do Serde quando o valor não puder ser lido como `Option<String>`.
fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.filter(|text| !text.is_empty()))
}
