//! Resultado de verificação reutilizado durante um prazo limitado.
//!
//! A tabela guarda um registro por UID Firebase, sem FK para usuário interno.
//! Uma nova resposta conclusiva pode substituir a anterior; indisponibilidade do
//! provedor não é registrada como ausência de vínculo. O repositório só reutiliza
//! resultados positivos, não expirados e compatíveis com a configuração atual.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Resultado persistido da última verificação institucional registrada.
///
/// Não representa uma sessão nem um usuário interno do CApp.
/// Tokens e credenciais não são armazenados neste registro.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InstitutionalVerification {
    /// UID autenticado no projeto Firebase configurado para o backend.
    pub firebase_uid: String,

    /// Identificador Microsoft confrontado com a identidade Firebase.
    pub microsoft_object_id: Uuid,

    /// Instituição à qual a consulta se refere.
    pub institution_id: Uuid,

    /// Tenant institucional usado durante a verificação.
    pub microsoft_tenant_id: Uuid,

    /// Grupo de estudantes consultado durante a verificação.
    pub microsoft_student_group_id: Uuid,

    /// Indica se o Graph confirmou participação no grupo.
    pub student_membership_confirmed: bool,

    /// Início da consulta institucional, usado para ordenar atualizações.
    pub checked_at: DateTime<Utc>,

    /// Limite para reutilizar uma confirmação positiva.
    pub expires_at: DateTime<Utc>,
}
