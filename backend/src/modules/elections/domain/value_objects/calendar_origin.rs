//! Calendário inicial e imutável de uma eleição.

use crate::modules::elections::domain::value_objects::{
    election_period::ElectionPeriod, identifiers::ElectionId,
};

/// Versão do formato do calendário inicial.
pub const CALENDAR_ORIGIN_VERSION: u8 = 1;

/// Calendário original publicado antes da preparação das chaves.
///
/// O hash deste objeto será usado como referência inicial da cadeia
/// de revisões do calendário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarOrigin {
    election_id: ElectionId,
    period: ElectionPeriod,
}

impl CalendarOrigin {
    /// Constrói a origem do calendário com um período previamente validado.
    pub fn new(election_id: ElectionId, period: ElectionPeriod) -> Self {
        Self {
            election_id,
            period,
        }
    }

    /// Retorna o identificador da eleição.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Retorna o período original.
    pub const fn period(&self) -> &ElectionPeriod {
        &self.period
    }

    /// Retorna a versão do formato.
    pub const fn version(&self) -> u8 {
        CALENDAR_ORIGIN_VERSION
    }

    /// Retorna o tipo do objeto no contrato.
    pub const fn kind(&self) -> &'static str {
        "calendar-origin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn election_id() -> ElectionId {
        ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap()
    }

    fn period() -> ElectionPeriod {
        let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
        let ends_at = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();

        ElectionPeriod::try_new(starts_at, ends_at).unwrap()
    }

    #[test]
    fn creates_calendar_origin_with_validated_period() {
        let id = election_id();
        let period = period();

        let origin = CalendarOrigin::new(id, period.clone());

        assert_eq!(origin.election_id(), id);
        assert_eq!(origin.period(), &period);
    }

    #[test]
    fn exposes_contract_version_and_kind() {
        let origin = CalendarOrigin::new(election_id(), period());

        assert_eq!(origin.version(), 1);
        assert_eq!(origin.kind(), "calendar-origin");
    }
}
