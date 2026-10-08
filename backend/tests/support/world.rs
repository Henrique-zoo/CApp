//! Contexto de cada cenário e requisições em memória ao router real.
//!
//! [`AppWorld`] nasce sem recursos externos. Os hooks associam banco e router
//! antes dos steps e os liberam ao término. Respostas e resultados pertencem
//! a cada cenário, mesmo quando o pool consultivo é compartilhado.

use std::{collections::HashMap, fmt};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde::de::DeserializeOwned;
use sqlx::PgPool;
use tower::ServiceExt;

use super::{
    database::{SuiteDatabase, TestDatabase},
    results::{TestDatabaseMutation, TestResponse},
};

/// Contexto independente criado pelo Cucumber para cada cenário.
///
/// [`Default`] inicia sem banco ou router, sem resultados e com contador de
/// requisições zerado. O hook `before` associa os recursos conforme a tag de
/// isolamento; o hook `after` os libera. Steps devem acessar o pool por
/// [`Self::database_pool`] e enviar requisições por [`Self::get`] ou
/// [`Self::get_json`].
///
/// Os campos de resultados SQL são preenchidos pelos steps que executarem
/// mutações. Os helpers HTTP não os atualizam automaticamente.
#[derive(cucumber::World, Default)]
pub(crate) struct AppWorld {
    /// Banco associado pelo hook `before` e liberado pelo hook `after`.
    database: Option<TestDatabase>,
    /// Router real com o estado associado ao pool desse cenário.
    app: Option<Router>,
    /// Resultados das operações concorrentes, na ordem definida pelo step.
    pub(crate) concurrent_database_mutations: Vec<TestDatabaseMutation>,
    /// Resultado da última tentativa de mutação sequencial.
    pub(crate) last_database_mutation: Option<TestDatabaseMutation>,
    /// Última resposta HTTP capturada pelos helpers de requisição.
    pub(crate) last_response: Option<TestResponse>,
    /// Número de tentativas de GET iniciadas após a obtenção do router.
    ///
    /// Inclui chamadas via `get_json` e aumenta antes de construir a requisição;
    /// uma URI inválida ou falha de leitura também pode incrementar o contador.
    pub(crate) request_count: usize,
    /// Estado e fixture dos cenários de provisionamento de usuário.
    pub(crate) user_provisioning: UserProvisioningFixture,
}

impl fmt::Debug for AppWorld {
    /// Formata o contexto do cenário sem expor pools ou o container.
    ///
    /// Inclui modo do banco, presença do router, resultados SQL, última resposta
    /// e contador de requisições. O corpo HTTP é incluído no diagnóstico.
    ///
    /// # Erros
    ///
    /// Propaga falhas de escrita retornadas pelo formatador.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppWorld")
            .field(
                "database_kind",
                &self.database.as_ref().map(TestDatabase::kind),
            )
            .field("app_ready", &self.app.is_some())
            .field(
                "concurrent_database_mutations",
                &self.concurrent_database_mutations,
            )
            .field("last_database_mutation", &self.last_database_mutation)
            .field("last_response", &self.last_response)
            .field("request_count", &self.request_count)
            .field("user_provisioning", &self.user_provisioning)
            .finish()
    }
}

impl AppWorld {
    /// Obtém o pool associado ao cenário pelo hook `before`.
    ///
    /// # Retorno
    ///
    /// Referência ao pool compartilhado de leitura ou ao pool isolado do cenário.
    /// Steps que escrevem devem estar em cenários com `@isolated_database`.
    ///
    /// # Panics
    ///
    /// Se o hook ainda não associou o banco ou a limpeza já o removeu do World.
    pub(crate) fn database_pool(&self) -> &PgPool {
        &self
            .database
            .as_ref()
            .expect("BDD database should be initialized")
            .pool
    }

    /// Associa o banco e o router ao World antes da execução dos steps.
    ///
    /// # Parâmetros
    ///
    /// - `suite`: recursos compartilhados e acesso administrativo ao banco de testes.
    /// - `isolated`: solicita uma cópia gravável da base-modelo quando `true`.
    ///
    /// Delega a preparação a [`SuiteDatabase::prepare_scenario`] e só armazena
    /// os recursos depois que essa operação termina. Não reinicializa os campos
    /// de resultados: cada cenário deve começar com um novo World.
    ///
    /// # Panics
    ///
    /// Em associação duplicada, falha de clonagem ou conexão. Uma base criada
    /// antes de falha de conexão poderá ser recolhida no encerramento da suíte.
    pub(super) async fn attach_database(&mut self, suite: &SuiteDatabase, isolated: bool) {
        assert!(
            self.database.is_none() && self.app.is_none(),
            "a Cucumber world should receive its database only once",
        );

        let (database, app) = suite.prepare_scenario(isolated).await;
        self.database = Some(database);
        self.app = Some(app);
    }

