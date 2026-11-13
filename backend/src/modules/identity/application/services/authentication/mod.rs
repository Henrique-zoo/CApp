//! Casos de uso de verificação institucional e acesso com evidência persistida.
//!
//! [`verify_institutional_identity`] consulta os provedores e registra o resultado.
//! [`authenticate_institutional_request`] valida Firebase e reutiliza uma evidência
//! vigente, estudantil ou não, sem chamar o Graph. As funções não conhecem HTTP, `AppState`,
//! pool SQLx ou DTOs da API; os adaptadores implementam as traits recebidas.
//!
//! O TTL da evidência é independente do prazo do Firebase ID token e do cache de
//! chaves de assinatura. A verificação provisiona uma conta incompleta de forma idempotente; nenhuma função emite credenciais.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::super::ports::InstitutionalDirectory;
use crate::modules::identity::{
    application::{
        error::AuthenticationError,
        ports::{
            IdentityTokenVerifier, InstitutionRepository, InstitutionalAffiliation,
            InstitutionalProfile, InstitutionalVerificationRepository, UserRepository,
            VerifiedIdentity,
        },
    },
    domain::entities::{
        institution::Institution, institutional_verification::InstitutionalVerification, user::User,
    },
};

/// Prazo de reutilização da confirmação institucional.
///
/// Valor inicial de política; alterações passam a valer nas novas verificações.
pub const VERIFICATION_TTL_DAYS: i64 = 120;

/// Identidade autorizada por uma confirmação institucional ainda válida.
///
/// Não contém credenciais nem depende de um usuário interno provisionado.
#[derive(Debug, Clone)]
pub struct AuthenticatedIdentity {
    /// UID autenticado no Firebase.
    pub firebase_uid: String,

    /// Instituição cuja elegibilidade foi confirmada.
    pub institution_id: Uuid,
}

/// Resultado público após persistir a verificação de um estudante ou não estudante.
///
/// O perfil vem do Graph e a validade é a mesma atribuída à evidência enviada ao
/// repositório. A camada HTTP converte este tipo para seu próprio DTO. Não contém
/// tokens, senha ou UID Firebase duplicado. Integra as respostas de autenticação
/// e de restauração.
#[derive(Debug, Clone)]
pub struct InstitutionalVerificationResult {
    /// Perfil externo obtido e associado à identidade autenticada nesta operação.
    pub profile: InstitutionalProfile,

    /// Instituição associada ao usuário.
    pub institution: Institution,

    /// Fim da validade da confirmação institucional, em UTC.
    ///
    /// Não é a expiração da sessão Firebase. O cliente pode usá-lo para antecipar a
    /// renovação; a decisão de acesso permanece no backend.
    pub expires_at: DateTime<Utc>,
}

/// Conta provisionada e evidência produzidas pela mesma verificação institucional.
/// Evita uma consulta HTTP posterior apenas para obter o perfil interno.
#[derive(Debug, Clone)]
pub struct InstitutionalAuthResult {
    /// Conta nova ou existente, com seu estado preservado pelo provisionamento.
    pub user: User,

    /// Evidência persistida nesta operação, sempre presente em caso de sucesso.
    pub verification: InstitutionalVerificationResult,
}

