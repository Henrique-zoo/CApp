//! Registro público confidencial e referências estruturais do encadeamento.
//!
//! O codec calcula/confere o digest do payload `board-entry`, incluindo a
//! assinatura aninhada da cédula. Este modelo não calcula hashes, não verifica
//! assinaturas nem comprova persistência ou inclusão no quadro publicado.

use super::ballot::{BallotContent, VerifiedBallot};
use crate::modules::elections::domain::value_objects::{
    fingerprint::Fingerprint, identifiers::ElectionId, voter_count::MAX_VOTER_COUNT,
};

/// Cédula pública e posição no quadro, sem conta interna ou escolha em claro.
///
/// O digest é uma referência fornecida pelo codec, não um campo adicional do
/// payload JSON. O repositório confirma o registro junto da participação e recibo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardEntry {
    ballot: VerifiedBallot,
    manifest_hash: Fingerprint,
    index: u32,
    previous_hash: Fingerprint,
    entry_hash: Fingerprint,
}

/// Incompatibilidade estrutural no registro ou entre registros consecutivos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBoardEntry {
    /// A identificada não participa deste quadro criptográfico.
    #[error("O quadro confidencial exige uma cédula confidencial")]
    IdentifiedBallot,
    /// O perfil limita os índices ao intervalo inclusivo de 1 a 100000.
    #[error("Índice inválido no quadro eleitoral")]
    InvalidIndex,
    /// A primeira referência deve ser o digest do manifesto.
    #[error("Referência inicial inválida no quadro eleitoral")]
    InvalidOrigin,
    /// Contexto, índice ou referência ao registro anterior divergem.
    #[error("Os registros eleitorais não são consecutivos")]
    InvalidPredecessor,
}

impl BoardEntry {
    /// Valida modalidade, índice e origem, conservando a cédula assinada original.
    ///
    /// O codec confere `entry_hash` antes da reconstituição. Para índices maiores
    /// que um, o repositório também chama [`Self::validate_successor_of`] com o
    /// último registro confirmado, dentro da transação que reserva a posição.
    ///
    /// # Erros
    ///
    /// Rejeita cédula identificada, índice fora do perfil ou primeiro elo que não
    /// referencia o manifesto da cédula.
    pub fn try_new(
        ballot: VerifiedBallot,
        index: u32,
        previous_hash: Fingerprint,
        entry_hash: Fingerprint,
    ) -> Result<Self, InvalidBoardEntry> {
        let BallotContent::Confidential { manifest_hash, .. } = ballot.ballot().content() else {
            return Err(InvalidBoardEntry::IdentifiedBallot);
        };
        if !(1..=MAX_VOTER_COUNT).contains(&index) {
            return Err(InvalidBoardEntry::InvalidIndex);
        }
        if index == 1 && &previous_hash != manifest_hash {
            return Err(InvalidBoardEntry::InvalidOrigin);
        }
        Ok(Self {
            manifest_hash: manifest_hash.clone(),
            ballot,
            index,
            previous_hash,
            entry_hash,
        })
    }

    /// Confere o elo com o predecessor informado, sem auditar o quadro completo.
    ///
    /// A operação não consulta armazenamento nem autentica os digests recebidos.
    /// Retorna [`InvalidBoardEntry::InvalidPredecessor`] para contexto diferente,
    /// índice não consecutivo ou referência que não corresponde ao predecessor.
    pub fn validate_successor_of(&self, previous: &Self) -> Result<(), InvalidBoardEntry> {
        if self.election_id() != previous.election_id()
            || self.manifest_hash != previous.manifest_hash
            || self.index != previous.index + 1
            || self.previous_hash != previous.entry_hash
        {
            return Err(InvalidBoardEntry::InvalidPredecessor);
        }
        Ok(())
    }

    /// Eleição do payload da cédula.
    pub const fn election_id(&self) -> ElectionId {
        self.ballot.ballot().election_id()
    }
    /// Manifesto imutável referenciado pela cédula.
    pub const fn manifest_hash(&self) -> &Fingerprint {
        &self.manifest_hash
    }
    /// Cédula original, incluindo a assinatura aninhada no payload do registro.
    pub const fn ballot(&self) -> &VerifiedBallot {
        &self.ballot
    }
    /// Posição confirmada, começando em um.
    pub const fn index(&self) -> u32 {
        self.index
    }
    /// Digest do manifesto no primeiro elo ou do registro anterior nos demais.
    pub const fn previous_hash(&self) -> &Fingerprint {
        &self.previous_hash
    }
    /// Digest do payload deste registro, calculado/conferido externamente.
    pub const fn entry_hash(&self) -> &Fingerprint {
        &self.entry_hash
    }
}
