//! Entidade de domínio representando o contexto ativo de navegação em Centro Acadêmico (`active_ca_context`).
//!
//! O contexto ativo expressa o Centro Acadêmico atualmente selecionado pelo usuário
//! na sessão de navegação da interface (em memória/sessão).
//!
//! # Regras Estritas de Domínio (Issue #45 / I01 e § 2.1)
//! 1. **Conveniência de Navegação em Sessão**: O contexto ativo determina quais informações e
//!    serviços do CA são apresentados em tela ao estudante durante sua sessão.
//! 2. **Não Sobrescrita da Preferência Gravada**: A alternância do contexto ativo de navegação
//!    (ex.: de CAER para CACC) é uma operação efêmera de sessão e **NÃO** sobrescreve nem altera
//!    a preferência persistida de CA favorito (`user_preference.favorite_ca_id`).
//! 3. **Isolamento Estrito de Privilégios**: Estar navegando com contexto ativo em um Centro Acadêmico
//!    **NÃO** concede privilégios administrativos, cargos de gestão, direito a voto nem vínculo
//!    acadêmico naquele CA. Privilégios decorrem exclusivamente de atribuições de gestão formais
//!    vigentes no CA ativo; ao navegar por outro CA no qual não exerça cargo, o estudante atua
//!    estritamente como visitante.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::modules::{
    ca::domain::entities::academic_center::{AcademicCenter, AcademicCenterStatus},
    identity::domain::entities::user_preference::UserPreference,
};

/// Erros decorrentes de validação e integridade do contexto ativo de navegação.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActiveContextError {
    /// O identificador do usuário não pode ser o UUID nulo.
    #[error("o identificador do usuário não pode ser o UUID nulo")]
    NilUserId,

    /// O identificador do Centro Acadêmico ativo não pode ser o UUID nulo.
    #[error("o identificador do Centro Acadêmico ativo não pode ser o UUID nulo")]
    NilAcademicCenterId,

    /// O Centro Acadêmico indicado não foi localizado entre os CAs cadastrados.
    #[error("o Centro Acadêmico '{0}' não foi encontrado entre os Centros Acadêmicos cadastrados")]
    AcademicCenterNotFound(Uuid),

    /// O Centro Acadêmico indicado não está ativo para seleção de contexto de navegação.
    #[error("o Centro Acadêmico '{0}' não se encontra ativo no ciclo de vida")]
    AcademicCenterNotActive(Uuid),

    /// Tentativa de associar preferência de usuário divergente do usuário do contexto ativo.
    #[error("o usuário da preferência não corresponde ao usuário da sessão")]
    UserMismatch,

    /// Operação inválida de privilégio: o contexto ativo não confere privilégios administrativos.
    #[error(
        "o contexto ativo representa estritamente navegação em sessão e não confere privilégios administrativos ou acadêmicos"
    )]
    PrivilegeConferralAttempt,

    /// O nome do cargo de gestão não pode ser vazio.
    #[error("o nome do cargo administrativo não pode ser vazio")]
    EmptyRoleName,
}

/// Origem da determinação do contexto ativo na sessão.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveContextSource {
    /// Inicializado na abertura do aplicativo a partir da preferência persistida de CA favorito.
    FavoritePreference,
    /// Selecionado ou alternado explicitamente pelo estudante durante a navegação.
    ExplicitSelection,
    /// Inicializado com contexto padrão da instituição quando o estudante não possui favorito definido.
    DefaultNavigation,
}

/// Entidade de domínio que representa o contexto ativo de navegação de um estudante em sessão/memória.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCaContext {
    /// Identificador do usuário navegando na sessão.
    pub user_id: Uuid,
    /// Identificador do Centro Acadêmico atualmente ativo na interface.
    pub active_ca_id: Uuid,
    /// Origem de definição do contexto atual na sessão.
    pub source: ActiveContextSource,
    /// Instante da última definição ou alternância de contexto.
    pub updated_at: DateTime<Utc>,
}

