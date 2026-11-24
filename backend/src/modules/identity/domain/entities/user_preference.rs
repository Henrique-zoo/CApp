//! Entidade de domínio representando a preferência do usuário (`user_preference`).
//!
//! A preferência armazena opções de personalização e conveniência de interface do usuário,
//! especialmente o Centro Acadêmico favorito utilizado como contexto inicial ao abrir o CApp.
//!
//! # Regra Estrita de Domínio (Issue #44 / I01)
//! O campo `favorite_ca_id` (UUID referenciando `academic_center`) representa **unicamente**
//! conveniência de navegação do usuário.
//!
//! A escolha de um Centro Acadêmico favorito:
//! - **NÃO** confere elegibilidade eleitoral (direito a voto ou participação em eleições);
//! - **NÃO** confere permissões administrativas nem cargos de gestão no CA;
//! - **NÃO** confere nem substitui o vínculo acadêmico oficial com o curso ou Centro Acadêmico.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Erros de validação ou integridade estrutural da preferência do usuário.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum UserPreferenceError {
    /// O identificador do usuário não pode ser o UUID nulo.
    #[error("o identificador do usuário não pode ser o UUID nulo")]
    NilUserId,

    /// O identificador do Centro Acadêmico favorito não pode ser o UUID nulo.
    #[error("o identificador do Centro Acadêmico favorito não pode ser o UUID nulo")]
    NilFavoriteCaId,

    /// Operação inválida de privilégio: a preferência não concede direitos eleitorais ou administrativos.
    #[error(
        "a preferência de Centro Acadêmico representa unicamente conveniência de navegação e não confere privilégios acadêmicos ou administrativos"
    )]
    PrivilegeConferralAttempt,
}

/// Entidade que encapsula a preferência de navegação do usuário no CApp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPreference {
    /// Identificador do usuário ao qual a preferência pertence (PK/FK referenciando `app_user`).
    pub user_id: Uuid,
    /// Identificador do Centro Acadêmico favorito para contexto de navegação (FK referenciando `academic_center`).
    pub favorite_ca_id: Option<Uuid>,
    /// Instante da última atualização da preferência.
    pub updated_at: DateTime<Utc>,
}

impl UserPreference {
    /// Cria uma nova instância de preferência do usuário.
    ///
    /// # Erros
    ///
    /// Retorna [`UserPreferenceError::NilUserId`] se `user_id` for nulo, ou
    /// [`UserPreferenceError::NilFavoriteCaId`] se `favorite_ca_id` contiver UUID nulo.
    pub fn new(user_id: Uuid, favorite_ca_id: Option<Uuid>) -> Result<Self, UserPreferenceError> {
        Self::with_timestamp(user_id, favorite_ca_id, Utc::now())
    }

    /// Cria a preferência padrão para um usuário recém-provisionado (sem CA favorito inicial).
    ///
    /// # Erros
    ///
    /// Retorna [`UserPreferenceError::NilUserId`] se `user_id` for nulo.
    pub fn default_for_user(user_id: Uuid) -> Result<Self, UserPreferenceError> {
        Self::new(user_id, None)
    }

