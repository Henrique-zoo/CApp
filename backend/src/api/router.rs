//! Composição das rotas de saúde, prontidão e verificação institucional.
//!
//! O chamador pode servir o router com Axum ou enviar requisições em memória.
//! Construir o router não abre conexões. `/ready` consulta PostgreSQL; a rota de
//! verificação consulta também os adaptadores externos e persiste a evidência.

use std::time::Duration;

use axum::{Router, extract::State, routing::get};
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
/// não instala proteção global. A rota de verificação interpreta suas próprias
/// credenciais; o extractor de evidência persistida só protege handlers que o
/// recebam explicitamente, e nenhuma rota atual o recebe.
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
/// let state = AppState::new(
///     PgDatabase::new(pool.clone()), Clients::new(firebase, microsoft),
/// );
/// let app = create_router().with_state(state);
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
        .merge(crate::modules::identity::api::router())
        .route("/health", get(|| async { "O servidor está saudável." }))
        .route(
            "/ready",
            get(|State(state): State<AppState>| async move {
                let result = timeout(
                    Duration::from_secs(2),
                    sqlx::query("SELECT 1 AS ok").fetch_one(&state.pg_database.pool),
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