/// Verifica identidade e elegibilidade, persiste a evidência e devolve a conta com sua verificação.
///
/// # Parâmetros
///
/// - `directory`: acesso ao perfil, tenant e participação em grupo Microsoft.
/// - `verifier`: validação do Firebase ID token para o projeto configurado.
/// - `repositories`: cadastro institucional e armazenamento da evidência.
/// - `firebase_token`: ID token Firebase sem prefixo `Bearer`.
/// - `microsoft_token`: access token Graph correspondente à mesma identidade.
///
/// # Processamento e persistência
///
/// Primeiro valida Firebase e interpreta o objeto Microsoft vinculado. Consulta
/// tenant, instituição e grupo, obtém o perfil com o vínculo correspondente e
/// confronta seu ID com o vínculo Firebase antes de persistir a evidência.
/// Nome, e-mail e curso ausentes não impedem a verificação;
/// `institution.status` não é usado como regra de acesso.
///
/// O prazo de 120 dias começa antes das consultas ao diretório. Somente respostas
/// conclusivas do grupo são persistidas. Uma negativa substitui a evidência anterior
/// se for mais recente e retorna um perfil não estudante; falhas de comunicação preservam
/// o registro existente sem estender sua validade. A ordem de atualização usa
/// `checked_at`; resultados concorrentes mais antigos não sobrescrevem os recentes.
/// Não há transação distribuída entre o Graph e o PostgreSQL. O usuário incompleto é
/// provisionado depois do confronto das identidades; uma falha na evidência pode
/// deixar a conta incompleta criada, sem conceder acesso. A repetição reutiliza seu ID.
///
/// # Retorno
///
/// Conta provisionada, perfil externo e validade da evidência registrada. Não
/// associa curso, emite token ou administra sessão. O relógio deve estar sincronizado
/// com os provedores e com as demais instâncias da aplicação.
///
/// # Erros
///
/// - Credencial inválida/expirada, objeto vinculado ausente ou identidades divergentes.
/// - `UnknownInstitution` quando o tenant não pertence ao cadastro.
/// - `InstitutionalVerificationRequired` se a verificação não substituiu
///   um registro com início igual ou posterior.
/// - `ProviderUnavailable` para falhas do diretório, inclusive ID de perfil inválido.
/// - `Internal` para falha de persistência; nesse caso não há resposta de sucesso.
///
/// O handler HTTP aplica seu próprio timeout. Cancelar a operação não equivale a
/// reverter uma escrita que o PostgreSQL já tenha concluído.
///
/// # Exemplos
///
/// O exemplo compila sem credenciais; executá-lo requer adaptadores configurados:
///
/// ```no_run
/// use backend::modules::identity::application::{
///     error::AuthenticationError,
///     ports::{IdentityTokenVerifier, InstitutionalDirectory, InstitutionRepository,
///         InstitutionalVerificationRepository, UserRepository},
///     services::authentication::verify_institutional_identity,
/// };
///
/// async fn verify<D, V, R>(directory: &D, verifier: &V, repositories: &R,
///     firebase_token: &str, microsoft_token: &str) -> Result<(), AuthenticationError>
/// where
///     D: InstitutionalDirectory,
///     V: IdentityTokenVerifier,
///     R: InstitutionRepository + InstitutionalVerificationRepository + UserRepository,
/// {
///     let result = verify_institutional_identity(
///         directory, verifier, repositories, firebase_token, microsoft_token,
///     ).await?;
///     assert!(result.verification.expires_at > chrono::Utc::now());
///     Ok(())
/// }
/// ```
pub async fn verify_institutional_identity(
    directory: &impl InstitutionalDirectory,
    verifier: &impl IdentityTokenVerifier,
    repositories: &(impl InstitutionRepository + InstitutionalVerificationRepository + UserRepository),
    firebase_token: &str,
    microsoft_token: &str,
) -> Result<InstitutionalAuthResult, AuthenticationError> {
    let identity = verifier.verify(firebase_token).await?;
    let expected_id = microsoft_object_id(&identity)?;

    // O TTL começa antes das consultas externas, sem incluir tempo adicional
    // de validade por atrasos na comunicação ou na persistência.
    let checked_at = Utc::now();

    let tenant_id = directory.get_tenant_id(microsoft_token).await?;

    let institution = repositories
        .find_by_microsoft_tenant_id(tenant_id)
        .await?
        .ok_or(AuthenticationError::UnknownInstitution)?;

    let is_student = directory
        .is_student_member(microsoft_token, institution.microsoft_student_group_id)
        .await?;

    let profile = directory.get_profile(microsoft_token, is_student).await?;
    let actual_id =
        Uuid::parse_str(&profile.id).map_err(|_| AuthenticationError::ProviderUnavailable)?;

    if actual_id != expected_id {
        return Err(AuthenticationError::IdentityMismatch);
    }

    let user = repositories.provision(&identity.firebase_uid).await?;

    if user.status() == crate::modules::identity::domain::UserStatus::Inactive {
        return Err(AuthenticationError::AccountInactive);
    }

    let verification = InstitutionalVerification {
        user_id: user.id(),
        profile_name: profile.name.clone(),
        profile_email: profile.mail.clone(),
        course_name: match &profile.affiliation {
            InstitutionalAffiliation::Student { course_name } => course_name.clone(),
            InstitutionalAffiliation::NonStudent => None,
        },
        microsoft_object_id: actual_id,
        institution_id: institution.id,
        microsoft_tenant_id: tenant_id,
        microsoft_student_group_id: institution.microsoft_student_group_id,
        student_membership_confirmed: is_student,
        checked_at,
        expires_at: checked_at + Duration::days(VERIFICATION_TTL_DAYS),
    };

    // Uma resposta negativa conclusiva também precisa substituir uma
    // confirmação estudantil anterior, atualizando o vínculo registrado.
    let saved = repositories.save(&verification).await?;

    if !saved {
        return Err(AuthenticationError::InstitutionalVerificationRequired);
    }

    Ok(InstitutionalAuthResult {
        user,
        verification: InstitutionalVerificationResult {
            profile,
            institution,
            expires_at: verification.expires_at,
        },
    })
}

