//! Steps provisórios da especificação de autenticação institucional (#3).
//!
//! Os `Dado` escolhem fixtures, os `Quando` aplicam resultados predeterminados
//! e os `Então` conferem a simulação. Estes cenários passam sem implementação
//! de login, chamadas Microsoft/Firebase ou persistência de usuários.
//! Na automação posterior (#11), substituir a simulação por chamadas à
//! implementação real e verificações dos seus resultados.

use cucumber::{given, then, when};

use crate::support::{
    AppWorld,
    authentication_fixture::{
        AuthMessageFixture, InstitutionalAuthFixture, LoginResponseFixture, ScreenFixture,
        SessionFixture,
    },
};

/// Inicia cada exemplo com uma fixture própria, sem autenticação anterior.
#[given("que o usuário não está autenticado no aplicativo")]
fn user_is_not_authenticated(world: &mut AppWorld) {
    world.authentication = InstitutionalAuthFixture::default();
}

/// Prepara o sucesso de uma conta da UnB, inclusive no primeiro acesso.
#[given(expr = "que o estudante possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(
    expr = "que um estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa da UnB"
)]
fn student_has_unb_account(world: &mut AppWorld, email: String) {
    world
        .authentication
        .prepare_login(email, LoginResponseFixture::UnbAccount);
}

/// Prepara a recusa de credenciais, sem consultar um provedor de identidade.
#[given(expr = "que um usuário possui credenciais institucionais inválidas para o e-mail {string}")]
fn user_has_invalid_credentials(world: &mut AppWorld, email: String) {
    world
        .authentication
        .prepare_login(email, LoginResponseFixture::InvalidCredentials);
}

/// Prepara a recusa de uma conta revogada, sem inferir situação acadêmica.
#[given(
    expr = "que o estudante possui uma conta institucional com o e-mail {string} revogada pela UnB"
)]
fn student_account_revoked(world: &mut AppWorld, email: String) {
    world
        .authentication
        .prepare_login(email, LoginResponseFixture::RevokedAccount);
}

/// Confirma que a recusa da conta foi preparada no contexto deste cenário.
#[given("o serviço de autenticação recusa o acesso dessa conta")]
fn provider_rejects_account(world: &mut AppWorld) {
    assert_eq!(
        world
            .authentication
            .prepared_login
            .as_ref()
            .expect("prepare a revoked account")
            .response,
        LoginResponseFixture::RevokedAccount,
    );
}

/// Prepara uma conta pessoal Microsoft, sem autorização para o CApp da UnB.
#[given("que o usuário possui uma conta pessoal Microsoft válida")]
fn user_has_personal_account(world: &mut AppWorld) {
    world.authentication.prepare_login(
        "pessoa@example.com".to_owned(),
        LoginResponseFixture::PersonalAccount,
    );
}

/// Prepara a identidade de outra universidade, sem catálogo de instituições.
#[given("que o usuário possui uma conta institucional Microsoft válida de outra universidade")]
fn user_has_other_university_account(world: &mut AppWorld) {
    world.authentication.prepare_login(
        "estudante@outra.example".to_owned(),
        LoginResponseFixture::OtherUniversity,
    );
}

/// Confirma a origem já definida na fixture, independentemente do e-mail.
#[given("o serviço de autenticação confirma que a conta pertence a essa outra universidade")]
fn provider_confirms_other_university(world: &mut AppWorld) {
    assert_eq!(
        world
            .authentication
            .prepared_login
            .as_ref()
            .expect("prepare another university account")
            .response,
        LoginResponseFixture::OtherUniversity,
    );
}

/// Prepara o fluxo ainda aberto que poderá ser cancelado pelo estudante.
#[given("que o estudante iniciou o login institucional")]
fn student_started_login(world: &mut AppWorld) {
    world.authentication.screen = ScreenFixture::Authentication;
}

/// Substitui a resposta preparada por indisponibilidade, sem rede ou espera.
#[given("o serviço de autenticação está temporariamente indisponível")]
fn provider_is_unavailable(world: &mut AppWorld) {
    world
        .authentication
        .prepared_login
        .as_mut()
        .expect("prepare a login attempt")
        .response = LoginResponseFixture::Unavailable;
}

