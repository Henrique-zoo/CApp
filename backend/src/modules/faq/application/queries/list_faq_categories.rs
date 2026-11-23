//! Query para listagem de categorias disponíveis no FAQ de um Centro Acadêmico.

use uuid::Uuid;

/// Parâmetros da consulta para listagem de categorias do FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFaqCategoriesQuery {
    /// Identificador do Centro Acadêmico cujas categorias serão listadas.
    pub academic_center_id: Uuid,
}

impl ListFaqCategoriesQuery {
    /// Cria uma nova requisição de listagem de categorias de FAQ.
    pub fn new(academic_center_id: Uuid) -> Self {
        Self { academic_center_id }
    }
}
