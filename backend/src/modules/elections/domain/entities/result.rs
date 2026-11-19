//! Resultado imutável da apuração eleitoral.

use crate::modules::elections::domain::value_objects::{
    fingerprint::Fingerprint,
    identifiers::{AuthorityId, ElectionId},
    modality::ElectionModality,
    option_count::OptionCount,
    voter_count::VoterCount,
};

/// Versão do contrato de resultado eleitoral.
pub const ELECTION_RESULT_VERSION: u8 = 1;

/// Estado publicado da apuração eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    /// Resultado válido e completo.
    Valid,

    /// Resultado anulado por inconsistência ou contagem fora do intervalo.
    Annulled,

    /// Apuração incompleta, aguardando contribuições válidas.
    Incomplete,

    /// Eleição cancelada administrativamente.
    Cancelled,
}

/// Motivo associado a um resultado eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultReason {
    /// Falta contribuição de uma ou mais autoridades.
    MissingAuthority,

    /// Uma ou mais contribuições de autoridade são inválidas.
    InvalidAuthorityShare,

    /// A soma das contagens não corresponde ao total de eleitores.
    AggregateSumMismatch,

    /// Uma contagem está fora do intervalo permitido.
    TallyOutOfRange,

    /// Cancelamento administrativo.
    ManualCancellation,
}

/// Dados imutáveis de um resultado eleitoral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectionResult {
    election_id: ElectionId,
    modality: ElectionModality,
    manifest_hash: Option<Fingerprint>,
    closure_hash: Option<Fingerprint>,
    aggregate_hash: Option<Fingerprint>,
    cancellation_hash: Option<Fingerprint>,
    voter_count: VoterCount,
    status: ResultStatus,
    reason: Option<ResultReason>,
    counts: Option<Vec<OptionCount>>,
    missing_authorities: Vec<AuthorityId>,
}

/// Erros encontrados ao construir um resultado eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidElectionResult {
    /// Uma contagem publicada deve pertencer ao intervalo `[0,N]`.
    #[error("Uma contagem excede o número de participantes")]
    CountOutOfRange,

    /// Hashes obrigatórios ausentes para o estado informado.
    #[error("Hashes obrigatórios ausentes para o estado do resultado")]
    MissingRequiredHash,

    /// Hash incompatível com o estado informado.
    #[error("Hash incompatível com o estado do resultado")]
    UnexpectedHash,

    /// Motivo incompatível com o estado informado.
    #[error("Motivo incompatível com o estado do resultado")]
    InvalidReason,

    /// Contagens obrigatórias ausentes.
    #[error("Contagens obrigatórias ausentes")]
    MissingCounts,

    /// Contagens não permitidas para o estado informado.
    #[error("Contagens não permitidas para o estado do resultado")]
    UnexpectedCounts,

    /// Quantidade de opções fora dos limites do contrato.
    #[error("A quantidade de opções deve estar entre 2 e 51")]
    InvalidCountLength,

    /// Identificadores de opção repetidos.
    #[error("Existem identificadores de opção repetidos")]
    DuplicateOption,

    /// Autoridades pendentes excedem o limite do contrato.
    #[error("A quantidade de autoridades pendentes excede 52")]
    TooManyMissingAuthorities,

    /// Autoridades pendentes repetidas.
    #[error("Existem autoridades pendentes repetidas")]
    DuplicateAuthority,

    /// Resultado válido não pode ter autoridades pendentes.
    #[error("Resultado válido não pode ter autoridades pendentes")]
    MissingAuthoritiesInValidResult,

    /// Resultado incompleto precisa de autoridades pendentes.
    #[error("Resultado incompleto exige autoridades pendentes")]
    NoMissingAuthorities,
}

