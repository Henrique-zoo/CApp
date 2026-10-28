//! Ciclo de vida do PostgreSQL descartável e das bases usadas pelos cenários.
//!
//! [`DatabaseServer`] mantém um container por execução de suíte.
//! [`SuiteDatabase`] compartilha o acesso administrativo, a base consultiva
//! e a coordenação de clonagens. [`TestDatabase`] identifica o recurso que
//! cada World deverá liberar.
//!
//! As migrations e a fixture preparam uma base-modelo uma única vez. Cenários
//! de escrita clonam esse modelo, em vez de iniciar outro container. A base
//! compartilhada usa `default_transaction_read_only`; como o usuário de testes
//! tem privilégios administrativos, essa opção não é uma barreira de segurança
//! contra steps que a desabilitem explicitamente.
//!
//! Os nomes de bases usados no DDL são internos, nunca dados recebidos de uma
//! requisição. Host e porta vêm do Testcontainers; `DATABASE_URL` não é usada.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use axum::Router;
use backend::{
    api::{self, state::AppState},
    infrastructure::database::MIGRATOR,
};
use futures::lock::Mutex;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use testcontainers::{ContainerAsync, GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner};

/// Base-modelo preparada uma vez com migrations e fixture para clonagem.
const POSTGRES_TEMPLATE_DB: &str = "capp_cucumber_template";
/// Cópia da base-modelo usada pelos cenários consultivos.
const POSTGRES_SHARED_DB: &str = "capp_cucumber_shared";
/// Usuário criado exclusivamente no PostgreSQL descartável da suíte.
const POSTGRES_USER: &str = "capp";
/// Senha de teste usada apenas no container descartável.
const POSTGRES_PASSWORD: &str = "capp";
/// Porta interna do PostgreSQL; a porta publicada no host é dinâmica.
const POSTGRES_PORT: u16 = 5432;
/// Prefixo dos nomes gerados para as bases isoladas e reconhecidos na limpeza.
const SCENARIO_DATABASE_PREFIX: &str = "capp_cucumber_scenario_";

/// Recurso de banco associado a um cenário e devolvido ao hook de limpeza.
///
/// `name = None` identifica o pool compartilhado de leitura. Um nome presente
/// identifica a base isolada que [`SuiteDatabase::release_scenario`] deve remover.
/// Descartar este valor sem chamar a limpeza não executa `DROP DATABASE`.
pub(super) struct TestDatabase {
    /// Pool usado pelo router e pelos steps SQL do cenário.
    pub(super) pool: PgPool,
    /// Nome da base isolada; `None` identifica o pool compartilhado.
    name: Option<String>,
}

impl TestDatabase {
    /// Identifica o modo do banco para o diagnóstico do World.
    ///
    /// Retorna `isolated` quando há um nome de base própria e `shared-read-only`
    /// quando o cenário usa o pool compartilhado. Não executa consultas.
    pub(super) fn kind(&self) -> &'static str {
        if self.name.is_some() {
            "isolated"
        } else {
            "shared-read-only"
        }
    }
}

/// Proprietário do container e dos recursos compartilhados por uma suíte.
///
/// O runner mantém este valor vivo durante a execução. Depois que os hooks
/// terminam, deve consumir o valor com [`Self::shutdown`] para realizar a
/// limpeza assíncrona explícita.
pub(super) struct DatabaseServer {
    /// Handle que mantém o PostgreSQL descartável vivo durante a suíte.
    container: ContainerAsync<GenericImage>,
    /// Pools, router e sincronização compartilhados pelos hooks.
    pub(super) suite: Arc<SuiteDatabase>,
}

