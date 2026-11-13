//! Contratos JSON da verificação institucional.
//!
//! A entrada usa `microsoftAccessToken`; o ID token Firebase pertence ao cabeçalho
//! `Authorization`. A saída reúne `user` e `verification`; a evidência contém
//! `profile`, `institution` e `expiresAt` em camelCase, com instante UTC serializado
//! pelo Chrono em RFC 3339. Campos opcionais ausentes do perfil são serializados
//! como `null`; não há UID Firebase, senha ou credencial na resposta.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::modules::identity::{
    application::{
        ports::{InstitutionalAffiliation, InstitutionalProfile},
        services::authentication::{InstitutionalAuthResult, InstitutionalVerificationResult},
    },
    domain::entities::institution::Institution,
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

/// Resposta da verificação: conta provisionada e evidência não nula.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InstitutionalAuthResponse {
    /// Perfil interno do mesmo usuário cuja identidade institucional foi confirmada.
    pub user: UserResponse,

    /// Evidência recém-confirmada, sem expor credenciais ou UID Firebase.
    pub verification: InstitutionalVerificationResponse,
}

impl From<InstitutionalAuthResult> for InstitutionalAuthResponse {
    fn from(result: InstitutionalAuthResult) -> Self {
        Self {
            user: result.user.into(),
            verification: result.verification.into(),
        }
    }
}

/// Evidência institucional presente nas respostas de autenticação e restauração.
///
/// # Formato JSON
///
/// ```json
/// {
///   "profile": {
///     "microsoftObjectId": "00000000-0000-0000-0000-000000000001",
///     "name": "Estudante",
///     "email": null,
///     "affiliation": { "type": "student", "courseName": null }
///   },
///   "institution": {
///     "id": "00000000-0000-0000-0000-000000000001",
///     "name": "Instituição",
///     "acronym": null
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
    /// Perfil Microsoft retornado pela verificação, obtido durante o provisionamento ou sincronização.
    pub profile: InstitutionalIdentityResponse,

    pub institution: InstitutionSummaryResponse,

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

    /// Filiação à instituição e dados associados.
    pub affiliation: InstitutionalAffiliationResponse,
}

/// Vínculo público com discriminador JSON `type`.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(super) enum InstitutionalAffiliationResponse {
    /// Estudante reconhecido pelo grupo institucional.
    Student {
        /// Curso externo opcional, ainda sem associação ao catálogo.
        #[serde(rename = "courseName")]
        course_name: Option<String>,
    },
    /// Usuário cuja participação no grupo estudantil não foi confirmada.
    NonStudent,
}

impl From<InstitutionalAffiliation> for InstitutionalAffiliationResponse {
    /// Converte o vínculo interno para o contrato JSON sem duplicar o curso.
    fn from(affiliation: InstitutionalAffiliation) -> Self {
        match affiliation {
            InstitutionalAffiliation::Student { course_name } => Self::Student { course_name },
            InstitutionalAffiliation::NonStudent => Self::NonStudent,
        }
    }
}

impl From<InstitutionalProfile> for InstitutionalIdentityResponse {
    /// Move os atributos externos para os nomes públicos da API, sem novas consultas.
    fn from(profile: InstitutionalProfile) -> Self {
        Self {
            microsoft_object_id: profile.id,
            name: profile.name,
            email: profile.mail,
            affiliation: profile.affiliation.into(),
        }
    }
}

impl From<InstitutionalVerificationResult> for InstitutionalVerificationResponse {
    /// Converte perfil e validade sem recalcular o prazo ou duplicar strings.
    fn from(result: InstitutionalVerificationResult) -> Self {
        Self {
            profile: result.profile.into(),
            institution: result.institution.into(),
            expires_at: result.expires_at,
        }
    }
}

/// Resumo público da instituição identificada na verificação institucional.
///
/// Expõe os dados necessários para identificar e apresentar a instituição ao
/// cliente. Omite a configuração da integração Microsoft, como tenant e grupo
/// de estudantes.
///
/// O identificador pertence ao cadastro interno do CApp. A presença deste resumo
/// não comprova vínculo estudantil nem concede permissões; o vínculo é informado
/// separadamente no perfil retornado pela verificação.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionSummaryResponse {
    /// Identificador único da instituição no CApp, serializado como UUID textual.
    pub id: Uuid,

    /// Nome da instituição registrado no cadastro do CApp.
    pub name: String,

    /// Sigla cadastrada, como `UnB`; serializada como `null` quando ausente.
    pub acronym: Option<String>,
}

impl From<Institution> for InstitutionSummaryResponse {
    fn from(institution: Institution) -> Self {
        Self {
            id: institution.id,
            name: institution.name,
            acronym: institution.acronym,
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

/// Conta interna sem credenciais ou identificadores dos provedores.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserResponse {
    /// Complementos declarados para ausências do Graph, sem valor de evidência.
    name: Option<String>,
    institutional_email: Option<String>,
    course_name: Option<String>,
    id: Uuid,
    nickname: Option<String>,
    email: Option<String>,
    current_semester: Option<i32>,
    institution_entry_semester: Option<String>,
    status: &'static str,
}

impl From<crate::modules::identity::domain::entities::user::User> for UserResponse {
    fn from(user: crate::modules::identity::domain::entities::user::User) -> Self {
        use crate::modules::identity::domain::UserStatus;

        Self {
            id: user.id(),
            name: user.name().map(str::to_owned),
            institutional_email: user.institutional_email().map(str::to_owned),
            course_name: user.course_name().map(str::to_owned),
            nickname: user.nickname().map(str::to_owned),
            email: user.email().map(str::to_owned),
            current_semester: user.current_semester(),
            institution_entry_semester: user.institution_entry_semester().map(str::to_owned),
            status: match user.status() {
                UserStatus::Incomplete => "incomplete",
                UserStatus::Active => "active",
                UserStatus::Inactive => "inactive",
            },
        }
    }
}

/// Sessão reutilizável sem access token Microsoft; evidência nula exige sincronização.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AccountSessionResponse {
    pub user: UserResponse,
    pub verification: Option<InstitutionalVerificationResponse>,
}

/// Dados e complementos do perfil; não aceita status, UID, privilégios ou e-mail de login.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CompleteProfileRequest {
    /// Aceitos apenas se o atributo correspondente estiver ausente na evidência.
    pub name: Option<String>,
    pub institutional_email: Option<String>,
    pub course_name: Option<String>,
    pub nickname: String,
    pub current_semester: i32,
    pub institution_entry_semester: String,
}
