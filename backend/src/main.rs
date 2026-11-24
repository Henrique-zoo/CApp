//! Inicialização do processo HTTPS do backend CApp.
//!
//! # Configuração
//!
//! - `DATABASE_URL`: URL de conexão PostgreSQL aceita pelo SQLx. É obrigatória.
//! - `FIREBASE_PROJECT_ID`: obrigatória; projeto esperado nos ID tokens, não vazio
//!   e sem espaços em branco. Não é verificada a existência externa do projeto.
//! - `FIREBASE_PROJECT_NUMBER`: número do projeto usado no emissor/audiência App Check.
//! - `FIREBASE_APP_CHECK_APP_IDS`: App IDs permitidos, separados por vírgula.
//! - `BACKEND_TLS_CERT_FILE` e `BACKEND_TLS_KEY_FILE`: caminhos obrigatórios
//!   dos arquivos PEM do servidor HTTPS.
//! - `ELECTION_SIGNING_KEY_FILE`: arquivo PEM PKCS#8 da chave RSA da eleição.
//!   Quando ausente, o servidor atende as rotas de leitura e cadastro e o envio de votos retorna 503.
//! - `RUST_LOG`: filtro opcional do `tracing`. O padrão é `info`.
//! - Endereço HTTPS: `0.0.0.0:3000`, fixo nesta implementação.
//!
//! O ambiente do processo tem precedência sobre o `.env` que dotenvy procura
//! no diretório de execução e nos ancestrais. O processo aplica as migrations
//! embutidas após conectar ao banco e antes de abrir a porta HTTPS.
//!
//! Os adaptadores Firebase e Graph compartilham um cliente HTTP com timeout de
//! conexão de três segundos, timeout total de dez segundos e redirecionamentos
//! desabilitados. A construção desses adaptadores não executa I/O.

use std::{env, net::TcpListener, sync::Arc};

