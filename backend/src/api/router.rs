//! Composição das rotas de saúde, prontidão e verificação institucional.
//!
//! O chamador pode servir o router com Axum ou enviar requisições em memória.
//! Construir o router não abre conexões. `/ready` consulta PostgreSQL; a rota de
//! verificação consulta também os adaptadores externos e persiste a evidência.

use std::time::Duration;

use axum::{Router, extract::State, middleware, routing::get};
use reqwest::StatusCode;
use tokio::time::timeout;

use crate::api::state::AppState;

/// Monta as rotas globais e de identidade, ainda dependentes de [`AppState`].
///
/// # Rotas
///
/// | Método e caminho | Comportamento |
/// | --- | --- |
/// | `POST /auth/institutional/verify` | Valida as credenciais, persiste a confirmação e retorna perfil e validade. |
/// | `GET /health` | Retorna HTTP 200 e `O servidor está saudável.` sem consultar dependências. |
/// | `GET /ready` | Retorna HTTP 503 se o servidor estiver em drenagem de encerramento (inclusive se iniciado durante a verificação); caso contrário, executa `SELECT 1 AS ok` no pool retornando HTTP 200 no sucesso e HTTP 503 em erro ou timeout. |
///
/// A prontidão limita a espera pela aquisição da conexão e execução da consulta
/// a dois segundos. Sua resposta não contém corpo; falhas são registradas em
/// `tracing` no nível `warn`, sem devolver o diagnóstico SQL ao cliente.
/// A consulta verifica comunicação com o banco, não a existência das tabelas,
/// a versão do esquema nem a disponibilidade das demais integrações.
///
/// # Retorno
///
/// Router já associado a [AppState], com fiscalização App Check nas rotas de
/// negócio. GET/HEAD /health e /ready são públicos. Construir não inicia servidor,
/// aplica migrations ou consulta chaves remotas. Após a comprovação do aplicativo,
/// cada rota continua verificando a identidade Firebase e suas regras de acesso.
///
/// # Exemplos
///
/// Uma verificação de saúde em memória dispensa um PostgreSQL em execução:
///
/// ```
/// use axum::{body::Body, http::{Request, StatusCode}};
/// use backend::{
///     api::{router::create_router, state::{AppState, Clients}},
///     infrastructure::database::PgDatabase,
///     modules::identity::infrastructure::{
///         firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient,
///     },
/// };
/// use sqlx::postgres::PgPoolOptions;
/// use tower::ServiceExt;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let pool = PgPoolOptions::new()
///     .connect_lazy("postgres://capp:capp@localhost:5432/capp")?;
/// let http = reqwest::Client::builder()
///     .redirect(reqwest::redirect::Policy::none())
///     .build()?;
/// let firebase = FirebaseTokenVerifier::new("capp-example".into(), http.clone())?;
/// let microsoft = MicrosoftGraphClient::new(http);
/// let app_check = backend::modules::identity::infrastructure::firebase::AppCheckVerifier::new(
///     "1234567890".into(), vec!["1:1234567890:android:capp".into()], reqwest::Client::new())?;
/// let state = AppState::new(
///     PgDatabase::new(pool.clone()), Clients::new(firebase, microsoft, app_check),
/// );
/// let app = create_router(state);
/// let response = app
///     .oneshot(Request::builder().uri("/health").body(Body::empty())?)
///     .await?;
/// assert_eq!(response.status(), StatusCode::OK);
/// pool.close().await;
/// # Ok(())
/// # }
/// ```
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(crate::modules::identity::api::router())
        .route("/health", get(|| async { "O servidor está saudável." }))
        .route(
            "/ready",
            get(|State(state): State<AppState>| async move {
                check_readiness_with(&state, || {
                    sqlx::query("SELECT 1 AS ok").fetch_one(&state.pg_database.pool)
                })
                .await
            }),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            super::app_check::require_app_check,
        ))
        .with_state(state)
}

