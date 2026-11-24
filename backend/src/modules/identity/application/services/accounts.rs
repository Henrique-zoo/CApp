//! Restauração e conclusão de uma conta já provisionada, sem consultar Microsoft.
//!
//! O Firebase autentica a requisição. A conta interna e a evidência persistida
//! determinam o perfil e o prazo de sincronização; cache cliente não é evidência.
//! Um perfil incompleto pode ser retomado, mas somente uma credencial password
//! validada no mesmo UID permite concluir o cadastro. Nenhuma senha chega aqui.
//! Um nome declarado preenche somente uma ausência do Graph no perfil interno,
//! sem substituir os dados externos ou a evidência persistida. O e-mail escolhido
//! vem do Firebase; o endereço de sugestão do Graph não é armazenado nem restaurado.
//!
//! # Participação no fluxo
//!
//! [`restore`] permite entrar com uma conta já provisionada sem renovar a consulta
//! institucional. Ela retorna o perfil incompleto para retomada do cadastro ou o
//! perfil ativo para uso do aplicativo. A ausência de evidência vigente acompanha
//! a conta como `None`, permitindo que a coordenação solicite sincronização.
//! [`complete_profile`] recebe os dados acadêmicos depois de vincular e autenticar
//! a credencial de senha no Firebase; só então valida e persiste a conclusão.
//!
//! Os serviços recebem portas, não clientes HTTP ou conexões SQL. O verificador
//! fornece a identidade confiável; os repositórios resolvem a conta e a evidência.
//! [`User::complete_profile`] concentra as invariantes estruturais, enquanto este
//! módulo confere a origem dos dados, a propriedade da evidência e sua validade.
//! Nenhuma operação emite tokens, recebe senha ou usa cache mobile como prova.

use super::authentication::InstitutionalVerificationResult;
use crate::modules::identity::{
    application::{
        error::AuthenticationError,
        ports::{
            IdentityTokenVerifier, InstitutionRepository, InstitutionalAffiliation,
            InstitutionalProfile, InstitutionalVerificationRepository, UserRepository,
            VerifiedIdentity,
        },
    },
    domain::{
        UserStatus,
        entities::{
            institutional_verification::InstitutionalVerification,
            user::{ProfileCompletion, User},
        },
    },
};

/// Fotografia autenticada da conta e de sua confirmação institucional vigente.
///
/// Não representa uma sessão criada pelo backend. A API converte a conta e a
/// evidência em DTOs; o cliente decide se deve retomar o perfil ou sincronizar.
/// A conta pode existir com verificação ausente, inclusive após sua expiração.
/// A sugestão de e-mail do primeiro acesso não faz parte desta restauração.
pub struct AccountSession {
    /// Conta interna pertencente ao UID do token verificado.
    pub user: User,

    /// Ausência exige sincronização; não significa que o Firebase expirou.
    pub verification: Option<InstitutionalVerificationResult>,
}