    /// Retira do World os recursos usados pelo cenário e libera seu banco.
    ///
    /// Descarta o router antes de delegar a limpeza a
    /// [`SuiteDatabase::release_scenario`]. Tolera um World sem banco e pode ser
    /// chamado novamente após a limpeza. Mantém os resultados das operações para
    /// diagnóstico, mas o World deixa de aceitar consultas e requisições.
    ///
    /// # Panics
    ///
    /// Se a remoção da base isolada falhar. O recurso já terá sido retirado do World;
    /// a busca de bases remanescentes poderá tentar a remoção no final da suíte.
    pub(super) async fn detach_database(&mut self, suite: &SuiteDatabase) {
        self.app.take();

        let Some(database) = self.database.take() else {
            return;
        };

        suite.release_scenario(database).await;
    }

    /// Executa GET no router real e preserva status e corpo da resposta.
    ///
    /// # Parâmetros
    ///
    /// - `path`: URI aceita por [`Request::builder`], incluindo query string.
    ///
    /// Usa [`ServiceExt::oneshot`] com corpo vazio, sem abrir um socket HTTP.
    /// Incrementa [`Self::request_count`] antes de construir a requisição e substitui
    /// [`Self::last_response`] somente após ler e converter o corpo por completo.
    /// Status 4xx e 5xx também são capturados, sem causar panic por si só.
    ///
    /// O helper destina-se a respostas textuais de teste: exige UTF-8 e coleta o
    /// corpo inteiro em memória, com limite de `usize::MAX` bytes. Não há timeout
    /// adicional além daqueles implementados pelo próprio router.
    ///
    /// # Panics
    ///
    /// Se o router não estiver associado, a URI for inválida ou ocorrer falha ao
    /// processar a requisição, coletar o corpo ou convertê-lo para UTF-8. Quando
    /// há falha antes de armazenar a nova resposta, a resposta anterior permanece.
    pub(crate) async fn get(&mut self, path: &str) {
        let app = self
            .app
            .as_ref()
            .expect("backend API should be initialized before requests")
            .clone();
        self.request_count += 1;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(path)
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should be handled by backend router");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        self.last_response = Some(TestResponse {
            status,
            body: String::from_utf8(body.to_vec()).expect("response body should be UTF-8"),
        });
    }

    /// Executa GET e desserializa o corpo de uma resposta HTTP 200.
    ///
    /// # Parâmetros
    ///
    /// - `path`: URI enviada a [`Self::get`].
    /// - `T`: tipo de destino que implementa [`DeserializeOwned`].
    ///
    /// # Retorno
    ///
    /// Corpo convertido para `T`. A resposta textual continua em
    /// [`Self::last_response`], inclusive se a validação de status ou JSON falhar.
    /// O contador aumenta uma única vez por meio de `get`. Não verifica o cabeçalho
    /// `Content-Type`; a validação usa o status e a desserialização do corpo.
    ///
    /// # Panics
    ///
    /// Nas falhas de [`Self::get`], em status diferente de 200 ou quando o corpo
    /// não puder ser desserializado para `T`.
    pub(crate) async fn get_json<T>(&mut self, path: &str) -> T
    where
        T: DeserializeOwned,
    {
        self.get(path).await;

        let response = self
            .last_response
            .as_ref()
            .expect("a response should have been captured");

        assert_eq!(response.status, StatusCode::OK);

        serde_json::from_str(&response.body).expect("response should match the expected JSON shape")
    }
}

/// Situação cadastral da instituição de ensino parceira.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstitutionStatus {
    /// Instituição ativa e disponível no CApp.
    Active,
    /// Instituição inativa ou desativada no CApp.
    Inactive,
}

/// Registro interno do usuário na aplicação (`app_user`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppUserRecord {
    /// Identificador único e estável do usuário na aplicação.
    pub(crate) id: uuid::Uuid,
    /// Nome de exibição do usuário no aplicativo.
    pub(crate) display_name: String,
    /// E-mail institucional do usuário.
    pub(crate) institutional_email: String,
    /// Situação cadastral da conta interna ("active" ou "inactive").
    pub(crate) status: String,
    /// Instante lógico de criação do registro.
    pub(crate) created_at: u64,
    /// Instante lógico da última atualização do registro.
    pub(crate) updated_at: u64,
}

