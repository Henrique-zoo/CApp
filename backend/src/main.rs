//! Inicialização do processo HTTP do backend CApp.
//!
//! # Configuração
//!
//! As configurações são centralizadas e validadas por [`backend::config::AppConfig`]:
//! - `DATABASE_URL`: obrigatória; URL confidencial de conexão PostgreSQL aceita pelo SQLx.
//! - `FIREBASE_PROJECT_ID`: obrigatória; projeto esperado nos ID tokens, não vazio
//!   e sem espaços em branco. Não é verificada a existência externa do projeto.
//! - `RUST_LOG`: filtro opcional do `tracing`; ausente ou inválido usa `info`.
//! - `SERVER_HOST`: endereço de escuta do listener HTTP (padrão: `0.0.0.0`).
//! - `BACKEND_PORT` ou `SERVER_PORT`: porta TCP do listener HTTP (padrão: `3000`).
//! - `APP_ENV`: ambiente de execução (`development`, `staging`, `production`).
//!
//! As variáveis são lidas do ambiente do processo com diagnóstico seguro: credenciais
//! e segredos nunca são expostos em mensagens de erro ou logs de inicialização.
//! As migrations embutidas são aplicadas após conectar ao banco e antes da abertura
//! da porta HTTP; uma falha impede a inicialização da API.
//!
//! O subscriber encaminha os eventos de diagnóstico para a saída padrão.
//! Após inicializá-lo, o processo configura um cliente HTTP compartilhado, com
//! timeout de conexão de três segundos, total de dez e sem redirecionamentos.
//! Os adaptadores Firebase e Graph não executam I/O nessa construção. Depois,
//! o processo conecta ao banco, aplica migrations e serve o
//! router da biblioteca. Não há tratamento próprio de sinais ou configuração
//! de encerramento gracioso nesta inicialização.

use std::{env, time::Duration};

use anyhow::Context;
use backend::{
    api::{
        self,
        state::{AppState, Clients},
    },
    config::AppConfig,
    infrastructure::database::{MIGRATOR, PgDatabase},
    modules::identity::infrastructure::{
        firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient,
    },
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

/// Inicializa logs, clientes externos, PostgreSQL, migrations e servidor HTTP.
///
/// O pool usa as opções padrão de [`PgPoolOptions`]. A conexão inicial precisa
/// funcionar e as migrations devem concluir antes da abertura da porta HTTP,
/// inclusive para servir `/health`. O banco deve ter sido criado previamente.
/// O servidor permanece aguardando requisições enquanto a operação `serve`
/// estiver ativa.
///
/// # Erros
///
/// Propaga falhas de validação da configuração, ausência ou invalidade de
/// `FIREBASE_PROJECT_ID`, falhas ao construir os clientes e erros ao conectar ao
/// PostgreSQL, ao validar ou aplicar migrations, ao abrir o socket TCP e os
/// erros retornados por `axum::serve`.
///
/// # Panics
///
/// A instalação do subscriber global pode falhar com panic se outro subscriber
/// já tiver sido instalado no mesmo processo.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = AppConfig::from_env()
        .context("Falha na inicialização segura das configurações do backend")?;

    tracing::info!("Ambiente de execução: {}", config.environment());

    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Não foi possível construir o cliente HTTP")?;

    let firebase = FirebaseTokenVerifier::new(
        env::var("FIREBASE_PROJECT_ID")
            .context("FIREBASE_PROJECT_ID não está definida ou é inválida")?,
        http.clone(),
    )?;

    let microsoft = MicrosoftGraphClient::new(http.clone());

    let clients = Clients::new(firebase, microsoft);

    let pool = PgPoolOptions::new()
        .connect(config.database_url().expose_secret())
        .await
        .context("Failed to connect to DB.")?;

    tracing::info!("Aplicando migrations do banco de dados");
    MIGRATOR
        .run(&pool)
        .await
        .context("Não foi possível aplicar as migrations do banco de dados")?;
    tracing::info!("Migrations do banco de dados atualizadas");

    let pg_database = PgDatabase::new(pool);

    let bind_addr = format!("{}:{}", config.server_host(), config.server_port());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .context("Não foi possível criar o socket TCP")?;

    tracing::info!("Servidor HTTP ouvindo em {}", bind_addr);

    let service = api::router::create_router().with_state(AppState::new(pg_database, clients));

    axum::serve(listener, service).await?;

    Ok(())
}