impl ElectionResult {
    /// Constrói o resultado confidencial conforme os campos do contrato v1.
    ///
    /// Valida forma, campos condicionais, duplicatas e limites `[0,N]`. Ordem,
    /// soma, fotografia e contribuições são conferidas por `Election::publish_tally`
    /// ao consumir `VerifiedTally`. Não considera a assinatura uma prova de bits.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        election_id: ElectionId,
        manifest_hash: Option<Fingerprint>,
        closure_hash: Option<Fingerprint>,
        aggregate_hash: Option<Fingerprint>,
        cancellation_hash: Option<Fingerprint>,
        voter_count: VoterCount,
        status: ResultStatus,
        reason: Option<ResultReason>,
        counts: Option<Vec<OptionCount>>,
        missing_authorities: Vec<AuthorityId>,
    ) -> Result<Self, InvalidElectionResult> {
        Self::try_with_modality(
            ElectionModality::Confidential,
            election_id,
            manifest_hash,
            closure_hash,
            aggregate_hash,
            cancellation_hash,
            voter_count,
            status,
            reason,
            counts,
            missing_authorities,
        )
    }

    /// Constrói a contagem identificada sem manifesto, agregado ou autoridades.
    ///
    /// A soma determina `valid` ou `annulled`; a correspondência das opções é
    /// conferida ao publicar no agregado. Não decide vencedor, quórum ou posse.
    pub fn try_identified(
        election_id: ElectionId,
        closure_hash: Fingerprint,
        voter_count: VoterCount,
        counts: Vec<OptionCount>,
    ) -> Result<Self, InvalidElectionResult> {
        let consistent = counts
            .iter()
            .map(|c| u64::from(c.count().value()))
            .sum::<u64>()
            == u64::from(voter_count.value());
        Self::try_with_modality(
            ElectionModality::Identified,
            election_id,
            None,
            Some(closure_hash),
            None,
            None,
            voter_count,
            if consistent {
                ResultStatus::Valid
            } else {
                ResultStatus::Annulled
            },
            if consistent {
                None
            } else {
                Some(ResultReason::AggregateSumMismatch)
            },
            Some(counts),
            vec![],
        )
    }

    /// Representa cancelamento sem exigir manifesto na modalidade identificada.
    ///
    /// O digest é calculado pelo codec; a associação ao registro administrativo
    /// deve ser feita por `Election::cancelled_result`. Não produz contagens.
    pub fn try_cancelled(
        election_id: ElectionId,
        modality: ElectionModality,
        manifest_hash: Option<Fingerprint>,
        cancellation_hash: Fingerprint,
        voter_count: VoterCount,
    ) -> Result<Self, InvalidElectionResult> {
        if modality == ElectionModality::Identified && manifest_hash.is_some() {
            return Err(InvalidElectionResult::UnexpectedHash);
        }
        Self::try_with_modality(
            modality,
            election_id,
            manifest_hash,
            None,
            None,
            Some(cancellation_hash),
            voter_count,
            ResultStatus::Cancelled,
            Some(ResultReason::ManualCancellation),
            None,
            vec![],
        )
    }

    /// Valida estrutura e campos condicionais; o agregado confere contexto e ordem.
    #[allow(clippy::too_many_arguments)]
    fn try_with_modality(
        modality: ElectionModality,
        election_id: ElectionId,
        manifest_hash: Option<Fingerprint>,
        closure_hash: Option<Fingerprint>,
        aggregate_hash: Option<Fingerprint>,
        cancellation_hash: Option<Fingerprint>,
        voter_count: VoterCount,
        status: ResultStatus,
        reason: Option<ResultReason>,
        counts: Option<Vec<OptionCount>>,
        missing_authorities: Vec<AuthorityId>,
    ) -> Result<Self, InvalidElectionResult> {
        if counts.as_ref().is_some_and(|counts| {
            counts
                .iter()
                .any(|c| c.count().value() > voter_count.value())
        }) {
            return Err(InvalidElectionResult::CountOutOfRange);
        }
        if missing_authorities.len() > 52 {
            return Err(InvalidElectionResult::TooManyMissingAuthorities);
        }

        for (index, authority) in missing_authorities.iter().enumerate() {
            if missing_authorities[..index].contains(authority) {
                return Err(InvalidElectionResult::DuplicateAuthority);
            }
        }

        match status {
            ResultStatus::Valid => {
                if closure_hash.is_none()
                    || (modality == ElectionModality::Confidential
                        && (manifest_hash.is_none() || aggregate_hash.is_none()))
                {
                    return Err(InvalidElectionResult::MissingRequiredHash);
                }

                if cancellation_hash.is_some() {
                    return Err(InvalidElectionResult::UnexpectedHash);
                }

                if reason.is_some() {
                    return Err(InvalidElectionResult::InvalidReason);
                }

                let values = counts
                    .as_ref()
                    .ok_or(InvalidElectionResult::MissingCounts)?;

                Self::validate_counts(values)?;

                if !missing_authorities.is_empty() {
                    return Err(InvalidElectionResult::MissingAuthoritiesInValidResult);
                }
            }
            ResultStatus::Annulled => {
                if closure_hash.is_none()
                    || (modality == ElectionModality::Confidential
                        && (manifest_hash.is_none() || aggregate_hash.is_none()))
                {
                    return Err(InvalidElectionResult::MissingRequiredHash);
                }

                if cancellation_hash.is_some() {
                    return Err(InvalidElectionResult::UnexpectedHash);
                }

                match reason {
                    Some(ResultReason::AggregateSumMismatch) => {
                        let values = counts
                            .as_ref()
                            .ok_or(InvalidElectionResult::MissingCounts)?;

                        Self::validate_counts(values)?;
                    }
                    Some(ResultReason::TallyOutOfRange) => {
                        if counts.is_some() {
                            return Err(InvalidElectionResult::UnexpectedCounts);
                        }
                    }
                    _ => return Err(InvalidElectionResult::InvalidReason),
                }

                if !missing_authorities.is_empty() {
                    return Err(InvalidElectionResult::MissingAuthoritiesInValidResult);
                }
            }
            ResultStatus::Incomplete => {
                if closure_hash.is_none()
                    || (modality == ElectionModality::Confidential
                        && (manifest_hash.is_none() || aggregate_hash.is_none()))
                {
                    return Err(InvalidElectionResult::MissingRequiredHash);
                }

                if cancellation_hash.is_some() {
                    return Err(InvalidElectionResult::UnexpectedHash);
                }

                if !matches!(
                    reason,
                    Some(ResultReason::MissingAuthority | ResultReason::InvalidAuthorityShare)
                ) {
                    return Err(InvalidElectionResult::InvalidReason);
                }

                if counts.is_some() {
                    return Err(InvalidElectionResult::UnexpectedCounts);
                }

                if missing_authorities.is_empty() {
                    return Err(InvalidElectionResult::NoMissingAuthorities);
                }
            }
            ResultStatus::Cancelled => {
                if cancellation_hash.is_none() {
                    return Err(InvalidElectionResult::MissingRequiredHash);
                }

                if closure_hash.is_some() || aggregate_hash.is_some() {
                    return Err(InvalidElectionResult::UnexpectedHash);
                }

                if reason != Some(ResultReason::ManualCancellation) {
                    return Err(InvalidElectionResult::InvalidReason);
                }

                if counts.is_some() {
                    return Err(InvalidElectionResult::UnexpectedCounts);
                }

                if !missing_authorities.is_empty() {
                    return Err(InvalidElectionResult::MissingAuthoritiesInValidResult);
                }
            }
        }

        Ok(Self {
            election_id,
            modality,
            manifest_hash,
            closure_hash,
            aggregate_hash,
            cancellation_hash,
            voter_count,
            status,
            reason,
            counts,
            missing_authorities,
        })
    }

    fn validate_counts(counts: &[OptionCount]) -> Result<(), InvalidElectionResult> {
        if !(2..=51).contains(&counts.len()) {
            return Err(InvalidElectionResult::InvalidCountLength);
        }

        for (index, count) in counts.iter().enumerate() {
            if counts[..index]
                .iter()
                .any(|previous| previous.option_id() == count.option_id())
            {
                return Err(InvalidElectionResult::DuplicateOption);
            }
        }

        Ok(())
    }

    /// Retorna a modalidade à qual as evidências do resultado pertencem.
    pub const fn modality(&self) -> ElectionModality {
        self.modality
    }

    /// Retorna o identificador da eleição.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Retorna o hash do manifesto, quando disponível.
    pub const fn manifest_hash(&self) -> Option<&Fingerprint> {
        self.manifest_hash.as_ref()
    }

    /// Retorna o hash do encerramento, quando disponível.
    pub const fn closure_hash(&self) -> Option<&Fingerprint> {
        self.closure_hash.as_ref()
    }

    /// Retorna o hash do agregado, quando disponível.
    pub const fn aggregate_hash(&self) -> Option<&Fingerprint> {
        self.aggregate_hash.as_ref()
    }

    /// Retorna o hash do cancelamento, quando disponível.
    pub const fn cancellation_hash(&self) -> Option<&Fingerprint> {
        self.cancellation_hash.as_ref()
    }

    /// Retorna a quantidade de eleitores distintos.
    pub const fn voter_count(&self) -> VoterCount {
        self.voter_count
    }

    /// Retorna o estado publicado do resultado.
    pub const fn status(&self) -> ResultStatus {
        self.status
    }

    /// Retorna o motivo do resultado, quando existente.
    pub const fn reason(&self) -> Option<ResultReason> {
        self.reason
    }

    /// Retorna as contagens por opção, quando disponíveis.
    pub fn counts(&self) -> Option<&[OptionCount]> {
        self.counts.as_deref()
    }

    /// Retorna as autoridades sem contribuição válida.
    pub fn missing_authorities(&self) -> &[AuthorityId] {
        &self.missing_authorities
    }

    /// Retorna a versão do contrato.
    pub const fn version(&self) -> u8 {
        ELECTION_RESULT_VERSION
    }

    /// Retorna o tipo do registro no contrato.
    pub const fn kind(&self) -> &'static str {
        "result"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::domain::value_objects::identifiers::OptionId;
    use uuid::Uuid;

    fn election_id() -> ElectionId {
        ElectionId::try_from_uuid(Uuid::from_u128(1)).unwrap()
    }

    fn authority_id(value: u128) -> AuthorityId {
        AuthorityId::try_from_uuid(Uuid::from_u128(value)).unwrap()
    }

    fn fingerprint(value: char) -> Fingerprint {
        Fingerprint::try_from_hex(value.to_string().repeat(64)).unwrap()
    }

    fn voter_count() -> VoterCount {
        VoterCount::try_new(10).unwrap()
    }

    fn counts() -> Vec<OptionCount> {
        vec![
            OptionCount::try_new(OptionId::from_uuid(Uuid::from_u128(2)), 6).unwrap(),
            OptionCount::try_new(OptionId::from_uuid(Uuid::nil()), 4).unwrap(),
        ]
    }

    fn valid_result() -> ElectionResult {
        ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Valid,
            None,
            Some(counts()),
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn creates_valid_result() {
        let result = valid_result();

        assert_eq!(result.status(), ResultStatus::Valid);
        assert_eq!(result.kind(), "result");
        assert_eq!(result.version(), 1);
        assert_eq!(result.counts().unwrap().len(), 2);
    }

    #[test]
    fn creates_annulled_result_for_sum_mismatch() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Annulled,
            Some(ResultReason::AggregateSumMismatch),
            Some(counts()),
            vec![],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn creates_annulled_result_for_out_of_range_tally() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Annulled,
            Some(ResultReason::TallyOutOfRange),
            None,
            vec![],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn creates_incomplete_result_with_missing_authority() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Incomplete,
            Some(ResultReason::MissingAuthority),
            None,
            vec![authority_id(10)],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn creates_cancelled_result() {
        let result = ElectionResult::try_new(
            election_id(),
            None,
            None,
            None,
            Some(fingerprint('d')),
            voter_count(),
            ResultStatus::Cancelled,
            Some(ResultReason::ManualCancellation),
            None,
            vec![],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_valid_result_without_counts() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Valid,
            None,
            None,
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::MissingCounts));
    }

    #[test]
    fn rejects_incomplete_result_without_missing_authorities() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Incomplete,
            Some(ResultReason::MissingAuthority),
            None,
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::NoMissingAuthorities));
    }

    #[test]
    fn rejects_duplicate_option_ids() {
        let duplicate = OptionId::from_uuid(Uuid::from_u128(2));
        let duplicate_counts = vec![
            OptionCount::try_new(duplicate, 4).unwrap(),
            OptionCount::try_new(duplicate, 6).unwrap(),
        ];

        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Valid,
            None,
            Some(duplicate_counts),
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::DuplicateOption));
    }

    #[test]
    fn rejects_duplicate_missing_authorities() {
        let authority = authority_id(10);

        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Incomplete,
            Some(ResultReason::MissingAuthority),
            None,
            vec![authority, authority],
        );

        assert_eq!(result, Err(InvalidElectionResult::DuplicateAuthority));
    }

    #[test]
    fn rejects_cancelled_result_with_closure_hash() {
        let result = ElectionResult::try_new(
            election_id(),
            None,
            Some(fingerprint('b')),
            None,
            Some(fingerprint('d')),
            voter_count(),
            ResultStatus::Cancelled,
            Some(ResultReason::ManualCancellation),
            None,
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::UnexpectedHash));
    }

    #[test]
    fn accepts_cancelled_result_with_manifest_hash() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            None,
            None,
            Some(fingerprint('d')),
            voter_count(),
            ResultStatus::Cancelled,
            Some(ResultReason::ManualCancellation),
            None,
            vec![],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_valid_result_with_cancellation_hash() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            Some(fingerprint('d')),
            voter_count(),
            ResultStatus::Valid,
            None,
            Some(counts()),
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::UnexpectedHash));
    }

    #[test]
    fn rejects_annulled_result_with_missing_authority_reason() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Annulled,
            Some(ResultReason::MissingAuthority),
            None,
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::InvalidReason));
    }

    #[test]
    fn rejects_incomplete_result_with_counts() {
        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Incomplete,
            Some(ResultReason::MissingAuthority),
            Some(counts()),
            vec![authority_id(10)],
        );

        assert_eq!(result, Err(InvalidElectionResult::UnexpectedCounts));
    }

    #[test]
    fn rejects_more_than_51_option_counts() {
        let values = (1..=52)
            .map(|value| {
                OptionCount::try_new(OptionId::from_uuid(Uuid::from_u128(value)), 0).unwrap()
            })
            .collect();

        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Valid,
            None,
            Some(values),
            vec![],
        );

        assert_eq!(result, Err(InvalidElectionResult::InvalidCountLength));
    }

    #[test]
    fn rejects_more_than_52_missing_authorities() {
        let authorities = (1..=53).map(authority_id).collect();

        let result = ElectionResult::try_new(
            election_id(),
            Some(fingerprint('a')),
            Some(fingerprint('b')),
            Some(fingerprint('c')),
            None,
            voter_count(),
            ResultStatus::Incomplete,
            Some(ResultReason::MissingAuthority),
            None,
            authorities,
        );

        assert_eq!(
            result,
            Err(InvalidElectionResult::TooManyMissingAuthorities)
        );
    }
}
