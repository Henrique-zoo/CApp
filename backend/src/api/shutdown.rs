//! Gerenciamento de ciclo de vida, sinais do sistema operacional e encerramento controlado (*graceful shutdown*).
//!
//! Este módulo implementa o protocolo de término gracioso da API Axum do CApp:
//!
//! 1. **Captura Portável de Sinais**: escuta `SIGINT` (Ctrl+C) e `SIGTERM` (padrão de orquestradores
//!    como Kubernetes e Docker) em sistemas Unix, e eventos equivalentes no Windows.
//! 2. **Interrupção de Aceite de Novas Conexões**: fecha o socket de recepção para que novas conexões
//!    não sejam admitidas.
//! 3. **Indicação de Prontidão**: atualiza o estado compartilhado da aplicação, fazendo com que o
//!    endpoint `/ready` responda imediatamente com `HTTP 503 Service Unavailable`, sinalizando
//!    aos balanceadores de carga para remover o nó do tráfego ativo.
//! 4. **Drenagem com Timeout (Deadline)**: concede uma janela temporal configurável para que requisições
//!    já em trânsito sejam concluídas; caso excedido o prazo, o encerramento é forçado.
//! 5. **Liberação Ordenada de Recursos**: fecha de forma assíncrona o pool de conexões com o PostgreSQL
//!    ([`sqlx::PgPool`]) somente após o término das requisições em voo.

use std::{future::IntoFuture, time::Duration};

use crate::api::state::AppState;

/// Aguarda de forma portável um sinal de término emitido pelo sistema operacional ou pelo usuário.
///
/// Em sistemas Unix, escuta tanto `SIGINT` (Ctrl+C no terminal) quanto `SIGTERM` (sinal padrão
/// emitido por Kubernetes, Docker e systemd ao solicitar a parada de um processo).
///
/// Em sistemas Windows, escuta eventos de console como Ctrl+C, Ctrl+Break, fechamento de janela
/// (Ctrl+Close) e encerramento do sistema (Ctrl+Shutdown).
///
/// Em outras plataformas, utiliza a captura padrão de Ctrl+C via [`tokio::signal::ctrl_c`].
///
/// # Exemplos
///
/// ```no_run
/// use backend::api::shutdown::wait_for_shutdown_signal;
///
/// # #[tokio::main]
/// # async fn main() {
/// tracing::info!("Aguardando sinal do sistema...");
/// wait_for_shutdown_signal().await;
/// tracing::info!("Sinal recebido, prosseguindo com encerramento.");
/// # }
/// ```
pub async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(sig) => sig,
            Err(err) => {
                tracing::error!(
                    %err,
                    "Falha ao registrar handler para o sinal SIGTERM. Aguardando apenas SIGINT (Ctrl+C)."
                );
                let _ = tokio::signal::ctrl_c().await;
                tracing::info!("Sinal SIGINT (Ctrl+C) recebido do terminal.");
                return;
            }
        };

        tokio::select! {
            res = tokio::signal::ctrl_c() => {
                match res {
                    Ok(()) => tracing::info!("Sinal SIGINT (Ctrl+C) recebido do terminal."),
                    Err(err) => tracing::error!(%err, "Erro ao aguardar sinal SIGINT."),
                }
            }
            _ = sigterm.recv() => {
                tracing::info!("Sinal SIGTERM recebido do orquestrador ou sistema operacional.");
            }
        }
    }

    #[cfg(windows)]
    {
        use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close, ctrl_shutdown};

        let mut sig_c = ctrl_c().ok();
        let mut sig_break = ctrl_break().ok();
        let mut sig_close = ctrl_close().ok();
        let mut sig_shutdown = ctrl_shutdown().ok();

        tokio::select! {
            _ = async {
                if let Some(ref mut sig) = sig_c {
                    sig.recv().await
                } else {
                    let _ = tokio::signal::ctrl_c().await;
                    Some(())
                }
            } => {
                tracing::info!("Sinal Ctrl+C recebido no Windows.");
            }
            _ = async {
                if let Some(ref mut sig) = sig_break {
                    sig.recv().await
                } else {
                    std::future::pending::<Option<()>>().await
                }
            } => {
                tracing::info!("Sinal Ctrl+Break recebido no Windows.");
            }
            _ = async {
                if let Some(ref mut sig) = sig_close {
                    sig.recv().await
                } else {
                    std::future::pending::<Option<()>>().await
                }
            } => {
                tracing::info!("Evento de fechamento de console recebido no Windows.");
            }
            _ = async {
                if let Some(ref mut sig) = sig_shutdown {
                    sig.recv().await
                } else {
                    std::future::pending::<Option<()>>().await
                }
            } => {
                tracing::info!("Evento de encerramento do sistema recebido no Windows.");
            }
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        match tokio::signal::ctrl_c().await {
            Ok(()) => tracing::info!("Sinal Ctrl+C recebido."),
            Err(err) => tracing::error!(%err, "Erro ao registrar ou aguardar sinal Ctrl+C."),
        }
    }
}