/// Prepara os estados sem validade usados nos exemplos do esquema de cenário.
///
/// # Panics
///
/// Se o exemplo pedir uma situação não especificada nesta fixture.
#[given(expr = "que o estudante possui uma sessão {string}")]
fn student_has_unusable_session(world: &mut AppWorld, status: String) {
    world.authentication.session = match status.as_str() {
        "expirada sem possibilidade de renovação" => SessionFixture::Expired,
        "inválida" => SessionFixture::Invalid,
        _ => panic!("situação de sessão sem fixture: {status}"),
    };
}

/// Prepara uma sessão da UnB para especificar o encerramento no aplicativo.
#[given("que o estudante possui uma sessão autenticada válida")]
fn student_has_valid_session(world: &mut AppWorld) {
    world.authentication.session = SessionFixture::Valid {
        email: "232012345@aluno.unb.br".to_owned(),
    };
    world.authentication.screen = ScreenFixture::Authenticated;
    world.authentication.access_granted = true;
}

/// Aplica a resposta de login predeterminada pelos passos de contexto.
#[when("ele realiza a autenticação com suas credenciais institucionais")]
#[when("ele conclui a validação de sua identidade institucional")]
#[when("ele tenta realizar o login institucional")]
#[when("ele tenta autenticar no CApp")]
#[when("ele tenta realizar o login institucional com essa conta")]
fn user_attempts_login(world: &mut AppWorld) {
    world.authentication.authenticate();
}

/// Simula o cancelamento do fluxo antes da criação da sessão.
#[when("ele cancela a autenticação antes de concluí-la")]
fn student_cancels_login(world: &mut AppWorld) {
    world.authentication.cancel_login();
}

/// Aplica o resultado de consulta correspondente à sessão fictícia.
#[when("ele tenta acessar um serviço restrito da comunidade acadêmica")]
fn user_accesses_restricted_service(world: &mut AppWorld) {
    world.authentication.access_restricted_service();
}

/// Simula uma ação protegida sem executar operações reais no banco.
#[when("ele tenta realizar uma ação que exige autenticação")]
fn user_attempts_restricted_action(world: &mut AppWorld) {
    world.authentication.submit_restricted_action();
}

/// Simula a saída da sessão local, sem revogar tokens em serviços externos.
#[when("ele solicita sair do CApp")]
fn student_logs_out(world: &mut AppWorld) {
    world.authentication.logout();
}

/// Confere que a resposta simulada concedeu acesso sem mensagem de erro.
#[then("o acesso à plataforma é concedido com sucesso")]
fn access_is_granted(world: &mut AppWorld) {
    assert!(world.authentication.access_granted);
    assert_eq!(world.authentication.message, None);
}

/// Confere a sessão criada pelo resultado de sucesso da fixture.
#[then("uma sessão autenticada é estabelecida para o estudante")]
#[then("o sistema estabelece a sessão autenticada do usuário")]
fn session_is_established(world: &mut AppWorld) {
    assert!(matches!(
        world.authentication.session,
        SessionFixture::Valid { .. }
    ));
    assert_eq!(world.authentication.screen, ScreenFixture::Authenticated);
}

/// Confere que a identidade simulada mantém o e-mail usado no exemplo.
#[then(expr = "a identidade autenticada do usuário fica disponível com o e-mail {string}")]
fn authenticated_identity_is_available(world: &mut AppWorld, email: String) {
    assert_eq!(
        world.authentication.session,
        SessionFixture::Valid { email }
    );
}

/// Confere a ausência de cadastro manual no resultado preparado de sucesso.
#[then("não é solicitado cadastro manual para entrar no CApp")]
fn manual_registration_is_not_requested(world: &mut AppWorld) {
    assert_eq!(world.authentication.registration_requested, Some(false));
}

/// Confere a mensagem e a ausência de acesso no resultado simulado de recusa.
fn assert_login_rejected(world: &AppWorld, message: AuthMessageFixture) {
    assert_eq!(world.authentication.message, Some(message));
    assert!(!world.authentication.access_granted);
    assert_eq!(world.authentication.session, SessionFixture::Absent);
}

