//! Referência pública ao registro confirmado e seu recibo assinado.

use super::{
    base64_url::Base64Url, fingerprint::Fingerprint, identifiers::ElectionId,
    voter_count::MAX_VOTER_COUNT,
};

/// Recibo confidencial emitido somente após commit, conforme contrato da porta.
///
/// A construção valida estrutura; o codec verifica a assinatura e o leitor
/// público confere inclusão. Uma assinatura válida não prova inclusão sozinha.
/// Não contém escolha em claro, chave privada ou aleatoriedade de cifragem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    election_id: ElectionId,
    manifest_hash: Fingerprint,
    ballot_hash: Fingerprint,
    index: u32,
    entry_hash: Fingerprint,
    signature: Base64Url,
}

/// Índice inválido no quadro, cujos registros começam em um.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("O índice do recibo deve estar entre 1 e 100000")]
pub struct InvalidReceiptIndex;

impl Receipt {
    /// Valida o índice sem afirmar confirmação transacional ou inclusão pública.
    pub fn try_new(
        election_id: ElectionId,
        manifest_hash: Fingerprint,
        ballot_hash: Fingerprint,
        index: u32,
        entry_hash: Fingerprint,
        signature: Base64Url,
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
    /// Assinatura do servidor, a verificar pelo codec.
    pub const fn signature(&self) -> &Base64Url {
        &self.signature
    }
}
