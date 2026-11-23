//! Comando para criação de nova demanda acadêmica.

use uuid::Uuid;

use crate::modules::demands::domain::entities::DemandForum;

/// Dados de entrada necessários para a criação de uma demanda em qualquer um dos dois fóruns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDemandCommand {
    /// Identificador do Centro Acadêmico alvo.
    pub academic_center_id: Uuid,

    /// Identificador do autor autenticado solicitante.
    pub author_id: Uuid,

    /// Fórum selecionado para a demanda (Institucional ou Espaço Físico).
    pub forum: DemandForum,

    /// Título ou resumo opcional da demanda.
    pub title: Option<String>,

    /// Descrição detalhada do problema ou necessidade.
    pub description: String,

    /// Categoria temática opcional para triagem.
    pub category: Option<String>,

    /// Nome do curso associado (obrigatório no fórum institucional, proibido no espaço físico).
    pub course_name: Option<String>,
}

impl CreateDemandCommand {
    /// Cria uma nova instância do comando de criação de demanda.
    pub fn new(
        academic_center_id: Uuid,
        author_id: Uuid,
        forum: DemandForum,
        title: Option<String>,
        description: String,
        category: Option<String>,
        course_name: Option<String>,
    ) -> Self {
        Self {
            academic_center_id,
            author_id,
            forum,
            title,
            description,
            category,
            course_name,
        }
    }
}
