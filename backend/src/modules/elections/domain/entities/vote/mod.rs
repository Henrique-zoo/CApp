//! Voto aceito, único por conta interna e eleição.
//!
//! O voto vincula a conta à cédula verificada no momento do aceite. A linha de
//! participação guarda a chave pública de assinatura dentro da cédula aceita
//! para aquela eleição. O armazenamento impõe UNIQUE(election_id, account_id)
//! e confirma o voto e suas evidências na mesma transação (#282/#381).
//! Na modalidade confidencial, [`board_entry::BoardEntry`] é a projeção do
//! voto no quadro público, e [`Receipt`] confirma o aceite ao eleitor.

use super::{
    ballot::{BallotContent, VerifiedBallot},
    election::{Election, ElectionModality, ElectionState},
};
use crate::modules::elections::domain::{
    Fingerprint,
    error::{InvalidBallot, InvalidReceiptIndex, VoteError},
    identifiers::{AccountId, ElectionId},
    profile::MAX_VOTER_COUNT,
};
use chrono::{DateTime, Utc};

pub mod board_entry;

/// Chave interna de unicidade do voto no armazenamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoteKey {
    election_id: ElectionId,
    account_id: AccountId,
}

impl VoteKey {
    /// Combina conta interna e eleição; chave/dispositivo não integram a unicidade.
    pub const fn new(election_id: ElectionId, account_id: AccountId) -> Self {
        Self {
            election_id,
            account_id,
        }
    }
    /// Eleição à qual o voto pertence.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }
    /// UUID da conta CApp resolvida a partir da identidade autenticada.
    pub const fn account_id(&self) -> AccountId {
        self.account_id
    }
}

/// Voto que vincula a conta à cédula verificada e ao instante do aceite.
///
/// Na identificada a cédula conserva a escolha. Na confidencial conserva apenas
/// cifras. Mudanças posteriores no vínculo institucional não desfazem este fato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    accepted: VerifiedBallot,
    accepted_at: DateTime<Utc>,
}

impl Vote {
    /// Prepara o primeiro aceite com identidade/elegibilidade já verificadas.
    ///
    /// A aplicação obtém a conta da identidade autenticada e a combina com o
    /// ID da eleição em `key`. Dentro da operação atômica, reavalia a
    /// elegibilidade e chama este método. Só o commit autoriza devolver um recibo.
    /// O instante vem do relógio confiável e deve ser obtido na operação que lê
    /// o calendário vigente. Para aceites históricos use a reconstituição.
    pub fn try_accept(
        election: &Election,
        key: VoteKey,
        ballot: VerifiedBallot,
        accepted_at: DateTime<Utc>,
    ) -> Result<Self, VoteError> {
        if !matches!(election.state(), ElectionState::Open(_)) {
            return Err(VoteError::ElectionNotOpen);
        }
        Self::try_reconstitute(election, key, ballot, accepted_at)
    }

    /// Reconstitui um aceite histórico sem exigir elegibilidade atual ou chave privada.
    ///
    /// O repositório fornece a chave da linha confirmada e a cédula reverificada.
    /// Se a eleição foi cancelada, confere a cédula com a configuração arquivada
    /// antes de chamar este método. A entidade confere identidade e horários
    /// disponíveis no estado vigente com a precisão do relógio do backend. A
    /// persistência ordena votos e cancelamento concorrentes.
    pub fn try_reconstitute(
        election: &Election,
        key: VoteKey,
        accepted: VerifiedBallot,
        accepted_at: DateTime<Utc>,
    ) -> Result<Self, VoteError> {
        if key.election_id() != election.id()
            || accepted.ballot().election_id() != key.election_id()
            || accepted.ballot().account_id() != key.account_id()
        {
            return Err(VoteError::IdentityMismatch);
        }
        if !election.current_period().contains(&accepted_at) {
            return Err(VoteError::InvalidAcceptanceTime);
        }

        match election.state() {
            ElectionState::Open(voting)
            | ElectionState::Closed { voting, .. }
            | ElectionState::CountingVotes { voting, .. }
            | ElectionState::Finalized { voting, .. } => {
                if accepted_at < voting.opened_at {
                    return Err(VoteError::InvalidAcceptanceTime);
                }
                accepted.ballot().validate_for(election)?;
            }
            ElectionState::Cancelled { cancelled_at, .. } => {
                if accepted_at > *cancelled_at {
                    return Err(VoteError::InvalidAcceptanceTime);
                }
                if !matches!(
                    (accepted.ballot().content(), election.modality()),
                    (BallotContent::Identified(_), ElectionModality::Identified)
                        | (
                            BallotContent::Confidential { .. },
                            ElectionModality::Confidential
                        )
                ) {
                    return Err(InvalidBallot::ConfigurationMismatch.into());
                }
            }
            ElectionState::Preparing | ElectionState::WaitingStart(_) => {
                return Err(VoteError::ElectionNotOpen);
            }
        }

        Ok(Self {
            accepted,
            accepted_at,
        })
    }

