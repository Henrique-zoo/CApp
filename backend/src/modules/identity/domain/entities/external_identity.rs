//! Entidade de domínio representando a Identidade Externa do usuário (`external_identity`).
//!
//! A identidade externa vincula a conta interna do CApp (`app_user`) à identidade
//! autenticada externamente (Firebase Authentication e Microsoft Entra ID institucional).
//!
//! Conforme o Modelo Entidade-Relacionamento (§ 1. Identidade e vínculo acadêmico),
//! a identidade externa e o perfil interno do usuário são tratados como entidades distintas.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Identificador estável da identidade externa do veterano 1 para cenários didáticos.
pub const VETERANO_1_EXTERNAL_IDENTITY_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0021);

/// Identificador estável da identidade externa do veterano 2 para cenários didáticos.
pub const VETERANO_2_EXTERNAL_IDENTITY_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0022);

/// Identificador estável da identidade externa do calouro para cenários didáticos.
pub const CALOURO_EXTERNAL_IDENTITY_ID: Uuid =
    Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0023);

/// Limite máximo em caracteres para o identificador Firebase UID.
pub const MAX_FIREBASE_UID_LENGTH: usize = 255;

/// Limite máximo em caracteres para o identificador Microsoft Subject.
pub const MAX_MICROSOFT_SUBJECT_LENGTH: usize = 255;

/// Erros decorrentes de violações de validação da identidade externa.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExternalIdentityError {
    /// O identificador estável da entidade não pode ser o UUID nulo.
    #[error("o identificador da identidade externa não pode ser o UUID nulo")]
    NilId,

    /// O identificador do usuário associado não pode ser o UUID nulo.
    #[error("o identificador do usuário associado não pode ser o UUID nulo")]
    NilUserId,

    /// O Firebase UID não pode ser vazio.
    #[error("o identificador Firebase UID não pode ser vazio")]
    EmptyFirebaseUid,

    /// O Firebase UID excede o limite máximo permitido.
    #[error("o identificador Firebase UID excede o limite de {max} caracteres")]
    FirebaseUidTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },

    /// O Microsoft Subject não pode ser vazio.
    #[error("o identificador Microsoft Subject não pode ser vazio")]
    EmptyMicrosoftSubject,

    /// O Microsoft Subject excede o limite máximo permitido.
    #[error("o identificador Microsoft Subject excede o limite de {max} caracteres")]
    MicrosoftSubjectTooLong {
        /// Limite máximo em caracteres.
        max: usize,
    },
}

/// Entidade de domínio que representa a identidade externa autenticada do usuário.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExternalIdentity {
    /// Identificador interno único deste registro de identidade externa.
    pub id: Uuid,
    /// Identificador do usuário interno no CApp (`app_user.id`).
    pub user_id: Uuid,
    /// Identificador opcional da instituição de ensino parceira (`institution.id`).
    pub institution_id: Option<Uuid>,
    /// Identificador estável do usuário no Firebase Authentication.
    pub firebase_uid: String,
    /// Identificador do sujeito autenticado na Microsoft institucional.
    pub microsoft_subject: String,
    /// Instante do último login registrado por esta identidade.
    pub last_login_at: Option<DateTime<Utc>>,
}

