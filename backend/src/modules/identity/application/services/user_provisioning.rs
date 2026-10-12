//! Serviço de aplicação para provisionamento automático do usuário (`UserProvisioningService`).
//!
//! Coordena as regras de primeiro acesso, criação do perfil interno no domínio da aplicação,
//! associação unívoca com a identidade externa (Firebase/Microsoft), inicialização de preferências
//! padrão e preservação estrita do isolamento de privilégios.
//!
//! # Regras e Invariantes (§ 1.2 do Relatório de Features / Issue #7 / I01):
//! - **Primeiro acesso**: Cria automaticamente o perfil em `app_user` sem formulário manual.
//! - **Idempotência**: Acessos posteriores reutilizam o perfil existente com o mesmo `id`.
//! - **Preferências padrão**: Inicializa `user_preference` com `favorite_ca_id = None`.
//! - **Isolamento de privilégios**: O provisionamento por si só **não** concede qualquer cargo ou permissão administrativa.
//! - **Validação de domínio UnB**: Valida e-mail discente `@aluno.unb.br` e proíbe estritamente a matrícula `242003979`.

use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::modules::identity::{
    domain::{
        PROHIBITED_MATRICULA,
        entities::{
            AppUser, AppUserError, AppUserStatus, ExternalIdentity, ExternalIdentityError,
            UserPreference, UserPreferenceError,
        },
    },
    infrastructure::repository::{
        AppUserRepository, ExternalIdentityRepository, IdentityRepositoryError,
        InMemoryAppUserRepository, InMemoryExternalIdentityRepository,
        InMemoryUserPreferenceRepository, UserPreferenceRepository,
    },
};

/// Domínio institucional oficial dos estudantes da Universidade de Brasília.
pub const UNB_STUDENT_EMAIL_DOMAIN: &str = "@aluno.unb.br";

