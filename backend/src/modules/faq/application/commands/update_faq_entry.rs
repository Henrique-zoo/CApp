//! Comando para atualização de uma pergunta e resposta de FAQ.

use uuid::Uuid;

/// Dados necessários para atualizar uma entrada de FAQ existente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateFaqEntryCommand {
    /// Identificador do Centro Acadêmico proprietário do FAQ.
    pub academic_center_id: Uuid,

    /// Identificador da pergunta e resposta a ser atualizada.
    pub faq_entry_id: Uuid,

    /// Identificador do autor autenticado solicitante da alteração.
    pub author_id: Uuid,

    /// Novo texto opcional da pergunta frequente.
    pub question: Option<String>,

    /// Novo texto opcional da resposta oficial.
    pub answer: Option<String>,

    /// Nova categoria temática opcional da dúvida frequente.
    pub category: Option<String>,
}

impl UpdateFaqEntryCommand {
    /// Cria uma nova instância do comando de atualização de FAQ.
    pub fn new(
        academic_center_id: Uuid,
        faq_entry_id: Uuid,
        author_id: Uuid,
        question: Option<String>,
        answer: Option<String>,
        category: Option<String>,
    ) -> Self {
        Self {
            academic_center_id,
            faq_entry_id,
            author_id,
            question,
            answer,
            category,
        }
    }
}
