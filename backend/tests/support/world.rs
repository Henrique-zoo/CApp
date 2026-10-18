//! Contexto de cada cenário e acesso ao banco de testes e ao router real.
//!
//! [`AppWorld`] nasce sem recursos externos. Os hooks associam banco e router
//! antes dos steps e os liberam ao término. Respostas e resultados pertencem
//! a cada cenário, mesmo quando o pool consultivo é compartilhado.

use std::fmt;

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
/// isolamento; o hook `after` os libera. Steps de integração acessam o pool por
/// [`Self::database_pool`] e enviam requisições por [`Self::get`] ou
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
    /// Gerenciamento em memória e regras de autenticação institucional da UnB (#11).
    pub(crate) institutional_auth: InstitutionalAuthFixture,
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
            .field("institutional_auth", &self.institutional_auth)
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

/// Natureza da conta ou credencial institucional apresentada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AccountKind {
    /// Conta institucional ativa da UnB.
    UnbActive {
        /// Endereço de e-mail institucional.
        email: String,
        /// Indica se a tentativa representa o primeiro acesso do estudante.
        first_access: bool,
    },
    /// Credenciais inválidas para o e-mail informado.
    InvalidCredentials {
        /// Endereço de e-mail institucional informado.
        email: String,
    },
    /// Conta institucional revogada pela administração da UnB.
    RevokedUnbAccount {
        /// Endereço de e-mail da conta revogada.
        email: String,
    },
    /// Conta pessoal da Microsoft, fora do domínio institucional da UnB.
    PersonalMicrosoft,
    /// Conta institucional de outra universidade.
    OtherUniversity {
        /// Confirmação de que a conta pertence à outra instituição de ensino.
        provider_confirmed: bool,
    },
}

/// Situação da sessão de acesso mantida no aplicativo.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum SessionState {
    /// Nenhuma sessão ativa estabelecida.
    #[default]
    Absent,
    /// Sessão válida associada a um e-mail institucional da UnB.
    Valid {
        /// E-mail do usuário autenticado.
        email: String,
    },
    /// Sessão expirada sem possibilidade de renovação.
    Expired,
    /// Sessão inválida.
    Invalid,
}

/// Estado observável da tela do aplicativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ScreenState {
    /// Tela de entrada inicial do aplicativo.
    #[default]
    Entry,
    /// Tela de autenticação institucional em andamento.
    Authentication,
    /// Tela principal após autenticação e concessão de acesso.
    Authenticated,
}

/// Mensagens funcionais esperadas na resposta da API ou interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthMessage {
    /// Credenciais institucionais inválidas.
    InvalidCredentials,
    /// Falha na autenticação da conta institucional.
    AccountRejected,
    /// Exigência de conta institucional da UnB.
    UnbAccountRequired,
    /// Indisponibilidade temporária do serviço de autenticação.
    Unavailable,
    /// Bloqueio por ausência de autenticação institucional.
    AuthenticationRequired,
    /// Bloqueio por sessão sem validade.
    InvalidSession,
}

impl AuthMessage {
    /// Retorna a mensagem amigável correspondente ao diagnóstico funcional.
    ///
    /// # Retorno
    ///
    /// Mensagem em português adequada para apresentação ao estudante.
    pub(crate) fn user_friendly_message(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "Credenciais institucionais inválidas.",
            Self::AccountRejected => "Não foi possível autenticar a conta institucional.",
            Self::UnbAccountRequired => "É necessário utilizar uma conta institucional da UnB.",
            Self::Unavailable => "Serviço de autenticação temporariamente indisponível.",
            Self::AuthenticationRequired => {
                "Acesso restrito: autenticação institucional necessária."
            }
            Self::InvalidSession => "Sessão inválida ou expirada.",
        }
    }
}

/// Valida se o endereço de e-mail institucional atende às regras da UnB.
///
/// # Regras institucionais da UnB
///
/// - Domínio estrito: `@aluno.unb.br`.
/// - A parte local deve corresponder à matrícula discente (composta por dígitos).
/// - Matrícula '242003979' é expressamente proibida no contexto de testes.
///
/// # Parâmetros
///
/// - `email`: endereço de e-mail a ser validado.
///
/// # Retorno
///
/// `true` se o endereço respeitar o padrão institucional discente da UnB; `false` caso contrário.
pub(crate) fn is_valid_unb_student_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };

    if domain != "aluno.unb.br" {
        return false;
    }

    if local == "242003979" {
        return false;
    }

    !local.is_empty() && local.chars().all(|c| c.is_ascii_digit())
}

