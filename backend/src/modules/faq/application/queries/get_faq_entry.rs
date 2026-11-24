//! Query para obtenção de uma pergunta e resposta frequente individual por seu identificador.

use uuid::Uuid;

/// Parâmetros da consulta para obter uma pergunta frequente específica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetFaqEntryQuery {
    /// Identificador do Centro Acadêmico responsável pelo FAQ.
    pub academic_center_id: Uuid,

    /// Identificador único da entrada de FAQ desejada.
    pub faq_entry_id: Uuid,
}

impl GetFaqEntryQuery {
    /// Cria uma nova requisição de detalhamento de FAQ.
    pub fn new(academic_center_id: Uuid, faq_entry_id: Uuid) -> Self {
        Self {
            academic_center_id,
            faq_entry_id,
        }
    }
}
