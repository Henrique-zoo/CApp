//! Configuração institucional consultada a partir do tenant Microsoft.

use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::modules::identity::domain::CatalogStatus;

/// Instituição cadastrada e configuração de verificação de estudantes.
///
/// Corresponde à tabela `institution`; o tenant possui unicidade no PostgreSQL.
/// O grupo é um identificador externo e sua correspondência com o tenant deve ser
/// garantida por quem configura a instituição. Não contém credenciais Microsoft.
/// O derive [`FromRow`] permite leitura direta das colunas pelo repositório.
#[derive(Debug, Clone, FromRow)]
pub struct Institution {
    /// Identificador interno gerado pelo banco quando omitido na inserção.
    pub id: Uuid,
    /// Nome público da instituição, usado também em mensagens de vínculo recusado.
    pub name: String,
    /// Sigla da instituição.
    pub acronym: Option<String>,
    /// UUID do diretório Microsoft; identifica unicamente a instituição cadastrada.
    pub microsoft_tenant_id: Uuid,
    /// UUID do grupo cuja participação comprova a elegibilidade exigida pelo CApp.
    pub microsoft_student_group_id: Uuid,
    /// Disponibilidade no catálogo; não é filtro de autorização na implementação atual.
    pub status: CatalogStatus,
}
