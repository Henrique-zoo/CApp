//! Executa as tarefas automáticas previstas no ciclo de vida da eleição.
//!
//! A cada ciclo, encerra pleitos com prazo vencido e entrega as contribuições
//! pendentes do servidor. A tarefa pertence ao processo que serve a API.

use std::{sync::Arc, time::Duration};

use tokio::{
    task::JoinHandle,
    time::{self, MissedTickBehavior},
};

use crate::{
    infrastructure::database::PgDatabase,
    modules::elections::{
        application::services::{board, tally},
        crypto_wire::ServerSigner,
    },
};

/// Inicia o processamento periódico com o banco compartilhado e a chave da instalação.
///
/// O primeiro ciclo começa imediatamente. Os seguintes usam um intervalo de
/// dois segundos. Uma falha é registrada e a tarefa volta a tentar no próximo
/// ciclo. O chamador recebe o handle para cancelar a tarefa ao encerrar a API.
pub fn start_election_clock(database: PgDatabase, signer: Arc<ServerSigner>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            if let Err(error) = board::close_due(&database, signer.as_ref()).await {
                tracing::error!(%error, "Falha ao consultar eleições a encerrar");
            }

            if let Err(error) =
                tally::submit_pending_server_shares(&database, &database, signer.as_ref()).await
            {
                tracing::error!(%error, "Falha ao consultar contribuições do servidor");
            }
        }
    })
}