/// Executa a verificação de prontidão com uma rotina de consulta ao banco de dados.
///
/// Avalia se a aplicação está em processo de encerramento controlado antes e depois da consulta,
/// garantindo que qualquer transição para drenagem durante a espera resulte imediatamente
/// em `HTTP 503 Service Unavailable`.
pub async fn check_readiness_with<F, Fut, T, E>(state: &AppState, ping_db: F) -> StatusCode
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    if state.is_shutting_down() {
        tracing::warn!("Verificação de prontidão rejeitada: servidor em processo de encerramento.");
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    let result = timeout(Duration::from_secs(2), ping_db()).await;

    if state.is_shutting_down() {
        tracing::warn!(
            "Verificação de prontidão rejeitada: servidor entrou em encerramento durante a consulta."
        );
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    match result {
        Ok(Ok(_)) => StatusCode::OK,
        Ok(Err(error)) => {
            tracing::warn!(%error, "Falha ao verificar o PostgreSQL.");
            StatusCode::SERVICE_UNAVAILABLE
        }
        Err(error) => {
            tracing::warn!(%error, "Timeout ao verificar o PostgreSQL.");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::state::Clients,
        infrastructure::database::PgDatabase,
        modules::identity::infrastructure::{
            firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient,
        },
    };
    use sqlx::postgres::PgPoolOptions;
    use tokio::sync::oneshot;

    fn test_app_state() -> AppState {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .expect("cria pool lazy de teste");
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("cliente HTTP de teste");
        let firebase = FirebaseTokenVerifier::new("capp-test-project".into(), http.clone())
            .expect("verificador Firebase de teste");
        let app_check = crate::modules::identity::infrastructure::firebase::AppCheckVerifier::new(
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

    #[tokio::test]
    async fn test_readiness_returns_503_when_already_shutting_down() {
        let state = test_app_state();
        state.mark_shutting_down();

        let mut db_called = false;
        let status = check_readiness_with(&state, || async {
            db_called = true;
            Ok::<_, &'static str>("ok")
        })
        .await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(
            !db_called,
            "não deve consultar o banco se já estiver em shutdown"
        );
    }

    #[tokio::test]
    async fn test_readiness_returns_503_when_shutdown_occurs_during_db_query_race() {
        let state = test_app_state();
        assert!(!state.is_shutting_down());

        let (started_tx, started_rx) = oneshot::channel();
        let (continue_tx, continue_rx) = oneshot::channel();

        let state_clone = state.clone();
        let check_task = tokio::spawn(async move {
            check_readiness_with(&state_clone, || async move {
                let _ = started_tx.send(());
                let _ = continue_rx.await;
                Ok::<_, &'static str>("ok")
            })
            .await
        });

        // Aguarda a consulta ao banco iniciar de fato (passou da primeira verificação de shutdown)
        started_rx.await.expect("consulta deve ter iniciado");

        // Simula recebimento de sinal de encerramento enquanto a consulta ao banco está em voo
        state.mark_shutting_down();

        // Libera a consulta ao banco para concluir com sucesso
        let _ = continue_tx.send(());

        let status = check_task.await.expect("tarefa deve finalizar");
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "deve reavaliar a flag de shutdown e rejeitar com 503 mesmo se a query do banco retornar Ok"
        );
    }

    #[tokio::test]
    async fn test_readiness_returns_200_when_healthy_and_not_shutting_down() {
        let state = test_app_state();
        assert!(!state.is_shutting_down());

        let status = check_readiness_with(&state, || async { Ok::<_, &'static str>("ok") }).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_returns_503_on_db_error() {
        let state = test_app_state();
        assert!(!state.is_shutting_down());

        let status = check_readiness_with(&state, || async {
            Err::<&'static str, _>("falha de conexão simulada")
        })
        .await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_readiness_returns_503_on_db_timeout() {
        let state = test_app_state();
        assert!(!state.is_shutting_down());

        let status = check_readiness_with(&state, || async {
            tokio::time::sleep(Duration::from_millis(2500)).await;
            Ok::<_, &'static str>("ok")
        })
        .await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }
}
