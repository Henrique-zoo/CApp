//! Revisão imutável do calendário eleitoral.

use chrono::{DateTime, Timelike, Utc};

use crate::modules::elections::domain::value_objects::{
    election_period::ElectionPeriod,
    fingerprint::Fingerprint,
    identifiers::{ActorId, ElectionId},
};

/// Versão do formato das revisões de calendário.
pub const SCHEDULE_CHANGE_VERSION: u8 = 1;

/// Maior número de revisão permitido pelo contrato.
pub const MAX_SCHEDULE_REVISION: u32 = 100_000;

/// Comprimento máximo da justificativa.
pub const MAX_SCHEDULE_REASON_LENGTH: usize = 2_000;

/// Revisão publicada do calendário de uma eleição.
///
/// Este tipo valida a estrutura da revisão. A permissão para alterar datas
/// conforme o estado da eleição é responsabilidade do agregado `Election`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleChange {
    election_id: ElectionId,
    manifest_hash: Option<Fingerprint>,
    revision: u32,
    previous_schedule_hash: Fingerprint,
    period: ElectionPeriod,
    changed_by: ActorId,
    changed_at: DateTime<Utc>,
    reason: String,
}

/// Erros encontrados ao construir uma revisão de calendário.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidScheduleChange {
    /// O número da revisão deve estar entre 1 e 100.000.
    #[error("O número da revisão deve estar entre 1 e 100000")]
    InvalidRevision,

    /// O instante da alteração deve possuir precisão exata de segundos.
    #[error("O instante da alteração deve ter precisão exata de segundos")]
    SubsecondPrecision,

    /// A justificativa deve possuir entre 1 e 2.000 caracteres.
    #[error("A justificativa deve possuir entre 1 e 2000 caracteres")]
    InvalidReasonLength,
}

