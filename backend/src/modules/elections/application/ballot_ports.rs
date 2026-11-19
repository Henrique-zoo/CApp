//! Fronteiras de verificação e aceite transacional previstas em `interfaces.json`.
//!
//! O verificador coordena `CryptographicCodec`/`ElectionCryptography`; o store
//! implementa `ElectionStore.recordBallot`. Aqui não há algoritmos, SQL ou HTTP.

use super::ports::AuthenticatedElectorContext;
use crate::modules::elections::domain::{
    entities::{
        ballot::{SignedBallot, VerifiedBallot},
        manifest::{Manifest, VerifiedManifest},
        participation::{Participation, ParticipationKey},
    },
    value_objects::{base64_url::Base64Url, receipt::Receipt},
};

/// Recusa criptográfica ou falha técnica distinguíveis pela aplicação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ElectionEvidenceError {
    /// Codificação, assinatura, parâmetro ou prova inválida.
    #[error("Evidência eleitoral inválida")]
    InvalidEvidence,
    /// A verificação não pôde ser concluída por falha técnica.
    #[error("Verificação eleitoral indisponível")]
    Unavailable,
}

/// Verifica material público antes de autorizar abertura ou novo aceite.
///
/// O adaptador rejeita propriedades desconhecidas/duplicadas no envelope original
/// antes de criar estes valores. Confere RSA/DER, digest, assinatura, subgrupo,
/// provas de autoridade e chave conjunta conforme o tipo. Não verifica bits da
/// cédula: essa garantia individual não existe na v1.
#[async_trait::async_trait]
pub trait ElectionEvidenceVerifier: Send + Sync {
    /// Confere manifesto e assinatura com a chave confiável do servidor.
    async fn verify_manifest(
        &self,
        manifest: Manifest,
        signature: Base64Url,
    ) -> Result<VerifiedManifest, ElectionEvidenceError>;
    /// Confere a assinatura com a chave presente e as cifras quando confidencial.
    async fn verify_ballot(
        &self,
        ballot: SignedBallot,
    ) -> Result<VerifiedBallot, ElectionEvidenceError>;
}

/// Aceite confirmado, com recibo obrigatório somente no protocolo confidencial.
///
/// O construtor verifica referências; cabe ao adaptador devolver este valor
/// somente depois de confirmar voto/participação/quadro/recibo na mesma transação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedBallot {
    participation: Participation,
    receipt: Option<Receipt>,
}

/// Recibo ausente ou associado a outro registro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Recibo incompatível com a participação confirmada")]
pub struct InvalidRecordedBallot;

impl RecordedBallot {
    /// Confere referências sem substituir o commit ou a verificação da assinatura.
    pub fn try_new(
        participation: Participation,
        receipt: Option<Receipt>,
    ) -> Result<Self, InvalidRecordedBallot> {
        use crate::modules::elections::domain::entities::ballot::BallotContent;
        match (participation.accepted_ballot().ballot().content(), &receipt) {
            (BallotContent::Identified(_), None) => {}
            (BallotContent::Confidential { manifest_hash, .. }, Some(receipt))
                if receipt.election_id() == participation.key().election_id()
                    && receipt.manifest_hash() == manifest_hash
                    && receipt.ballot_hash() == participation.accepted_ballot().ballot_hash() => {}
            _ => return Err(InvalidRecordedBallot),
        }
        Ok(Self {
            participation,
            receipt,
        })
    }
    /// Primeiro aceite, preservado em recuperações idempotentes.
    pub const fn participation(&self) -> &Participation {
        &self.participation
    }
    /// Recibo confidencial do registro confirmado, sem promessa de inclusão isolada.
    pub const fn receipt(&self) -> Option<&Receipt> {
        self.receipt.as_ref()
    }
}

/// O store distingue gravação nova de recuperação sem efeitos adicionais.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordBallotOutcome {
    /// Nova participação e evidências confirmadas.
    Accepted(RecordedBallot),
    /// Mesmo pacote/operação recuperados, inclusive após encerramento.
    Recovered(RecordedBallot),
}

/// Diagnósticos da operação atômica de aceite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RecordBallotError {
    /// Outra operação alterou a eleição; repetir após leitura atualizada.
    #[error("Conflito de concorrência eleitoral")]
    Conflict,
    /// Existe participação desta conta por outra operação/cédula.
    #[error("A conta já participou da eleição")]
    AlreadyParticipated,
    /// Mesmo identificador de envio com outro payload.
    #[error("Envio eleitoral conflitante")]
    SubmissionConflict,
    /// Estado, prazo, configuração, identidade ou elegibilidade recusados.
    #[error("Novo voto recusado")]
    Rejected,
    /// Falha de armazenamento; não confirma o aceite ao chamador.
    #[error("Armazenamento eleitoral indisponível")]
    Unavailable,
}

/// Operação especializada de #282/#381, distinta do `save` genérico.
///
/// Impõe unicidade por conta/eleição; autentica antes de recuperar um aceite.
/// Repetição idêntica é resolvida antes das regras de novo voto e não exige vínculo
/// institucional atual. Novo aceite reavalia elegibilidade/estado/prazo e confirma
/// chave, participação, cédula, índice, cadeia e recibo atomicamente. Encerramento,
/// calendário e cancelamento usam a mesma ordem transacional. Não emite recibo
/// para rollback e não aceita conta ou curso declarados no corpo da requisição.
#[async_trait::async_trait]
pub trait BallotStore: Send + Sync {
    /// Confirma ou recupera uma submissão sem criar participação duplicada.
    async fn record_ballot(
        &self,
        elector: &AuthenticatedElectorContext,
        ballot: VerifiedBallot,
        expected_version: u64,
    ) -> Result<RecordBallotOutcome, RecordBallotError>;
    /// Recupera a participação pela conta autenticada, sem exigir chave privada.
    /// A aplicação autoriza a consulta da própria conta antes de chamar a porta.
    async fn find_participation(
        &self,
        key: ParticipationKey,
    ) -> Result<Option<RecordedBallot>, RecordBallotError>;
}
