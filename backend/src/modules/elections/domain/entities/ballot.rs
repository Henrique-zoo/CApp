//! Cédulas assinadas e evidências públicas, sem segredos ou escolha secreta em claro.
//!
//! A cédula conserva os dados públicos usados no aceite e na auditoria. O codec
//! lê o envelope original e converte os componentes decimais em inteiros. O
//! ElGamal valida esses componentes uma vez, e a mesma cédula cifrada pode ser
//! usada na agregação depois do aceite.

use std::collections::HashSet;

use uuid::Uuid;

use super::{configuration::ElectionConfiguration, election::Election};
use crate::modules::elections::domain::{
    Fingerprint,
    cryptography::elgamal::{ElGamalCiphertext, EncryptedBallot},
    error::InvalidBallot,
    identifiers::{AccountId, ElectionId, SubmissionId},
    profile::{MAX_OPTIONS, MIN_OPTIONS},
};

/// Associa os componentes públicos de uma cifra ElGamal à respectiva opção.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext {
    option_id: Uuid,
    components: ElGamalCiphertext,
}

impl Ciphertext {
    /// Associa os componentes à opção. A verificação do grupo depende de a
    /// cifra representar uma cédula individual ou o agregado da votação.
    pub const fn new(option_id: Uuid, components: ElGamalCiphertext) -> Self {
        Self {
            option_id,
            components,
        }
    }

    /// Opção correspondente à posição da cifra.
    pub const fn option_id(&self) -> Uuid {
        self.option_id
    }

    /// Componentes numéricos da cifra usados na verificação e na apuração.
    pub const fn components(&self) -> &ElGamalCiphertext {
        &self.components
    }
}

/// Opções e cifras validadas de uma cédula confidencial, na ordem recebida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BallotCiphertexts {
    option_ids: Vec<Uuid>,
    encrypted: EncryptedBallot,
}

impl BallotCiphertexts {
    /// Confere a quantidade de opções e a unicidade dos IDs, e valida os
    /// componentes ElGamal de uma cédula individual. Os IDs e as cifras
    /// permanecem alinhados pela posição. A comparação com o manifesto ocorre
    /// em [`SignedBallot::validate_for`].
    ///
    /// Retorna [`InvalidBallot::InvalidOptions`] se a dimensão ou os IDs forem
    /// inválidos, ou [`InvalidBallot::InvalidCiphertext`] se algum componente
    /// violar a faixa, a pertinência ao subgrupo ou a regra de identidade.
    pub fn try_new(ciphertexts: Vec<Ciphertext>) -> Result<Self, InvalidBallot> {
        validate_ciphertexts(&ciphertexts)?;

        let (option_ids, components) = ciphertexts
            .into_iter()
            .map(|cipher| (cipher.option_id, cipher.components))
            .unzip();
        let encrypted =
            EncryptedBallot::new(components).map_err(|_| InvalidBallot::InvalidCiphertext)?;

        Ok(Self {
            option_ids,
            encrypted,
        })
    }

    /// IDs das opções na mesma ordem das cifras.
    pub fn option_ids(&self) -> &[Uuid] {
        &self.option_ids
    }

    /// Cédula com componentes ElGamal prontos para agregação após o aceite.
    pub const fn encrypted_ballot(&self) -> &EncryptedBallot {
        &self.encrypted
    }
}

/// Conteúdo permitido para cada modalidade, sem variante de voto branco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BallotContent {
    /// Escolha associável ao eleitor; acesso é controlado pela aplicação.
    Identified(Uuid),
    /// Vetor cifrado completo vinculado ao manifesto imutável.
    Confidential {
        /// Digest do manifesto conferido pelo codec.
        manifest_hash: Fingerprint,
        /// Uma cifra por opção, incluindo Nulo.
        ciphertexts: BallotCiphertexts,
    },
}

/// Cédula com o UUID da conta CApp, a operação e a chave pública de assinatura.
///
/// Na modalidade confidencial, o quadro público identifica a participação
/// pela conta e publica a escolha somente cifrada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBallot {
    election_id: ElectionId,
    account_id: AccountId,
    submission_id: SubmissionId,
    public_key: Vec<u8>,
    signature: Vec<u8>,
    content: BallotContent,
}

