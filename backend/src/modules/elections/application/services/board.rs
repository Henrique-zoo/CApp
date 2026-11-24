//! Aceite de votos e consultas das evidências eleitorais mantidas pelo backend.
//!
//! Eleitores e auditores consultam o quadro e seu checkpoint para verificar a
//! inclusão de cédulas. A consulta privada permite à conta autenticada recuperar
//! o recibo original, inclusive quando o resultado do envio do voto se perdeu.
//! O aceite obtém o contexto confiável, chama o verificador e distingue um
//! reenvio de uma segunda cédula quando já há participação confirmada.

use crate::modules::elections::domain::{
    cryptography::elgamal::accumulate_ballot,
    entities::{
        ballot::{BallotContent, SignedBallot},
        vote::VoteKey,
    },
    error::InvalidBallot,
    identifiers::ElectionId,
    profile::MAX_VOTER_COUNT,
    rules::registration::{RegisteredKeyError, validate_registered_key},
};
use serde_json::Value;

use super::super::{
    board::{BoardError, BoardSnapshot, FrozenBoard, NewVote, RecordedBallot, VoteState},
    ports::{
        BallotVerifier, BoardClosureTransaction, BoardEvidenceSigner, BoardSnapshotStream,
        ElectionBoardReader, ElectionBoardRepository, VoteEvidenceSigner, VoteRepository,
        VoteTransaction,
    },
    registration::VoterIdentity,
};

/// Verifica a origem assinada e publica a configuração imutável da eleição.
/// Uma publicação idêntica devolve o quadro existente. Um conflito de origem
/// para a mesma eleição retorna [`BoardError::AlreadyExists`].
pub async fn publish(
    boards: &impl ElectionBoardRepository,
    signer: &impl BoardEvidenceSigner,
    origin: FrozenBoard,
) -> Result<BoardSnapshot, BoardError> {
    let board = signer.verify_origin(origin)?;
    boards.publish(board).await
}

/// Encerra o quadro após o fim do prazo segundo o relógio do banco.
/// O bloqueio impede novos aceites enquanto o serviço assina os dados finais.
/// Repetir o encerramento recupera as evidências publicadas no primeiro commit.
pub async fn close(
    boards: &impl ElectionBoardRepository,
    signer: &impl BoardEvidenceSigner,
    election_id: ElectionId,
) -> Result<BoardSnapshot, BoardError> {
    let (closing, transaction) = boards.lock_closure(election_id).await?;
    let Some(closing) = closing else {
        return transaction.finish(None).await;
    };

    if closing.now < closing.ends_at {
        return Err(BoardError::NotYetEnded);
    }
    if closing.server_key != signer.public_key()? {
        return Err(BoardError::InvalidArtifact(
            "chave de assinatura diferente do manifesto",
        ));
    }

    let evidence = signer.sign_closure(closing)?;
    transaction.finish(Some(evidence)).await
}

/// Encerra os quadros vencidos usando o mesmo serviço de fechamento transacional.
/// Uma falha em um quadro é registrada e os demais continuam sendo processados.
pub async fn close_due(
    boards: &impl ElectionBoardRepository,
    signer: &impl BoardEvidenceSigner,
) -> Result<(), BoardError> {
    for election in boards.due_for_closure().await? {
        if let Err(error) = close(boards, signer, election).await {
            tracing::error!(%error, election_id = %election.as_uuid(), "Falha ao encerrar quadro eleitoral");
        }
    }
    Ok(())
}

/// Exporta as evidências públicas em ordem de aceite.
///
/// # Parâmetros
/// - `reader`: lê os artefatos da mesma versão confirmada do quadro.
/// - `election_id`: identifica a eleição a ser exportada.
///
/// # Retorno
/// Retorna um fluxo de fragmentos do JSON público. Se a eleição ainda não tem
/// quadro publicado, retorna [`BoardError::NotFound`] antes de abrir o fluxo.
pub async fn stream_snapshot(
    reader: &impl ElectionBoardReader,
    election_id: ElectionId,
) -> Result<BoardSnapshotStream, BoardError> {
    reader.stream_snapshot(election_id).await
}

/// Obtém a evidência que ancora o estado atual do quadro.
///
/// # Parâmetros
/// - `reader`: consulta as evidências já persistidas.
/// - `election_id`: identifica a eleição consultada.
///
/// # Retorno
/// Último recibo assinado ou manifesto assinado se ainda não houver votos.
/// Quem audita pode guardar a evidência para comparar com uma exportação futura.
pub async fn read_checkpoint(
    reader: &impl ElectionBoardReader,
    election_id: ElectionId,
) -> Result<Value, BoardError> {
    reader.read_checkpoint(election_id).await
}

