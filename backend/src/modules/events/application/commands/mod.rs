//! Comandos da camada de aplicação para criação, edição e publicação de eventos.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::events::domain::EventRegistrationInfo;

/// Comando para criar um novo evento (em rascunho ou publicado diretamente) em um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateEventCommand {
    /// Identificador do Centro Acadêmico responsável pelo evento.
    pub academic_center_id: Uuid,
    /// Identificador do gestor/usuário autor do evento.
    pub author_id: Uuid,
    /// Título do evento.
    pub title: String,
    /// Descrição do evento.
    pub description: String,
    /// Local de realização do evento.
    pub location: String,
    /// Data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Data e horário de término do evento.
    pub ends_at: DateTime<Utc>,
    /// Quando `true`, publica o evento diretamente sem exigir homologação; quando `false`, salva como rascunho.
    pub publish_directly: bool,
    /// Configuração de inscrição e limite opcional de vagas.
    pub registration: EventRegistrationInfo,
}

/// Comando para editar as informações de um evento existente no Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEventCommand {
    /// Identificador do evento a ser editado.
    pub event_id: Uuid,
    /// Identificador opcional do Centro Acadêmico no qual a operação está sendo solicitada.
    pub academic_center_id: Option<Uuid>,
    /// Identificador opcional do usuário que solicita a edição.
    pub actor_id: Option<Uuid>,
    /// Novo título do evento.
    pub title: String,
    /// Nova descrição do evento.
    pub description: String,
    /// Novo local de realização do evento.
    pub location: String,
    /// Nova data e horário de início do evento.
    pub starts_at: DateTime<Utc>,
    /// Nova data e horário de término do evento.
    pub ends_at: DateTime<Utc>,
    /// Nova configuração opcional de inscrição.
    pub registration: Option<EventRegistrationInfo>,
}

/// Comando para publicar diretamente um evento salvo em rascunho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishEventCommand {
    /// Identificador do evento a ser publicado.
    pub event_id: Uuid,
    /// Identificador opcional do Centro Acadêmico no qual a publicação é solicitada.
    pub academic_center_id: Option<Uuid>,
    /// Identificador opcional do gestor que executa a publicação direta.
    pub actor_id: Option<Uuid>,
}
