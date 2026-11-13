//! Restauração e conclusão de uma conta já provisionada, sem consultar Microsoft.
//!
//! O Firebase autentica a requisição. A conta interna e a evidência persistida
//! determinam o perfil e o prazo de sincronização; cache cliente não é evidência.
//! Um perfil incompleto pode ser retomado, mas somente uma credencial password
//! validada no mesmo UID permite concluir o cadastro. Nenhuma senha chega aqui.
//! Complementos opcionais preenchem apenas ausências do Graph no perfil interno:
//! os dados externos não nulos e a evidência persistida nunca são substituídos.

use super::authentication::InstitutionalVerificationResult;
use crate::modules::identity::{
    application::{error::AuthenticationError, ports::*},
    domain::{
        UserStatus,
        entities::user::{ProfileCompletion, User},
    },
};

/// Fotografia autenticada da conta e de sua confirmação institucional vigente.
pub struct AccountSession {
    /// Conta interna pertencente ao UID do token verificado.
    pub user: User,

    /// Ausência exige sincronização; não significa que o Firebase expirou.
    pub verification: Option<InstitutionalVerificationResult>,
}

/// Restaura somente contas provisionadas. Nunca cria conta com UID do cliente.
///
/// Ausência de conta produz AccountNotProvisioned, permitindo ao cliente iniciar
/// a primeira confirmação. Uma conta existente sem evidência vigente é retornada
/// com verification nulo. Recusas e falhas de infraestrutura propagam: não são
/// confundidas com primeiro acesso nem autorizam contornar a consulta via Graph.
pub async fn restore(
    verifier: &impl IdentityTokenVerifier,
    repositories: &(impl UserRepository + InstitutionRepository + InstitutionalVerificationRepository),
    token: &str,
) -> Result<AccountSession, AuthenticationError> {
    let identity = verifier.verify(token).await?;
    let user = repositories
        .get_by_firebase_uid(&identity.firebase_uid)
        .await?
        .ok_or(AuthenticationError::AccountNotProvisioned)?;

    if user.status() == UserStatus::Inactive {
        return Err(AuthenticationError::AccountInactive);
    }

    let evidence = repositories
        .find_valid_by_firebase_uid(&identity.firebase_uid)
        .await?;
    let verification = match evidence {
        Some(evidence) => {
            let object_id = identity
                .microsoft_object_id
                .as_deref()
                .and_then(|id| uuid::Uuid::parse_str(id).ok())
                .ok_or(AuthenticationError::InvalidMicrosoftId)?;

            if object_id != evidence.microsoft_object_id || evidence.user_id != user.id() {
                return Err(AuthenticationError::IdentityMismatch);
            }

            if evidence.expires_at <= chrono::Utc::now() {
                None
            } else {
                let institution = repositories
                    .find_by_microsoft_tenant_id(evidence.microsoft_tenant_id)
                    .await?
                    .ok_or(AuthenticationError::UnknownInstitution)?;

                Some(InstitutionalVerificationResult {
                    profile: InstitutionalProfile {
                        id: evidence.microsoft_object_id.to_string(),
                        name: evidence.profile_name,
                        mail: evidence.profile_email,
                        affiliation: if evidence.student_membership_confirmed {
                            InstitutionalAffiliation::Student {
                                course_name: evidence.course_name,
                            }
                        } else {
                            InstitutionalAffiliation::NonStudent
                        },
                    },
                    institution,
                    expires_at: evidence.expires_at,
                })
            }
        }
        None => None,
    };

    Ok(AccountSession { user, verification })
}

/// Conclui/atualiza o perfil após senha Firebase e confirmação institucional.
/// O e-mail vem do JWT; é independente do endereço institucional do Graph.
pub async fn complete_profile(
    verifier: &impl IdentityTokenVerifier,
    repositories: &(impl UserRepository + InstitutionalVerificationRepository),
    token: &str,
    data: ProfileCompletion,
) -> Result<User, AuthenticationError> {
    let identity = verifier.verify(token).await?;

    if identity.sign_in_provider.as_deref() != Some("password") {
        return Err(AuthenticationError::PasswordRequired);
    }

    let email = identity
        .email
        .as_deref()
        .filter(|email| !email.is_empty())
        .ok_or(AuthenticationError::PasswordRequired)?;
    let mut user = repositories
        .get_by_firebase_uid(&identity.firebase_uid)
        .await?
        .ok_or(AuthenticationError::InstitutionalVerificationRequired)?;

    if user.status() == UserStatus::Inactive {
        return Err(AuthenticationError::AccountInactive);
    }

    let evidence = repositories
        .find_valid_by_firebase_uid(&identity.firebase_uid)
        .await?
        .ok_or(AuthenticationError::InstitutionalVerificationRequired)?;
    let object_id = identity
        .microsoft_object_id
        .as_deref()
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .ok_or(AuthenticationError::InvalidMicrosoftId)?;

    if evidence.microsoft_object_id != object_id || evidence.expires_at <= chrono::Utc::now() {
        return Err(AuthenticationError::InstitutionalVerificationRequired);
    }

    if evidence.user_id != user.id() {
        return Err(AuthenticationError::IdentityMismatch);
    }

    validate_supplements(&data, &evidence)?;
    user.complete_profile(data, email, chrono::Utc::now())
        .map_err(|_| AuthenticationError::InvalidProfile)?;

    if !repositories.update_profile(&user).await? {
        return Err(AuthenticationError::AccountInactive);
    }

    Ok(user)
}

/// Campos preenchidos pelo Graph não podem ser substituídos pelo formulário.
/// Complementos nunca mudam a evidência ou tornam um não estudante em estudante.
fn validate_supplements(
    data: &ProfileCompletion,
    evidence: &crate::modules::identity::domain::entities::institutional_verification::InstitutionalVerification,
) -> Result<(), AuthenticationError> {
    if (data.name.is_some() && evidence.profile_name.is_some())
        || (data.institutional_email.is_some() && evidence.profile_email.is_some())
        || (data.course_name.is_some()
            && (evidence.course_name.is_some() || !evidence.student_membership_confirmed))
    {
        return Err(AuthenticationError::InvalidProfile);
    }

    Ok(())
}