/// Gerenciamento do estado e validação de autenticação institucional da UnB (#11).
#[derive(Debug)]
pub(crate) struct InstitutionalAuthFixture {
    /// Conta institucional ou credencial preparada pelo cenário.
    pub(crate) prepared_account: Option<AccountKind>,
    /// Disponibilidade do serviço externo de autenticação.
    pub(crate) provider_available: bool,
    /// Situação atual da sessão de acesso.
    pub(crate) session: SessionState,
    /// Tela atual observável.
    pub(crate) screen: ScreenState,
    /// E-mail da identidade autenticada ativa, quando disponível.
    pub(crate) authenticated_email: Option<String>,
    /// Última mensagem funcional gerada.
    pub(crate) message: Option<AuthMessage>,
    /// Indica se o acesso à plataforma foi concedido com sucesso.
    pub(crate) access_granted: bool,
    /// Indica se o acesso a recurso protegido foi bloqueado.
    pub(crate) protected_access_blocked: bool,
    /// Indica se uma nova tentativa de login está disponível.
    pub(crate) retry_available: bool,
    /// Indica se o sistema solicitou autenticação institucional.
    pub(crate) login_requested: bool,
    /// Indica se foi solicitado cadastro manual para entrar no app.
    pub(crate) manual_registration_required: Option<bool>,
    /// Resultado de uma ação protegida solicitada; `None` antes da tentativa.
    pub(crate) action_performed: Option<bool>,
    /// Registra se o encerramento da sessão foi concluído.
    pub(crate) logout_completed: bool,
}

impl Default for InstitutionalAuthFixture {
    fn default() -> Self {
        Self {
            prepared_account: None,
            provider_available: true,
            session: SessionState::Absent,
            screen: ScreenState::Entry,
            authenticated_email: None,
            message: None,
            access_granted: false,
            protected_access_blocked: false,
            retry_available: true,
            login_requested: false,
            manual_registration_required: None,
            action_performed: None,
            logout_completed: false,
        }
    }
}

impl InstitutionalAuthFixture {
    /// Reinicializa o estado para um usuário não autenticado.
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    /// Configura uma conta institucional ativa da UnB para o estudante.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail institucional discente da UnB.
    /// - `first_access`: indica se representa o primeiro acesso do estudante.
    pub(crate) fn set_unb_account(&mut self, email: &str, first_access: bool) {
        self.prepared_account = Some(AccountKind::UnbActive {
            email: email.to_string(),
            first_access,
        });
    }

    /// Configura credenciais inválidas para o e-mail informado.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail informado na tentativa inválida.
    pub(crate) fn set_invalid_credentials(&mut self, email: &str) {
        self.prepared_account = Some(AccountKind::InvalidCredentials {
            email: email.to_string(),
        });
    }

    /// Configura uma conta institucional revogada pela UnB.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail da conta revogada.
    pub(crate) fn set_revoked_unb_account(&mut self, email: &str) {
        self.prepared_account = Some(AccountKind::RevokedUnbAccount {
            email: email.to_string(),
        });
    }

    /// Configura uma conta pessoal da Microsoft.
    pub(crate) fn set_personal_microsoft_account(&mut self) {
        self.prepared_account = Some(AccountKind::PersonalMicrosoft);
    }

    /// Configura uma conta institucional de outra universidade.
    pub(crate) fn set_other_university_account(&mut self) {
        self.prepared_account = Some(AccountKind::OtherUniversity {
            provider_confirmed: false,
        });
    }

    /// Confirma que o serviço de autenticação identificou o vínculo com outra universidade.
    pub(crate) fn confirm_other_university(&mut self) {
        if let Some(AccountKind::OtherUniversity { provider_confirmed }) =
            &mut self.prepared_account
        {
            *provider_confirmed = true;
        }
    }

    /// Simula indisponibilidade temporária do serviço de autenticação.
    pub(crate) fn set_provider_unavailable(&mut self) {
        self.provider_available = false;
    }

    /// Registra início do fluxo de autenticação institucional na interface.
    pub(crate) fn start_login(&mut self) {
        self.screen = ScreenState::Authentication;
    }

