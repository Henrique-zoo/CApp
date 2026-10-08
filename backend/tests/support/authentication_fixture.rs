//! Respostas predeterminadas para especificar o login institucional da UnB.
//!
//! Suporte temporário da issue #3: os steps executam uma simulação em memória.
//! Nenhuma credencial é validada, nenhum provedor é consultado e nenhum usuário
//! é persistido. A automação posterior (#11) deverá verificar a implementação
//! real, substituindo estas respostas preparadas nos passos de ação.

/// Resposta de login escolhida pelo `Dado`, sem consultar Microsoft ou Firebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoginResponseFixture {
    /// Conta da UnB aceita no exemplo.
    UnbAccount,
    /// Credenciais recusadas no exemplo.
    InvalidCredentials,
    /// Conta revogada e recusada no exemplo.
    RevokedAccount,
    /// Conta pessoal Microsoft, fora do escopo inicial.
    PersonalAccount,
    /// Conta institucional de outra universidade, fora do escopo inicial.
    OtherUniversity,
    /// Serviço temporariamente indisponível no exemplo.
    Unavailable,
}

/// Dados fictícios de uma tentativa, independentes de qualquer cadastro real.
#[derive(Debug)]
pub(crate) struct LoginFixture {
    /// E-mail apresentado pelo cenário; não é usado para decidir a instituição.
    pub(crate) email: String,
    /// Resultado predeterminado dessa tentativa.
    pub(crate) response: LoginResponseFixture,
}

/// Situação de sessão preparada pelo cenário, sem tokens ou relógio real.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum SessionFixture {
    /// Usuário sem sessão no aplicativo simulado.
    #[default]
    Absent,
    /// Sessão válida que disponibiliza uma identidade fictícia da UnB.
    Valid { email: String },
    /// Sessão expirada que não pode ser renovada no exemplo.
    Expired,
    /// Sessão cuja validade foi recusada no exemplo.
    Invalid,
}

/// Tela observável na simulação, sem executar widgets Flutter.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum ScreenFixture {
    /// Entrada do aplicativo.
    #[default]
    Entry,
    /// Autenticação iniciada e ainda não concluída.
    Authentication,
    /// Acesso autenticado ao aplicativo.
    Authenticated,
}

/// Mensagem esperada na especificação, sem impor o texto final da interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthMessageFixture {
    /// As credenciais foram recusadas.
    InvalidCredentials,
    /// A conta não pôde ser autenticada.
    AccountRejected,
    /// É necessária uma conta institucional da UnB.
    UnbAccountRequired,
    /// A autenticação está temporariamente indisponível.
    Unavailable,
    /// O recurso exige autenticação.
    AuthenticationRequired,
    /// O recurso exige uma sessão válida.
    InvalidSession,
}

/// Estado descartável dos exemplos de autenticação, exclusivo de cada World.
#[derive(Debug, Default)]
pub(crate) struct InstitutionalAuthFixture {
    /// Resposta escolhida pelos passos de contexto para o próximo login.
    pub(crate) prepared_login: Option<LoginFixture>,
    /// Sessão da simulação.
    pub(crate) session: SessionFixture,
    /// Tela atual da simulação.
    pub(crate) screen: ScreenFixture,
    /// Última mensagem apresentada no exemplo.
    pub(crate) message: Option<AuthMessageFixture>,
    /// Resultado do último login ou acesso a recurso protegido.
    pub(crate) access_granted: bool,
    /// Indica que a simulação oferece nova tentativa de login.
    pub(crate) retry_available: bool,
    /// Indica que o acesso protegido solicitou autenticação.
    pub(crate) login_requested: bool,
    /// Pedido de cadastro no resultado de login; `None` antes de um sucesso.
    pub(crate) registration_requested: Option<bool>,
    /// Resultado da tentativa de ação protegida; `None` antes da tentativa.
    pub(crate) action_performed: Option<bool>,
    /// Registra que o passo de saída foi executado.
    pub(crate) logout_completed: bool,
}

impl InstitutionalAuthFixture {
    /// Prepara uma resposta explícita; não infere a origem a partir do e-mail.
    pub(crate) fn prepare_login(&mut self, email: String, response: LoginResponseFixture) {
        self.prepared_login = Some(LoginFixture { email, response });
    }

    /// Consome a resposta preparada e aplica o resultado fictício do login.
    ///
    /// # Panics
    ///
    /// Se o cenário não tiver preparado uma tentativa no `Dado`.
    pub(crate) fn authenticate(&mut self) {
        let login = self.prepared_login.take().expect("prepare a login fixture");
        self.session = SessionFixture::Absent;
        self.screen = ScreenFixture::Entry;
        self.access_granted = false;
        self.retry_available = true;
        self.login_requested = false;
        self.registration_requested = None;
        self.logout_completed = false;
        self.message = match login.response {
            LoginResponseFixture::UnbAccount => {
                self.session = SessionFixture::Valid { email: login.email };
                self.screen = ScreenFixture::Authenticated;
                self.access_granted = true;
                self.retry_available = false;
                self.registration_requested = Some(false);
                None
            }
            LoginResponseFixture::InvalidCredentials => {
                Some(AuthMessageFixture::InvalidCredentials)
            }
            LoginResponseFixture::RevokedAccount => Some(AuthMessageFixture::AccountRejected),
            LoginResponseFixture::PersonalAccount | LoginResponseFixture::OtherUniversity => {
                Some(AuthMessageFixture::UnbAccountRequired)
            }
            LoginResponseFixture::Unavailable => Some(AuthMessageFixture::Unavailable),
        };
    }

    /// Simula o cancelamento e o retorno à entrada, sem criar sessão.
    ///
    /// # Panics
    ///
    /// Se o `Dado` não tiver preparado uma autenticação em andamento.
    pub(crate) fn cancel_login(&mut self) {
        assert_eq!(self.screen, ScreenFixture::Authentication);
        self.prepared_login = None;
        self.session = SessionFixture::Absent;
        self.screen = ScreenFixture::Entry;
        self.access_granted = false;
        self.retry_available = true;
        self.message = None;
    }

    /// Aplica o resultado de acesso esperado para a sessão fictícia preparada.
    pub(crate) fn access_restricted_service(&mut self) {
        self.message = match self.session {
            SessionFixture::Valid { .. } => None,
            SessionFixture::Absent => Some(AuthMessageFixture::AuthenticationRequired),
            SessionFixture::Expired | SessionFixture::Invalid => {
                Some(AuthMessageFixture::InvalidSession)
            }
        };
        self.access_granted = self.message.is_none();
        self.login_requested = !self.access_granted;
    }

    /// Registra somente o resultado simulado de uma ação; não escreve no banco.
    pub(crate) fn submit_restricted_action(&mut self) {
        self.access_restricted_service();
        self.action_performed = Some(self.access_granted);
    }

    /// Simula o encerramento da sessão local e o retorno à tela de entrada.
    ///
    /// # Panics
    ///
    /// Se o cenário não tiver preparado uma sessão válida.
    pub(crate) fn logout(&mut self) {
        assert!(matches!(self.session, SessionFixture::Valid { .. }));
        self.session = SessionFixture::Absent;
        self.screen = ScreenFixture::Entry;
        self.access_granted = false;
        self.retry_available = true;
        self.message = None;
        self.logout_completed = true;
    }
}