/// Recupera a conta do UID autenticado e reconstitui sua evidência reutilizável.
///
/// # Parâmetros
///
/// - `verifier`: valida o Firebase ID token e fornece UID e vínculo Microsoft.
/// - `repositories`: consulta a conta, a evidência vigente e a instituição. A
///   porta de evidência deve filtrar expiração e alterações de tenant/grupo.
/// - `token`: Firebase ID token sem o prefixo HTTP `Bearer`; não é token Graph.
///
/// # Retorno e efeitos
///
/// Devolve [`AccountSession`] para uma conta existente e não desativada. Um perfil
/// incompleto também é retornado: a conclusão do cadastro pertence a outra etapa.
/// `verification: None` significa ausência de evidência reutilizável, não expiração
/// da autenticação Firebase. A função confere novamente a expiração após a leitura.
///
/// Havendo evidência, confronta seu proprietário e objeto Microsoft com a conta
/// autenticada antes de reconstruir perfil e instituição. Evidências antigas com
/// curso ausente ou em branco são reconstituídas com acesso não estudantil.
/// O e-mail do Graph não é recuperado; o e-mail interno permanece na conta.
///
/// Não consulta o Graph, provisiona conta, escreve nos repositórios ou estende o
/// prazo institucional. Falhas técnicas não são convertidas em conta ausente.
///
/// # Erros
///
/// - Propaga recusas e indisponibilidade do verificador Firebase.
/// - [`AuthenticationError::AccountNotProvisioned`] se o UID ainda não tem conta.
/// - [`AuthenticationError::AccountInactive`] se a conta está desativada.
/// - [`AuthenticationError::InvalidMicrosoftId`] se há evidência, mas o vínculo
///   Microsoft do token está ausente ou não é um UUID.
/// - [`AuthenticationError::IdentityMismatch`] se a evidência pertence a outro
///   usuário interno ou objeto Microsoft.
/// - [`AuthenticationError::UnknownInstitution`] se o tenant da evidência não
///   permite recuperar o cadastro institucional.
/// - [`AuthenticationError::Internal`] em falhas de persistência.
///
/// # Exemplos
///
/// O exemplo compila; a execução depende de adaptadores configurados:
///
/// ```no_run
/// use backend::modules::identity::application::{
///     error::AuthenticationError,
///     ports::{IdentityTokenVerifier, InstitutionRepository,
///         InstitutionalVerificationRepository, UserRepository},
///     services::accounts::restore,
/// };
///
/// async fn needs_institutional_sync<V, R>(verifier: &V, repositories: &R,
///     token: &str) -> Result<bool, AuthenticationError>
/// where
///     V: IdentityTokenVerifier,
///     R: UserRepository + InstitutionRepository + InstitutionalVerificationRepository,
/// {
///     let session = restore(verifier, repositories, token).await?;
///     Ok(session.verification.is_none())
/// }
/// ```
pub async fn restore(
    verifier: &impl IdentityTokenVerifier,
    repositories: &(impl UserRepository + InstitutionRepository + InstitutionalVerificationRepository),
    token: &str,
) -> Result<AccountSession, AuthenticationError> {
    let (user, identity) = authenticate_account(verifier, repositories, token).await?;

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

            if object_id != evidence.microsoft_object_id() || evidence.user_id() != user.id() {
                return Err(AuthenticationError::IdentityMismatch);
            }

            if evidence.expires_at() <= chrono::Utc::now() {
                None
            } else {
                let institution = repositories
                    .find_by_microsoft_tenant_id(evidence.microsoft_tenant_id())
                    .await?
                    .ok_or(AuthenticationError::UnknownInstitution)?;

                Some(InstitutionalVerificationResult {
                    profile: InstitutionalProfile {
                        id: evidence.microsoft_object_id().to_string(),
                        name: evidence.profile_name().map(ToString::to_string),
                        mail: None,
                        affiliation: InstitutionalAffiliation::from_directory(
                            evidence.student_membership_confirmed(),
                            evidence.course_name().map(ToString::to_string),
                        ),
                    },
                    institution,
                    expires_at: evidence.expires_at(),
                })
            }
        }
        None => None,
    };

    Ok(AccountSession { user, verification })
}

/// Resolve a conta CApp a partir do token Firebase. A conta precisa estar
/// provisionada e disponível. A consulta de evidência acadêmica cabe à
/// operação que exige vínculo, ao passo que recuperar um recibo exige a conta.
pub(crate) async fn authenticate_account(
    verifier: &impl IdentityTokenVerifier,
    repository: &impl UserRepository,
    token: &str,
) -> Result<(User, VerifiedIdentity), AuthenticationError> {
    let identity = verifier.verify(token).await?;
    let user = repository
        .get_by_firebase_uid(&identity.firebase_uid)
        .await?
        .ok_or(AuthenticationError::AccountNotProvisioned)?;

    if user.status() == UserStatus::Inactive {
        return Err(AuthenticationError::AccountInactive);
    }

    Ok((user, identity))
}