    /// Configura a situação da sessão (ex.: expirada ou inválida).
    ///
    /// # Parâmetros
    ///
    /// - `status`: descrição textual da situação da sessão.
    ///
    /// # Panics
    ///
    /// Se a situação informada não for reconhecida.
    pub(crate) fn set_session_state(&mut self, status: &str) {
        self.session = match status {
            "expirada sem possibilidade de renovação" => SessionState::Expired,
            "inválida" => SessionState::Invalid,
            "válida" => SessionState::Valid {
                email: "231012345@aluno.unb.br".to_string(),
            },
            _ => panic!("situação de sessão não reconhecida: {status}"),
        };
        self.access_granted = matches!(self.session, SessionState::Valid { .. });
        self.screen = if self.access_granted {
            ScreenState::Authenticated
        } else {
            ScreenState::Entry
        };
    }

    /// Configura uma sessão autenticada válida.
    ///
    /// # Parâmetros
    ///
    /// - `email`: e-mail opcional da conta institucional.
    pub(crate) fn set_valid_session(&mut self, email: Option<&str>) {
        let student_email = email.unwrap_or("231012345@aluno.unb.br");
        self.session = SessionState::Valid {
            email: student_email.to_string(),
        };
        self.authenticated_email = Some(student_email.to_string());
        self.access_granted = true;
        self.screen = ScreenState::Authenticated;
    }

    /// Executa a autenticação com base na conta e no estado do provedor.
    ///
    /// # Panics
    ///
    /// Se nenhuma conta ou credencial foi preparada no cenário.
    pub(crate) fn authenticate(&mut self) {
        if !self.provider_available {
            self.access_granted = false;
            self.session = SessionState::Absent;
            self.screen = ScreenState::Entry;
            self.message = Some(AuthMessage::Unavailable);
            self.retry_available = true;
            return;
        }

        let account = self
            .prepared_account
            .as_ref()
            .expect("uma conta ou credencial deve ser preparada antes da autenticação");

        self.session = SessionState::Absent;
        self.screen = ScreenState::Entry;
        self.access_granted = false;
        self.retry_available = true;
        self.login_requested = false;
        self.manual_registration_required = None;
        self.logout_completed = false;

        match account {
            AccountKind::UnbActive { email, .. } => {
                if is_valid_unb_student_email(email) {
                    self.session = SessionState::Valid {
                        email: email.clone(),
                    };
                    self.authenticated_email = Some(email.clone());
                    self.screen = ScreenState::Authenticated;
                    self.access_granted = true;
                    self.retry_available = false;
                    self.manual_registration_required = Some(false);
                    self.message = None;
                } else {
                    self.message = Some(AuthMessage::UnbAccountRequired);
                }
            }
            AccountKind::InvalidCredentials { .. } => {
                self.message = Some(AuthMessage::InvalidCredentials);
            }
            AccountKind::RevokedUnbAccount { .. } => {
                self.message = Some(AuthMessage::AccountRejected);
            }
            AccountKind::PersonalMicrosoft | AccountKind::OtherUniversity { .. } => {
                self.message = Some(AuthMessage::UnbAccountRequired);
            }
        }
    }

    /// Cancela o fluxo de autenticação em andamento.
    pub(crate) fn cancel_login(&mut self) {
        self.prepared_account = None;
        self.session = SessionState::Absent;
        self.screen = ScreenState::Entry;
        self.access_granted = false;
        self.retry_available = true;
        self.message = None;
    }

    /// Valida o acesso a um serviço restrito à comunidade acadêmica.
    pub(crate) fn access_restricted_service(&mut self) {
        match &self.session {
            SessionState::Valid { .. } => {
                self.access_granted = true;
                self.protected_access_blocked = false;
                self.message = None;
                self.login_requested = false;
            }
            SessionState::Absent => {
                self.access_granted = false;
                self.protected_access_blocked = true;
                self.message = Some(AuthMessage::AuthenticationRequired);
                self.login_requested = true;
            }
            SessionState::Expired | SessionState::Invalid => {
                self.access_granted = false;
                self.protected_access_blocked = true;
                self.message = Some(AuthMessage::InvalidSession);
                self.login_requested = true;
            }
        }
    }

    /// Executa uma ação que exige autenticação prévia.
    pub(crate) fn perform_restricted_action(&mut self) {
        self.access_restricted_service();
        self.action_performed = Some(self.access_granted);
    }

    /// Encerra a sessão ativa no aplicativo.
    ///
    /// # Panics
    ///
    /// Se não houver uma sessão válida ativa no momento do logout.
    pub(crate) fn logout(&mut self) {
        assert!(
            matches!(self.session, SessionState::Valid { .. }),
            "deve haver uma sessão válida para realizar logout"
        );
        self.session = SessionState::Absent;
        self.authenticated_email = None;
        self.screen = ScreenState::Entry;
        self.access_granted = false;
        self.retry_available = true;
        self.message = None;
        self.logout_completed = true;
    }
}