impl ScheduleChange {
    /// Constrói uma revisão validando os limites definidos pelo contrato.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        election_id: ElectionId,
        manifest_hash: Option<Fingerprint>,
        revision: u32,
        previous_schedule_hash: Fingerprint,
        period: ElectionPeriod,
        changed_by: ActorId,
        changed_at: DateTime<Utc>,
        reason: impl Into<String>,
    ) -> Result<Self, InvalidScheduleChange> {
        if !(1..=MAX_SCHEDULE_REVISION).contains(&revision) {
            return Err(InvalidScheduleChange::InvalidRevision);
        }

        if changed_at.nanosecond() != 0 {
            return Err(InvalidScheduleChange::SubsecondPrecision);
        }

        let reason = reason.into();
        let reason_length = reason.chars().count();

        if !(1..=MAX_SCHEDULE_REASON_LENGTH).contains(&reason_length) {
            return Err(InvalidScheduleChange::InvalidReasonLength);
        }

        Ok(Self {
            election_id,
            manifest_hash,
            revision,
            previous_schedule_hash,
            period,
            changed_by,
            changed_at,
            reason,
        })
    }

    /// Retorna o identificador da eleição.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Retorna o hash do manifesto, quando já publicado.
    pub const fn manifest_hash(&self) -> Option<&Fingerprint> {
        self.manifest_hash.as_ref()
    }

    /// Retorna o número da revisão.
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    /// Retorna o hash da revisão anterior.
    pub const fn previous_schedule_hash(&self) -> &Fingerprint {
        &self.previous_schedule_hash
    }

    /// Retorna o período desta revisão.
    pub const fn period(&self) -> &ElectionPeriod {
        &self.period
    }

    /// Retorna o identificador do responsável pela alteração.
    pub const fn changed_by(&self) -> ActorId {
        self.changed_by
    }

    /// Retorna o instante da alteração.
    pub const fn changed_at(&self) -> &DateTime<Utc> {
        &self.changed_at
    }

    /// Retorna a justificativa da alteração.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Retorna a versão do formato.
    pub const fn version(&self) -> u8 {
        SCHEDULE_CHANGE_VERSION
    }

    /// Retorna o tipo do objeto no contrato.
    pub const fn kind(&self) -> &'static str {
        "schedule-change"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::domain::value_objects::fingerprint::Fingerprint;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn election_id() -> ElectionId {
        ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap()
    }

    fn actor_id() -> ActorId {
        ActorId::try_from_uuid(Uuid::from_u128(2)).unwrap()
    }

    fn fingerprint() -> Fingerprint {
        Fingerprint::try_from_hex("a".repeat(64)).unwrap()
    }

    fn period() -> ElectionPeriod {
        let starts_at = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
        let ends_at = Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap();

        ElectionPeriod::try_new(starts_at, ends_at).unwrap()
    }

    fn changed_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap()
    }

    fn build_change(
        revision: u32,
        manifest_hash: Option<Fingerprint>,
        changed_at: DateTime<Utc>,
        reason: &str,
    ) -> Result<ScheduleChange, InvalidScheduleChange> {
        ScheduleChange::try_new(
            election_id(),
            manifest_hash,
            revision,
            fingerprint(),
            period(),
            actor_id(),
            changed_at,
            reason,
        )
    }

    #[test]
    fn creates_valid_schedule_change_before_manifest() {
        let change = build_change(1, None, changed_at(), "Adiamento aprovado").unwrap();

        assert_eq!(change.election_id(), election_id());
        assert_eq!(change.manifest_hash(), None);
        assert_eq!(change.revision(), 1);
        assert_eq!(change.changed_by(), actor_id());
        assert_eq!(change.reason(), "Adiamento aprovado");
    }

    #[test]
    fn creates_valid_schedule_change_with_manifest() {
        let manifest_hash = fingerprint();

        let change =
            build_change(2, Some(manifest_hash.clone()), changed_at(), "Prorrogação").unwrap();

        assert_eq!(change.manifest_hash(), Some(&manifest_hash));
    }

    #[test]
    fn exposes_contract_version_and_kind() {
        let change = build_change(1, None, changed_at(), "Motivo").unwrap();

        assert_eq!(change.version(), 1);
        assert_eq!(change.kind(), "schedule-change");
    }

    #[test]
    fn rejects_zero_revision() {
        assert_eq!(
            build_change(0, None, changed_at(), "Motivo"),
            Err(InvalidScheduleChange::InvalidRevision)
        );
    }

    #[test]
    fn accepts_maximum_revision() {
        assert!(build_change(MAX_SCHEDULE_REVISION, None, changed_at(), "Motivo").is_ok());
    }

    #[test]
    fn rejects_revision_above_contract_limit() {
        assert_eq!(
            build_change(MAX_SCHEDULE_REVISION + 1, None, changed_at(), "Motivo"),
            Err(InvalidScheduleChange::InvalidRevision)
        );
    }

    #[test]
    fn rejects_fractional_seconds_in_changed_at() {
        let instant = changed_at().with_nanosecond(1).unwrap();

        assert_eq!(
            build_change(1, None, instant, "Motivo"),
            Err(InvalidScheduleChange::SubsecondPrecision)
        );
    }

    #[test]
    fn rejects_empty_reason() {
        assert_eq!(
            build_change(1, None, changed_at(), ""),
            Err(InvalidScheduleChange::InvalidReasonLength)
        );
    }

    #[test]
    fn accepts_reason_with_exactly_2000_characters() {
        let reason = "a".repeat(MAX_SCHEDULE_REASON_LENGTH);

        assert!(build_change(1, None, changed_at(), &reason).is_ok());
    }

    #[test]
    fn rejects_reason_longer_than_2000_characters() {
        let reason = "a".repeat(MAX_SCHEDULE_REASON_LENGTH + 1);

        assert_eq!(
            build_change(1, None, changed_at(), &reason),
            Err(InvalidScheduleChange::InvalidReasonLength)
        );
    }

    #[test]
    fn accepts_reason_containing_only_spaces_as_schema_allows() {
        assert!(build_change(1, None, changed_at(), "   ").is_ok());
    }
}