impl ActiveCaContext {
    /// Cria uma nova instância de contexto ativo a partir de seleção explícita.
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError::NilUserId`] se `user_id` for nulo, ou
    /// [`ActiveContextError::NilAcademicCenterId`] se `active_ca_id` for nulo.
    pub fn new(user_id: Uuid, active_ca_id: Uuid) -> Result<Self, ActiveContextError> {
        Self::with_source_and_timestamp(
            user_id,
            active_ca_id,
            ActiveContextSource::ExplicitSelection,
            Utc::now(),
        )
    }

    /// Cria uma nova instância de contexto ativo especificando a origem da seleção.
    ///
    /// # Erros
    ///
    /// Retorna erro se algum identificador for nulo.
    pub fn with_source(
        user_id: Uuid,
        active_ca_id: Uuid,
        source: ActiveContextSource,
    ) -> Result<Self, ActiveContextError> {
        Self::with_source_and_timestamp(user_id, active_ca_id, source, Utc::now())
    }

    /// Inicializa o contexto ativo a partir da preferência persistida de CA favorito do estudante.
    ///
    /// Se o usuário possuir um CA favorito configurado (`preference.favorite_ca_id()`), o contexto
    /// ativo inicial é estabelecido com esse CA sob a origem [`ActiveContextSource::FavoritePreference`].
    /// Caso contrário, utiliza o `fallback_ca_id` sob [`ActiveContextSource::DefaultNavigation`].
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError::UserMismatch`] se a preferência não pertencer ao `user_id`,
    /// ou erro caso os identificadores sejam nulos.
    pub fn from_preference(
        user_id: Uuid,
        preference: &UserPreference,
        fallback_ca_id: Uuid,
    ) -> Result<Self, ActiveContextError> {
        if user_id.is_nil() {
            return Err(ActiveContextError::NilUserId);
        }
        if preference.user_id() != user_id {
            return Err(ActiveContextError::UserMismatch);
        }

        if let Some(fav_id) = preference.favorite_ca_id() {
            if fav_id.is_nil() {
                return Err(ActiveContextError::NilAcademicCenterId);
            }
            Self::with_source(user_id, fav_id, ActiveContextSource::FavoritePreference)
        } else {
            if fallback_ca_id.is_nil() {
                return Err(ActiveContextError::NilAcademicCenterId);
            }
            Self::with_source(
                user_id,
                fallback_ca_id,
                ActiveContextSource::DefaultNavigation,
            )
        }
    }

    /// Reconstrói o contexto ativo com timestamp e origem explícitos.
    ///
    /// # Erros
    ///
    /// Retorna erro se algum identificador for nulo.
    pub fn with_source_and_timestamp(
        user_id: Uuid,
        active_ca_id: Uuid,
        source: ActiveContextSource,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, ActiveContextError> {
        if user_id.is_nil() {
            return Err(ActiveContextError::NilUserId);
        }
        if active_ca_id.is_nil() {
            return Err(ActiveContextError::NilAcademicCenterId);
        }

        Ok(Self {
            user_id,
            active_ca_id,
            source,
            updated_at,
        })
    }

    /// Alterna o contexto ativo de navegação para outro Centro Acadêmico.
    ///
    /// A transição altera o CA em foco na sessão e define a origem como
    /// [`ActiveContextSource::ExplicitSelection`].
    ///
    /// # Regra Estrita de Negócio
    /// A alteração de `active_ca_id` nesta entidade afeta unicamente o estado em memória/sessão;
    /// ela **NÃO** altera nem sobrescreve a preferência persistida de CA favorito.
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError::NilAcademicCenterId`] se `new_ca_id` for o UUID nulo.
    pub fn switch_to(&mut self, new_ca_id: Uuid) -> Result<(), ActiveContextError> {
        if new_ca_id.is_nil() {
            return Err(ActiveContextError::NilAcademicCenterId);
        }
        self.active_ca_id = new_ca_id;
        self.source = ActiveContextSource::ExplicitSelection;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Alterna o contexto ativo de navegação garantindo que o novo CA pertença à lista de CAs cadastrados e ativos.
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError::NilAcademicCenterId`] se `new_ca_id` for nulo,
    /// [`ActiveContextError::AcademicCenterNotFound`] se o CA não estiver na lista de cadastrados, ou
    /// [`ActiveContextError::AcademicCenterNotActive`] se o status do CA for diferente de [`AcademicCenterStatus::Active`].
    pub fn switch_to_registered(
        &mut self,
        new_ca_id: Uuid,
        registered_cas: &[AcademicCenter],
    ) -> Result<(), ActiveContextError> {
        if new_ca_id.is_nil() {
            return Err(ActiveContextError::NilAcademicCenterId);
        }

        let target_ca = registered_cas
            .iter()
            .find(|ca| ca.id == new_ca_id)
            .ok_or(ActiveContextError::AcademicCenterNotFound(new_ca_id))?;

        if target_ca.status != AcademicCenterStatus::Active {
            return Err(ActiveContextError::AcademicCenterNotActive(new_ca_id));
        }

        self.switch_to(new_ca_id)
    }

    /// Retorna o identificador do usuário proprietário da sessão de navegação.
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o identificador do Centro Acadêmico ativo na sessão.
    pub const fn active_ca_id(&self) -> Uuid {
        self.active_ca_id
    }

    /// Retorna a origem da definição do contexto ativo.
    pub const fn source(&self) -> ActiveContextSource {
        self.source
    }

    /// Retorna o instante da última alteração de contexto ativo.
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Verifica se o Centro Acadêmico informado corresponde ao contexto atualmente ativo.
    pub fn is_active(&self, ca_id: &Uuid) -> bool {
        self.active_ca_id == *ca_id
    }

    /// Confirma que este contexto possui finalidade restrita à navegação da interface.
    ///
    /// Sempre retorna `true` conforme especificação da Issue #45 e § 2.1.
    pub const fn is_navigation_only(&self) -> bool {
        true
    }

    /// Regra estrita de domínio: o contexto ativo **não** confere vínculo acadêmico com o curso do CA.
    ///
    /// Sempre retorna `false`.
    pub const fn confers_academic_enrollment(&self) -> bool {
        false
    }

    /// Regra estrita de domínio: o contexto ativo **não** confere elegibilidade eleitoral no CA.
    ///
    /// Sempre retorna `false`.
    pub const fn confers_electoral_eligibility(&self) -> bool {
        false
    }

    /// Regra estrita de domínio: o contexto ativo **não** confere permissões administrativas.
    ///
    /// Sempre retorna `false`. Permissões decorrem unicamente de cargos formais de gestão
    /// atribuídos especificamente ao usuário no CA correspondente.
    pub const fn confers_administrative_permissions(&self) -> bool {
        false
    }
}

