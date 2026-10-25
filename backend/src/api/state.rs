//! Recursos fornecidos aos handlers por meio do extrator `State` do Axum.
//!
//! O pool é criado por quem inicializa a aplicação; este módulo somente o
//! transporta entre os handlers e gerencia a sinalização atômica de ciclo de
//! vida e encerramento controlado da aplicação.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use sqlx::PgPool;

/// Estado compartilhado entre as requisições atendidas pelo router.
///
/// Clonar o estado clona o handle de [`PgPool`], compartilhando o mesmo conjunto
/// de conexões, e compartilha a referência atômica para o indicador de encerramento
/// em andamento. Não cria um pool ou uma conexão por requisição. A configuração
/// e o encerramento do pool cabem ao chamador; fechar um clone afeta os demais.
#[derive(Clone)]
pub struct AppState {
    /// Pool PostgreSQL usado pelos handlers.
    ///
    /// Pode ser acessado diretamente para executar consultas ou iniciar transações.
    /// A criação deste estado não verifica a conectividade do pool.
    pub pool: PgPool,

    /// Indicador atômico de que o processo de encerramento gracioso foi iniciado.
    is_shutting_down: Arc<AtomicBool>,
}

impl AppState {
    /// Constrói o estado assumindo a posse de um handle do pool e inicializando
    /// o indicador de prontidão de encerramento como inativo.
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
    /// assert!(!state.is_shutting_down());
    /// state.mark_shutting_down();
    /// assert!(another_handle.is_shutting_down());
    /// state.pool.close().await;
    /// assert!(another_handle.pool.is_closed());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Informa se o servidor iniciou o encerramento gracioso e está drenando requisições.
    ///
    /// Quando `true`, indica aos handlers de prontidão (como `/ready`) que a aplicação
    /// não deve aceitar novas requisições de balanceadores de carga ou orquestradores.
    ///
    /// # Retorno
    ///
    /// `true` se [`AppState::mark_shutting_down`] já tiver sido invocado; `false` caso contrário.
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Relaxed)
    }

    /// Sinaliza que o processo de encerramento gracioso foi iniciado.
    ///
    /// Essa alteração é visível imediatamente para todas as instâncias e clones deste [`AppState`].
    pub fn mark_shutting_down(&self) {
        self.is_shutting_down.store(true, Ordering::Relaxed);
    }
}