    /// Recupera o aceite idêntico, inclusive após encerramento ou troca de curso.
    ///
    /// Autenticar a conta antes desta consulta. Não chama a regra de novo aceite:
    /// não exige eleição aberta, nova elegibilidade ou posse da chave privada.
    pub fn retry(&self, key: VoteKey, ballot: &VerifiedBallot) -> Result<&Self, VoteError> {
        let accepted_key = self.key();
        if key != accepted_key
            || ballot.ballot().election_id() != accepted_key.election_id()
            || ballot.ballot().account_id() != accepted_key.account_id()
        {
            return Err(VoteError::IdentityMismatch);
        }
        if ballot.ballot().submission_id() != self.accepted.ballot().submission_id() {
            return Err(VoteError::AlreadyParticipated);
        }
        if ballot.ballot_hash() != self.accepted.ballot_hash() {
            return Err(VoteError::SubmissionConflict);
        }
        Ok(self)
    }
    /// Par de eleição e conta já conferido no aceite ou na reconstituição.
    pub const fn key(&self) -> VoteKey {
        let ballot = self.accepted.ballot();
        VoteKey::new(ballot.election_id(), ballot.account_id())
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

/// Recibo confidencial emitido somente após commit, conforme contrato da porta.
///
/// O recibo identifica o registro da cédula cifrada no quadro público. Quem o
/// recebe verifica a assinatura do servidor e consulta o quadro para conferir
/// a inclusão da cédula naquela posição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    election_id: ElectionId,
    manifest_hash: Fingerprint,
    ballot_hash: Fingerprint,
    index: u32,
    entry_hash: Fingerprint,
    signature: Vec<u8>,
}

impl Receipt {
    /// Valida o índice sem afirmar confirmação transacional ou inclusão pública.
    pub fn try_new(
        election_id: ElectionId,
        manifest_hash: Fingerprint,
        ballot_hash: Fingerprint,
        index: u32,
        entry_hash: Fingerprint,
        signature: Vec<u8>,
    ) -> Result<Self, InvalidReceiptIndex> {
        if !(1..=MAX_VOTER_COUNT).contains(&index) {
            return Err(InvalidReceiptIndex);
        }

        Ok(Self {
            election_id,
            manifest_hash,
            ballot_hash,
            index,
            entry_hash,
            signature,
        })
    }

    /// Eleição do registro.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Digest do manifesto congelado.
    pub const fn manifest_hash(&self) -> &Fingerprint {
        &self.manifest_hash
    }

    /// Digest da cédula aceita.
    pub const fn ballot_hash(&self) -> &Fingerprint {
        &self.ballot_hash
    }

    /// Índice monotônico do registro confirmado.
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Hash do registro, também usado como checkpoint da cadeia.
    pub const fn entry_hash(&self) -> &Fingerprint {
        &self.entry_hash
    }

    /// Assinatura que autentica o recibo emitido pelo servidor.
    pub const fn signature(&self) -> &[u8] {
        self.signature.as_slice()
    }
}

#[cfg(test)]
mod tests;
