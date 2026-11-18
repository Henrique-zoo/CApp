//! Período de vigência de uma eleição.

use chrono::{DateTime, Timelike, Utc};

/// Intervalo de tempo de uma eleição.
///
/// O intervalo segue a convenção `[starts_at, ends_at)`: o instante inicial
/// está incluído e o instante final não está incluído.
///
/// Os instantes são armazenados em UTC e devem ter precisão exata de segundos,
/// conforme o contrato eleitoral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionPeriod {
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

/// Erro ao construir um período eleitoral inválido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionPeriod {
    /// O início deve ser estritamente anterior ao fim.
    #[error("O início do período deve ser anterior ao fim")]
    StartMustPrecedeEnd,

    /// O contrato eleitoral não permite frações de segundo.
    #[error("Os instantes do período devem ter precisão exata de segundos")]
    SubsecondPrecision,
}

impl ElectionPeriod {
    /// Constrói um período eleitoral validando ordem e precisão temporal.
    pub fn try_new(
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<Self, InvalidElectionPeriod> {
        if starts_at.nanosecond() != 0 || ends_at.nanosecond() != 0 {
            return Err(InvalidElectionPeriod::SubsecondPrecision);
        }

        if starts_at >= ends_at {
            return Err(InvalidElectionPeriod::StartMustPrecedeEnd);
        }

        Ok(Self { starts_at, ends_at })
    }

    /// Retorna o instante de início do período.
    pub fn starts_at(&self) -> &DateTime<Utc> {
        &self.starts_at
    }

    /// Retorna o instante de encerramento do período.
    pub fn ends_at(&self) -> &DateTime<Utc> {
        &self.ends_at
    }

    /// Informa se o instante está dentro do intervalo `[starts_at, ends_at)`.
    pub fn contains(&self, instant: &DateTime<Utc>) -> bool {
        instant >= &self.starts_at && instant < &self.ends_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    fn instant(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 16, hour, minute, second)
            .single()
            .unwrap()
    }

    #[test]
    fn accepts_valid_period() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0);

        let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

        assert_eq!(period.starts_at(), &starts_at);
        assert_eq!(period.ends_at(), &ends_at);
    }

    #[test]
    fn rejects_equal_start_and_end() {
        let value = instant(10, 0, 0);

        assert_eq!(
            ElectionPeriod::try_new(value, value),
            Err(InvalidElectionPeriod::StartMustPrecedeEnd)
        );
    }

    #[test]
    fn rejects_start_after_end() {
        let starts_at = instant(12, 0, 0);
        let ends_at = instant(10, 0, 0);

        assert_eq!(
            ElectionPeriod::try_new(starts_at, ends_at),
            Err(InvalidElectionPeriod::StartMustPrecedeEnd)
        );
    }

    #[test]
    fn rejects_fractional_seconds_in_start() {
        let starts_at = instant(10, 0, 0).with_nanosecond(1).unwrap();
        let ends_at = instant(12, 0, 0);

        assert_eq!(
            ElectionPeriod::try_new(starts_at, ends_at),
            Err(InvalidElectionPeriod::SubsecondPrecision)
        );
    }

    #[test]
    fn rejects_fractional_seconds_in_end() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0).with_nanosecond(500_000_000).unwrap();

        assert_eq!(
            ElectionPeriod::try_new(starts_at, ends_at),
            Err(InvalidElectionPeriod::SubsecondPrecision)
        );
    }

    #[test]
    fn contains_start_instant() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0);
        let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

        assert!(period.contains(&starts_at));
    }

    #[test]
    fn excludes_end_instant() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0);
        let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

        assert!(!period.contains(&ends_at));
    }

    #[test]
    fn contains_instant_inside_period() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0);
        let inside = instant(11, 0, 0);
        let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

        assert!(period.contains(&inside));
    }

    #[test]
    fn excludes_instant_before_period() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0);
        let before = instant(9, 59, 59);
        let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

        assert!(!period.contains(&before));
    }

    #[test]
    fn excludes_instant_after_period() {
        let starts_at = instant(10, 0, 0);
        let ends_at = instant(12, 0, 0);
        let after = instant(12, 0, 1);
        let period = ElectionPeriod::try_new(starts_at, ends_at).unwrap();

        assert!(!period.contains(&after));
    }
}
