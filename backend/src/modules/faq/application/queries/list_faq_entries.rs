//! Query para listagem e navegação de entradas de FAQ de um Centro Acadêmico.

use uuid::Uuid;

/// Parâmetros da consulta para listagem de perguntas e respostas de FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFaqEntriesQuery {
    /// Identificador do Centro Acadêmico cujas perguntas serão listadas.
    pub academic_center_id: Uuid,

    /// Filtro opcional por categoria temática.
    pub category: Option<String>,

    /// Termo de busca textual opcional.
    pub search: Option<String>,
}

impl ListFaqEntriesQuery {
    /// Cria uma nova requisição de listagem de FAQ.
    pub fn new(academic_center_id: Uuid, category: Option<String>, search: Option<String>) -> Self {
        Self {
            academic_center_id,
            category,
            search,
        }
    }
}
