//! Recursos fornecidos aos handlers por meio do extrator `State` do Axum.
//!
//! O chamador constrói o adaptador PostgreSQL e os clientes externos antes de
//! compor o router. Clones do estado compartilham conexões e o cache de chaves
//! Firebase; não contêm identidade ou tokens de uma requisição específica.
//! Nos testes, o adaptador PostgreSQL usa o banco associado ao cenário.

use std::sync::Arc;

use crate::{
    infrastructure::database::PgDatabase,
    modules::{
        elections::{
            crypto_wire::ServerSigner,
            infrastructure::{
                PgElectionBoardRepository, PgVoterRegistrationRepository, RsaPssBallotVerifier,
                RsaPssVoterKeyProofVerifier,
            },
        },
        identity::infrastructure::{
            firebase::{AppCheckVerifier, FirebaseTokenVerifier},
            microsoft::MicrosoftGraphClient,
        },
    },
};

/// Clientes de identidade reutilizados pela API e pelos casos de uso.
///
/// O verificador Firebase compartilha seu cache entre clones; o cliente Microsoft
/// compartilha o pool HTTP e recebe o access token em cada operação. Os clientes
/// podem ser construídos com clones de um único [`reqwest::Client`].
#[derive(Clone)]
pub struct Clients {
    /// Verificador da origem do aplicativo, independente do Firebase UID.
    pub app_check: AppCheckVerifier,

    /// Verificador de Firebase ID tokens para o projeto configurado.
    pub firebase: FirebaseTokenVerifier,

    /// Adaptador de perfil, tenant e participação em grupos do Microsoft Graph.
    pub microsoft: MicrosoftGraphClient,
}

impl Clients {
    /// Agrupa adaptadores já configurados, sem executar requisições externas.
    ///
    /// # Parâmetros
    ///
    /// - `firebase`: verificador cuja audiência e emissor correspondem ao projeto.
    /// - `microsoft`: cliente do Graph; seu cliente HTTP deve recusar redirecionamentos.
    /// - `app_check`: verificador do número do projeto e dos App IDs permitidos.
    ///
    /// # Retorno
    ///
    /// Contêiner que assume a posse dos adaptadores. A operação não confirma
    /// permissões, disponibilidade dos provedores ou existência do projeto.
    pub fn new(
        firebase: FirebaseTokenVerifier,
        microsoft: MicrosoftGraphClient,
        app_check: AppCheckVerifier,
    ) -> Self {
        Self {
            firebase,
            microsoft,
            app_check,
        }
    }
}

/// Estado compartilhado entre as requisições atendidas pelo router.
///
/// Clonar o estado compartilha o [`sqlx::PgPool`], os clientes HTTP e o cache
/// Firebase. Não cria um pool por requisição. A configuração e o encerramento
/// dos recursos cabem ao chamador; fechar o pool original afeta seus clones.
/// As identidades autenticadas pertencem às requisições, não a este estado.
#[derive(Clone)]
pub struct AppState {
    /// Adaptador PostgreSQL que implementa os contratos de repositório.
    ///
    /// Serviços recebem referências às traits implementadas por esse adaptador.
    /// Construir o estado não verifica a conectividade nem aplica migrations.
    pub pg_database: PgDatabase,

    /// Clientes externos compartilhados, sem credenciais de usuários armazenadas.
    pub clients: Clients,

    /// Adaptador do cadastro da chave eleitoral que reutiliza o pool de `pg_database`.
    pub voter_registration_repository: PgVoterRegistrationRepository,

    /// Verificador usado pelo serviço de aceite das cédulas.
    pub(crate) ballot_verifier: RsaPssBallotVerifier,

    /// Verificador tipado de prova de posse de chave para o cadastro eleitoral.
    pub(crate) voter_key_proof_verifier: RsaPssVoterKeyProofVerifier,

