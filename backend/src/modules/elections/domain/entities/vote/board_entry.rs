//! Projeção pública de um voto confidencial e sua posição no quadro.
//!
//! O repositório pode montar a projeção a partir da cédula já lida na linha
//! do voto. O codec calcula o digest do payload quando for encadear o quadro
//! ou emitir o recibo.

use crate::modules::elections::domain::{
    Fingerprint,
    entities::ballot::{BallotContent, VerifiedBallot},
    error::InvalidBoardEntry,
    identifiers::ElectionId,
    profile::MAX_VOTER_COUNT,
};

/// Cédula pública e posição no quadro, com a conta CApp e a escolha cifrada.
///
/// A cédula é emprestada do voto para preservar o original ao publicar o quadro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardEntry<'a> {
    ballot: &'a VerifiedBallot,
    index: u32,
    previous_hash: Fingerprint,
}

impl<'a> BoardEntry<'a> {
    /// Valida modalidade, índice e origem, conservando a cédula assinada original.
    ///
    /// O chamador fornece a cédula já aceita em `Vote`, o índice e a referência
    /// ao registro anterior. Para índices maiores que um, o repositório também
    /// confere o elo com [`Self::validate_successor_of`].
    ///
    /// # Erros
    ///
    /// Rejeita cédula identificada, índice fora do perfil ou primeiro elo que não
    /// referencia o manifesto da cédula.
    pub fn try_new(
        ballot: &'a VerifiedBallot,
        index: u32,
        previous_hash: Fingerprint,
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
            ballot,
            index,
            previous_hash,
        })
    }

    /// Confere o elo com o predecessor informado, sem auditar o quadro completo.
    ///
    /// `previous_entry_hash` é o digest do payload de `previous`, calculado pelo
    /// codec a partir do registro anterior.
    /// Retorna [`InvalidBoardEntry::InvalidPredecessor`] para contexto diferente,
    /// índice não consecutivo ou referência que não corresponde ao predecessor.
    pub fn validate_successor_of(
        &self,
        previous: &BoardEntry<'_>,
        previous_entry_hash: &Fingerprint,
    ) -> Result<(), InvalidBoardEntry> {
        if self.election_id() != previous.election_id()
            || self.manifest_hash() != previous.manifest_hash()
            || self.index != previous.index + 1
            || &self.previous_hash != previous_entry_hash
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
        match self.ballot.ballot().content() {
            BallotContent::Confidential { manifest_hash, .. } => manifest_hash,
            BallotContent::Identified(_) => unreachable!(),
        }
    }
    /// Cédula original, incluindo a assinatura aninhada no payload do registro.
    pub const fn ballot(&self) -> &VerifiedBallot {
        self.ballot
    }
    /// Posição confirmada, começando em um.
    pub const fn index(&self) -> u32 {
        self.index
    }
    /// Digest do manifesto no primeiro elo ou do registro anterior nos demais.
    pub const fn previous_hash(&self) -> &Fingerprint {
        &self.previous_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::elections::{
        domain::entities::{
            ballot::SignedBallot,
            election::ElectionModality,
            vote::{Vote, VoteKey},
        },
        test_support::{account, at, ballot, decoded, eid, fp, opened},
    };
    use uuid::Uuid;

    #[test]
    fn first_entry_is_rooted_in_the_ballot_manifest_and_preserves_the_signed_ballot() {
        let election = opened(ElectionModality::Confidential);
        let vote = Vote::try_accept(
            &election,
            VoteKey::new(eid(), account(50)),
            ballot(ElectionModality::Confidential, 'e'),
            at(20, 11),
        )
        .unwrap();
        let entry = BoardEntry::try_new(vote.accepted_ballot(), 1, fp('b')).unwrap();
        assert_eq!(entry.election_id(), eid());
        assert_eq!(entry.manifest_hash(), &fp('b'));
        assert!(std::ptr::eq(entry.ballot(), vote.accepted_ballot()));
        assert_eq!(entry.index(), 1);
        assert_eq!(entry.previous_hash(), &fp('b'));
        assert_eq!(
            BoardEntry::try_new(vote.accepted_ballot(), 1, fp('a')),
            Err(InvalidBoardEntry::InvalidOrigin)
        );
    }

    #[test]
    fn board_indices_obey_the_same_profile_limit_as_receipts() {
        let accepted = ballot(ElectionModality::Confidential, 'e');
        for index in [0, 100001, u32::MAX] {
            assert_eq!(
                BoardEntry::try_new(&accepted, index, fp('b')),
                Err(InvalidBoardEntry::InvalidIndex)
            );
        }
        assert!(BoardEntry::try_new(&accepted, 100000, fp('c')).is_ok());
        assert_eq!(
            BoardEntry::try_new(&ballot(ElectionModality::Identified, 'e'), 1, fp('b')),
            Err(InvalidBoardEntry::IdentifiedBallot)
        );
    }

    #[test]
    fn consecutive_entries_require_the_same_context_and_previous_digest() {
        let accepted = ballot(ElectionModality::Confidential, 'e');
        let first = BoardEntry::try_new(&accepted, 1, fp('b')).unwrap();
        let first_hash = fp('c');
        let second = BoardEntry::try_new(&accepted, 2, first_hash).unwrap();
        assert_eq!(second.validate_successor_of(&first, &first_hash), Ok(()));
        for entry in [
            first.clone(),
            BoardEntry::try_new(&accepted, 3, first_hash).unwrap(),
            BoardEntry::try_new(&accepted, 2, fp('a')).unwrap(),
        ] {
            assert_eq!(
                entry.validate_successor_of(&first, &first_hash),
                Err(InvalidBoardEntry::InvalidPredecessor)
            );
        }
        for (election_id, manifest_hash) in [
            (
                ElectionId::try_from_uuid(Uuid::from_u128(2)).unwrap(),
                fp('b'),
            ),
            (eid(), fp('a')),
        ] {
            let other = SignedBallot::new(
                election_id,
                account(50),
                decoded("YQ"),
                decoded("Yg"),
                BallotContent::Confidential {
                    manifest_hash,
                    ciphertexts: match accepted.ballot().content() {
                        BallotContent::Confidential { ciphertexts, .. } => ciphertexts.clone(),
                        _ => unreachable!(),
                    },
                },
            );
            let other = VerifiedBallot::from_verified_parts(other, fp('e'));
            let entry = BoardEntry::try_new(&other, 2, first_hash).unwrap();
            assert_eq!(
                entry.validate_successor_of(&first, &first_hash),
                Err(InvalidBoardEntry::InvalidPredecessor)
            );
        }
        let last = BoardEntry::try_new(&accepted, 100000, fp('c')).unwrap();
        assert_eq!(
            second.validate_successor_of(&last, &fp('d')),
            Err(InvalidBoardEntry::InvalidPredecessor)
        );
    }
}