/// Valida a origem dos dados e persiste a conclusão ou atualização do perfil.
///
/// # Parâmetros
///
/// - `verifier`: valida o token e identifica o provedor que autenticou a sessão.
/// - `repositories`: recupera conta/evidência e grava o perfil validado.
/// - `token`: Firebase ID token emitido após autenticação pelo provedor `password`
///   da mesma conta institucional; não inclui prefixo `Bearer`.
/// - `data`: apelido, período atual, ingresso e eventual nome declarado para uma
///   ausência do Graph. A credencial e a senha já foram tratadas no Firebase.
///
/// # Validação, persistência e retorno
///
/// O e-mail interno vem do JWT validado, não do formulário ou da sugestão Graph.
/// Exige conta não desativada e evidência vigente do mesmo usuário e objeto
/// Microsoft. A confirmação pode ser não estudantil: concluir o perfil não cria
/// direitos de estudante nem modifica a evidência institucional.
///
/// Após conferir a origem do nome declarado, delega as invariantes a
/// [`User::complete_profile`] e solicita a escrita ao repositório. A porta deve
/// recusar uma conta desativada concorrentemente. O retorno é a fotografia
/// atualizada em memória após a confirmação da escrita, sem uma releitura SQL.
/// Repetir a conclusão preserva os identificadores e a primeira data de conclusão.
/// Não consulta o Graph, renova evidência ou modifica credenciais Firebase.
///
/// # Erros
///
/// - Propaga falhas do verificador e converte falhas dos repositórios em
///   [`AuthenticationError::Internal`].
/// - [`AuthenticationError::PasswordRequired`] se a sessão não foi autenticada
///   por senha ou não contém e-mail não vazio.
/// - [`AuthenticationError::InstitutionalVerificationRequired`] se falta conta
///   ou evidência, ela expirou ou seu objeto Microsoft diverge do token.
/// - [`AuthenticationError::InvalidMicrosoftId`] para vínculo Microsoft ausente
///   ou malformado; [`AuthenticationError::IdentityMismatch`] para outro `user_id`.
/// - [`AuthenticationError::InvalidProfile`] para dados inválidos ou tentativa
///   de fornecer nome quando o Graph já o informou.
/// - [`AuthenticationError::AccountInactive`] para conta desativada na leitura
///   ou recusa da atualização condicional pelo repositório.
///
/// # Exemplos
///
/// A senha é configurada no cliente antes desta chamada. O exemplo exige
/// adaptadores e um token do provedor correto para ser executado:
///
/// ```no_run
/// use backend::modules::identity::{
///     application::{
///         error::AuthenticationError,
///         ports::{IdentityTokenVerifier, InstitutionalVerificationRepository, UserRepository},
///         services::accounts::complete_profile,
///     },
///     domain::entities::user::{ProfileCompletion, User},
/// };
///
/// async fn finish<V, R>(verifier: &V, repositories: &R, password_id_token: &str)
///     -> Result<User, AuthenticationError>
/// where
///     V: IdentityTokenVerifier,
///     R: UserRepository + InstitutionalVerificationRepository,
/// {
///     complete_profile(verifier, repositories, password_id_token, ProfileCompletion {
///         nickname: "Pessoa".into(), current_semester: 4,
///         institution_entry_semester: "2024.1".into(), name: None,
///     }).await
/// }
/// ```
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

    if evidence.microsoft_object_id() != object_id || evidence.expires_at() <= chrono::Utc::now() {
        return Err(AuthenticationError::InstitutionalVerificationRequired);
    }

    if evidence.user_id() != user.id() {
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

/// Impede que o nome declarado substitua um atributo já informado pelo Graph.
///
/// `data` contém a solicitação do cliente e `evidence` é o registro confiável já
/// associado à conta. Qualquer `Some` no nome é recusado se a evidência também
/// contém nome, mesmo quando os textos coincidem ou estão vazios. A normalização
/// do complemento permitido cabe a [`User::complete_profile`].
///
/// Retorna [`AuthenticationError::InvalidProfile`] para essa substituição e
/// `Ok(())` quando o complemento é admissível. Não modifica argumentos, verifica
/// expiração ou concede vínculo estudantil; essas responsabilidades pertencem
/// às demais etapas do caso de uso.
fn validate_supplements(
    data: &ProfileCompletion,
    evidence: &InstitutionalVerification,
) -> Result<(), AuthenticationError> {
    if data.name.is_some() && evidence.profile_name().is_some() {
        return Err(AuthenticationError::InvalidProfile);
    }

    Ok(())
}