/// Recursos reutilizados pelos hooks de preparação e limpeza dos cenários.
///
/// O [`Arc`] compartilha estes recursos entre Worlds concorrentes. Um contador
/// atômico gera nomes únicos durante a suíte; o mutex serializa os comandos de
/// clonagem da base-modelo. Cada cenário gravável recebe pool e router próprios.
pub(super) struct SuiteDatabase {
    /// Endereço do PostgreSQL fornecido pelo Testcontainers.
    host: String,
    /// Porta do PostgreSQL publicada dinamicamente no host.
    port: u16,
    /// Pool conectado à base `postgres` para criar e remover bancos.
    admin_pool: PgPool,
    /// Pool de leitura reutilizado pelos cenários sem a tag de isolamento.
    shared_pool: PgPool,
    /// Router associado ao pool compartilhado.
    shared_app: Router,
    /// Contador atômico usado para gerar nomes de bases isoladas.
    next_database: AtomicU64,
    /// Mutex que serializa os comandos de clonagem da base-modelo.
    template_ddl: Mutex<()>,
}

/// Monta as opções de conexão do PostgreSQL descartável, sem abrir conexão.
///
/// # Parâmetros
///
/// - `host`: endereço informado pelo Testcontainers.
/// - `port`: porta publicada dinamicamente no host.
/// - `database`: nome interno da base administrativa, modelo, compartilhada ou isolada.
///
/// # Retorno
///
/// Opções SQLx com as credenciais fixas exclusivas do container de testes.
/// A configuração não lê `DATABASE_URL` da aplicação.
fn connect_options(host: &str, port: u16, database: &str) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(POSTGRES_USER)
        .password(POSTGRES_PASSWORD)
        .database(database)
}

/// Abre um pool no PostgreSQL de testes com o limite indicado.
///
/// # Parâmetros
///
/// - `host`, `port`, `database`: destino usado por [`connect_options`].
/// - `max_connections`: limite de conexões simultâneas do pool; deve ser positivo.
///
/// # Retorno
///
/// Pool conectado à base solicitada, usando as demais opções padrão do SQLx.
///
/// # Panics
///
/// Se a configuração do pool for inválida ou a conexão inicial falhar.
/// A falha de conexão inclui o nome da base no diagnóstico.
async fn connect_pool(host: &str, port: u16, database: &str, max_connections: u32) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_with(connect_options(host, port, database))
        .await
        .unwrap_or_else(|error| panic!("Cucumber should connect to database {database}: {error}"))
}

