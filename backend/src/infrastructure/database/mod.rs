//! Adaptador PostgreSQL e migrations usados pelo servidor e pelos testes.
//!
//! [`MIGRATOR`] incorpora `backend/migrations/` na compilação. O executável o aplica
//! antes de abrir a porta HTTP; a suíte BDD prepara a base-modelo antes da fixture.
//! [`PgDatabase`] encapsula o pool e recebe implementações de traits de repositório
//! nos módulos de negócio. Cada chamador administra seu próprio pool.

use sqlx::{PgPool, migrate::Migrator};

/// Migrations versionadas do backend, embutidas no binário.
///
/// A declaração não acessa o banco. O chamador deve executar [`Migrator::run`]
/// com o pool do ambiente que deseja preparar. A execução cria, se necessário,
/// a tabela de controle `_sqlx_migrations` e aplica somente versões pendentes.
/// Migrations já aplicadas têm seus checksums validados; alterações ou versões
/// ausentes são retornadas como [`sqlx::migrate::MigrateError`].
///
/// O lock de migrations do SQLx permanece habilitado para coordenar execuções
/// concorrentes no mesmo banco. Erros de conexão, permissão e execução SQL são
/// propagados ao chamador. O banco deve existir antes da chamada.
///
/// As migrations atuais criam `catalog_status`, `institution` e
/// `institutional_verification`, sem inserir uma instituição real ou usuário.
/// Novos arquivos exigem recompilar o binário; `build.rs` observa o diretório
/// para que o Cargo detecte essas adições. A fixture BDD é aplicada separadamente.
///
/// # Exemplos
///
/// A execução exige um pool conectado ao PostgreSQL:
///
/// ```no_run
/// use backend::infrastructure::database::MIGRATOR;
///
/// # async fn prepare(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
/// MIGRATOR.run(pool).await?;
/// # Ok(())
/// # }
/// ```
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Acesso PostgreSQL compartilhado pelas implementações de repositório.
///
/// Clonar este tipo compartilha o mesmo pool de conexões.
#[derive(Debug, Clone)]
pub struct PgDatabase {
    /// Pool utilizado pelos adaptadores de persistência do backend.
    pub(crate) pool: PgPool,
}

impl PgDatabase {
    /// Encapsula um pool existente sem abrir conexões nem aplicar migrations.
    ///
    /// # Parâmetros
    ///
    /// - `pool`: handle SQLx conectado ou de conexão tardia, preparado pelo chamador.
    ///
    /// # Retorno
    ///
    /// Adaptador pronto para receber chamadas pelas traits dos módulos. Clonar o
    /// adaptador compartilha o pool; fechar qualquer handle afeta os demais. O tipo não
    /// cria transações, carrega dados ou verifica disponibilidade na construção.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use backend::infrastructure::database::PgDatabase;
    /// use sqlx::postgres::PgPoolOptions;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), sqlx::Error> {
    /// let pool = PgPoolOptions::new()
    ///     .connect_lazy("postgres://capp:capp@localhost:5432/capp")?;
    /// let database = PgDatabase::new(pool.clone());
    /// let _shared = database.clone();
    /// pool.close().await;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