impl SignedBallot {
    /// Reúne a identificação, os bytes da chave pública e da assinatura e o
    /// conteúdo da cédula. Na modalidade confidencial, recebe cifras já
    /// validadas por [`BallotCiphertexts::try_new`].
    ///
    /// Não verifica a assinatura. Para o aceite é necessário obter também
    /// [`VerifiedBallot`] da porta criptográfica e conferir a conta autenticada.
    pub fn new(
        election_id: ElectionId,
        account_id: AccountId,
        submission_id: SubmissionId,
        public_key: Vec<u8>,
        signature: Vec<u8>,
        content: BallotContent,
    ) -> Self {
        Self {
            election_id,
            account_id,
            submission_id,
            public_key,
            signature,
            content,
        }
    }
    /// Identificador da eleição assinada.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }
    /// UUID da conta CApp gravado no campo `voterId` do contrato assinado.
    pub const fn account_id(&self) -> AccountId {
        self.account_id
    }
    /// Identificador idempotente do pacote preparado no dispositivo.
    pub const fn submission_id(&self) -> SubmissionId {
        self.submission_id
    }
    /// Chave pública do par de assinatura gerado para esta eleição.
    pub const fn public_key(&self) -> &[u8] {
        self.public_key.as_slice()
    }
    /// Assinatura do payload completo, a verificar pelo codec.
    pub const fn signature(&self) -> &[u8] {
        self.signature.as_slice()
    }
    /// Conteúdo assinado, sem acesso mutável.
    pub const fn content(&self) -> &BallotContent {
        &self.content
    }

    /// Confere eleição, modalidade, manifesto e opções contra o agregado.
    pub fn validate_for(&self, election: &Election) -> Result<(), InvalidBallot> {
        if self.election_id != election.id() {
            return Err(InvalidBallot::ConfigurationMismatch);
        }
        let configuration = election
            .configuration()
            .ok_or(InvalidBallot::ConfigurationMismatch)?;
        match (&self.content, configuration) {
            (BallotContent::Identified(choice), ElectionConfiguration::Identified(options)) => {
                if !options.as_slice().iter().any(|o| o.id() == *choice) {
                    return Err(InvalidBallot::InvalidOptions);
                }
            }
            (
                BallotContent::Confidential {
                    manifest_hash,
                    ciphertexts,
                },
                ElectionConfiguration::Confidential(verified),
            ) => {
                if verified.hash() != manifest_hash {
                    return Err(InvalidBallot::ConfigurationMismatch);
                }
                if !verified
                    .manifest()
                    .options()
                    .iter()
                    .map(|o| o.id())
                    .eq(ciphertexts.option_ids().iter().copied())
                {
                    return Err(InvalidBallot::InvalidOptions);
                }
            }
            _ => return Err(InvalidBallot::ConfigurationMismatch),
        }
        Ok(())
    }
}

/// Confere dimensão e IDs sem interpretar os inteiros como bits.
fn validate_ciphertexts(ciphertexts: &[Ciphertext]) -> Result<(), InvalidBallot> {
    if !(MIN_OPTIONS..=MAX_OPTIONS).contains(&ciphertexts.len()) {
        return Err(InvalidBallot::InvalidOptions);
    }
    let ids: HashSet<_> = ciphertexts.iter().map(Ciphertext::option_id).collect();
    if ids.len() != ciphertexts.len() {
        return Err(InvalidBallot::InvalidOptions);
    }
    Ok(())
}

/// Cédula cuja assinatura, codificação e cifras foram verificadas pelas portas.
///
/// O digest calculado pelo codec cobre o payload inteiro, inclusive a chave
/// pública e o identificador de envio. A verificação autentica esse conteúdo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedBallot {
    ballot: SignedBallot,
    ballot_hash: Fingerprint,
}

impl VerifiedBallot {
    /// Constrói a evidência somente após validação nos adaptadores do crate.
    #[allow(
        dead_code,
        reason = "Adaptadores criptográficos pertencem às issues #378/#380."
    )]
    pub(crate) fn from_verified_parts(ballot: SignedBallot, ballot_hash: Fingerprint) -> Self {
        Self {
            ballot,
            ballot_hash,
        }
    }
    /// Material público conferido.
    pub const fn ballot(&self) -> &SignedBallot {
        &self.ballot
    }
    /// Digest do payload aceito.
    pub const fn ballot_hash(&self) -> &Fingerprint {
        &self.ballot_hash
    }
}

