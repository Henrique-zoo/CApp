//! Recursos fornecidos aos handlers por meio do extrator `State` do Axum.
//!
//! O chamador constrói o adaptador PostgreSQL e os clientes externos antes de
//! compor o router. Clones do estado compartilham conexões e o cache de chaves
//! Firebase; não contêm identidade ou tokens de uma requisição específica.
//! Nos testes, o adaptador PostgreSQL usa o banco associado ao cenário.

use crate::{
    infrastructure::database::PgDatabase,
    modules::identity::infrastructure::{
        firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient,
    },
};

/// Clientes externos reutilizados pelos casos de uso de identidade.
///
/// O verificador Firebase compartilha seu cache entre clones; o cliente Microsoft
/// compartilha o pool HTTP e recebe o access token em cada operação. Os clientes
/// podem ser construídos com clones de um único [`reqwest::Client`].
#[derive(Clone)]
pub struct Clients {
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
    ///
    /// # Retorno
    ///
    /// Contêiner que assume a posse dos adaptadores. A operação não confirma
    /// permissões, disponibilidade dos provedores ou existência do projeto.
    pub fn new(firebase: FirebaseTokenVerifier, microsoft: MicrosoftGraphClient) -> Self {
        Self {
            firebase,
            microsoft,
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
    /// let state = AppState::new(
    ///     PgDatabase::new(pool.clone()),
    ///     Clients::new(firebase, microsoft),
    /// );
    /// let _another_handle = state.clone();
    /// pool.close().await; // Fecha o pool compartilhado com os dois estados.
    /// assert!(pool.is_closed());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pg_database: PgDatabase, clients: Clients) -> Self {
        Self {
            pg_database,
            clients,
        }
    }
}
