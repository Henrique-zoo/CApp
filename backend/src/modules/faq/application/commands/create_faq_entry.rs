//! Comando para cadastro de uma nova pergunta e resposta de FAQ.

use uuid::Uuid;

/// Dados necessários para cadastrar uma nova entrada de FAQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateFaqEntryCommand {
    /// Identificador do Centro Acadêmico proprietário do FAQ.
    pub academic_center_id: Uuid,

    /// Identificador do autor autenticado solicitante.
    pub author_id: Uuid,

    /// Texto da pergunta frequente.
    pub question: String,

    /// Texto da resposta oficial.
    pub answer: String,

    /// Categoria opcional da dúvida frequente.
    pub category: Option<String>,
}

impl CreateFaqEntryCommand {
    /// Cria uma nova instância do comando de cadastro de FAQ.
    pub fn new(
        academic_center_id: Uuid,
        author_id: Uuid,
        question: String,
        answer: String,
        category: Option<String>,
    ) -> Self {
        Self {
            academic_center_id,
            author_id,
            question,
            answer,
            category,
        }
    }
}
