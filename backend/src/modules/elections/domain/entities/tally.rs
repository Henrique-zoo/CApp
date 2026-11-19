//! Evidência de apuração recebida das portas criptográficas.
//!
//! A verificação de provas e assinaturas pertence às issues #378–#380. A lista
//! indica contribuições válidas para o mesmo agregado final; o agregado eleitoral
//! confere sua correspondência exata com as autoridades do manifesto.

use super::{
    ballot::{Ciphertext, InvalidBallot, validate_ciphertexts},
    result::{ElectionResult, ResultStatus},
};
use crate::modules::elections::domain::value_objects::{
    fingerprint::Fingerprint,
    identifiers::{AuthorityId, ElectionId},
    modality::ElectionModality,
};

/// Agregado público vinculado a uma única fotografia de encerramento.
///
/// Os componentes podem ser a identidade multiplicativa para zero cédulas.
/// Este valor não efetua agregação nem garante sozinho que o quadro foi completo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aggregate {
    election_id: ElectionId,
    manifest_hash: Fingerprint,
    closure_hash: Fingerprint,
    ciphertexts: Vec<Ciphertext>,
}

impl Aggregate {
    /// Valida dimensão/IDs; a porta confere grupo, ordem e produto da fotografia.
    pub fn try_new(
        election_id: ElectionId,
        manifest_hash: Fingerprint,
        closure_hash: Fingerprint,
        ciphertexts: Vec<Ciphertext>,
    ) -> Result<Self, InvalidBallot> {
        validate_ciphertexts(&ciphertexts)?;
        Ok(Self {
            election_id,
            manifest_hash,
            closure_hash,
            ciphertexts,
        })
    }
    /// Eleição agregada.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }
    /// Manifesto que define a ordem das opções.
    pub const fn manifest_hash(&self) -> &Fingerprint {
        &self.manifest_hash
    }
    /// Fotografia comum a contagem, cifras e contribuições.
    pub const fn closure_hash(&self) -> &Fingerprint {
        &self.closure_hash
    }
    /// Cifras agregadas na ordem do manifesto, incluindo Nulo.
    pub fn ciphertexts(&self) -> &[Ciphertext] {
        &self.ciphertexts
    }
}

/// Resultado acompanhado das referências e contribuições verificadas externamente.
///
/// Não prova validade individual das cédulas. `valid` significa apenas contagem
/// consistente sob as premissas aprovadas. Identificada não possui agregado nem
/// contribuições criptográficas e usa [`Self::try_identified`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedTally {
    result: ElectionResult,
    aggregate: Option<Aggregate>,
    aggregate_hash: Option<Fingerprint>,
    contributing_authorities: Vec<AuthorityId>,
}

/// Recusa ao tratar evidência confidencial ou cancelamento como contagem identificada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidIdentifiedTally {
    /// A contagem confidencial precisa das verificações de agregado/contribuições.
    #[error("Resultado confidencial exige evidências criptográficas")]
    ConfidentialResult,
    /// Cancelamento administrativo não constitui uma apuração.
    #[error("Cancelamento administrativo não é uma contagem")]
    AdministrativeCancellation,
}

impl VerifiedTally {
    /// Constrói a evidência após conferir agregado, provas e assinaturas.
    #[allow(
        dead_code,
        reason = "A verificação de produção pertence às issues #378–#380/#139."
    )]
    pub(crate) fn from_verified_parts(
        result: ElectionResult,
        aggregate: Aggregate,
        aggregate_hash: Fingerprint,
        contributing_authorities: Vec<AuthorityId>,
    ) -> Self {
        Self {
            result,
            aggregate: Some(aggregate),
            aggregate_hash: Some(aggregate_hash),
            contributing_authorities,
        }
    }
    /// Encapsula somente contagem identificada, sem dispensar a fotografia final.
    ///
    /// Recusa resultado confidencial ou cancelamento. O agregado continua
    /// conferindo eleição, fotografia, ordem e consistência antes da publicação.
    pub fn try_identified(result: ElectionResult) -> Result<Self, InvalidIdentifiedTally> {
        if result.modality() != ElectionModality::Identified {
            return Err(InvalidIdentifiedTally::ConfidentialResult);
        }
        if result.status() == ResultStatus::Cancelled {
            return Err(InvalidIdentifiedTally::AdministrativeCancellation);
        }
        Ok(Self {
            result,
            aggregate: None,
            aggregate_hash: None,
            contributing_authorities: vec![],
        })
    }
    /// Resultado contextual a publicar.
    pub const fn result(&self) -> &ElectionResult {
        &self.result
    }
    /// Agregado confidencial cuja fotografia foi verificada.
    pub const fn aggregate(&self) -> Option<&Aggregate> {
        self.aggregate.as_ref()
    }
    /// Digest independente do agregado, conferido pelo codec.
    pub const fn aggregate_hash(&self) -> Option<&Fingerprint> {
        self.aggregate_hash.as_ref()
    }
    /// Autoridades com contribuição válida para este agregado.
    pub fn contributing_authorities(&self) -> &[AuthorityId] {
        &self.contributing_authorities
    }
}