/// Autentica uma requisição usando evidência institucional já persistida.
///
/// # Parâmetros
///
/// - `verifier`: validação Firebase; pode atualizar as chaves públicas se necessário.
/// - `repository`: consulta da evidência vigente para o UID autenticado.
/// - `firebase_token`: ID token sem prefixo HTTP. Não recebe access token Microsoft.
///
/// # Retorno
///
/// UID Firebase e instituição autenticados para a requisição.
/// Confronta novamente o objeto vinculado e verifica a expiração após a consulta,
/// sem atualizar o registro ou prorrogar seu prazo. Não consulta o Graph nem cria
/// usuário interno. A identidade retornada não concede permissões administrativas.
///
/// # Erros
///
/// Propaga erros do verificador. Retorna `InvalidMicrosoftId` se o vínculo não puder
/// ser interpretado; `InstitutionalVerificationRequired` se não houver evidência
/// válida, se ela expirar durante a consulta ou se o objeto Microsoft divergir;
/// e `Internal` para falha de persistência. O caller HTTP limita o tempo total.
///
/// # Exemplos
///
/// ```no_run
/// use backend::modules::identity::application::{
///     error::AuthenticationError,
///     ports::{IdentityTokenVerifier, InstitutionalVerificationRepository, UserRepository},
///     services::authentication::authenticate_institutional_request,
/// };
///
/// async fn authorize<V: IdentityTokenVerifier, R: InstitutionalVerificationRepository>(
///     verifier: &V, repository: &R, token: &str,
/// ) -> Result<String, AuthenticationError> {
///     let identity = authenticate_institutional_request(verifier, repository, token).await?;
///     Ok(identity.firebase_uid)
/// }
/// ```
pub async fn authenticate_institutional_request(
    verifier: &impl IdentityTokenVerifier,
    repository: &impl InstitutionalVerificationRepository,
    firebase_token: &str,
) -> Result<AuthenticatedIdentity, AuthenticationError> {
    let identity = verifier.verify(firebase_token).await?;
    let object_id = microsoft_object_id(&identity)?;

    let verification = repository
        .find_valid_by_firebase_uid(&identity.firebase_uid)
        .await?
        .ok_or(AuthenticationError::InstitutionalVerificationRequired)?;

    // Não reutiliza evidência de uma identidade Microsoft diferente.
    // Confere também a expiração após a conclusão da consulta.

    if verification.microsoft_object_id != object_id || verification.expires_at <= Utc::now() {
        return Err(AuthenticationError::InstitutionalVerificationRequired);
    }

    Ok(AuthenticatedIdentity {
        firebase_uid: identity.firebase_uid,
        institution_id: verification.institution_id,
    })
}

/// Interpreta o identificador Microsoft da identidade Firebase verificada.
///
/// # Erros
///
/// Retorna erro quando o identificador está ausente ou não é um UUID válido.
fn microsoft_object_id(identity: &VerifiedIdentity) -> Result<Uuid, AuthenticationError> {
    identity
        .microsoft_object_id
        .as_deref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or(AuthenticationError::InvalidMicrosoftId)
}

#[cfg(test)]
mod tests;
