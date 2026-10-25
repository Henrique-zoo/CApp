//! Inicialização do processo HTTP do backend CApp.
//!
//! # Configuração
//!
//! - `DATABASE_URL`: obrigatória; URL de conexão PostgreSQL aceita pelo SQLx.
//! - `RUST_LOG`: filtro opcional do `tracing`; ausente ou inválido usa `info`.
//! - `BACKEND_BIND_ADDRESS`: opcional; endereço e porta para escuta HTTP (padrão: `0.0.0.0:3000`).
//! - `SHUTDOWN_TIMEOUT_SECS`: opcional; prazo máximo em segundos para drenagem de conexões (padrão: 15s).
//!
//! As variáveis são lidas do ambiente do processo. O executável não carrega
//! arquivos `.env`. As migrations embutidas são aplicadas após conectar ao banco
//! e antes da abertura da porta HTTP; uma falha impede a inicialização da API.
//!
//! O subscriber encaminha os eventos de diagnóstico para a saída padrão.
//! Após inicializá-lo, o processo conecta ao banco, aplica migrations e serve o
//! router da biblioteca com tratamento portável de sinais (`SIGINT`, `SIGTERM` e eventos
//! do Windows), drenagem de conexões ativas com prazo configurável e liberação
//! ordenada do pool PostgreSQL.

use std::env::{self, VarError};

use anyhow::Context;
use backend::{
    api::{self, state::AppState},
    config::ServerConfig,
    infrastructure::database::MIGRATOR,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

/// Inicializa logs, configurações, pool PostgreSQL, migrations e servidor HTTP no runtime Tokio.
///
/// O pool usa as opções padrão de [`PgPoolOptions`]. A conexão inicial precisa
/// funcionar e as migrations devem concluir antes da abertura da porta HTTP,
/// inclusive para servir `/health`. O banco deve ter sido criado previamente.
///
/// O servidor é iniciado com tratamento portável de sinais do sistema operacional
/// e encerramento controlado (*graceful shutdown*). Ao interceptar um sinal de término:
/// 1. Interrompe a recepção de novas conexões no socket TCP.
/// 2. Atualiza o estado da aplicação para que `/ready` responda HTTP 503.
/// 3. Drena conexões ativas até a sua conclusão ou até atingir o prazo limite configurado.
/// 4. Fecha de forma limpa e ordenada o pool de conexões do PostgreSQL.
///
/// # Erros
///
/// Propaga a ausência ou invalidade de `DATABASE_URL`, erros de sintaxe ou limites em
/// `SHUTDOWN_TIMEOUT_SECS`, erros ao conectar ao PostgreSQL, ao validar ou aplicar
/// migrations, ao abrir o socket TCP e os erros retornados durante a execução do servidor.
///
/// # Panics
///
/// A instalação do subscriber global pode falhar com panic se outro subscriber
/// já tiver sido instalado no mesmo processo.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let listener = tokio::net::TcpListener::bind(&server_config.bind_address)
        .await
        .context("Não foi possível criar o socket TCP")?;

    tracing::info!(
        address = %server_config.bind_address,
        shutdown_timeout_secs = server_config.shutdown_timeout.as_secs(),
        "Servidor HTTP iniciado e pronto para receber requisições"
    );

    let state = AppState::new(pool);
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