    /// Persistência dos votos, usando o pool compartilhado.
    pub(crate) vote_repository: PgElectionBoardRepository,

    /// A instalação fornece a chave para habilitar a assinatura de recibos.
    pub(crate) election_signer: Option<Arc<ServerSigner>>,
}

impl AppState {
    /// Constrói o estado com os adaptadores preparados pelo chamador.
    ///
    /// A operação não realiza consultas, não abre conexões nem aplica migrations.
    /// O adaptador pode encapsular um pool conectado ou de conexão tardia.
    /// A disponibilidade do banco e dos provedores só é avaliada nas operações.
    ///
    /// # Parâmetros
    ///
    /// - `pg_database`: adaptador vinculado ao banco do ambiente ou do cenário.
    /// - `clients`: verificadores e clientes externos previamente configurados.
    ///
    /// # Retorno
    ///
    /// Estado pronto para ser passado a [`axum::Router::with_state`].
    ///
    /// # Exemplos
    ///
    /// A conexão tardia permite construir e clonar o estado sem acessar a rede:
    ///
    /// ```
    /// use backend::{
    ///     api::state::{AppState, Clients},
    ///     infrastructure::database::PgDatabase,
    ///     modules::identity::infrastructure::{
    ///         firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient,
    ///     },
    /// };
    /// use sqlx::postgres::PgPoolOptions;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let pool = PgPoolOptions::new()
    ///     .max_connections(5)
    ///     .connect_lazy("postgres://capp:capp@localhost:5432/capp")?;
    /// let http = reqwest::Client::builder()
    ///     .redirect(reqwest::redirect::Policy::none())
    ///     .build()?;
    /// let firebase = FirebaseTokenVerifier::new("capp-example".into(), http.clone())?;
    /// let microsoft = MicrosoftGraphClient::new(http);
    /// let app_check = backend::modules::identity::infrastructure::firebase::AppCheckVerifier::new(
    ///     "1234567890".into(), vec!["1:1234567890:android:capp".into()], reqwest::Client::new())?;
    /// let state = AppState::new(
    ///     PgDatabase::new(pool.clone()),
    ///     Clients::new(firebase, microsoft, app_check),
    /// );
    /// let _another_handle = state.clone();
    /// pool.close().await; // Fecha o pool compartilhado com os dois estados.
    /// assert!(pool.is_closed());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pg_database: PgDatabase, clients: Clients) -> Self {
        let voter_registration_repository =
            PgVoterRegistrationRepository::new(pg_database.pool.clone());

        let vote_repository = PgElectionBoardRepository::new(pg_database.pool.clone());

        Self {
            pg_database,
            clients,
            voter_registration_repository,
            ballot_verifier: RsaPssBallotVerifier,
            voter_key_proof_verifier: RsaPssVoterKeyProofVerifier,
            vote_repository,
            election_signer: None,
        }
    }

    /// Habilita o aceite transacional de cédulas usando a chave prevista no manifesto.
    pub fn with_election_signer(mut self, signer: Arc<ServerSigner>) -> Self {
        self.election_signer = Some(signer);
        self
    }

    /// Inicia o relógio que encerra eleições publicadas quando o prazo vence.
    /// O processo encerra a tarefa junto com o servidor.
    pub fn start_election_clock(&self) -> Option<tokio::task::JoinHandle<()>> {
        let signer = self.election_signer.clone()?;
        let boards = self.vote_repository.clone();
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                interval.tick().await;
                if let Err(error) =
                    crate::modules::elections::application::services::board::close_due(
                        &boards, &*signer,
                    )
                    .await
                {
                    tracing::error!(%error, "Falha ao consultar eleições a encerrar");
                }
                if let Err(error) = crate::modules::elections::application::services::tally::submit_pending_server_shares(
                    &boards, &boards, &*signer,
                ).await {
                    tracing::error!(%error, "Falha ao consultar contribuições do servidor");
                }
            }
        }))
    }
}
