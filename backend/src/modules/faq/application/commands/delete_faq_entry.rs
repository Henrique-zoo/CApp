//! Comando para remoção de uma pergunta e resposta de FAQ.

use uuid::Uuid;

/// Dados necessários para remover uma entrada de FAQ existente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteFaqEntryCommand {
    /// Identificador do Centro Acadêmico proprietário do FAQ.
    pub academic_center_id: Uuid,

    /// Identificador da pergunta e resposta a ser removida.
    pub faq_entry_id: Uuid,

    /// Identificador do autor autenticado solicitante da remoção.
    pub author_id: Uuid,
}

impl DeleteFaqEntryCommand {
    /// Cria uma nova instância do comando de remoção de FAQ.
    pub fn new(academic_center_id: Uuid, faq_entry_id: Uuid, author_id: Uuid) -> Self {
        Self {
            academic_center_id,
            faq_entry_id,
            author_id,
        }
    }
}