#[cfg(test)]
mod tests {
    //! Validação da cédula individual e sua compatibilidade com a configuração.

    use super::*;
    use crate::modules::elections::{
        domain::entities::election::ElectionModality,
        test_support::{account, ballot, ciphertext, decoded, eid, fp, opened, option},
    };
    use crypto_bigint::U2048;

    #[test]
    fn ballot_cannot_cross_modalities_or_choose_an_unknown_slate() {
        let identified = opened(ElectionModality::Identified);
        assert!(
            ballot(ElectionModality::Confidential, 60, 'e')
                .ballot()
                .validate_for(&identified)
                .is_err()
        );
        let confidential = opened(ElectionModality::Confidential);
        assert!(
            ballot(ElectionModality::Identified, 60, 'e')
                .ballot()
                .validate_for(&confidential)
                .is_err()
        );
        let unknown = SignedBallot::new(
            eid(),
            account(50),
            SubmissionId::try_from_uuid(Uuid::from_u128(60)).unwrap(),
            decoded("YQ"),
            decoded("Yg"),
            BallotContent::Identified(option(999)),
        );
        assert!(unknown.validate_for(&identified).is_err());
    }

    #[test]
    fn ciphertext_dimension_order_ids_manifest_and_degenerate_components_are_checked() {
        let election = opened(ElectionModality::Confidential);
        let make = |ciphertexts, manifest_hash| {
            let ciphertexts = BallotCiphertexts::try_new(ciphertexts)?;
            Ok(SignedBallot::new(
                eid(),
                account(50),
                SubmissionId::try_from_uuid(Uuid::from_u128(60)).unwrap(),
                decoded("YQ"),
                decoded("Yg"),
                BallotContent::Confidential {
                    manifest_hash,
                    ciphertexts,
                },
            ))
        };
        assert_eq!(
            make(vec![ciphertext(option(30), 2, 4)], fp('b')),
            Err(InvalidBallot::InvalidOptions)
        );
        assert_eq!(
            make(vec![ciphertext(option(30), 2, 4); 2], fp('b')),
            Err(InvalidBallot::InvalidOptions)
        );
        let two = U2048::from_u8(2);
        let four = U2048::from_u8(4);
        for (a, b) in [
            (U2048::ZERO, four),
            (U2048::ONE, four),
            (two, U2048::ZERO),
            (U2048::MAX, four),
            (two, U2048::MAX),
        ] {
            assert_eq!(
                make(
                    vec![
                        Ciphertext::new(option(30), ElGamalCiphertext { a, b }),
                        ciphertext(option(0), 2, 4),
                    ],
                    fp('b')
                ),
                Err(InvalidBallot::InvalidCiphertext)
            );
        }
        let reversed = vec![ciphertext(option(0), 2, 4), ciphertext(option(30), 2, 4)];
        assert_eq!(
            make(reversed, fp('b')).unwrap().validate_for(&election),
            Err(InvalidBallot::InvalidOptions)
        );
        let correct = vec![ciphertext(option(30), 2, 4), ciphertext(option(0), 2, 4)];
        assert_eq!(
            make(correct, fp('f')).unwrap().validate_for(&election),
            Err(InvalidBallot::ConfigurationMismatch)
        );
    }

    #[test]
    fn null_is_a_regular_valid_choice_in_both_modalities() {
        let identified = opened(ElectionModality::Identified);
        let ballot = SignedBallot::new(
            eid(),
            account(50),
            SubmissionId::try_from_uuid(Uuid::from_u128(60)).unwrap(),
            decoded("YQ"),
            decoded("Yg"),
            BallotContent::Identified(option(0)),
        );
        assert!(ballot.validate_for(&identified).is_ok());
        assert_eq!(
            opened(ElectionModality::Confidential)
                .configuration()
                .unwrap()
                .options()
                .last()
                .unwrap()
                .id(),
            option(0)
        );
    }
}
