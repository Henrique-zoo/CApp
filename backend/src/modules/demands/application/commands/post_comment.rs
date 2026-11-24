//! Comando para publicação de comentário ou interação em uma demanda.

use uuid::Uuid;

/// Dados de entrada necessários para publicar um comentário em uma demanda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostDemandCommentCommand {
    /// Identificador do Centro Acadêmico responsável pela demanda.
    pub academic_center_id: Uuid,

    /// Identificador da demanda alvo.
    pub demand_id: Uuid,

    /// Identificador do usuário autor do comentário.
    pub author_id: Uuid,

    /// Conteúdo textual do comentário.
    pub content: String,
}

impl PostDemandCommentCommand {
    /// Cria uma nova instância do comando de publicação de comentário.
    pub fn new(
        academic_center_id: Uuid,
        demand_id: Uuid,
        author_id: Uuid,
        content: String,
    ) -> Self {
        Self {
            academic_center_id,
            demand_id,
            author_id,
            content,
        }
    }
}
