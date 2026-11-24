//! Verificação da origem do quadro e codificação dos artefatos de encerramento.
//!
//! A aplicação decide quando publicar ou encerrar. Este adaptador confere a
//! origem assinada e assina os dados finais usando o protocolo público.

use chrono::SecondsFormat;
use serde_json::json;

use crate::modules::elections::{
    application::{
        board::{BoardClosing, BoardClosure, BoardError, BoardPublication, FrozenBoard},
        ports::BoardEvidenceSigner,
    },
    crypto_wire::{CryptoWireError, ServerSigner, object_digest, verify_manifest},
    domain::identifiers::AcademicCenterId,
};

use super::ballot_codec::ciphertexts_payload;

impl From<CryptoWireError> for BoardError {
    fn from(error: CryptoWireError) -> Self {
        match error {
            CryptoWireError::Signing => Self::SigningUnavailable,
            invalid => Self::InvalidEvidence(Box::new(invalid)),
        }
    }
}

impl BoardEvidenceSigner for ServerSigner {
    fn verify_origin(&self, board: FrozenBoard) -> Result<BoardPublication, BoardError> {
        let server_key = self.public_key_base64()?;
        let manifest = verify_manifest(&board.manifest, &server_key)?;
        if manifest.election_id != board.election_id.into_uuid() {
            return Err(CryptoWireError::Context.into());
        }

        Ok(BoardPublication {
            election_id: board.election_id,
            academic_center_id: AcademicCenterId::try_from_uuid(manifest.academic_center_id)
                .map_err(|error| BoardError::InvalidEvidence(Box::new(error)))?,
            manifest: manifest.envelope,
            manifest_hash: manifest.manifest_hash,
            starts_at: *board.period.starts_at(),
            ends_at: *board.period.ends_at(),
            option_ids: manifest.option_ids,
            requires_registration: board.requires_voter_registration,
        })
    }

    fn sign_closure(&self, closing: BoardClosing) -> Result<BoardClosure, BoardError> {
        let closure_payload = json!({
            "version": 1,
            "kind": "closure",
            "electionId": closing.election_id.into_uuid(),
            "manifestHash": closing.manifest_hash,
            "headHash": closing.head_hash,
            "voterCount": closing.voter_count,
            "closedAt": closing.now.to_rfc3339_opts(SecondsFormat::Secs, true),
        });
        let aggregate_payload = json!({
            "version": 1,
            "kind": "aggregate",
            "electionId": closing.election_id.into_uuid(),
            "manifestHash": closing.manifest_hash,
            "closureHash": object_digest(&closure_payload)?,
            "ciphertexts": ciphertexts_payload(&closing.option_ids, &closing.accumulator),
        });

        Ok(BoardClosure {
            closure: self.sign_payload(&closure_payload)?,
            aggregate: self.sign_payload(&aggregate_payload)?,
        })
    }
}