/// Associação da identidade institucional externa (`external_identity`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalIdentityRecord {
    /// Identificador único deste registro de identidade externa.
    pub(crate) id: uuid::Uuid,
    /// Identificador do usuário interno ao qual este registro pertence.
    pub(crate) user_id: uuid::Uuid,
    /// Nome da instituição de ensino associada.
    pub(crate) institution_name: String,
    /// Identificador do sujeito na Microsoft/provedor institucional.
    pub(crate) external_subject: String,
    /// Instante lógico do último login registrado para a identidade.
    pub(crate) last_login_at: u64,
}

/// Fixture que mantém o estado de provisionamento de usuários no cenário BDD.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserProvisioningFixture {
    /// Instituições parceiras cadastradas, indexadas pelo nome.
    pub(crate) institutions: HashMap<String, InstitutionStatus>,
    /// Usuários internos cadastrados, indexados pelo identificador único.
    pub(crate) users: HashMap<uuid::Uuid, AppUserRecord>,
    /// Identidades externas cadastradas, indexadas pelo identificador do sujeito externo.
    pub(crate) external_identities: HashMap<String, ExternalIdentityRecord>,
    /// Mapeamento de e-mail institucional para o identificador do sujeito externo.
    pub(crate) email_to_subject: HashMap<String, String>,
    /// E-mail do estudante em foco no cenário.
    pub(crate) current_email: Option<String>,
    /// Identificador interno do usuário em foco no cenário.
    pub(crate) current_user_id: Option<uuid::Uuid>,
    /// Identificador interno original capturado antes de novas operações.
    pub(crate) initial_user_id: Option<uuid::Uuid>,
    /// Quantidade de usuários cadastrados antes da última operação.
    pub(crate) user_count_before: usize,
    /// Quantidade de usuários cadastrados após a última operação.
    pub(crate) user_count_after: usize,
    /// Indica se um novo perfil interno foi criado na última operação.
    pub(crate) was_user_created: bool,
    /// Indica se o perfil existente foi recuperado na última operação.
    pub(crate) was_user_recovered: bool,
    /// Instante do último login antes da operação mais recente.
    pub(crate) last_login_before: Option<u64>,
    /// Instante do último login após a operação mais recente.
    pub(crate) last_login_after: Option<u64>,
    /// Relógio lógico monotonicamente crescente.
    pub(crate) clock: u64,
    /// Indica se o fluxo de autenticação exigiu formulário manual de cadastro.
    pub(crate) registration_form_required: bool,
}

impl UserProvisioningFixture {
    /// Registra uma instituição de ensino no catálogo da fixture.
    ///
    /// # Parâmetros
    ///
    /// - `name`: nome da instituição parceira.
    /// - `status`: situação cadastral da instituição.
    pub(crate) fn register_institution(&mut self, name: &str, status: InstitutionStatus) {
        self.institutions.insert(name.to_string(), status);
    }

    /// Cadastra previamente um estudante existente na aplicação.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail institucional do estudante.
    /// - `display_name`: nome de exibição no aplicativo.
    ///
    /// # Retorno
    ///
    /// Identificador único gerado para o usuário pré-cadastrado.
    pub(crate) fn register_existing_user(&mut self, email: &str, display_name: &str) -> uuid::Uuid {
        self.clock += 1;
        let user_id = uuid::Uuid::new_v4();
        let subject = format!("sub-unb-{}", email);

        let user = AppUserRecord {
            id: user_id,
            display_name: display_name.to_string(),
            institutional_email: email.to_string(),
            status: "active".to_string(),
            created_at: self.clock,
            updated_at: self.clock,
        };

        let identity = ExternalIdentityRecord {
            id: uuid::Uuid::new_v4(),
            user_id,
            institution_name: "Universidade de Brasília".to_string(),
            external_subject: subject.clone(),
            last_login_at: self.clock,
        };

        self.users.insert(user_id, user);
        self.external_identities.insert(subject.clone(), identity);
        self.email_to_subject.insert(email.to_string(), subject);
        self.user_count_before = self.users.len();
        self.user_count_after = self.users.len();
        self.current_user_id = Some(user_id);
        self.current_email = Some(email.to_string());

        user_id
    }