impl DatabaseServer {
    /// Inicializa o container, a base-modelo e os recursos consultivos da suíte.
    ///
    /// # Preparação
    ///
    /// 1. Inicia `postgres:17-alpine` com porta dinâmica e credenciais de teste.
    /// 2. Aplica o mesmo [`MIGRATOR`] do servidor e, em seguida, a fixture
    ///    `tests/fixtures/api_bdd.sql` à base-modelo.
    /// 3. Fecha o pool do modelo, marca a base como template e cria sua cópia
    ///    compartilhada com `default_transaction_read_only = on`.
    /// 4. Bloqueia conexões ao modelo e constrói o router com o pool compartilhado.
    ///
    /// O pool temporário do modelo aceita até cinco conexões; o administrativo,
    /// oito; o compartilhado, dezesseis. Nenhum servidor HTTP é iniciado.
    /// Requer Docker acessível e a imagem disponível localmente ou no registry.
    ///
    /// # Retorno
    ///
    /// Proprietário do container e dos recursos da suíte. O chamador deve executar
    /// [`Self::shutdown`] depois que todos os cenários terminarem.
    ///
    /// # Panics
    ///
    /// Em falhas de Docker, descoberta de endereço, conexão, migrations, fixture
    /// ou preparação das bases. Se a inicialização falhar, não há valor retornado
    /// para executar a sequência explícita de `shutdown`.
    pub(super) async fn start() -> Self {
        let container = GenericImage::new("postgres", "17-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", POSTGRES_TEMPLATE_DB)
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .start()
            .await
            .expect("PostgreSQL testcontainer should start");

        let host = container
            .get_host()
            .await
            .expect("PostgreSQL testcontainer should expose host")
            .to_string();
        let port = container
            .get_host_port_ipv4(POSTGRES_PORT)
            .await
            .expect("PostgreSQL testcontainer should expose port");
        let template_pool = connect_pool(&host, port, POSTGRES_TEMPLATE_DB, 5).await;

        MIGRATOR
            .run(&template_pool)
            .await
            .expect("Cucumber template migrations should run");

        sqlx::raw_sql(include_str!("../fixtures/api_bdd.sql"))
            .execute(&template_pool)
            .await
            .expect("the deterministic Cucumber template fixture should load");

        template_pool.close().await;

        let admin_pool = connect_pool(&host, port, "postgres", 8).await;
        sqlx::query(&format!(
            "ALTER DATABASE \"{POSTGRES_TEMPLATE_DB}\" IS_TEMPLATE true"
        ))
        .execute(&admin_pool)
        .await
        .expect("Cucumber database should be marked as a template");
        sqlx::query(&format!(
            "CREATE DATABASE \"{POSTGRES_SHARED_DB}\" TEMPLATE \"{POSTGRES_TEMPLATE_DB}\""
        ))
        .execute(&admin_pool)
        .await
        .expect("Cucumber should create the shared read-only database");
        sqlx::query(&format!(
            "ALTER DATABASE \"{POSTGRES_SHARED_DB}\" SET default_transaction_read_only = on"
        ))
        .execute(&admin_pool)
        .await
        .expect("Cucumber shared database should default to read-only transactions");
        sqlx::query(&format!(
            "ALTER DATABASE \"{POSTGRES_TEMPLATE_DB}\" ALLOW_CONNECTIONS false"
        ))
        .execute(&admin_pool)
        .await
        .expect("Cucumber template database should reject ordinary connections");

        let shared_pool = connect_pool(&host, port, POSTGRES_SHARED_DB, 16).await;
        let shared_app =
            api::router::create_router().with_state(AppState::new(shared_pool.clone()));
        let suite = Arc::new(SuiteDatabase {
            host,
            port,
            admin_pool,
            shared_pool,
            shared_app,
            next_database: AtomicU64::new(1),
            template_ddl: Mutex::new(()),
        });

        Self { container, suite }
    }

    /// Consome o servidor e encerra os recursos após o término dos cenários.
    ///
    /// Primeiro tenta remover bases isoladas remanescentes e fecha os pools
    /// compartilhado e administrativo. Em seguida, solicita a parada do container
    /// com tolerância de cinco segundos e o remove. O fechamento dos pools aguarda
    /// a devolução das conexões; os steps não devem manter tarefas usando-as após
    /// o encerramento do cenário.
    ///
    /// Falhas individuais ao remover bases e falhas na parada do container são
    /// registradas em stderr; a rotina continua para a remoção final do container.
    /// O timeout de parada não limita a duração total desta função.
    ///
    /// # Panics
    ///
    /// Se a consulta de bases remanescentes ou a remoção final do container falhar.
    /// Nesse caso, a execução das etapas restantes pode ser interrompida.
    pub(super) async fn shutdown(self) {
        self.suite.remove_orphaned_databases().await;
        self.suite.shared_pool.close().await;
        self.suite.admin_pool.close().await;

        if let Err(error) = self.container.stop_with_timeout(Some(5)).await {
            eprintln!("failed to stop PostgreSQL testcontainer cleanly: {error}");
        }

        self.container
            .rm()
            .await
            .expect("PostgreSQL testcontainer should be removed");
    }
}

