//! Rotas globais de saúde e prontidão do backend.
//!
//! A composição é independente do transporte: o chamador pode servir o router
//! com Axum ou enviar requisições em memória. Nenhuma conexão é aberta ao
//! construir o router; o acesso ao PostgreSQL ocorre no handler de prontidão.

use std::time::Duration;

use axum::{Router, extract::State, routing::get};
use reqwest::StatusCode;
use tokio::time::timeout;

use crate::api::state::AppState;

/// Monta as rotas globais, ainda dependentes de [`AppState`].
///
/// # Rotas
///
/// | Método e caminho | Comportamento |
/// | --- | --- |
/// | `GET /health` | Retorna HTTP 200 e `O servidor está saudável.` sem consultar dependências. |
/// | `GET /ready` | Executa `SELECT 1 AS ok` no pool; retorna HTTP 200 no sucesso e HTTP 503 em erro ou timeout. |
///
/// A prontidão limita a espera pela aquisição da conexão e execução da consulta
/// a dois segundos. Sua resposta não contém corpo; falhas são registradas em
/// `tracing` no nível `warn`, sem devolver o diagnóstico SQL ao cliente.
/// A consulta verifica comunicação com o banco, não a existência das tabelas,
/// a versão do esquema nem a disponibilidade das demais integrações.
///
/// # Retorno
///
/// Um [`Router<AppState>`] que deve receber o estado por [`Router::with_state`]
/// antes de atender requisições. Não inicia servidor, não aplica migrations e
/// não instala autenticação ou outros middlewares.
///
/// # Exemplos
///
/// Uma verificação de saúde em memória dispensa um PostgreSQL em execução:
///
/// ```
/// use axum::{body::Body, http::{Request, StatusCode}};
/// use backend::api::{router::create_router, state::AppState};
/// use sqlx::postgres::PgPoolOptions;
/// use tower::ServiceExt;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPoolOptions::new()
///     .connect_lazy("postgres://capp:capp@localhost:5432/capp")?;
/// let app = create_router().with_state(AppState::new(pool.clone()));
/// let response = app
///     .oneshot(Request::builder().uri("/health").body(Body::empty())?)
///     .await?;
/// assert_eq!(response.status(), StatusCode::OK);
/// pool.close().await;
/// # Ok(())
/// # }
/// ```
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(|| async { "O servidor está saudável." }))
        .route(
            "/ready",
            get(|State(state): State<AppState>| async move {
                let result = timeout(
                    Duration::from_secs(2),
                    sqlx::query("SELECT 1 AS ok").fetch_one(&state.pool),
                )
                .await;

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
            }),
        )
}