/// Atribuição de cargo de gestão a um estudante em determinado Centro Acadêmico.
///
/// Modela os privilégios administrativos exercidos formalmente pelo usuário em um CA específico,
/// permitindo aferir o isolamento de privilégios quando o usuário navega entre diferentes CAs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementRoleAssignment {
    /// Identificador do usuário detentor do cargo.
    pub user_id: Uuid,
    /// Identificador do Centro Acadêmico em que o cargo é exercido.
    pub ca_id: Uuid,
    /// Nome descritivo do cargo na diretoria ou gestão (ex.: "Diretor de Comunicação", "Presidente").
    pub role_name: String,
    /// Lista de permissões administrativas conferidas pelo cargo.
    pub permissions: Vec<String>,
}

impl ManagementRoleAssignment {
    /// Cria uma nova atribuição de cargo de gestão.
    ///
    /// # Erros
    ///
    /// Retorna [`ActiveContextError::NilUserId`] se `user_id` for nulo,
    /// [`ActiveContextError::NilAcademicCenterId`] se `ca_id` for nulo, ou
    /// [`ActiveContextError::EmptyRoleName`] se o nome do cargo for vazio.
    pub fn new(
        user_id: Uuid,
        ca_id: Uuid,
        role_name: impl Into<String>,
        permissions: Vec<String>,
    ) -> Result<Self, ActiveContextError> {
        if user_id.is_nil() {
            return Err(ActiveContextError::NilUserId);
        }
        if ca_id.is_nil() {
            return Err(ActiveContextError::NilAcademicCenterId);
        }
        let role_name = role_name.into().trim().to_string();
        if role_name.is_empty() {
            return Err(ActiveContextError::EmptyRoleName);
        }

        Ok(Self {
            user_id,
            ca_id,
            role_name,
            permissions,
        })
    }