/// Recupera o recibo original do voto da própria conta.
///
/// # Parâmetros
/// - `reader`: busca recibos já confirmados.
/// - `key`: identifica a eleição e a conta obtida na autenticação.
///
/// # Retorno
/// Recibo assinado ou `Ok(None)` quando a conta não tem recibo nessa eleição.
/// É possível recuperar o recibo após o encerramento ou o fim da elegibilidade.
pub async fn read_own_receipt(
    reader: &impl ElectionBoardReader,
    key: VoteKey,
) -> Result<Option<Value>, BoardError> {
    reader.find_receipt(key).await
}

/// Confirma a cédula assinada da conta autenticada.
///
/// # Parâmetros
/// - `verifier`: verifica a cédula contra a conta e o manifesto oficiais.
/// - `votes`: consulta o contexto e confirma a participação e suas evidências.
/// - `election_id`: identifica a eleição solicitada.
/// - `signer`: emite as evidências associadas ao novo aceite.
/// - `voter`: conta e vínculo obtidos da autenticação.
/// - `ballot`: cédula com os IDs provenientes da rota e da conta autenticada.
///
/// # Retorno
/// Devolve o registro confirmado e o recibo original. Uma cédula verificada
/// com o mesmo hash recupera ambos, mesmo após o término da votação.
pub async fn record_ballot(
    verifier: &impl BallotVerifier,
    votes: &impl VoteRepository,
    signer: &impl VoteEvidenceSigner,
    voter: VoterIdentity,
    ballot: SignedBallot,
) -> Result<RecordedBallot, BoardError> {
    if ballot.account_id() != voter.account_id {
        return Err(BoardError::VoterMismatch);
    }

    let election_id = ballot.election_id();
    let context = votes.ballot_context(election_id).await?;
    let BallotContent::Confidential {
        manifest_hash,
        ciphertexts,
    } = ballot.content()
    else {
        return Err(BoardError::InvalidEvidence(Box::new(
            InvalidBallot::ConfigurationMismatch,
        )));
    };
    if *manifest_hash != context.manifest_hash || ciphertexts.option_ids() != context.option_ids {
        return Err(BoardError::InvalidEvidence(Box::new(
            InvalidBallot::ConfigurationMismatch,
        )));
    }

    let verified = verifier
        .verify_ballot(ballot)
        .map_err(|error| BoardError::InvalidEvidence(Box::new(error)))?;
    let (state, transaction) = votes.lock_vote(election_id, voter).await?;

    let window = match state {
        VoteState::Pending(window) => window,
        VoteState::Accepted(accepted) => {
            if &accepted.ballot_hash == verified.ballot_hash() {
                return Ok(accepted.record);
            }
            return Err(BoardError::AlreadyParticipated);
        }
    };

    if window.cancelled {
        return Err(BoardError::Cancelled);
    }
    if window.closed {
        return Err(BoardError::Closed);
    }
    if window.now < window.starts_at {
        return Err(BoardError::VotingNotStarted);
    }
    if window.now >= window.ends_at {
        return Err(BoardError::VotingEnded);
    }
    if window.eligible_until.is_none_or(|end| window.now >= end) {
        return Err(BoardError::NotEligible);
    }

    let ballot = verified.ballot();
    if window.requires_registration {
        validate_registered_key(window.registered_key.as_deref(), ballot.public_key()).map_err(
            |error| match error {
                RegisteredKeyError::Missing => BoardError::RegistrationRequired,
                RegisteredKeyError::Mismatch => BoardError::RegisteredKeyMismatch,
            },
        )?;
    }
    if window.server_key != signer.public_key()? {
        return Err(BoardError::InvalidArtifact(
            "chave de assinatura diferente do manifesto",
        ));
    }
    if window.voter_count >= MAX_VOTER_COUNT {
        return Err(BoardError::LimitExceeded);
    }

    let BallotContent::Confidential { ciphertexts, .. } = ballot.content() else {
        return Err(BoardError::InvalidEvidence(Box::new(
            InvalidBallot::ConfigurationMismatch,
        )));
    };
    let accumulator = accumulate_ballot(window.accumulator, ciphertexts.encrypted_ballot())
        .map_err(|_| BoardError::InvalidArtifact("acumulador"))?;
    let index = window.voter_count + 1;
    let evidence = signer.sign_vote(&verified, index, &window.head_hash)?;

    transaction
        .save(NewVote {
            ballot_hash: *verified.ballot_hash(),
            public_key: ballot.public_key().to_vec(),
            accepted_at: window.now,
            index,
            entry_hash: evidence.entry_hash,
            accumulator,
            record: evidence.record,
        })
        .await
}