    /// Reconstrói a preferência com timestamp explícito (ex.: ao carregar do banco).
    ///
    /// # Erros
    ///
    /// Retorna erro se algum identificador for o UUID nulo.
    pub fn with_timestamp(
        user_id: Uuid,
        favorite_ca_id: Option<Uuid>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, UserPreferenceError> {
        if user_id.is_nil() {
            return Err(UserPreferenceError::NilUserId);
        }
        if matches!(favorite_ca_id, Some(ca_id) if ca_id.is_nil()) {
            return Err(UserPreferenceError::NilFavoriteCaId);
        }

        Ok(Self {
            user_id,
            favorite_ca_id,
            updated_at,
        })
    }

    /// Define um Centro Acadêmico favorito para a navegação do usuário.
    ///
    /// Atualiza o timestamp `updated_at` para o instante presente.
    ///
    /// # Erros
    ///
    /// Retorna [`UserPreferenceError::NilFavoriteCaId`] se `ca_id` for o UUID nulo.
    pub fn set_favorite_ca(&mut self, ca_id: Uuid) -> Result<(), UserPreferenceError> {
        if ca_id.is_nil() {
            return Err(UserPreferenceError::NilFavoriteCaId);
        }
        self.favorite_ca_id = Some(ca_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Limpa a preferência de CA favorito, indicando que o usuário não possui seleção pré-definida.
    ///
    /// Atualiza o timestamp `updated_at` para o instante presente.
    pub fn clear_favorite_ca(&mut self) {
        self.favorite_ca_id = None;
        self.updated_at = Utc::now();
    }

    /// Retorna o identificador do usuário proprietário da preferência.
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o identificador do CA favorito, se definido.
    pub const fn favorite_ca_id(&self) -> Option<Uuid> {
        self.favorite_ca_id
    }

    /// Retorna o instante da última alteração da preferência.
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Informa se o usuário possui um CA favorito configurado.
    pub const fn has_favorite_ca(&self) -> bool {
        self.favorite_ca_id.is_some()
    }

    /// Verifica se o CA informado coincide com o CA favorito atual.
    pub fn is_favorite(&self, ca_id: &Uuid) -> bool {
        self.favorite_ca_id.as_ref() == Some(ca_id)
    }

    /// Confirma que o escopo deste registro é estritamente de conveniência de navegação.
    ///
    /// Sempre retorna `true` conforme diretriz da Issue #44 e modelo ER § 2.3.
    pub const fn is_navigation_convenience_only(&self) -> bool {
        true
    }

    /// Regra estrita de domínio: a seleção de um CA favorito **não** confere vínculo acadêmico oficial.
    ///
    /// Sempre retorna `false`. O vínculo acadêmico oficial provém exclusivamente
    /// de integração institucional (`academic_enrollment`).
    pub const fn confers_academic_enrollment(&self) -> bool {
        false
    }

    /// Regra estrita de domínio: a seleção de um CA favorito **não** confere elegibilidade eleitoral.
    ///
    /// Sempre retorna `false`. A aptidão de votar ou se candidatar depende de critérios
    /// formais da eleição e do colégio eleitoral oficial.
    pub const fn confers_electoral_eligibility(&self) -> bool {
        false
    }

    /// Regra estrita de domínio: a seleção de um CA favorito **não** confere permissões administrativas.
    ///
    /// Sempre retorna `false`. Permissões e cargos dependem do exercício vigente em uma gestão
    /// (`management_assignment`), não da preferência de tela inicial.
    pub const fn confers_administrative_permissions(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);
    const USER_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0011);

    #[test]
    fn should_create_default_preference_without_favorite() {
        let pref =
            UserPreference::default_for_user(USER_ID).expect("should create default preference");
        assert_eq!(pref.user_id(), USER_ID);
        assert_eq!(pref.favorite_ca_id(), None);
        assert!(!pref.has_favorite_ca());
        assert!(!pref.is_favorite(&CAER_ID));
    }

    #[test]
    fn should_set_and_change_favorite_ca() {
        let mut pref = UserPreference::default_for_user(USER_ID).expect("valid default preference");

        pref.set_favorite_ca(CAER_ID)
            .expect("should set CAER as favorite");
        assert_eq!(pref.favorite_ca_id(), Some(CAER_ID));
        assert!(pref.has_favorite_ca());
        assert!(pref.is_favorite(&CAER_ID));
        assert!(!pref.is_favorite(&CACC_ID));

        pref.set_favorite_ca(CACC_ID)
            .expect("should change favorite to CACC");
        assert_eq!(pref.favorite_ca_id(), Some(CACC_ID));
        assert!(pref.is_favorite(&CACC_ID));
        assert!(!pref.is_favorite(&CAER_ID));
    }

    #[test]
    fn should_clear_favorite_ca() {
        let mut pref =
            UserPreference::new(USER_ID, Some(CAER_ID)).expect("valid preference with CAER");
        assert!(pref.has_favorite_ca());

        pref.clear_favorite_ca();
        assert_eq!(pref.favorite_ca_id(), None);
        assert!(!pref.has_favorite_ca());
    }

    #[test]
    fn should_reject_nil_user_id() {
        let err = UserPreference::default_for_user(Uuid::nil()).unwrap_err();
        assert_eq!(err, UserPreferenceError::NilUserId);
    }

    #[test]
    fn should_reject_nil_favorite_ca_id() {
        let err = UserPreference::new(USER_ID, Some(Uuid::nil())).unwrap_err();
        assert_eq!(err, UserPreferenceError::NilFavoriteCaId);

        let mut pref = UserPreference::default_for_user(USER_ID).unwrap();
        let set_err = pref.set_favorite_ca(Uuid::nil()).unwrap_err();
        assert_eq!(set_err, UserPreferenceError::NilFavoriteCaId);
    }

    #[test]
    fn should_strictly_enforce_no_academic_or_administrative_privileges() {
        let pref = UserPreference::new(USER_ID, Some(CAER_ID)).expect("valid preference");

        // Regra estrita de domínio (Issue #44)
        assert!(pref.is_navigation_convenience_only());
        assert!(!pref.confers_academic_enrollment());
        assert!(!pref.confers_electoral_eligibility());
        assert!(!pref.confers_administrative_permissions());
    }

    #[test]
    fn should_serialize_and_deserialize_json() {
        let original = UserPreference::new(USER_ID, Some(CAER_ID)).expect("valid preference");
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: UserPreference = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(original.user_id(), deserialized.user_id());
        assert_eq!(original.favorite_ca_id(), deserialized.favorite_ca_id());
    }
}
