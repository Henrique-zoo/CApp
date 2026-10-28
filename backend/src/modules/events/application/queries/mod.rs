//! Consultas da camada de aplicação do módulo de eventos.

use uuid::Uuid;

/// Consulta para buscar um evento específico pelo seu identificador único.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetEventByIdQuery {
    /// Identificador único do evento.
    pub event_id: Uuid,
    /// Quando `true`, oculta eventos que ainda não estejam publicados (ex.: visão pública de estudantes).
    pub only_published: bool,
}

/// Consulta para listar os eventos pertencentes a um Centro Acadêmico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListCaEventsQuery {
    /// Identificador do Centro Acadêmico.
    pub academic_center_id: Uuid,
    /// Quando `true`, retorna apenas os eventos publicados na agenda oficial do CA.
    pub only_published: bool,
}
