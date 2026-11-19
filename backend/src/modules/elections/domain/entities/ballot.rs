//! Cédulas assinadas e evidências públicas, sem segredos ou escolha secreta em claro.
//!
//! Estes valores representam dados necessários ao aceite, não DTOs HTTP. O codec
//! verifica o envelope original antes do mapeamento; a criptografia verifica
//! subgrupo e assinatura. Nenhum desses mecanismos comprova conteúdo binário.

use std::collections::HashSet;

use super::election::Election;
use crate::modules::elections::domain::value_objects::{
    base64_url::Base64Url,
    big_uint::BigUInt,
    fingerprint::Fingerprint,
    identifiers::{ElectionId, OptionId, SubmissionId, VoterId},
    modality::ElectionModality,
    options::{MAX_OPTIONS, MIN_OPTIONS},
};

/// Cifra pública de uma opção; a identidade é permitida em agregados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext {
    option_id: OptionId,
    a: BigUInt,
    b: BigUInt,
}

impl Ciphertext {
    /// Armazena componentes com representação decimal validada.
    /// A pertinência ao grupo pertence à porta criptográfica.
    pub fn new(option_id: OptionId, a: BigUInt, b: BigUInt) -> Self {
        Self { option_id, a, b }
    }
    /// Opção correspondente à posição da cifra.
    pub const fn option_id(&self) -> OptionId {
        self.option_id
    }
    /// Primeiro componente público.
    pub const fn a(&self) -> &BigUInt {
        &self.a
    }
    /// Segundo componente público.
    pub const fn b(&self) -> &BigUInt {
        &self.b
    }
}

/// Conteúdo permitido para cada modalidade, sem variante de voto branco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BallotContent {
    /// Escolha associável ao eleitor; acesso é controlado pela aplicação.
    Identified(OptionId),
    /// Vetor cifrado completo vinculado ao manifesto imutável.
    Confidential {
        /// Digest do manifesto conferido pelo codec.
        manifest_hash: Fingerprint,
        /// Uma cifra por opção, incluindo Nulo.
        ciphertexts: Vec<Ciphertext>,
    },
}

/// Cédula com identidade eleitoral opaca, operação e chave pública de assinatura.
///
/// Não contém a conta interna. Na modalidade confidencial não há campo para
/// escolha em claro, chave privada ou aleatoriedade de cifragem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBallot {
    election_id: ElectionId,
    voter_id: VoterId,
    submission_id: SubmissionId,
    public_key: Base64Url,
    signature: Base64Url,
    content: BallotContent,
}

/// Falha estrutural ou contextual da cédula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBallot {
    /// Configuração, eleição, modalidade ou manifesto divergente.
    #[error("A cédula não corresponde à configuração eleitoral")]
    ConfigurationMismatch,
    /// Vetor incompleto, repetido ou fora da ordem configurada.
    #[error("As opções da cédula divergem da configuração")]
    InvalidOptions,
    /// Componente zero ou primeiro componente individual igual a um.
    #[error("A cédula contém uma cifra degenerada")]
    DegenerateCiphertext,
}

impl SignedBallot {
    /// Valida a estrutura, preservando os dados assinados sem normalizá-los.
    ///
    /// Não verifica a assinatura. Para o aceite é necessário obter também
    /// [`VerifiedBallot`] da porta criptográfica e conferir a conta autenticada.
    pub fn try_new(
        election_id: ElectionId,
        voter_id: VoterId,
        submission_id: SubmissionId,
        public_key: Base64Url,
        signature: Base64Url,
        content: BallotContent,
    ) -> Result<Self, InvalidBallot> {
        if let BallotContent::Confidential { ciphertexts, .. } = &content {
            validate_ciphertexts(ciphertexts)?;
            if ciphertexts
                .iter()
                .any(|c| matches!(c.a.as_str(), "0" | "1") || c.b.as_str() == "0")
            {
                return Err(InvalidBallot::DegenerateCiphertext);
            }
        }
        Ok(Self {
            election_id,
            voter_id,
            submission_id,
            public_key,
            signature,
            content,
        })
    }
    /// Identificador da eleição assinada.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }
    /// Identidade pública opaca nesta eleição.
    pub const fn voter_id(&self) -> VoterId {
        self.voter_id
    }
    /// Identificador idempotente do pacote preparado no dispositivo.
    pub const fn submission_id(&self) -> SubmissionId {
        self.submission_id
    }
    /// Chave pública enviada na primeira cédula aceita.
    pub const fn public_key(&self) -> &Base64Url {
        &self.public_key
    }
    /// Assinatura do payload completo, a verificar pelo codec.
    pub const fn signature(&self) -> &Base64Url {
        &self.signature
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
        let options = election
            .options()
            .ok_or(InvalidBallot::ConfigurationMismatch)?;
        match (&self.content, election.modality()) {
            (BallotContent::Identified(choice), ElectionModality::Identified) => {
                if !options.iter().any(|o| o.id() == *choice) {
                    return Err(InvalidBallot::InvalidOptions);
                }
            }
            (
                BallotContent::Confidential {
                    manifest_hash,
                    ciphertexts,
                },
                ElectionModality::Confidential,
            ) => {
                if election.manifest_hash() != Some(manifest_hash) {
                    return Err(InvalidBallot::ConfigurationMismatch);
                }
                if !options
                    .iter()
                    .map(|o| o.id())
                    .eq(ciphertexts.iter().map(Ciphertext::option_id))
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
pub(super) fn validate_ciphertexts(ciphertexts: &[Ciphertext]) -> Result<(), InvalidBallot> {
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
/// O digest cobre todo o payload, inclusive chave e operação. O fingerprint é o
/// SHA-256 do DER público, calculado pelo codec. Não se afirma validade 0/1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedBallot {
    ballot: SignedBallot,
    ballot_hash: Fingerprint,
    key_fingerprint: Fingerprint,
}

impl VerifiedBallot {
    /// Constrói a evidência somente após validação nos adaptadores do crate.
    #[allow(
        dead_code,
        reason = "Adaptadores criptográficos pertencem às issues #378/#380."
    )]
    pub(crate) fn from_verified_parts(
        ballot: SignedBallot,
        ballot_hash: Fingerprint,
        key_fingerprint: Fingerprint,
    ) -> Self {
        Self {
            ballot,
            ballot_hash,
            key_fingerprint,
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
    /// Referência à chave vinculada no primeiro aceite.
    pub const fn key_fingerprint(&self) -> &Fingerprint {
        &self.key_fingerprint
    }
}
