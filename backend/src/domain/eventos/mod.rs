//! Reexportação dos tipos, entidades e regras de negócio do domínio de Eventos.
//!
//! Permite acessar o modelo de domínio de eventos tanto via `backend::domain::eventos`
//! quanto via `backend::modules::events::domain`.

pub use crate::modules::events::domain::*;

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::*;

    #[test]
    fn should_access_event_domain_from_eventos_module() {
        let starts_at = Utc.with_ymd_and_hms(2026, 11, 10, 14, 0, 0).unwrap();
        let ends_at = Utc.with_ymd_and_hms(2026, 11, 10, 18, 0, 0).unwrap();

        let event = Event::create_published(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Semana Acadêmica de Engenharia de Redes",
            "Ciclo de palestras e oficinas técnicas abertas ao curso",
            "Auditório do Departamento de Engenharia Elétrica",
            starts_at,
            ends_at,
            EventRegistrationInfo::unrestricted(),
        )
        .expect("deve criar evento válido via reexportação");

        assert_eq!(event.status, EventPublicationStatus::Published);
    }
}