    /// Verifica se a atribuição de cargo confere a permissão informada (comparação insensível a maiúsculas/minúsculas).
    pub fn has_permission(&self, permission: &str) -> bool {
        let perm_clean = permission.trim();
        self.permissions
            .iter()
            .any(|p| p.eq_ignore_ascii_case(perm_clean))
    }

    /// Retorna o identificador do usuário do cargo.
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o identificador do Centro Acadêmico ao qual o cargo pertence.
    pub const fn ca_id(&self) -> Uuid {
        self.ca_id
    }

    /// Retorna o nome do cargo administrativo.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Retorna a lista de permissões administrativas atribuídas.
    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ca::domain::entities::academic_center::{
        CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    };

    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);
    const OTHER_USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0012);

    #[test]
    fn should_create_valid_active_context() {
        let context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("valid context");
        assert_eq!(context.user_id(), USER_ID);
        assert_eq!(context.active_ca_id(), CAER_DIDACTIC_ID);
        assert_eq!(context.source(), ActiveContextSource::ExplicitSelection);
        assert!(context.is_active(&CAER_DIDACTIC_ID));
        assert!(!context.is_active(&CACC_DIDACTIC_ID));
        assert!(context.is_navigation_only());
        assert!(!context.confers_administrative_permissions());
        assert!(!context.confers_academic_enrollment());
        assert!(!context.confers_electoral_eligibility());
    }

    #[test]
    fn should_reject_nil_user_id_or_ca_id() {
        assert_eq!(
            ActiveCaContext::new(Uuid::nil(), CAER_DIDACTIC_ID).unwrap_err(),
            ActiveContextError::NilUserId
        );
        assert_eq!(
            ActiveCaContext::new(USER_ID, Uuid::nil()).unwrap_err(),
            ActiveContextError::NilAcademicCenterId
        );
    }

    #[test]
    fn should_initialize_from_preference_with_favorite_ca() {
        let pref = UserPreference::new(USER_ID, Some(CAER_DIDACTIC_ID)).expect("valid pref");
        let context = ActiveCaContext::from_preference(USER_ID, &pref, CACC_DIDACTIC_ID)
            .expect("should init from favorite");

        assert_eq!(context.active_ca_id(), CAER_DIDACTIC_ID);
        assert_eq!(context.source(), ActiveContextSource::FavoritePreference);
    }

    #[test]
    fn should_initialize_from_preference_with_fallback_when_no_favorite() {
        let pref = UserPreference::default_for_user(USER_ID).expect("valid pref without favorite");
        let context = ActiveCaContext::from_preference(USER_ID, &pref, CACC_DIDACTIC_ID)
            .expect("should init with fallback");

        assert_eq!(context.active_ca_id(), CACC_DIDACTIC_ID);
        assert_eq!(context.source(), ActiveContextSource::DefaultNavigation);
    }

    #[test]
    fn should_reject_initialization_when_user_mismatch() {
        let pref = UserPreference::default_for_user(OTHER_USER_ID).expect("pref for other user");
        assert_eq!(
            ActiveCaContext::from_preference(USER_ID, &pref, CAER_DIDACTIC_ID).unwrap_err(),
            ActiveContextError::UserMismatch
        );
    }

    #[test]
    fn should_switch_active_ca_successfully() {
        let mut context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("initial caer");
        assert!(context.is_active(&CAER_DIDACTIC_ID));

        context
            .switch_to(CACC_DIDACTIC_ID)
            .expect("should switch to cacc");
        assert_eq!(context.active_ca_id(), CACC_DIDACTIC_ID);
        assert!(context.is_active(&CACC_DIDACTIC_ID));
        assert!(!context.is_active(&CAER_DIDACTIC_ID));
        assert_eq!(context.source(), ActiveContextSource::ExplicitSelection);
    }

    #[test]
    fn should_reject_switch_to_nil_id() {
        let mut context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("initial caer");
        assert_eq!(
            context.switch_to(Uuid::nil()).unwrap_err(),
            ActiveContextError::NilAcademicCenterId
        );
    }

    #[test]
    fn should_switch_to_registered_active_ca() {
        let caer = AcademicCenter::caer_didactic();
        let cacc = AcademicCenter::cacc_didactic();
        let registered = vec![caer, cacc];

        let mut context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("initial caer");
        context
            .switch_to_registered(CACC_DIDACTIC_ID, &registered)
            .expect("should switch to registered cacc");
        assert_eq!(context.active_ca_id(), CACC_DIDACTIC_ID);
    }

    #[test]
    fn should_reject_switch_to_unregistered_or_inactive_ca() {
        let caer = AcademicCenter::caer_didactic();
        let unknown_id = Uuid::new_v4();
        let registered = vec![caer];

        let mut context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("initial caer");
        assert_eq!(
            context
                .switch_to_registered(unknown_id, &registered)
                .unwrap_err(),
            ActiveContextError::AcademicCenterNotFound(unknown_id)
        );

        let mut draft_cacc = AcademicCenter::cacc_didactic();
        draft_cacc.status = AcademicCenterStatus::Draft;
        let registered_with_draft = vec![AcademicCenter::caer_didactic(), draft_cacc];

        assert_eq!(
            context
                .switch_to_registered(CACC_DIDACTIC_ID, &registered_with_draft)
                .unwrap_err(),
            ActiveContextError::AcademicCenterNotActive(CACC_DIDACTIC_ID)
        );
    }

    #[test]
    fn should_create_management_role_assignment_and_check_permissions() {
        let assignment = ManagementRoleAssignment::new(
            USER_ID,
            CAER_DIDACTIC_ID,
            "Diretor de Comunicação",
            vec!["Publicar notícias diretamente".to_string()],
        )
        .expect("valid assignment");

        assert_eq!(assignment.user_id(), USER_ID);
        assert_eq!(assignment.ca_id(), CAER_DIDACTIC_ID);
        assert_eq!(assignment.role_name(), "Diretor de Comunicação");
        assert!(assignment.has_permission("Publicar notícias diretamente"));
        assert!(assignment.has_permission("publicar notícias diretamente"));
        assert!(!assignment.has_permission("Administrar integrantes da gestão"));
    }

    #[test]
    fn should_reject_invalid_role_assignment() {
        assert_eq!(
            ManagementRoleAssignment::new(Uuid::nil(), CAER_DIDACTIC_ID, "Presidente", vec![])
                .unwrap_err(),
            ActiveContextError::NilUserId
        );
        assert_eq!(
            ManagementRoleAssignment::new(USER_ID, Uuid::nil(), "Presidente", vec![]).unwrap_err(),
            ActiveContextError::NilAcademicCenterId
        );
        assert_eq!(
            ManagementRoleAssignment::new(USER_ID, CAER_DIDACTIC_ID, "   ", vec![]).unwrap_err(),
            ActiveContextError::EmptyRoleName
        );
    }

    #[test]
    fn should_serialize_and_deserialize_active_context() {
        let context = ActiveCaContext::new(USER_ID, CAER_DIDACTIC_ID).expect("valid context");
        let json = serde_json::to_string(&context).expect("serialize");
        let deserialized: ActiveCaContext = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(context, deserialized);
    }
}
