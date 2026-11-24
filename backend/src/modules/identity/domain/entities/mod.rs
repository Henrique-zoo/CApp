//! Entidades usadas na verificação institucional e ciclo de vida de dados discentes.
//!
//! [`institution`] configura o tenant e o grupo de estudantes.
//! [`institutional_verification`] registra evidência por UID Firebase, independente
//! de provisionamento de usuário interno. Os derives SQLx mapeiam linhas para esses
//! tipos; não aplicam migrations nem consultam o banco.
//! [`user`] modela o usuário interno provisionado na plataforma.

pub mod institution;
pub mod institutional_verification;
pub mod user;
pub use crate::shared::retention::StudentRecord;
pub use academic_data::*;
pub use app_user::AppUser;
pub use app_user::AppUserError;
pub use app_user::AppUserStatus;
pub use app_user::CALOURO_DIDACTIC_ID;
pub use app_user::MAX_DISPLAY_NAME_LENGTH;
pub use app_user::MAX_EMAIL_LENGTH;
pub use app_user::PROHIBITED_MATRICULA;
pub use app_user::VETERANO_1_DIDACTIC_ID;
pub use app_user::VETERANO_2_DIDACTIC_ID;
use chrono::DateTime;
use chrono::Utc;
pub use institution::*;
pub use institutional_verification::*;
use serde::Deserialize;
use serde::Serialize;
pub use user::*;
pub use user_preference::UserPreference;
pub use user_preference::UserPreferenceError;
use uuid::Uuid;
/// Estado de ciclo de vida e retenção da identidade de um discente na plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityRetentionStatus {
    /// Discente com vínculo regular ativo na UnB.
    Active,

    /// Discente desvinculado (formatura, trancamento ou desistência) em janela de tolerância de 180 dias.
    InactiveGracePeriod {
        /// Data em que o vínculo regular com a UnB foi formalmente encerrado.
        unlinked_at: DateTime<Utc>,
    },

    /// Registro discente cujos dados pessoais foram irreversivelmente anonimizados.
    Anonymized {
        /// Data em que a anonimização irreversível foi executada.
        anonymized_at: DateTime<Utc>,
    },
}
/// Entidade de domínio que representa a identidade acadêmica discente no CApp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudentIdentity {
    /// Identificador global único do usuário (UUID).
    pub id: Uuid,

    /// Matrícula institucional na UnB (9 dígitos numéricos).
    pub matricula: String,

    /// E-mail institucional oficial (`<matricula>@aluno.unb.br`).
    pub email: String,

    /// Nome civil ou social do discente.
    pub name: String,

    /// Status atual de retenção e ciclo de vida do registro.
    pub retention_status: IdentityRetentionStatus,
}
impl StudentIdentity {
    /// Constrói uma nova identidade discente ativa.
    pub fn new_active(
        id: Uuid,
        matricula: impl Into<String>,
        email: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, crate::shared::retention::RetentionError> {
        let matricula = matricula.into();
        let email = email.into();

        crate::shared::retention::PiiSanitizer::validate_matricula(&matricula)?;
        crate::shared::retention::PiiSanitizer::validate_unb_email(&email)?;

        Ok(Self {
            id,
            matricula,
            email,
            name: name.into(),
            retention_status: IdentityRetentionStatus::Active,
        })
    }
}
pub mod academic_data;
pub mod app_user;
pub mod user_preference;