use anyhow::{Context, ensure};
use backend::{
    api::{
        self,
        state::{AppState, Clients},
    },
    infrastructure::{
        database::{MIGRATOR, PgDatabase},
        tls,
    },
    modules::elections::{crypto_wire::ServerSigner, infrastructure::start_election_clock},
    modules::identity::infrastructure::{
        firebase::{AppCheckVerifier, FirebaseTokenVerifier},
        microsoft::MicrosoftGraphClient,
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::postgres::PgPoolOptions;

/// Inicializa logs, TLS, clientes externos, PostgreSQL, migrations e a API.
///
/// O pool usa as opções padrão de [`PgPoolOptions`]. A conexão inicial precisa
/// funcionar e as migrations devem concluir antes da abertura da porta HTTPS,
/// inclusive para servir `/health`. O banco deve ter sido criado previamente.
/// O servidor permanece aguardando requisições enquanto a operação `serve`
/// estiver ativa.
///
/// # Erros
///
/// Propaga erros na configuração TLS, em `DATABASE_URL` ou `FIREBASE_PROJECT_ID`,
/// falhas ao construir o cliente HTTP e o verificador Firebase, erros ao conectar ao
/// PostgreSQL, ao validar ou aplicar migrations, ao abrir o socket TCP e os
/// erros retornados pelo servidor HTTPS.
///
/// # Panics
///
/// A instalação do subscriber global pode falhar com panic se outro subscriber
/// já tiver sido instalado no mesmo processo.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    api::logging::init_logging()?;

    let server_config = backend::config::ServerConfig::from_env()?;
    let https_address = &server_config.bind_address;
    let config = backend::config::Config::from_env().context("Configuração inválida")?;
    let app_config = backend::config::AppConfig::from_env()?;
    let database_url = app_config.database_url().expose_secret();

    let http = config
        .http
        .build_client()
        .context("Não foi possível construir o cliente HTTP")?;

    let firebase = FirebaseTokenVerifier::with_config(
        env::var("FIREBASE_PROJECT_ID")
            .context("FIREBASE_PROJECT_ID não está definida ou é inválida")?,
        http.clone(),
        config.http.clone(),
    )?;

    let microsoft = MicrosoftGraphClient::with_config(http.clone(), config.http.clone());

    let app_check = AppCheckVerifier::with_config(
        env::var("FIREBASE_PROJECT_NUMBER")
            .context("FIREBASE_PROJECT_NUMBER não está definida ou é inválida")?,
        env::var("FIREBASE_APP_CHECK_APP_IDS")
            .context("FIREBASE_APP_CHECK_APP_IDS não está definida ou é inválida")?
            .split(',')
            .map(|id| id.trim().to_owned())
            .collect(),
        http.clone(),
        config.http.clone(),
    )?;

    let pool = PgPoolOptions::new()
        .connect(database_url)
        .await
        .map_err(|error| {
            anyhow::anyhow!(
                "Não foi possível conectar ao PostgreSQL: {}",
                backend::config::sanitize_connection_error(&error)
            )
        })?;

    let (tls_config, tls_public_key) = tls::load_from_env()?;
    let election_signer = load_election_signer(&tls_public_key).await?;

    tracing::info!("Aplicando migrations do banco de dados");

    MIGRATOR
        .run(&pool)
        .await
        .context("Não foi possível aplicar as migrations do banco de dados")?;

    tracing::info!("Migrations do banco de dados atualizadas");

    let state = AppState::new(PgDatabase::new(pool), Clients::new(firebase, microsoft));

    let listener = TcpListener::bind(https_address)
        .with_context(|| format!("Não foi possível abrir o socket HTTPS em {https_address}"))?;

    let server = axum_server::from_tcp_rustls(listener, tls_config)
        .context("Não foi possível configurar a escuta HTTPS")?;

    let (state, election_clock) = match election_signer {
        Some(signer) => {
            let signer = Arc::new(signer);
            let clock = start_election_clock(state.pg_database.clone(), Arc::clone(&signer));
            (state.with_election_signer(signer), Some(clock))
        }
        None => (state, None),
    };

    tracing::info!(address = https_address, "Backend HTTPS ouvindo");

    let handle = axum_server::Handle::new();
    let shutdown_state = state.clone();
    let shutdown_handle = handle.clone();
    let timeout = server_config.shutdown_timeout;
    let shutdown = tokio::spawn(async move {
        api::shutdown::wait_for_shutdown_signal().await;
        shutdown_state.mark_shutting_down();
        shutdown_handle.graceful_shutdown(Some(timeout));
    });
    let pg_pool = state.pg_database.pool().clone();
    let result = server
        .handle(handle)
        .serve(api::router::create_router(state, app_check).into_make_service())
        .await;

    if let Some(clock) = election_clock {
        clock.abort();
    }

    shutdown.abort();
    pg_pool.close().await;
    result.context("Falha no servidor HTTPS")
}

/// Carrega a chave eleitoral opcional e verifica que ela difere da chave TLS.
///
/// Se `ELECTION_SIGNING_KEY_FILE` foi configurada, o arquivo deve conter uma
/// chave RSA privada em PEM PKCS#8. Um arquivo ilegível, uma chave inválida ou
/// a reutilização da chave TLS impedem a inicialização.
async fn load_election_signer(tls_public_key_der: &[u8]) -> anyhow::Result<Option<ServerSigner>> {
    let Some(path) = env::var_os("ELECTION_SIGNING_KEY_FILE") else {
        return Ok(None);
    };

    let pem = tokio::fs::read_to_string(path)
        .await
        .context("Não foi possível ler ELECTION_SIGNING_KEY_FILE")?;

    let signer = ServerSigner::from_pkcs8_pem(&pem).context(
        "ELECTION_SIGNING_KEY_FILE exige chave RSA privada PEM PKCS#8 do protocolo eleitoral",
    )?;

    ensure!(
        signer.public_key_base64()? != URL_SAFE_NO_PAD.encode(tls_public_key_der),
        "BACKEND_TLS_KEY_FILE e ELECTION_SIGNING_KEY_FILE devem usar chaves distintas"
    );

    Ok(Some(signer))
}