    /// Executa o login e provisionamento automático do estudante com base no e-mail institucional.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail institucional autenticado externamente.
    pub(crate) fn provision_or_authenticate(&mut self, email: &str) {
        self.user_count_before = self.users.len();
        self.clock += 1;

        let subject = self
            .email_to_subject
            .get(email)
            .cloned()
            .unwrap_or_else(|| format!("sub-unb-{}", email));

        if let Some(identity) = self.external_identities.get_mut(&subject) {
            self.last_login_before = Some(identity.last_login_at);
            identity.last_login_at = self.clock;
            self.last_login_after = Some(self.clock);

            let user_id = identity.user_id;
            if let Some(user) = self.users.get_mut(&user_id) {
                user.updated_at = self.clock;
            }

            self.was_user_created = false;
            self.was_user_recovered = true;
            self.current_user_id = Some(user_id);
        } else {
            let user_id = uuid::Uuid::new_v4();
            let user = AppUserRecord {
                id: user_id,
                display_name: format!(
                    "Estudante {}",
                    &email[..email.find('@').unwrap_or(email.len())]
                ),
                institutional_email: email.to_string(),
                status: "active".to_string(),
                created_at: self.clock,
                updated_at: self.clock,
            };

            let identity = ExternalIdentityRecord {
                id: uuid::Uuid::new_v4(),
                user_id,
                institution_name: "Universidade de Brasília".to_string(),
                external_subject: subject.clone(),
                last_login_at: self.clock,
            };

            self.users.insert(user_id, user);
            self.external_identities.insert(subject.clone(), identity);
            self.email_to_subject.insert(email.to_string(), subject);

            self.was_user_created = true;
            self.was_user_recovered = false;
            self.current_user_id = Some(user_id);
            self.last_login_before = None;
            self.last_login_after = Some(self.clock);
        }

        self.user_count_after = self.users.len();
        self.registration_form_required = false;
        self.current_email = Some(email.to_string());
    }

    /// Executa múltiplas tentativas consecutivas de reautenticação para o mesmo e-mail.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail institucional do estudante.
    /// - `times`: quantidade de tentativas de login.
    pub(crate) fn reauthenticate_multiple(&mut self, email: &str, times: usize) {
        for _ in 0..times {
            self.provision_or_authenticate(email);
        }
    }

    /// Obtém uma referência ao usuário pelo seu identificador único.
    ///
    /// # Parâmetros
    ///
    /// - `id`: identificador do usuário na aplicação.
    ///
    /// # Retorno
    ///
    /// Referência ao registro do usuário, se encontrado.
    pub(crate) fn get_user(&self, id: &uuid::Uuid) -> Option<&AppUserRecord> {
        self.users.get(id)
    }

    /// Obtém uma referência ao usuário pelo seu e-mail institucional.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail institucional pesquisado.
    ///
    /// # Retorno
    ///
    /// Referência ao registro do usuário, se cadastrado.
    pub(crate) fn get_user_by_email(&self, email: &str) -> Option<&AppUserRecord> {
        self.users
            .values()
            .find(|user| user.institutional_email == email)
    }

    /// Obtém a identidade externa associada a um determinado usuário interno.
    ///
    /// # Parâmetros
    ///
    /// - `user_id`: identificador do usuário interno.
    ///
    /// # Retorno
    ///
    /// Referência ao registro de identidade externa correspondente.
    pub(crate) fn get_identity_for_user(
        &self,
        user_id: &uuid::Uuid,
    ) -> Option<&ExternalIdentityRecord> {
        self.external_identities
            .values()
            .find(|identity| identity.user_id == *user_id)
    }

    /// Conta quantos usuários internos estão associados ao mesmo identificador externo.
    ///
    /// # Parâmetros
    ///
    /// - `subject`: identificador do sujeito externo.
    ///
    /// # Retorno
    ///
    /// Quantidade de usuários vinculados àquela identidade externa.
    pub(crate) fn count_users_for_identity(&self, subject: &str) -> usize {
        let Some(identity) = self.external_identities.get(subject) else {
            return 0;
        };
        self.users
            .values()
            .filter(|user| user.id == identity.user_id)
            .count()
    }

    /// Conta quantos registros de usuários possuem o e-mail institucional informado.
    ///
    /// # Parâmetros
    ///
    /// - `email`: endereço de e-mail institucional.
    ///
    /// # Retorno
    ///
    /// Quantidade de usuários cadastrados com esse e-mail.
    pub(crate) fn count_users_with_email(&self, email: &str) -> usize {
        self.users
            .values()
            .filter(|user| user.institutional_email == email)
            .count()
    }
}