impl SuiteDatabase {
    /// Prepara o recurso de banco e o router de um cenário.
    ///
    /// # Parâmetros
    ///
    /// - `isolated`: quando `true`, cria uma cópia gravável da base-modelo; quando
    ///   `false`, reutiliza a base e o router compartilhados de leitura.
    ///
    /// # Retorno
    ///
    /// Par com o banco a ser liberado pelo hook `after` e o router associado ao
    /// mesmo pool. No modo isolado, o nome usa um contador exclusivo da suíte,
    /// a clonagem é serializada pelo mutex e o pool aceita até cinco conexões.
    /// As migrations e a fixture já estão presentes na base clonada.
    ///
    /// # Panics
    ///
    /// Em falha de clonagem ou conexão. Se a base for criada e a conexão falhar,
    /// a limpeza de remanescentes no fim da suíte poderá removê-la.
    pub(super) async fn prepare_scenario(&self, isolated: bool) -> (TestDatabase, Router) {
        if !isolated {
            return (
                TestDatabase {
                    pool: self.shared_pool.clone(),
                    name: None,
                },
                self.shared_app.clone(),
            );
        }

        let database_number = self.next_database.fetch_add(1, Ordering::Relaxed);
        let database_name = format!("{SCENARIO_DATABASE_PREFIX}{database_number}");

        {
            let _template_guard = self.template_ddl.lock().await;
            sqlx::query(&format!(
                "CREATE DATABASE \"{database_name}\" TEMPLATE \"{POSTGRES_TEMPLATE_DB}\""
            ))
            .execute(&self.admin_pool)
            .await
            .unwrap_or_else(|error| {
                panic!("Cucumber should clone isolated database {database_name}: {error}")
            });
        }

        let pool = connect_pool(&self.host, self.port, &database_name, 5).await;
        let app = api::router::create_router().with_state(AppState::new(pool.clone()));
        (
            TestDatabase {
                pool,
                name: Some(database_name),
            },
            app,
        )
    }

    /// Libera o banco de um cenário depois que seu router foi descartado.
    ///
    /// No modo compartilhado, descarta somente o handle local, preservando o pool
    /// da suíte. No modo isolado, fecha o pool e remove a base. O chamador deve
    /// encerrar as tarefas e devolver as conexões adquiridas pelo cenário antes
    /// da limpeza, pois o fechamento do pool aguarda essas devoluções.
    ///
    /// # Panics
    ///
    /// Se a remoção da base isolada falhar.
    pub(super) async fn release_scenario(&self, database: TestDatabase) {
        let Some(database_name) = database.name else {
            drop(database.pool);
            return;
        };

        database.pool.close().await;
        self.drop_database(&database_name)
            .await
            .unwrap_or_else(|error| {
                panic!("Cucumber should remove isolated database {database_name}: {error}")
            });
    }

    /// Remove uma base isolada, forçando o encerramento de conexões remanescentes.
    ///
    /// # Parâmetros
    ///
    /// - `database_name`: identificador interno gerado pela suíte ou selecionado
    ///   pela busca de bases remanescentes. Não pode conter nomes arbitrários ou
    ///   entrada externa: é interpolado no DDL sem escape adicional.
    ///
    /// # Retorno
    ///
    /// `Ok(())` quando `DROP DATABASE IF EXISTS ... WITH (FORCE)` é concluído,
    /// inclusive se a base já não existir.
    ///
    /// # Erros
    ///
    /// Propaga falhas do SQLx ao obter conexão ou executar a remoção no PostgreSQL.
    async fn drop_database(&self, database_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"
        ))
        .execute(&self.admin_pool)
        .await?;
        Ok(())
    }

    /// Tenta remover bases isoladas que permaneceram após os hooks.
    ///
    /// Seleciona somente nomes com o prefixo da suíte seguido de dígitos. Falhas
    /// individuais de remoção são registradas em stderr e não impedem a tentativa
    /// nas demais bases. A remoção posterior do container descarta os dados restantes.
    ///
    /// # Panics
    ///
    /// Se a consulta ao catálogo `pg_database` falhar.
    async fn remove_orphaned_databases(&self) {
        let database_names = sqlx::query_scalar::<_, String>(
            "SELECT datname
             FROM pg_database
             WHERE datname ~ '^capp_cucumber_scenario_[0-9]+$'",
        )
        .fetch_all(&self.admin_pool)
        .await
        .expect("Cucumber should list orphaned scenario databases");

        for database_name in database_names {
            if let Err(error) = self.drop_database(&database_name).await {
                eprintln!("failed to remove orphaned database {database_name}: {error}");
            }
        }
    }
}