/// Confere a mensagem correspondente à fixture de credenciais inválidas.
#[then("a autenticação é recusada indicando credenciais inválidas")]
fn invalid_credentials_are_rejected(world: &mut AppWorld) {
    assert_login_rejected(world, AuthMessageFixture::InvalidCredentials);
}

/// Confere a recusa da conta sem afirmar que o vínculo acadêmico está inativo.
#[then("o aplicativo informa que não foi possível autenticar a conta institucional")]
fn account_rejection_is_reported(world: &mut AppWorld) {
    assert_login_rejected(world, AuthMessageFixture::AccountRejected);
}

/// Confere a restrição à UnB para contas pessoais ou de outra universidade.
#[then("o aplicativo informa que é necessário utilizar uma conta institucional da UnB")]
fn unb_account_is_required(world: &mut AppWorld) {
    assert_login_rejected(world, AuthMessageFixture::UnbAccountRequired);
}

/// Confere a mensagem de indisponibilidade preparada no cenário.
#[then("o aplicativo informa a indisponibilidade temporária da autenticação")]
fn provider_unavailability_is_reported(world: &mut AppWorld) {
    assert_login_rejected(world, AuthMessageFixture::Unavailable);
}

/// Confere que a simulação não criou sessão nem concedeu acesso.
#[then("nenhuma sessão autenticada é estabelecida")]
#[then("o acesso à plataforma permanece bloqueado")]
fn no_session_is_established(world: &mut AppWorld) {
    assert_eq!(world.authentication.session, SessionFixture::Absent);
    assert!(!world.authentication.access_granted);
}

/// Confere o retorno à entrada, sem executar uma interface Flutter.
#[then("o aplicativo retorna à tela de entrada")]
fn entry_screen_is_displayed(world: &mut AppWorld) {
    assert_eq!(world.authentication.screen, ScreenFixture::Entry);
}

/// Confere que o resultado simulado disponibiliza outra tentativa de entrada.
#[then("o estudante pode iniciar uma nova tentativa de login")]
fn retry_is_available(world: &mut AppWorld) {
    assert!(world.authentication.retry_available);
    assert_eq!(world.authentication.screen, ScreenFixture::Entry);
}

/// Confere a recusa de consulta ou ação realizada sem autenticação.
#[then("o acesso ao recurso é bloqueado por ausência de autenticação")]
#[then("a operação é impedida por ausência de autenticação")]
fn unauthenticated_access_is_blocked(world: &mut AppWorld) {
    assert_login_rejected(world, AuthMessageFixture::AuthenticationRequired);
}

/// Confere que a recusa decorreu da sessão expirada ou inválida preparada.
#[then("o acesso ao recurso é bloqueado por falta de uma sessão válida")]
fn invalid_session_access_is_blocked(world: &mut AppWorld) {
    assert_eq!(
        world.authentication.message,
        Some(AuthMessageFixture::InvalidSession)
    );
    assert!(!world.authentication.access_granted);
}

/// Confere a solicitação de login no resultado do acesso protegido.
#[then("o aplicativo solicita que o usuário realize a autenticação institucional")]
fn authentication_is_requested(world: &mut AppWorld) {
    assert!(world.authentication.login_requested);
}

/// Confere o resultado negativo da tentativa de ação, sem presumir persistência.
#[then("a ação solicitada não é realizada")]
fn restricted_action_is_not_performed(world: &mut AppWorld) {
    assert_eq!(world.authentication.action_performed, Some(false));
}

/// Confere que a saída foi executada e removeu a sessão da simulação.
#[then("a sessão do estudante é encerrada no aplicativo")]
fn session_is_closed(world: &mut AppWorld) {
    assert!(world.authentication.logout_completed);
    assert_eq!(world.authentication.session, SessionFixture::Absent);
}

/// Confere o estado sem acesso autenticado após a saída local.
#[then("o acesso a recursos restritos passa a exigir nova autenticação")]
fn protected_access_requires_new_login(world: &mut AppWorld) {
    assert!(world.authentication.logout_completed);
    assert_eq!(world.authentication.session, SessionFixture::Absent);
    assert!(!world.authentication.access_granted);
}
