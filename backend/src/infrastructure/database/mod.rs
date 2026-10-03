//! Migrations PostgreSQL compartilhadas pelo servidor e pelos testes.
//!
//! [`MIGRATOR`] incorpora o conteúdo de `backend/migrations/` na compilação.
//! O executável o aplica antes de abrir a porta HTTP; a infraestrutura BDD usa
//! o mesmo migrador ao preparar sua base-modelo, antes de carregar a fixture.
//! Cada chamador cria e administra seu próprio pool.

use sqlx::migrate::Migrator;

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
/// O diretório pode permanecer sem arquivos SQL enquanto o modelo de dados não
/// estiver definido. Nesse caso, somente a tabela de controle é preparada.
/// Novos arquivos exigem recompilar o binário; `build.rs` observa o diretório
/// para que o Cargo detecte essas adições.
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
