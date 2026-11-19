//! Registro imutável do encerramento de uma eleição.

use chrono::{DateTime, Timelike, Utc};

use crate::modules::elections::domain::rules::board_snapshot::validate_confidential_board_snapshot;
use crate::modules::elections::domain::value_objects::{
    fingerprint::Fingerprint, identifiers::ElectionId, modality::ElectionModality,
    voter_count::VoterCount,
};

/// Versão do formato do encerramento eleitoral.
pub const ELECTION_CLOSURE_VERSION: u8 = 1;

/// Registro imutável da fotografia final de uma votação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionClosure {
    election_id: ElectionId,
    manifest_hash: Option<Fingerprint>,
    schedule_hash: Fingerprint,
    head_hash: Option<Fingerprint>,
    voter_count: VoterCount,
    closed_at: DateTime<Utc>,
}

/// Erros encontrados ao construir um encerramento eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionClosure {
    /// A origem do quadro confidencial corresponde exatamente à contagem zero.
    #[error("A cabeça do quadro é incompatível com a contagem de participantes")]
    InvalidEmptyBoard,
    /// O instante do encerramento deve possuir precisão exata de segundos.
    #[error("O instante do encerramento deve ter precisão exata de segundos")]
    SubsecondPrecision,
}

impl ElectionClosure {
    /// Constrói o registro confidencial conferindo precisão e origem do quadro vazio.
    ///
    /// A aplicação fornece a fotografia confirmada. O agregado confere também
    /// estado, referências ao calendário e prazo antes de aceitar o encerramento.
    pub fn try_new(
        election_id: ElectionId,
        manifest_hash: Fingerprint,
        schedule_hash: Fingerprint,
        head_hash: Fingerprint,
        voter_count: VoterCount,
        closed_at: DateTime<Utc>,
    ) -> Result<Self, InvalidElectionClosure> {
        if closed_at.nanosecond() != 0 {
            return Err(InvalidElectionClosure::SubsecondPrecision);
        }
        validate_confidential_board_snapshot(Some(&manifest_hash), Some(&head_hash), voter_count)
            .map_err(|_| InvalidElectionClosure::InvalidEmptyBoard)?;

        Ok(Self {
            election_id,
            manifest_hash: Some(manifest_hash),
            schedule_hash,
            head_hash: Some(head_hash),
            voter_count,
            closed_at,
        })
    }

    /// Constrói uma fotografia identificada sem exigir artefatos confidenciais.
    ///
    /// A contagem é obtida dos registros identificados pela aplicação. Nenhum
    /// segredo, escolha individual ou dado pessoal integra esta fotografia.
    pub fn try_identified(
        election_id: ElectionId,
        schedule_hash: Fingerprint,
        voter_count: VoterCount,
        closed_at: DateTime<Utc>,
    ) -> Result<Self, InvalidElectionClosure> {
        if closed_at.nanosecond() != 0 {
            return Err(InvalidElectionClosure::SubsecondPrecision);
        }
        Ok(Self {
            election_id,
            manifest_hash: None,
            schedule_hash,
            head_hash: None,
            voter_count,
            closed_at,
        })
    }

    /// Retorna a modalidade da fotografia.
    pub const fn modality(&self) -> ElectionModality {
        if self.manifest_hash.is_some() {
            ElectionModality::Confidential
        } else {
            ElectionModality::Identified
        }
    }

    /// Retorna o identificador da eleição.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Retorna o hash do manifesto.
    pub const fn manifest_hash(&self) -> Option<&Fingerprint> {
        self.manifest_hash.as_ref()
    }

    /// Retorna o hash do calendário vigente no encerramento.
    pub const fn schedule_hash(&self) -> &Fingerprint {
        &self.schedule_hash
    }

    /// Retorna o hash da cabeça do quadro de votos.
    pub const fn head_hash(&self) -> Option<&Fingerprint> {
        self.head_hash.as_ref()
    }

    /// Retorna a quantidade final de eleitores distintos.
    pub const fn voter_count(&self) -> VoterCount {
        self.voter_count
    }

    /// Retorna o instante do encerramento.
    pub const fn closed_at(&self) -> &DateTime<Utc> {
        &self.closed_at
    }

    /// Retorna a versão do formato.
    pub const fn version(&self) -> u8 {
        ELECTION_CLOSURE_VERSION
    }

    /// Retorna o tipo do objeto no contrato.
    pub const fn kind(&self) -> &'static str {
        "closure"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::domain::value_objects::voter_count::VoterCount;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn election_id() -> ElectionId {
        ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap()
    }

    fn fingerprint(value: char) -> Fingerprint {
        Fingerprint::try_from_hex(value.to_string().repeat(64)).unwrap()
    }

    fn voter_count() -> VoterCount {
        VoterCount::try_new(42).unwrap()
    }

    fn closed_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 20, 18, 0, 0).unwrap()
    }

    fn build_closure(closed_at: DateTime<Utc>) -> Result<ElectionClosure, InvalidElectionClosure> {
        ElectionClosure::try_new(
            election_id(),
            fingerprint('a'),
            fingerprint('b'),
            fingerprint('c'),
            voter_count(),
            closed_at,
        )
    }

    #[test]
    fn creates_valid_closure() {
        let closure = build_closure(closed_at()).unwrap();

        assert_eq!(closure.election_id(), election_id());
        assert_eq!(closure.manifest_hash(), Some(&fingerprint('a')));
        assert_eq!(closure.schedule_hash(), &fingerprint('b'));
        assert_eq!(closure.head_hash(), Some(&fingerprint('c')));
        assert_eq!(closure.voter_count().value(), 42);
        assert_eq!(closure.closed_at(), &closed_at());
        assert_eq!(closure.version(), ELECTION_CLOSURE_VERSION);
        assert_eq!(closure.kind(), "closure");
    }

    #[test]
    fn accepts_zero_voters() {
        let closure = ElectionClosure::try_new(
            election_id(),
            fingerprint('a'),
            fingerprint('b'),
            fingerprint('a'),
            VoterCount::try_new(0).unwrap(),
            closed_at(),
        )
        .unwrap();

        assert_eq!(closure.voter_count().value(), 0);
        assert_eq!(closure.head_hash(), closure.manifest_hash());
    }

    #[test]
    fn rejects_fractional_seconds() {
        let instant = closed_at().with_nanosecond(1).unwrap();

        assert_eq!(
            build_closure(instant),
            Err(InvalidElectionClosure::SubsecondPrecision)
        );
    }
}
