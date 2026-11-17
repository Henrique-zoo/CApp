//! Estados reutilizados pelos cadastros institucionais.

/// Disponibilidade de um registro no catálogo.
///
/// Mapeado ao enum PostgreSQL `catalog_status`, com rótulos `active` e `inactive`.
/// O derive não cria o tipo no banco; isso é responsabilidade da migration.
/// Este estado não comprova vínculo estudantil e não é usado atualmente como
/// critério de autorização pelo serviço de autenticação.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "catalog_status", rename_all = "lowercase")]
pub enum CatalogStatus {
    /// Disponível para novos usos.
    #[default]
    Active,
    /// Indisponível para novos usos, preservando os registros históricos.
    Inactive,
}

/// Ciclo de vida interno: incompleta, habilitada ou desativada.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    /// Identidade provisionada, aguardando os dados e a credencial interna.
    #[default]
    Incomplete,
    /// Conta CApp habilitada.
    Active,
    /// Conta CApp desativada.
    Inactive,
}