#[derive(Clone)]
struct DrainTracker {
    cancel_rx: tokio::sync::watch::Receiver<bool>,
    active_tx: tokio::sync::watch::Sender<usize>,
}

async fn drain_middleware(
    axum::extract::State(tracker): axum::extract::State<DrainTracker>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    if *tracker.cancel_rx.borrow() {
        return reqwest::StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    struct DrainGuard(tokio::sync::watch::Sender<usize>);
    impl Drop for DrainGuard {
        fn drop(&mut self) {
            self.0.send_modify(|c| *c = c.saturating_sub(1));
        }
    }

    tracker.active_tx.send_modify(|c| *c += 1);
    let _guard = DrainGuard(tracker.active_tx.clone());

    let mut cancel_rx = tracker.cancel_rx.clone();

    tokio::select! {
        biased;
        _ = cancel_rx.wait_for(|cancelled| *cancelled) => {
            tracing::warn!("Requisição HTTP em andamento cancelada por expiração do timeout de encerramento.");
            reqwest::StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        response = next.run(request) => response,
    }
}

/// Executa o servidor HTTP Axum com protocolo de encerramento controlado (*graceful shutdown*).
///
/// O servidor aceita requisições normalmente até que o futuro `signal` seja concluído.
/// Ao receber o sinal:
///
/// 1. Interrompe o aceite de novas conexões no socket TCP.
/// 2. Atualiza o indicador de ciclo de vida em `state` (se fornecido), fazendo com que requisições
///    a `/ready` retornem `503 Service Unavailable`.
/// 3. Inicia o período de drenagem de requisições ativas com duração máxima de `shutdown_timeout`.
/// 4. Caso requisições continuem em andamento após o `shutdown_timeout`, força o cancelamento
///    efetivo de todos os handlers e tarefas em voo, aguardando sua finalização antes de seguir.
/// 5. Fecha de maneira ordenada o pool de conexões do banco de dados ([`sqlx::PgPool`]) caso
///    um [`AppState`] tenha sido associado, garantindo que o pool permaneça ativo durante a drenagem.
///
/// # Parâmetros
///
/// - `listener`: socket TCP previamente aberto onde o servidor aceitará conexões.
/// - `router`: roteador Axum configurado para atender as rotas da aplicação.
/// - `signal`: futuro assíncrono que dispara o início da drenagem ao completar.
/// - `shutdown_timeout`: prazo máximo concedido para a conclusão de requisições em trânsito.
/// - `state`: estado compartilhado da aplicação contendo os recursos gerenciados (ex: pool PostgreSQL).
///
/// # Retorno
///
/// Retorna `Ok(())` quando o servidor conclui o ciclo de vida e encerramento com sucesso,
/// ou um erro [`std::io::Error`] se ocorrer falha irrecuperável de rede.
///
/// # Erros
///
/// Retorna erro se o runtime do Axum encontrar falhas de E/S na camada de transporte.
pub async fn serve_with_graceful_shutdown<F>(
    listener: tokio::net::TcpListener,
    router: axum::Router,
    signal: F,
    shutdown_timeout: Duration,
    state: Option<AppState>,
) -> Result<(), std::io::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    let (active_tx, mut active_rx) = tokio::sync::watch::channel(0usize);

    let tracker = DrainTracker {
        cancel_rx,
        active_tx,
    };

    let router = router.layer(axum::middleware::from_fn_with_state(
        tracker,
        drain_middleware,
    ));

    let server_future = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        })
        .into_future();

    tokio::pin!(server_future);
    tokio::pin!(signal);

    tokio::select! {
        biased;

        res = &mut server_future => {
            match res {
                Ok(()) => tracing::info!("Servidor HTTP finalizado antes de receber sinal de encerramento."),
                Err(ref err) => tracing::error!(%err, "Servidor HTTP encerrou com erro antes de receber sinal de encerramento."),
            }
            if let Some(ref st) = state {
                tracing::info!("Fechando pool de conexões com o PostgreSQL...");
                st.pg_database.pool.close().await;
                tracing::info!("Pool PostgreSQL fechado com sucesso.");
            }
            return res;
        }
        _ = &mut signal => {
            tracing::info!(
                timeout_secs = shutdown_timeout.as_secs(),
                "Sinal de encerramento recebido. Interrompendo novos aceites e iniciando drenagem das requisições..."
            );
            if let Some(ref st) = state {
                st.mark_shutting_down();
            }
            let _ = shutdown_tx.send(());
        }
    }

    match tokio::time::timeout(shutdown_timeout, &mut server_future).await {
        Ok(Ok(())) => {
            tracing::info!("Drenagem de requisições HTTP concluída com sucesso dentro do prazo.");
        }
        Ok(Err(err)) => {
            tracing::error!(%err, "Erro retornado pelo servidor HTTP durante a drenagem de conexões.");
        }
        Err(_) => {
            tracing::warn!(
                timeout_secs = shutdown_timeout.as_secs(),
                "Prazo limite de drenagem de requisições HTTP expirou. Forçando cancelamento das conexões ativas."
            );
            let _ = cancel_tx.send(true);

            let verify_drain = active_rx.wait_for(|count| *count == 0);
            if tokio::time::timeout(Duration::from_secs(2), verify_drain)
                .await
                .is_err()
            {
                tracing::error!("Timeout ao aguardar finalização dos handlers cancelados.");
            } else {
                tracing::info!(
                    "Todos os handlers de requisições em andamento foram cancelados e finalizados."
                );
            }

            let _ = tokio::time::timeout(Duration::from_millis(500), &mut server_future).await;
        }
    }

    if let Some(ref st) = state {
        tracing::info!("Fechando pool de conexões com o PostgreSQL de forma ordenada...");
        st.pg_database.pool.close().await;
        tracing::info!("Pool PostgreSQL fechado com sucesso. Recursos liberados.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use sqlx::postgres::PgPoolOptions;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::net::TcpListener;
    use tokio::sync::{Mutex, oneshot};
    use tokio::time::sleep;

    /// Compõe o estado atual com adaptadores sem executar chamadas aos provedores.
    fn test_state(pool: sqlx::PgPool) -> AppState {
        use crate::{
            api::state::Clients,
            infrastructure::database::PgDatabase,
            modules::identity::infrastructure::{
                firebase::{AppCheckVerifier, FirebaseTokenVerifier},
                microsoft::MicrosoftGraphClient,
            },
        };
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("cliente HTTP de teste");
        let firebase = FirebaseTokenVerifier::new("capp-shutdown-test".into(), http.clone())
            .expect("configuração Firebase de teste");
        let app_check = AppCheckVerifier::new(
            "1234567890".into(),
            vec!["1:1234567890:android:capp".into()],
            http.clone(),
        )
        .expect("verificador App Check de teste");
        AppState::new(
            PgDatabase::new(pool),
            Clients::new(firebase, MicrosoftGraphClient::new(http), app_check),
        )
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_graceful_shutdown_drains_in_flight_request_and_rejects_new_connections() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("deve criar listener");
        let addr = listener.local_addr().expect("deve obter endereço local");

        let (started_tx, started_rx) = oneshot::channel();
        let started_tx = Arc::new(Mutex::new(Some(started_tx)));

        let app = Router::new().route(
            "/slow",
            get({
                let started_tx = Arc::clone(&started_tx);
                move || {
                    let started_tx = Arc::clone(&started_tx);
                    async move {
                        if let Some(tx) = started_tx.lock().await.take() {
                            let _ = tx.send(());
                        }
                        sleep(Duration::from_millis(150)).await;
                        "ok"
                    }
                }
            }),
        );

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server_task = tokio::spawn(async move {
            serve_with_graceful_shutdown(
                listener,
                app,
                async move {
                    let _ = shutdown_rx.await;
                },
                Duration::from_secs(2),
                None,
            )
            .await
        });

        // Dispara requisição HTTP em background que leva 150ms
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("deve construir client HTTP sem proxy");
        let slow_url = format!("http://{addr}/slow");
        let client_for_req = client.clone();
        let request_task = tokio::spawn(async move { client_for_req.get(slow_url).send().await });

        // Aguarda a requisição chegar ao handler e ser aceita
        started_rx.await.expect("handler deve ter iniciado");

        // Dispara sinal de shutdown
        shutdown_tx.send(()).expect("deve enviar sinal de shutdown");

        // Aguarda a parada do listener propagar no Axum
        sleep(Duration::from_millis(20)).await;

        // Comprova recusa efetiva de novas conexões TCP durante o encerramento
        let new_tcp_res = tokio::net::TcpStream::connect(addr).await;
        assert!(
            new_tcp_res.is_err(),
            "novas conexões TCP devem ser rejeitadas durante o shutdown"
        );

        // Comprova recusa efetiva de novas requisições HTTP durante o encerramento
        let new_http_res = client.get(format!("http://{addr}/slow")).send().await;
        assert!(
            new_http_res.is_err(),
            "novas requisições HTTP devem falhar na camada de transporte durante o shutdown"
        );

        // A requisição original em voo deve ser concluída com sucesso (drenada)
        let response = request_task
            .await
            .expect("join deve ter sucesso")
            .expect("requisição deve ter resposta");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), "ok");

        // O servidor deve finalizar com sucesso
        let server_res = server_task.await.expect("join deve ter sucesso");
        assert!(server_res.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_shutdown_timeout_forces_termination_when_deadline_exceeded() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("deve criar listener");
        let addr = listener.local_addr().expect("deve obter endereço local");

        let (started_tx, started_rx) = oneshot::channel();
        let started_tx = Arc::new(Mutex::new(Some(started_tx)));

        let handler_dropped = Arc::new(AtomicBool::new(false));
        let handler_completed_normally = Arc::new(AtomicBool::new(false));

        let handler_dropped_clone = Arc::clone(&handler_dropped);
        let handler_completed_clone = Arc::clone(&handler_completed_normally);

        struct HandlerDropCheck(Arc<AtomicBool>);
        impl Drop for HandlerDropCheck {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let app = Router::new().route(
            "/hang",
            get({
                let started_tx = Arc::clone(&started_tx);
                move || {
                    let started_tx = Arc::clone(&started_tx);
                    let handler_dropped_clone = Arc::clone(&handler_dropped_clone);
                    let handler_completed_clone = Arc::clone(&handler_completed_clone);
                    async move {
                        let _drop_check = HandlerDropCheck(handler_dropped_clone);
                        if let Some(tx) = started_tx.lock().await.take() {
                            let _ = tx.send(());
                        }
                        // Simula uma requisição excessivamente longa (5 segundos)
                        sleep(Duration::from_millis(5000)).await;
                        handler_completed_clone.store(true, Ordering::SeqCst);
                        "concluído"
                    }
                }
            }),
        );

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        // Timeout de drenagem estrito de apenas 100ms
        let drain_timeout = Duration::from_millis(100);

        let server_task = tokio::spawn(async move {
            serve_with_graceful_shutdown(
                listener,
                app,
                async move {
                    let _ = shutdown_rx.await;
                },
                drain_timeout,
                None,
            )
            .await
        });

        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("client sem proxy");
        let hang_url = format!("http://{addr}/hang");
        let request_task = tokio::spawn(async move { client.get(hang_url).send().await });

        // Aguarda a requisição iniciar de fato dentro do handler
        started_rx.await.expect("handler deve ter iniciado");

        // Dispara o shutdown
        let start_shutdown = std::time::Instant::now();
        shutdown_tx.send(()).expect("deve enviar sinal de shutdown");

        // O servidor deve terminar próximo ao deadline de 100ms, sem esperar os 5 segundos
        let server_res = server_task.await.expect("join deve ter sucesso");
        let elapsed = start_shutdown.elapsed();
        assert!(server_res.is_ok());
        assert!(
            elapsed < Duration::from_millis(1500),
            "o servidor deve encerrar forçadamente dentro do deadline configurado (demorou {elapsed:?})"
        );

        // Comprova o cancelamento e descarte efetivo do handler lento após timeout
        assert!(
            handler_dropped.load(Ordering::SeqCst),
            "o handler lento deve ser cancelado e descartado ao expirar o timeout"
        );
        assert!(
            !handler_completed_normally.load(Ordering::SeqCst),
            "o handler não deve ter completado normalmente sua execução"
        );

        // A requisição do cliente deve finalizar com 503 ou erro de conexão decorrente do cancelamento
        let req_res = request_task.await.expect("join deve ter sucesso");
        if let Ok(resp) = req_res {
            assert_eq!(resp.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn test_orderly_resource_release_order_on_shutdown() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .expect("cria pool lazy");

        let state = test_state(pool);
        let pool_handle = state.pg_database.pool.clone();
        assert!(!pool_handle.is_closed());

        let events = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let (started_tx, started_rx) = oneshot::channel();
        let started_tx = Arc::new(Mutex::new(Some(started_tx)));

        let events_handler = Arc::clone(&events);
        struct HandlerOrderGuard(Arc<Mutex<Vec<&'static str>>>);
        impl Drop for HandlerOrderGuard {
            fn drop(&mut self) {
                if let Ok(mut evts) = self.0.try_lock() {
                    evts.push("handler_finished");
                }
            }
        }

        let app = Router::new().route(
            "/work",
            get({
                let started_tx = Arc::clone(&started_tx);
                let events_handler = Arc::clone(&events_handler);
                move || {
                    let started_tx = Arc::clone(&started_tx);
                    let events_handler = Arc::clone(&events_handler);
                    async move {
                        let _guard = HandlerOrderGuard(events_handler);
                        if let Some(tx) = started_tx.lock().await.take() {
                            let _ = tx.send(());
                        }
                        sleep(Duration::from_millis(50)).await;
                        "ok"
                    }
                }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("deve criar listener");
        let addr = listener.local_addr().expect("deve obter endereço");

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let events_server = Arc::clone(&events);
        let server_state = state.clone();

        let server_task = tokio::spawn(async move {
            let res = serve_with_graceful_shutdown(
                listener,
                app,
                async move {
                    let _ = shutdown_rx.await;
                },
                Duration::from_secs(2),
                Some(server_state),
            )
            .await;
            events_server.lock().await.push("server_stopped");
            res
        });

        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("client sem proxy");
        let request_task =
            tokio::spawn(async move { client.get(format!("http://{addr}/work")).send().await });

        started_rx.await.expect("handler deve ter iniciado");
        shutdown_tx.send(()).expect("envia sinal de shutdown");

        let req_res = request_task
            .await
            .expect("join req deve ter sucesso")
            .expect("requisição deve ter resposta");
        assert_eq!(req_res.status(), reqwest::StatusCode::OK);

        let server_res = server_task.await.expect("join server deve ter sucesso");
        assert!(server_res.is_ok());

        assert!(
            pool_handle.is_closed(),
            "o pool de banco deve estar fechado ao término do shutdown"
        );

        let recorded_events = events.lock().await.clone();
        assert_eq!(
            recorded_events,
            vec!["handler_finished", "server_stopped"],
            "a execução HTTP deve ser finalizada antes do fechamento dos recursos de banco"
        );
    }

    #[tokio::test]
    async fn test_readiness_probe_returns_503_during_shutdown() {
        use tower::ServiceExt;

        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .expect("cria pool lazy");

        let state = test_state(pool);
        assert!(!state.is_shutting_down());

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("deve criar listener");

        let app = crate::api::router::create_router(state.clone());

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let state_for_server = state.clone();

        let server_task = tokio::spawn(async move {
            serve_with_graceful_shutdown(
                listener,
                app,
                async move {
                    let _ = shutdown_rx.await;
                },
                Duration::from_secs(1),
                Some(state_for_server),
            )
            .await
        });

        // Sinaliza encerramento
        shutdown_tx.send(()).expect("envia sinal");

        // Aguarda a marcação de encerramento propagar
        sleep(Duration::from_millis(20)).await;
        assert!(state.is_shutting_down());

        // Comprova que uma nova requisição a /ready é rejeitada com 503 SERVICE_UNAVAILABLE
        let test_router = crate::api::router::create_router(state.clone());
        let res = test_router
            .oneshot(
                axum::http::Request::builder()
                    .uri("/ready")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);

        let server_res = server_task.await.expect("join deve ter sucesso");
        assert!(server_res.is_ok());
        assert!(state.pg_database.pool.is_closed());
    }

    #[tokio::test]
    async fn test_orderly_pool_closing_on_shutdown() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .expect("cria pool lazy");

        let state = test_state(pool);
        assert!(!state.pg_database.pool.is_closed());

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("deve criar listener");

        let app = Router::new();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server_state = state.clone();

        let server_task = tokio::spawn(async move {
            serve_with_graceful_shutdown(
                listener,
                app,
                async move {
                    let _ = shutdown_rx.await;
                },
                Duration::from_millis(500),
                Some(server_state),
            )
            .await
        });

        shutdown_tx.send(()).expect("envia sinal");
        let server_res = server_task.await.expect("join deve ter sucesso");
        assert!(server_res.is_ok());
        assert!(
            state.pg_database.pool.is_closed(),
            "o pool PostgreSQL deve ser fechado ordenadamente no encerramento"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_orderly_resource_release_order_on_forced_shutdown() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .expect("cria pool lazy");

        let state = test_state(pool);
        let pool_handle = state.pg_database.pool.clone();
        assert!(!pool_handle.is_closed());

        let events = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let (started_tx, started_rx) = oneshot::channel();
        let started_tx = Arc::new(Mutex::new(Some(started_tx)));

        let events_handler = Arc::clone(&events);
        struct ForcedHandlerGuard(Arc<Mutex<Vec<&'static str>>>);
        impl Drop for ForcedHandlerGuard {
            fn drop(&mut self) {
                if let Ok(mut evts) = self.0.try_lock() {
                    evts.push("handler_cancelled");
                }
            }
        }

        let app = Router::new().route(
            "/hang",
            get({
                let started_tx = Arc::clone(&started_tx);
                let events_handler = Arc::clone(&events_handler);
                move || {
                    let started_tx = Arc::clone(&started_tx);
                    let events_handler = Arc::clone(&events_handler);
                    async move {
                        let _guard = ForcedHandlerGuard(events_handler);
                        if let Some(tx) = started_tx.lock().await.take() {
                            let _ = tx.send(());
                        }
                        sleep(Duration::from_millis(5000)).await;
                        "ok"
                    }
                }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("deve criar listener");
        let addr = listener.local_addr().expect("obter endereço");

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let events_server = Arc::clone(&events);
        let server_state = state.clone();

        let server_task = tokio::spawn(async move {
            let res = serve_with_graceful_shutdown(
                listener,
                app,
                async move {
                    let _ = shutdown_rx.await;
                },
                Duration::from_millis(100),
                Some(server_state),
            )
            .await;
            events_server.lock().await.push("server_stopped");
            res
        });

        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let request_task =
            tokio::spawn(async move { client.get(format!("http://{addr}/hang")).send().await });

        started_rx.await.expect("handler deve ter iniciado");
        shutdown_tx.send(()).expect("envia sinal de shutdown");

        let _ = request_task.await;
        let server_res = server_task.await.expect("join server deve ter sucesso");
        assert!(server_res.is_ok());

        assert!(
            pool_handle.is_closed(),
            "o pool de banco deve estar fechado ao término do shutdown forçado"
        );

        let recorded_events = events.lock().await.clone();
        assert_eq!(
            recorded_events,
            vec!["handler_cancelled", "server_stopped"],
            "o handler lento deve ser cancelado e finalizado antes do encerramento do pool"
        );
    }
}