/// Erros decorrentes do fluxo de provisionamento de usuário.
#[derive(Debug, Error)]
pub enum UserProvisioningError {
    /// Violação de invariante de negócio da entidade de usuário interno.
    #[error("erro de domínio de usuário: {0}")]
    User(#[from] AppUserError),

    /// Violação de invariante da entidade de identidade externa.
    #[error("erro de domínio de identidade externa: {0}")]
    ExternalIdentity(#[from] ExternalIdentityError),

    /// Violação de invariante da preferência de usuário.
    #[error("erro de domínio de preferência: {0}")]
    Preference(#[from] UserPreferenceError),

    /// Falha na camada de persistência/repositório.
    #[error("erro na camada de persistência: {0}")]
    Repository(#[from] IdentityRepositoryError),

    /// O usuário referenciado pela identidade externa não foi localizado.
    #[error("usuário interno com identificador '{0}' não foi encontrado")]
    UserNotFound(Uuid),

    /// O e-mail informado não pertence ao domínio discente institucional esperado.
    #[error("o e-mail '{received}' deve pertencer ao domínio discente institucional '{expected}'")]
    InvalidInstitutionalDomain {
        /// Endereço recebido.
        received: String,
        /// Domínio esperado.
        expected: &'static str,
    },
}

/// Parâmetros de comando para o provisionamento ou reautenticação de usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionUserCommand {
    /// Identificador estável do provedor externo Firebase Authentication.
    pub firebase_uid: String,
    /// Identificador do sujeito autenticado na Microsoft institucional.
    pub microsoft_subject: String,
    /// Endereço de e-mail institucional discente (`<matricula>@aluno.unb.br`).
    pub institutional_email: String,
    /// Nome de exibição opcional fornecido no momento do login externo.
    pub display_name: Option<String>,
    /// Identificador opcional da instituição de ensino parceira.
    pub institution_id: Option<Uuid>,
}

impl ProvisionUserCommand {
    /// Constrói um novo comando de provisionamento com os parâmetros obrigatórios.
    pub fn new(
        firebase_uid: impl Into<String>,
        microsoft_subject: impl Into<String>,
        institutional_email: impl Into<String>,
    ) -> Self {
        Self {
            firebase_uid: firebase_uid.into(),
            microsoft_subject: microsoft_subject.into(),
            institutional_email: institutional_email.into(),
            display_name: None,
            institution_id: None,
        }
    }

    /// Define o nome de exibição para a criação do perfil.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Define a instituição parceira associada.
    pub fn with_institution_id(mut self, institution_id: Uuid) -> Self {
        self.institution_id = Some(institution_id);
        self
    }
}

/// Resultado do processamento do provisionamento de usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisioningResult {
    /// Entidade do usuário interno da aplicação (`app_user`).
    pub user: AppUser,
    /// Identidade institucional externa associada unicamente (`external_identity`).
    pub external_identity: ExternalIdentity,
    /// Preferências padrão do usuário (`user_preference`).
    pub preference: UserPreference,
    /// Indica se este é o primeiro acesso e um novo perfil foi criado (`true`),
    /// ou se o perfil pré-existente foi recuperado (`false`).
    pub is_new_user: bool,
    /// Indica se o usuário provisionado possui privilégios administrativos.
    ///
    /// Por definição de domínio e preservação de isolamento de privilégios,
    /// novos estudantes provisionados recebem sempre `false`.
    pub has_administrative_privileges: bool,
}

/// Serviço de domínio e aplicação para provisionamento automático de usuários.
#[derive(Clone)]
pub struct UserProvisioningService<U, E, P>
where
    U: AppUserRepository,
    E: ExternalIdentityRepository,
    P: UserPreferenceRepository,
{
    user_repo: Arc<U>,
    identity_repo: Arc<E>,
    preference_repo: Arc<P>,
}

impl<U, E, P> UserProvisioningService<U, E, P>
where
    U: AppUserRepository,
    E: ExternalIdentityRepository,
    P: UserPreferenceRepository,
{
    /// Constrói uma nova instância do serviço com os repositórios injetados.
    pub fn new(user_repo: Arc<U>, identity_repo: Arc<E>, preference_repo: Arc<P>) -> Self {
        Self {
            user_repo,
            identity_repo,
            preference_repo,
        }
    }

    /// Processa o login institucional com provisionamento automático e idempotência.
    ///
    /// # Fluxo de Execução:
    /// 1. Valida o e-mail discente (formato, domínio `@aluno.unb.br` e proibição de `242003979`).
    /// 2. Valida o Firebase UID e o Microsoft Subject.
    /// 3. Localiza se já existe uma identidade externa vinculada ao Firebase UID ou Subject:
    ///    - **Existente**: Atualiza `last_login_at`, recupera o perfil em `app_user` e `user_preference`.
    ///      Preserva estritamente o identificador `id` original (idempotência).
    ///    - **Novo Usuário**: Cria novo registro em `app_user` (`status = active`), associa a
    ///      `external_identity` unívoca e inicializa `user_preference` com `favorite_ca_id = None`.
    /// 4. Assegura que nenhum cargo administrativo ou permissão especial seja concedido.
    ///
    /// # Erros
    ///
    /// Retorna [`UserProvisioningError`] caso ocorra violação de regras de domínio ou falha no repositório.
    pub async fn provision(
        &self,
        command: ProvisionUserCommand,
    ) -> Result<ProvisioningResult, UserProvisioningError> {
        let normalized_email = command.institutional_email.trim().to_lowercase();

        // 1. Validação estrita da matrícula proibida por diretriz UnB
        if normalized_email.contains(PROHIBITED_MATRICULA) {
            return Err(UserProvisioningError::User(
                AppUserError::ProhibitedRegistration(PROHIBITED_MATRICULA.to_string()),
            ));
        }

        // 2. Validação do domínio de e-mail institucional
        if !normalized_email.contains('@') {
            return Err(UserProvisioningError::User(
                AppUserError::InvalidEmailFormat(normalized_email),
            ));
        }

        if !normalized_email.ends_with(UNB_STUDENT_EMAIL_DOMAIN) {
            return Err(UserProvisioningError::InvalidInstitutionalDomain {
                received: normalized_email,
                expected: UNB_STUDENT_EMAIL_DOMAIN,
            });
        }

        let firebase_uid = command.firebase_uid.trim().to_string();
        if firebase_uid.is_empty() {
            return Err(UserProvisioningError::ExternalIdentity(
                ExternalIdentityError::EmptyFirebaseUid,
            ));
        }

        let microsoft_subject = command.microsoft_subject.trim().to_string();
        if microsoft_subject.is_empty() {
            return Err(UserProvisioningError::ExternalIdentity(
                ExternalIdentityError::EmptyMicrosoftSubject,
            ));
        }

        let now = Utc::now();

        // 3. Busca por identidade externa previamente cadastrada (idempotência por Firebase UID)
        let existing_identity_by_uid = self
            .identity_repo
            .find_by_firebase_uid(&firebase_uid)
            .await?;

        // Busca complementar por Microsoft Subject
        let existing_identity = match existing_identity_by_uid {
            Some(identity) => Some(identity),
            None => {
                self.identity_repo
                    .find_by_microsoft_subject(&microsoft_subject)
                    .await?
            }
        };

        if let Some(mut identity) = existing_identity {
            // Reutilização de perfil e atualização do instante de último login
            identity.record_login(now);
            self.identity_repo.save(&identity).await?;

            let user = self
                .user_repo
                .find_by_id(identity.user_id)
                .await?
                .ok_or(UserProvisioningError::UserNotFound(identity.user_id))?;

            let preference = match self.preference_repo.find_by_user_id(user.id).await? {
                Some(p) => p,
                None => {
                    let default_pref = UserPreference::default_for_user(user.id)?;
                    self.preference_repo.save(&default_pref).await?;
                    default_pref
                }
            };

            return Ok(ProvisioningResult {
                user,
                external_identity: identity,
                preference,
                is_new_user: false,
                has_administrative_privileges: false,
            });
        }

        // Verifica se o usuário já possui cadastro prévio pelo e-mail institucional
        if let Some(user) = self.user_repo.find_by_email(&normalized_email).await? {
            // Cria e associa a identidade externa unívoca ao usuário existente
            let identity_id = Uuid::new_v4();
            let identity = ExternalIdentity::new(
                identity_id,
                user.id,
                command.institution_id,
                firebase_uid,
                microsoft_subject,
                Some(now),
            )?;
            self.identity_repo.save(&identity).await?;

            let preference = match self.preference_repo.find_by_user_id(user.id).await? {
                Some(p) => p,
                None => {
                    let default_pref = UserPreference::default_for_user(user.id)?;
                    self.preference_repo.save(&default_pref).await?;
                    default_pref
                }
            };

            return Ok(ProvisioningResult {
                user,
                external_identity: identity,
                preference,
                is_new_user: false,
                has_administrative_privileges: false,
            });
        }

        // 4. Primeiro acesso: provisionamento automático de novo estudante
        let user_id = Uuid::new_v4();
        let display_name = command
            .display_name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| {
                // Deriva nome de apresentação a partir da matrícula no e-mail
                normalized_email
                    .split('@')
                    .next()
                    .map(|mat| format!("Estudante {mat}"))
                    .unwrap_or_else(|| "Estudante UnB".to_string())
            });

        let user = AppUser::new(
            user_id,
            display_name,
            normalized_email,
            AppUserStatus::Active,
            now,
            now,
        )?;
        self.user_repo.save(&user).await?;

        // Associação unívoca entre identidade externa e interna
        let identity_id = Uuid::new_v4();
        let identity = ExternalIdentity::new(
            identity_id,
            user_id,
            command.institution_id,
            firebase_uid,
            microsoft_subject,
            Some(now),
        )?;
        self.identity_repo.save(&identity).await?;

        // Inicialização das preferências padrão com favorite_ca_id = None
        let preference = UserPreference::default_for_user(user_id)?;
        self.preference_repo.save(&preference).await?;

        // 5. Isolamento de privilégios: o provisionamento cria um estudante comum
        Ok(ProvisioningResult {
            user,
            external_identity: identity,
            preference,
            is_new_user: true,
            has_administrative_privileges: false,
        })
    }

    /// Verifica se o usuário possui qualquer privilégio ou concessão administrativa.
    ///
    /// Preserva a regra de isolamento de privilégios da Issue #7: o provisionamento
    /// nunca concede poderes administrativos.
    pub async fn has_administrative_privileges(&self, _user_id: Uuid) -> bool {
        // Conforme especificado, o provisionamento cria apenas um estudante discente comum.
        false
    }
}

/// Constrói um serviço de provisionamento configurado inteiramente em memória para testes.
pub fn create_in_memory_user_provisioning_service() -> UserProvisioningService<
    InMemoryAppUserRepository,
    InMemoryExternalIdentityRepository,
    InMemoryUserPreferenceRepository,
> {
    UserProvisioningService::new(
        Arc::new(InMemoryAppUserRepository::new()),
        Arc::new(InMemoryExternalIdentityRepository::new()),
        Arc::new(InMemoryUserPreferenceRepository::new()),
    )
}

/// Constrói um serviço de provisionamento em memória pré-populado com os dados didáticos da UnB.
pub fn create_didactic_user_provisioning_service() -> UserProvisioningService<
    InMemoryAppUserRepository,
    InMemoryExternalIdentityRepository,
    InMemoryUserPreferenceRepository,
> {
    UserProvisioningService::new(
        Arc::new(InMemoryAppUserRepository::with_didactic_data()),
        Arc::new(InMemoryExternalIdentityRepository::with_didactic_data()),
        Arc::new(InMemoryUserPreferenceRepository::with_didactic_data()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::identity::domain::VETERANO_1_DIDACTIC_ID;

    #[tokio::test]
    async fn should_provision_new_calouro_automatically() {
        let service = create_in_memory_user_provisioning_service();

        let cmd = ProvisionUserCommand::new(
            "firebase-uid-calouro-123",
            "ms-sub-calouro-123",
            "262001234@aluno.unb.br",
        )
        .with_display_name("Estudante Calouro UnB");

        let result = service.provision(cmd).await.expect("provision new user");

        // 1. Registro unívoco criado em app_user
        assert!(result.is_new_user);
        assert_eq!(result.user.status, AppUserStatus::Active);
        assert_eq!(result.user.institutional_email, "262001234@aluno.unb.br");
        assert_eq!(result.user.display_name, "Estudante Calouro UnB");

        // 2. Identidade externa associada unicamente
        assert_eq!(result.external_identity.user_id, result.user.id);
        assert_eq!(
            result.external_identity.firebase_uid,
            "firebase-uid-calouro-123"
        );
        assert!(result.external_identity.last_login_at.is_some());

        // 3. Inicialização de preferências padrão com favorite_ca_id = None
        assert_eq!(result.preference.user_id, result.user.id);
        assert_eq!(result.preference.favorite_ca_id, None);

        // 4. Isolamento estrito de privilégios
        assert!(!result.has_administrative_privileges);
        assert!(!service.has_administrative_privileges(result.user.id).await);
    }

    #[tokio::test]
    async fn should_ensure_idempotency_on_subsequent_login() {
        let service = create_in_memory_user_provisioning_service();

        let cmd1 = ProvisionUserCommand::new(
            "firebase-uid-veterano-1",
            "ms-sub-veterano-1",
            "231012345@aluno.unb.br",
        )
        .with_display_name("Estudante Veterano UnB");

        // Primeiro acesso: cria o usuário
        let first_result = service.provision(cmd1.clone()).await.expect("first login");
        assert!(first_result.is_new_user);
        let original_id = first_result.user.id;
        let original_login_time = first_result.external_identity.last_login_at;

        // Segundo acesso: recupera usuário existente sem recriar
        let second_result = service.provision(cmd1.clone()).await.expect("second login");
        assert!(!second_result.is_new_user);
        assert_eq!(
            second_result.user.id, original_id,
            "o identificador interno deve ser idêntico"
        );
        assert_eq!(
            second_result.user.institutional_email,
            "231012345@aluno.unb.br"
        );
        assert_eq!(
            second_result.preference.favorite_ca_id, None,
            "preferências existentes devem ser preservadas"
        );
        assert!(
            second_result.external_identity.last_login_at >= original_login_time,
            "data do último acesso deve ser atualizada"
        );

        // Terceiro acesso consecutivo (Cenário 4 de BDD: reautenticações consecutivas)
        let third_result = service.provision(cmd1).await.expect("third login");
        assert!(!third_result.is_new_user);
        assert_eq!(third_result.user.id, original_id);
    }

    #[tokio::test]
    async fn should_reject_prohibited_matricula() {
        let service = create_in_memory_user_provisioning_service();

        let prohibited_email = format!("{PROHIBITED_MATRICULA}@aluno.unb.br");
        let cmd = ProvisionUserCommand::new("uid-proibido", "sub-proibido", prohibited_email);

        let err = service.provision(cmd).await.unwrap_err();
        match err {
            UserProvisioningError::User(AppUserError::ProhibitedRegistration(mat)) => {
                assert_eq!(mat, PROHIBITED_MATRICULA);
            }
            other => panic!("erro inesperado: {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_invalid_email_domain() {
        let service = create_in_memory_user_provisioning_service();

        let cmd = ProvisionUserCommand::new("uid-invalido", "sub-invalido", "estudante@gmail.com");

        let err = service.provision(cmd).await.unwrap_err();
        match err {
            UserProvisioningError::InvalidInstitutionalDomain { received, expected } => {
                assert_eq!(received, "estudante@gmail.com");
                assert_eq!(expected, UNB_STUDENT_EMAIL_DOMAIN);
            }
            other => panic!("erro inesperado: {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_empty_credentials() {
        let service = create_in_memory_user_provisioning_service();

        let cmd_empty_uid = ProvisionUserCommand::new("   ", "sub-valid", "262001234@aluno.unb.br");
        let err1 = service.provision(cmd_empty_uid).await.unwrap_err();
        assert!(matches!(
            err1,
            UserProvisioningError::ExternalIdentity(ExternalIdentityError::EmptyFirebaseUid)
        ));

        let cmd_empty_sub = ProvisionUserCommand::new("uid-valid", "   ", "262001234@aluno.unb.br");
        let err2 = service.provision(cmd_empty_sub).await.unwrap_err();
        assert!(matches!(
            err2,
            UserProvisioningError::ExternalIdentity(ExternalIdentityError::EmptyMicrosoftSubject)
        ));
    }

    #[tokio::test]
    async fn should_handle_existing_didactic_veterano() {
        let service = create_didactic_user_provisioning_service();

        // Login do veterano 1 já pré-cadastrado na base didática
        let cmd = ProvisionUserCommand::new(
            "firebase-uid-veterano-1",
            "sub-unb-231012345@aluno.unb.br",
            "231012345@aluno.unb.br",
        );

        let result = service.provision(cmd).await.expect("login veterano");
        assert!(!result.is_new_user);
        assert_eq!(result.user.id, VETERANO_1_DIDACTIC_ID);
        assert_eq!(result.user.institutional_email, "231012345@aluno.unb.br");
        assert_eq!(
            result.preference.favorite_ca_id,
            Some(Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001))
        );
    }
}
