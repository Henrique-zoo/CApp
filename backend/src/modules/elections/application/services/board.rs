//! Consultas que distribuem as evidências eleitorais mantidas pelo backend.
//!
//! Eleitores e auditores consultam o quadro e seu checkpoint para verificar a
//! inclusão de cédulas. A consulta privada permite à conta autenticada recuperar
//! o recibo original, inclusive quando o resultado do envio do voto se perdeu.

use serde_json::Value;
use uuid::Uuid;

use super::super::{
    board::{BoardError, ElectionBoard, RecordedBallot},
    ports::{BoardSnapshotStream, ElectionBoardReader},
    registration::VoterIdentity,
};

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
    election_id: Uuid,
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
    election_id: Uuid,
) -> Result<Value, BoardError> {
    reader.read_checkpoint(election_id).await
}

/// Recupera o recibo original do voto da própria conta.
///
/// # Parâmetros
/// - `reader`: busca recibos já confirmados.
/// - `election_id`: identifica a eleição consultada.
/// - `authenticated_account_id`: UUID da conta obtido na autenticação.
///
/// # Retorno
/// Recibo assinado ou `Ok(None)` quando a conta não tem recibo nessa eleição.
/// É possível recuperar o recibo após o encerramento ou o fim da elegibilidade.
pub async fn read_own_receipt(
    reader: &impl ElectionBoardReader,
    election_id: Uuid,
    authenticated_account_id: Uuid,
) -> Result<Option<Value>, BoardError> {
    reader
        .find_receipt(election_id, authenticated_account_id)
        .await
}

/// Confirma a cédula assinada da conta autenticada.
///
/// # Parâmetros
/// - `board`: persiste a participação, o registro, o recibo e o agregado.
/// - `election_id`: identifica a eleição solicitada.
/// - `voter`: conta e vínculo obtidos da autenticação.
/// - `signed_ballot`: bytes originais do envelope assinado.
///
/// # Retorno
/// Devolve o registro confirmado e o recibo original. Uma nova tentativa da
/// mesma submissão recupera ambos sem acrescentar outro voto ao acumulador.
pub async fn record_ballot(
    board: &impl ElectionBoard,
    election_id: Uuid,
    voter: VoterIdentity,
    signed_ballot: &[u8],
) -> Result<RecordedBallot, BoardError> {
    board.record(election_id, voter, signed_ballot).await
}
