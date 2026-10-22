//! Infraestrutura compartilhada pelos testes de integração, independente do Cucumber.

use backend::infrastructure::database::MIGRATOR;
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner};

/// PostgreSQL descartável com migrations aplicadas e pool próprio.
///
/// Cada instância possui container e banco isolados. Os casos de um mesmo teste
/// reutilizam seu pool; testes paralelos não compartilham dados nem conexões.
/// O suporte não lê `.env` ou `DATABASE_URL`. O container também é descartado
/// quando a instância sai de escopo, inclusive durante o desenrolamento de panic.
pub(super) struct TestDatabase {
    /// Pool disponível aos repositórios, routers e fixtures do teste.
    pub(super) pool: PgPool,
    /// Mantém o container vivo até o fim do teste.
    container: ContainerAsync<GenericImage>,
}

impl TestDatabase {
    /// Inicia PostgreSQL 17 em porta dinâmica e aplica as migrations do backend.
    ///
    /// # Panics
    ///
    /// Falha o teste se Docker, a imagem, a conexão ou as migrations não estiverem
    /// disponíveis. Nenhum banco externo ao container é acessado.
    pub(super) async fn new() -> Self {
        let container = GenericImage::new("postgres", "17-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_USER", "integration_test")
            .with_env_var("POSTGRES_PASSWORD", "integration_test")
            .with_env_var("POSTGRES_DB", "integration_test")
            .start()
            .await
            .unwrap();
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!(
                "postgres://integration_test:integration_test@{host}:{port}/integration_test"
            ))
            .await
            .unwrap();
        MIGRATOR.run(&pool).await.unwrap();
        Self { pool, container }
    }

    /// Fecha o pool e remove o container ao concluir os casos do teste.
    ///
    /// # Panics
    ///
    /// Falha o teste se Docker não conseguir remover o container.
    pub(super) async fn close(self) {
        self.pool.close().await;
        self.container.rm().await.unwrap();
    }
}
