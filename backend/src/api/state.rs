//! Recursos fornecidos aos handlers por meio do extrator `State` do Axum.
//!
//! O pool é criado por quem inicializa a aplicação; este módulo somente o
//! transporta entre os handlers. Nos testes, o pool pertence ao banco do cenário.

use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    api::auth::HasAuthorizationService,
    modules::ca::application::{
        ports::AuthorizationServicePort, services::authorization_service::AuthorizationService,
    },
};

/// Estado compartilhado entre as requisições atendidas pelo router.
///
/// Clonar o estado clona o handle de [`PgPool`] e do serviço de autorização, compartilhando
/// as conexões e instâncias. Não cria conexões por requisição.
#[derive(Clone)]
pub struct AppState {
    /// Pool PostgreSQL usado pelos handlers.
    ///
    /// Pode ser acessado diretamente para executar consultas ou iniciar transações.
    /// A criação deste estado não verifica a conectividade do pool.
    pub pool: PgPool,

    /// Serviço de autorização e controle de permissões de Centros Acadêmicos.
    pub authorization: Arc<dyn AuthorizationServicePort>,
}

impl AppState {
    /// Constrói o estado assumindo a posse de um handle do pool e configurando a autorização padrão.
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
            authorization: Arc::new(AuthorizationService::with_in_memory()),
        }
    }

    /// Constrói o estado injetando explicitamente a porta do serviço de autorização.
    pub fn with_authorization(
        pool: PgPool,
        authorization: Arc<dyn AuthorizationServicePort>,
    ) -> Self {
        Self {
            pool,
            authorization,
        }
    }
}

impl HasAuthorizationService for AppState {
    fn authorization_service(&self) -> Arc<dyn AuthorizationServicePort> {
        Arc::clone(&self.authorization)
    }
}
