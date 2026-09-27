//! Inicialização do processo HTTP do backend CApp.
//!
//! # Configuração
//!
//! - `DATABASE_URL`: obrigatória; URL de conexão PostgreSQL aceita pelo SQLx.
//! - `RUST_LOG`: filtro opcional do `tracing`; ausente ou inválido usa `info`.
//! - Endereço HTTP: `0.0.0.0:3000`, fixo nesta implementação.
//!
//! As variáveis são lidas do ambiente do processo. O executável não carrega
//! arquivos `.env`. As migrations embutidas são aplicadas após conectar ao banco
//! e antes da abertura da porta HTTP; uma falha impede a inicialização da API.
//!
//! O subscriber encaminha os eventos de diagnóstico para a saída padrão.
//! Após inicializá-lo, o processo conecta ao banco, aplica migrations e serve o
//! router da biblioteca. Não há tratamento próprio de sinais ou configuração
//! de encerramento gracioso nesta inicialização.

use std::env::{self, VarError};

use anyhow::Context;
use backend::{
    api::{self, state::AppState},
    infrastructure::database::MIGRATOR,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

/// Inicializa logs, pool PostgreSQL, migrations e servidor HTTP no runtime Tokio.
///
/// O pool usa as opções padrão de [`PgPoolOptions`]. A conexão inicial precisa
/// funcionar e as migrations devem concluir antes da abertura da porta HTTP,
/// inclusive para servir `/health`. O banco deve ter sido criado previamente.
/// O servidor permanece aguardando requisições enquanto a operação `serve`
/// estiver ativa.
///
/// # Erros
///
/// Propaga a ausência ou invalidade de `DATABASE_URL`, erros ao conectar ao
/// PostgreSQL, ao validar ou aplicar migrations, ao abrir o socket TCP e os
/// erros retornados por `axum::serve`.
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("Não foi possível criar o socket TCP")?;

    let service = api::router::create_router().with_state(AppState::new(pool));

    axum::serve(listener, service).await?;

    Ok(())
}
