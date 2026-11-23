//! Query para busca textual de perguntas frequentes de um Centro Acadêmico.

use uuid::Uuid;

/// Parâmetros da consulta para busca textual no FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFaqEntriesQuery {
    /// Identificador do Centro Acadêmico cujas perguntas serão pesquisadas.
    pub academic_center_id: Uuid,

    /// Termo de busca textual a ser procurado na pergunta e na resposta.
    pub search_term: String,

    /// Filtro opcional complementar por categoria temática.
    pub category: Option<String>,
}

impl SearchFaqEntriesQuery {
    /// Cria uma nova requisição de busca no FAQ.
    pub fn new(academic_center_id: Uuid, search_term: String, category: Option<String>) -> Self {
        Self {
            academic_center_id,
            search_term,
            category,
        }
    }
}