impl ExternalIdentity {
    /// Cria uma nova instância de identidade externa validando suas invariantes.
    ///
    /// # Parâmetros
    ///
    /// - `id`: identificador estável da identidade externa.
    /// - `user_id`: identificador do usuário interno associado.
    /// - `institution_id`: identificador opcional da instituição parceira.
    /// - `firebase_uid`: identificador do Firebase Authentication.
    /// - `microsoft_subject`: identificador do sujeito institucional Microsoft.
    /// - `last_login_at`: instante do último acesso registrado.
    ///
    /// # Erros
    ///
    /// Retorna [`ExternalIdentityError`] se algum identificador for nulo ou se
    /// os campos de texto forem vazios ou ultrapassarem os limites máximos.
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        institution_id: Option<Uuid>,
        firebase_uid: impl Into<String>,
        microsoft_subject: impl Into<String>,
        last_login_at: Option<DateTime<Utc>>,
    ) -> Result<Self, ExternalIdentityError> {
        let instance = Self {
            id,
            user_id,
            institution_id,
            firebase_uid: firebase_uid.into().trim().to_string(),
            microsoft_subject: microsoft_subject.into().trim().to_string(),
            last_login_at,
        };
        instance.validate()?;
        Ok(instance)
    }

    /// Cria a identidade externa didática para o estudante veterano 1 da UnB.
    pub fn veterano_didactic(user_id: Uuid) -> Self {
        let reference_time = DateTime::from_timestamp(1_767_225_600, 0);
        Self {
            id: VETERANO_1_EXTERNAL_IDENTITY_ID,
            user_id,
            institution_id: None,
            firebase_uid: "firebase-uid-veterano-1".to_string(),
            microsoft_subject: "sub-unb-231012345@aluno.unb.br".to_string(),
            last_login_at: reference_time,
        }
    }

    /// Cria a identidade externa didática para o estudante calouro da UnB.
    pub fn calouro_didactic(user_id: Uuid) -> Self {
        let reference_time = DateTime::from_timestamp(1_767_225_600, 0);
        Self {
            id: CALOURO_EXTERNAL_IDENTITY_ID,
            user_id,
            institution_id: None,
            firebase_uid: "firebase-uid-calouro".to_string(),
            microsoft_subject: "sub-unb-262001234@aluno.unb.br".to_string(),
            last_login_at: reference_time,
        }
    }

    /// Valida as invariantes de domínio da identidade externa.
    ///
    /// # Erros
    ///
    /// Retorna [`ExternalIdentityError`] caso algum campo viole as restrições de integridade.
    pub fn validate(&self) -> Result<(), ExternalIdentityError> {
        if self.id.is_nil() {
            return Err(ExternalIdentityError::NilId);
        }
        if self.user_id.is_nil() {
            return Err(ExternalIdentityError::NilUserId);
        }

        if self.firebase_uid.is_empty() {
            return Err(ExternalIdentityError::EmptyFirebaseUid);
        }
        if self.firebase_uid.chars().count() > MAX_FIREBASE_UID_LENGTH {
            return Err(ExternalIdentityError::FirebaseUidTooLong {
                max: MAX_FIREBASE_UID_LENGTH,
            });
        }

        if self.microsoft_subject.is_empty() {
            return Err(ExternalIdentityError::EmptyMicrosoftSubject);
        }
        if self.microsoft_subject.chars().count() > MAX_MICROSOFT_SUBJECT_LENGTH {
            return Err(ExternalIdentityError::MicrosoftSubjectTooLong {
                max: MAX_MICROSOFT_SUBJECT_LENGTH,
            });
        }

        Ok(())
    }

    /// Atualiza o instante do último login autenticado com esta identidade.
    pub fn record_login(&mut self, timestamp: DateTime<Utc>) {
        self.last_login_at = Some(timestamp);
    }

    /// Retorna o identificador do registro da identidade externa.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Retorna o identificador do usuário associado.
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Retorna o identificador opcional da instituição de ensino parceira.
    pub const fn institution_id(&self) -> Option<Uuid> {
        self.institution_id
    }

    /// Retorna o identificador único do Firebase Authentication.
    pub fn firebase_uid(&self) -> &str {
        &self.firebase_uid
    }

    /// Retorna o identificador do sujeito da Microsoft institucional.
    pub fn microsoft_subject(&self) -> &str {
        &self.microsoft_subject
    }

    /// Retorna o instante do último login registrado, se houver.
    pub const fn last_login_at(&self) -> Option<DateTime<Utc>> {
        self.last_login_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_external_identity() {
        let now = Utc::now();
        let user_id = Uuid::new_v4();
        let id = Uuid::new_v4();

        let identity = ExternalIdentity::new(
            id,
            user_id,
            None,
            "firebase-12345",
            "sub-ms-54321",
            Some(now),
        )
        .expect("should create valid identity");

        assert_eq!(identity.id(), id);
        assert_eq!(identity.user_id(), user_id);
        assert_eq!(identity.firebase_uid(), "firebase-12345");
        assert_eq!(identity.microsoft_subject(), "sub-ms-54321");
        assert_eq!(identity.last_login_at(), Some(now));
    }

    #[test]
    fn should_record_login_update() {
        let user_id = Uuid::new_v4();
        let mut identity = ExternalIdentity::calouro_didactic(user_id);
        let previous = identity.last_login_at();

        let new_login = Utc::now();
        identity.record_login(new_login);

        assert_eq!(identity.last_login_at(), Some(new_login));
        assert_ne!(identity.last_login_at(), previous);
    }

    #[test]
    fn should_reject_nil_identifiers() {
        let now = Utc::now();
        let valid_id = Uuid::new_v4();

        let err1 = ExternalIdentity::new(Uuid::nil(), valid_id, None, "uid", "sub", Some(now))
            .unwrap_err();
        assert_eq!(err1, ExternalIdentityError::NilId);

        let err2 = ExternalIdentity::new(valid_id, Uuid::nil(), None, "uid", "sub", Some(now))
            .unwrap_err();
        assert_eq!(err2, ExternalIdentityError::NilUserId);
    }

    #[test]
    fn should_reject_empty_tokens() {
        let now = Utc::now();
        let valid_id = Uuid::new_v4();

        let err1 =
            ExternalIdentity::new(valid_id, valid_id, None, "   ", "sub", Some(now)).unwrap_err();
        assert_eq!(err1, ExternalIdentityError::EmptyFirebaseUid);

        let err2 =
            ExternalIdentity::new(valid_id, valid_id, None, "uid", "   ", Some(now)).unwrap_err();
        assert_eq!(err2, ExternalIdentityError::EmptyMicrosoftSubject);
    }
}
