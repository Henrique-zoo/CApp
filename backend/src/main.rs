//! Inicialização do processo HTTP do backend CApp.
//!
//! # Configuração
//!
//! - `DATABASE_URL`: obrigatória; URL de conexão PostgreSQL aceita pelo SQLx.
//! - `FIREBASE_PROJECT_ID`: obrigatória; projeto esperado nos ID tokens, não vazio
//!   e sem espaços em branco. Não é verificada a existência externa do projeto.
//! - `RUST_LOG`: filtro opcional do `tracing`; ausente ou inválido usa `info`.
//! - `BACKEND_BIND_ADDRESS`: endereço e porta HTTP, com padrão `0.0.0.0:3000`.
//! - `SHUTDOWN_TIMEOUT_SECS`: prazo de drenagem, com padrão de 15 segundos.
//!
//! As variáveis são lidas do ambiente do processo. O executável não carrega
//! arquivos `.env`. As migrations embutidas são aplicadas após conectar ao banco
//! e antes da abertura da porta HTTP; uma falha impede a inicialização da API.
//!
//! O subscriber encaminha os eventos de diagnóstico para a saída padrão.
//! Após inicializá-lo, o processo configura um cliente HTTP compartilhado, com
//! timeout de conexão de três segundos, total de dez e sem redirecionamentos.
//! Os adaptadores Firebase e Graph não executam I/O nessa construção. Depois,
//! o processo conecta ao banco, aplica migrations e serve o
//! router da biblioteca com tratamento de sinais, drenagem com prazo configurável
//! e fechamento ordenado do pool PostgreSQL.

use std::{
    env::{self, VarError},
    time::Duration,
};

use anyhow::Context;
use backend::{
    api::{
        self,
        state::{AppState, Clients},
    },
    config::ServerConfig,
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
/// O servidor trata sinais do sistema operacional, interrompe novos aceites,
/// sinaliza indisponibilidade em `/ready` e drena requisições em andamento com
/// o prazo configurado antes de fechar o pool PostgreSQL.
///
/// # Erros
///
/// Propaga a ausência ou invalidade de `DATABASE_URL` ou `FIREBASE_PROJECT_ID`,
/// configuração inválida de `SHUTDOWN_TIMEOUT_SECS`, falhas ao construir o cliente
/// HTTP e o verificador Firebase, erros ao conectar ao
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

    let server_config = ServerConfig::from_env().context("Configuração do servidor inválida")?;

    let db_url = env::var("DATABASE_URL").map_err(|e| match e {
        VarError::NotPresent => anyhow::anyhow!(e).context(
            "DATABASE_URL is not set or contains invalid characters ('=' or '\\0') in its name.",
        ),
        VarError::NotUnicode(_) => {
            anyhow::anyhow!(e).context("DATABASE_URL contains an invalid UTF-8 value.")
        }
    })?;

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
        .connect(&db_url)
        .await
        .context("Failed to connect to DB.")?;

    tracing::info!("Aplicando migrations do banco de dados");
    MIGRATOR
        .run(&pool)
        .await
        .context("Não foi possível aplicar as migrations do banco de dados")?;
    tracing::info!("Migrations do banco de dados atualizadas");

    let pg_database = PgDatabase::new(pool);

    let listener = tokio::net::TcpListener::bind(&server_config.bind_address)
        .await
        .context("Não foi possível criar o socket TCP")?;

    tracing::info!(
        address = %server_config.bind_address,
        shutdown_timeout_secs = server_config.shutdown_timeout.as_secs(),
        "Servidor HTTP iniciado e pronto para receber requisições"
    );

    let state = AppState::new(pg_database, clients);
    let service = api::router::create_router().with_state(state.clone());

    api::shutdown::serve_with_graceful_shutdown(
        listener,
        service,
        api::shutdown::wait_for_shutdown_signal(),
        server_config.shutdown_timeout,
        Some(state),
    )
    .await
    .context("Falha durante a execução ou encerramento do servidor HTTP")?;

    tracing::info!("Processo backend finalizado com sucesso.");

    Ok(())
}
