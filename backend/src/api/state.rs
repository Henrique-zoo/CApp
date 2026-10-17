//! Recursos fornecidos aos handlers por meio do extrator `State` do Axum.
//!
//! O pool é criado por quem inicializa a aplicação; este módulo somente o
//! transporta entre os handlers. Nos testes, o pool pertence ao banco do cenário.

use std::sync::Arc;

use crate::{
    infrastructure::firebase::FirebaseIdentityTokenVerifier,
    modules::identity::application::queries::current_identity::IdentityTokenVerifier,
};
use sqlx::PgPool;

/// Estado compartilhado entre as requisições atendidas pelo router.
///
/// Clonar o estado clona o handle de [`PgPool`], compartilhando o mesmo conjunto
/// de conexões. Não cria um pool ou uma conexão por requisição. A configuração
/// e o encerramento do pool cabem ao chamador; fechar um clone afeta os demais.
#[derive(Clone)]
pub struct AppState {
    /// Pool PostgreSQL usado pelos handlers.
    ///
    /// Pode ser acessado diretamente para executar consultas ou iniciar transações.
    /// A criação deste estado não verifica a conectividade do pool.
    pub pool: PgPool,

    /// Adaptador usado para validar tokens e recuperar a identidade autenticada.
    pub(crate) identity_token_verifier: Arc<dyn IdentityTokenVerifier>,
}

impl AppState {
    /// Constrói o estado assumindo a posse de um handle do pool.
    ///
    /// A operação não realiza consultas, não abre conexões nem aplica migrations.
    /// O chamador pode fornecer um pool já conectado ou um pool de conexão tardia;
    /// a disponibilidade efetiva é verificada ao executar consultas.
    ///
    /// # Exemplos
    ///
    /// O exemplo é compilado, mas não executado, pois depende de PostgreSQL:
    ///
    /// ```no_run
    /// use backend::api::state::AppState;
    /// use sqlx::postgres::PgPoolOptions;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), sqlx::Error> {
    /// let pool = PgPoolOptions::new()
    ///     .max_connections(5)
    ///     .connect("postgres://capp:capp@localhost:5432/capp")
    ///     .await?;
    /// let state = AppState::new(pool);
    /// let another_handle = state.clone();
    /// state.pool.close().await;
    /// assert!(another_handle.pool.is_closed());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            identity_token_verifier: Arc::new(FirebaseIdentityTokenVerifier),
        }
    }

    /// Constrói o estado com um verificador de identidade fornecido pelo chamador.
    ///
    /// Permite substituir o provedor concreto, por exemplo, em testes que precisam
    /// simular identidades válidas sem acessar serviços externos.
    pub fn with_identity_token_verifier(
        pool: PgPool,
        identity_token_verifier: Arc<dyn IdentityTokenVerifier>,
    ) -> Self {
        Self {
            pool,
            identity_token_verifier,
        }
    }
}
