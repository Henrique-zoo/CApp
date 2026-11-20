//! Eventos de domínio do módulo de identidade e dados acadêmicos.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Evento emitido quando dados acadêmicos institucionais são associados a um perfil de usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicDataAssociated {
    /// Identificador do usuário interno.
    pub user_id: Uuid,
    /// Identificador da instituição de ensino associada.
    pub institution_id: Uuid,
    /// Nome do curso identificado, se disponível.
    pub course_name: Option<String>,
    /// Indica se o vínculo discente regular foi confirmado.
    pub is_student: bool,
    /// Instante em que o evento ocorreu.
    pub occurred_at: DateTime<Utc>,
}

impl AcademicDataAssociated {
    /// Cria uma nova instância de evento de associação de dados acadêmicos.
    pub fn new(
        user_id: Uuid,
        institution_id: Uuid,
        course_name: Option<String>,
        is_student: bool,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            institution_id,
            course_name,
            is_student,
            occurred_at,
        }
    }
}
