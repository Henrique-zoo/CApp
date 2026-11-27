//! Evidências públicas da apuração lidas ou confirmadas pelo serviço eleitoral.

use crypto_bigint::U2048;
use serde::Serialize;
use serde_json::Value;

use super::super::domain::{
    Fingerprint,
    cryptography::elgamal::DecryptionShare,
    identifiers::{AuthorityId, ElectionId},
};
use uuid::Uuid;

/// Contribuição de uma opção, ainda não verificada.
pub struct OptionShare {
    /// Identificador da opção, inclusive o UUID reservado ao Nulo.
    pub option_id: Uuid,
    /// Valor parcial e prova Chaum-Pedersen.
    pub share: DecryptionShare,
}

/// Contribuição assinada para um agregado específico, ainda não verificada.
/// Os hashes e identificadores fazem parte da mensagem autenticada.
pub struct ShareSubmission {
    /// Eleição declarada na mensagem assinada.
    pub election: ElectionId,
    /// Autoridade declarada na mensagem assinada.
    pub authority: AuthorityId,
    /// Digest do manifesto.
    pub manifest_hash: Fingerprint,
    /// Digest do encerramento.
    pub closure_hash: Fingerprint,
    /// Digest do agregado.
    pub aggregate_hash: Fingerprint,
    /// Valores parciais na ordem das opções.
    pub shares: Vec<OptionShare>,
    /// Assinatura RSA-PSS ainda não autenticada.
    pub signature: Vec<u8>,
}

/// Conjunto de contribuições aceitas e último resultado assinado.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TallyEvidence {
    /// Contribuições das autoridades, com suas assinaturas e provas públicas.
    pub shares: Vec<Value>,
    /// Resultado assinado. Antes da primeira contribuição, o valor é ausente.
    /// Uma apuração incompleta informa as autoridades pendentes, sem totais.
    pub result: Option<Value>,
}

/// Dados de uma contribuição confirmada que permitem detectar reenvio.
pub struct StoredShare {
    /// Autoridade registrada para este agregado.
    pub authority: AuthorityId,
    /// Digest do payload, independente da aleatoriedade da assinatura.
    pub digest: String,
    /// Envelope público canônico, reconstruído e autenticado pelo verificador.
    pub envelope: Value,
}

/// Situação lida enquanto a eleição está bloqueada para gravar uma contribuição.
pub struct TallyState {
    /// Contribuições já confirmadas para a eleição.
    pub shares: Vec<StoredShare>,
    /// Último resultado assinado publicado junto da respectiva contribuição.
    pub result: Option<Value>,
}

impl From<TallyState> for TallyEvidence {
    fn from(state: TallyState) -> Self {
        Self {
            shares: state
                .shares
                .into_iter()
                .map(|share| share.envelope)
                .collect(),
            result: state.result,
        }
    }
}

/// Chave própria do servidor para uma eleição encerrada. Nunca é serializada.
pub struct PendingServerShare {
    /// Eleição cujo fechamento autorizou a contribuição.
    pub election: ElectionId,
    /// Autoridade do servidor congelada no manifesto.
    pub authority: AuthorityId,
    /// Segredo privado do servidor.
    pub secret: U2048,
}
