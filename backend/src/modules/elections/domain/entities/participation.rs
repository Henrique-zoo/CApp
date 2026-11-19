//! Participação única por conta interna e eleição, sem inscrição eleitoral prévia.
//!
//! O armazenamento impõe UNIQUE(election_id, account_id) e confirma participação,
//! cédula e recibo juntos (#282/#381). Este modelo não mantém uma lista de todos
//! os eleitores no agregado nem promete resolver concorrência apenas em memória.

use super::{
    ballot::{InvalidBallot, VerifiedBallot},
    election::{Election, ElectionStatus},
};
use crate::modules::elections::domain::value_objects::identifiers::{
    AccountId, ElectionId, VoterId,
};
use chrono::{DateTime, Timelike, Utc};

/// Chave privada de unicidade; nunca é o identificador publicado no quadro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticipationKey {
    election_id: ElectionId,
    account_id: AccountId,
}

impl ParticipationKey {
    /// Combina conta interna e eleição; chave/dispositivo não integram a unicidade.
    pub const fn new(election_id: ElectionId, account_id: AccountId) -> Self {
        Self {
            election_id,
            account_id,
        }
    }
    /// Eleição à qual a participação pertence.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }
    /// Conta interna resolvida pela porta de identidade, sem exposição pública.
    pub const fn account_id(&self) -> AccountId {
        self.account_id
    }
}

/// Primeiro aceite e vínculo imutável da conta com cédula/chave pública.
///
/// Na identificada a cédula conserva a escolha. Na confidencial conserva apenas
/// cifras. Mudanças posteriores no vínculo institucional não desfazem este fato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participation {
    key: ParticipationKey,
    accepted: VerifiedBallot,
    accepted_at: DateTime<Utc>,
}

/// Recusa de novo aceite, repetição conflitante ou reconstituição inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ParticipationError {
    /// Somente eleição aberta admite novo aceite.
    #[error("A eleição não está aberta")]
    ElectionNotOpen,
    /// Aceite fora do intervalo vigente ou com precisão incompatível.
    #[error("O instante do aceite é inválido")]
    InvalidAcceptanceTime,
    /// A conta/identidade eleitoral não corresponde à operação autenticada.
    #[error("A identidade não corresponde à participação")]
    IdentityMismatch,
    /// Outra operação não pode criar nova participação.
    #[error("A conta já participou desta eleição")]
    AlreadyParticipated,
    /// O mesmo identificador de envio foi usado com outro payload.
    #[error("O identificador de envio já possui outro conteúdo")]
    SubmissionConflict,
    /// Contexto ou opções inválidos na cédula.
    #[error(transparent)]
    InvalidBallot(#[from] InvalidBallot),
}

impl Participation {
    /// Prepara o primeiro aceite com identidade/elegibilidade já verificadas.
    ///
    /// A aplicação deve reavaliar a elegibilidade vigente e chamar este caminho
    /// dentro da operação atômica. Só o commit autoriza devolver um recibo.
    pub fn try_accept(
        election: &Election,
        account_id: AccountId,
        voter_id: VoterId,
        ballot: VerifiedBallot,
        accepted_at: DateTime<Utc>,
    ) -> Result<Self, ParticipationError> {
        if election.status() != ElectionStatus::Open {
            return Err(ParticipationError::ElectionNotOpen);
        }
        Self::try_reconstitute(election, account_id, voter_id, ballot, accepted_at)
    }

    /// Reconstitui um aceite histórico sem exigir elegibilidade atual ou chave privada.
    ///
    /// A cédula pública é reverificada pelo adaptador antes deste método. Confere
    /// configuração e instante contra abertura, encerramento e cancelamento; a
    /// transação de persistência ordena operações concorrentes no mesmo segundo.
    pub fn try_reconstitute(
        election: &Election,
        account_id: AccountId,
        voter_id: VoterId,
        accepted: VerifiedBallot,
        accepted_at: DateTime<Utc>,
    ) -> Result<Self, ParticipationError> {
        accepted.ballot().validate_for(election)?;
        if accepted.ballot().voter_id() != voter_id {
            return Err(ParticipationError::IdentityMismatch);
        }
        if accepted_at.nanosecond() != 0
            || !election.current_period().contains(&accepted_at)
            || election
                .opened_at()
                .is_none_or(|opened| accepted_at < opened)
            || election
                .cancellation()
                .is_some_and(|c| accepted_at > *c.cancelled_at())
        {
            return Err(ParticipationError::InvalidAcceptanceTime);
        }
        Ok(Self {
            key: ParticipationKey::new(election.id(), account_id),
            accepted,
            accepted_at,
        })
    }

    /// Recupera o aceite idêntico, inclusive após encerramento ou troca de curso.
    ///
    /// Autenticar a conta antes desta consulta. Não chama a regra de novo aceite:
    /// não exige eleição aberta, nova elegibilidade ou posse da chave privada.
    pub fn retry(
        &self,
        key: ParticipationKey,
        ballot: &VerifiedBallot,
    ) -> Result<&Self, ParticipationError> {
        if key != self.key
            || ballot.ballot().election_id() != self.key.election_id
            || ballot.ballot().voter_id() != self.accepted.ballot().voter_id()
        {
            return Err(ParticipationError::IdentityMismatch);
        }
        if ballot.ballot().submission_id() != self.accepted.ballot().submission_id() {
            return Err(ParticipationError::AlreadyParticipated);
        }
        if ballot.ballot_hash() != self.accepted.ballot_hash()
            || ballot.key_fingerprint() != self.accepted.key_fingerprint()
        {
            return Err(ParticipationError::SubmissionConflict);
        }
        Ok(self)
    }
    /// Identidade privada usada na restrição transacional.
    pub const fn key(&self) -> ParticipationKey {
        self.key
    }
    /// Cédula aceita e referências públicas de chave/digest.
    pub const fn accepted_ballot(&self) -> &VerifiedBallot {
        &self.accepted
    }
    /// Instante confiável do aceite original.
    pub const fn accepted_at(&self) -> DateTime<Utc> {
        self.accepted_at
    }
}
