//! Implementação dos steps de BDD para autenticação institucional da UnB.
//!
//! Os passos operam sobre [`AppWorld`], interagindo com a fixture de
//! autenticação institucional para validar os cenários da entrega incremental I01
//! em `features/identidade_e_acesso/autenticacao_institucional.feature`:
//!
//! - Login bem-sucedido com conta institucional ativa da UnB (@aluno.unb.br).
//! - Primeiro acesso discente sem exigência de formulário manual de cadastro.
//! - Recusa de contas pessoais Microsoft e de outras universidades.
//! - Bloqueio de recursos restritos e ações protegidas para usuários não autenticados
//!   ou com sessões inválidas/expiradas.
//! - Suporte completo a variações gramaticais de conjunções ("E") e sujeito oculto.

use cucumber::{given, then, when};

use crate::support::{
    AppWorld,
    world::{AccountKind, AuthMessage, ScreenState, SessionState, is_valid_unb_student_email},
};

/// Contextualiza que o usuário não possui sessão autenticada no aplicativo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário não está autenticado no aplicativo")]
#[given("o usuário não está autenticado no aplicativo")]
#[given("que o estudante não está autenticado no aplicativo")]
#[given("o estudante não está autenticado no aplicativo")]
#[given("que o usuário não possui sessão autenticada")]
#[given("o usuário não possui sessão autenticada")]
#[given("que não está autenticado no aplicativo")]
#[given("não está autenticado no aplicativo")]
fn user_is_not_authenticated(world: &mut AppWorld) {
    world.institutional_auth.reset();
}

/// Contextualiza uma conta institucional ativa da UnB para o estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: e-mail institucional do estudante.
#[given(expr = "que o estudante possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(expr = "o estudante possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(
    expr = "que um estudante possui uma conta institucional ativa da UnB com o e-mail {string}"
)]
#[given(expr = "um estudante possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(expr = "que possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(expr = "possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(
    expr = "que um estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa da UnB"
)]
#[given(
    expr = "um estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa da UnB"
)]
#[given(
    expr = "que o estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa da UnB"
)]
#[given(
    expr = "o estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa da UnB"
)]
fn student_has_unb_account(world: &mut AppWorld, email: String) {
    let first_access = email.contains("262001234");
    world
        .institutional_auth
        .set_unb_account(&email, first_access);
}

/// Contextualiza credenciais institucionais inválidas para o e-mail informado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: e-mail alvo da tentativa.
#[given(expr = "que um usuário possui credenciais institucionais inválidas para o e-mail {string}")]
#[given(expr = "um usuário possui credenciais institucionais inválidas para o e-mail {string}")]
#[given(expr = "que o usuário possui credenciais institucionais inválidas para o e-mail {string}")]
#[given(expr = "o usuário possui credenciais institucionais inválidas para o e-mail {string}")]
#[given(expr = "que possui credenciais institucionais inválidas para o e-mail {string}")]
#[given(expr = "possui credenciais institucionais inválidas para o e-mail {string}")]
fn user_has_invalid_credentials(world: &mut AppWorld, email: String) {
    world.institutional_auth.set_invalid_credentials(&email);
}

/// Contextualiza uma conta institucional revogada pela UnB.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: e-mail da conta revogada.
#[given(
    expr = "que o estudante possui uma conta institucional com o e-mail {string} revogada pela UnB"
)]
#[given(
    expr = "o estudante possui uma conta institucional com o e-mail {string} revogada pela UnB"
)]
#[given(
    expr = "que um estudante possui uma conta institucional com o e-mail {string} revogada pela UnB"
)]
#[given(
    expr = "um estudante possui uma conta institucional com o e-mail {string} revogada pela UnB"
)]
#[given(expr = "que possui uma conta institucional com o e-mail {string} revogada pela UnB")]
#[given(expr = "possui uma conta institucional com o e-mail {string} revogada pela UnB")]
fn student_account_revoked(world: &mut AppWorld, email: String) {
    world.institutional_auth.set_revoked_unb_account(&email);
}

/// Verifica que o serviço de autenticação recusa a conta institucional revogada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("o serviço de autenticação recusa o acesso dessa conta")]
#[given("que o serviço de autenticação recusa o acesso dessa conta")]
#[given("o serviço de autenticação recusa o acesso da conta")]
#[given("que o serviço de autenticação recusa o acesso da conta")]
fn provider_rejects_account(world: &mut AppWorld) {
    assert!(
        matches!(
            world.institutional_auth.prepared_account,
            Some(AccountKind::RevokedUnbAccount { .. })
        ),
        "a conta preparada deve ser uma conta revogada"
    );
}

/// Contextualiza conta pessoal Microsoft do usuário.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário possui uma conta pessoal Microsoft válida")]
#[given("o usuário possui uma conta pessoal Microsoft válida")]
#[given("que possui uma conta pessoal Microsoft válida")]
#[given("possui uma conta pessoal Microsoft válida")]
#[given("que um usuário possui uma conta pessoal Microsoft válida")]
#[given("um usuário possui uma conta pessoal Microsoft válida")]
fn user_has_personal_account(world: &mut AppWorld) {
    world.institutional_auth.set_personal_microsoft_account();
}

/// Contextualiza conta institucional pertencente a outra universidade.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário possui uma conta institucional Microsoft válida de outra universidade")]
#[given("o usuário possui uma conta institucional Microsoft válida de outra universidade")]
#[given("que possui uma conta institucional Microsoft válida de outra universidade")]
#[given("possui uma conta institucional Microsoft válida de outra universidade")]
#[given("que um usuário possui uma conta institucional Microsoft válida de outra universidade")]
#[given("um usuário possui uma conta institucional Microsoft válida de outra universidade")]
fn user_has_other_university_account(world: &mut AppWorld) {
    world.institutional_auth.set_other_university_account();
}

/// Registra a confirmação pelo provedor de que a conta pertence a outra universidade.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("o serviço de autenticação confirma que a conta pertence a essa outra universidade")]
#[given("que o serviço de autenticação confirma que a conta pertence a essa outra universidade")]
#[given("o serviço de autenticação confirma que a conta pertence a outra universidade")]
#[given("que o serviço de autenticação confirma que a conta pertence a outra universidade")]
fn provider_confirms_other_university(world: &mut AppWorld) {
    world.institutional_auth.confirm_other_university();
}

/// Registra o início do fluxo de login institucional.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o estudante iniciou o login institucional")]
#[given("o estudante iniciou o login institucional")]
#[given("que ele iniciou o login institucional")]
#[given("ele iniciou o login institucional")]
#[given("que iniciou o login institucional")]
#[given("iniciou o login institucional")]
fn student_started_login(world: &mut AppWorld) {
    world.institutional_auth.start_login();
}

/// Simula indisponibilidade temporária do serviço de autenticação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("o serviço de autenticação está temporariamente indisponível")]
#[given("que o serviço de autenticação está temporariamente indisponível")]
#[given("o serviço de autenticação encontra-se temporariamente indisponível")]
#[given("que o serviço de autenticação encontra-se temporariamente indisponível")]
fn provider_is_unavailable(world: &mut AppWorld) {
    world.institutional_auth.set_provider_unavailable();
}

/// Contextualiza uma sessão em situação sem validade (expirada ou inválida).
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `status`: texto descrevendo a situação da sessão.
#[given(expr = "que o estudante possui uma sessão {string}")]
#[given(expr = "o estudante possui uma sessão {string}")]
#[given(expr = "que ele possui uma sessão {string}")]
#[given(expr = "ele possui uma sessão {string}")]
#[given(expr = "que possui uma sessão {string}")]
#[given(expr = "possui uma sessão {string}")]
#[given(expr = "que o usuário possui uma sessão {string}")]
#[given(expr = "o usuário possui uma sessão {string}")]
fn student_has_unusable_session(world: &mut AppWorld, status: String) {
    world.institutional_auth.set_session_state(&status);
}

/// Contextualiza uma sessão autenticada ativa e válida para o estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o estudante possui uma sessão autenticada válida")]
#[given("o estudante possui uma sessão autenticada válida")]
#[given("que ele possui uma sessão autenticada válida")]
#[given("ele possui uma sessão autenticada válida")]
#[given("que possui uma sessão autenticada válida")]
#[given("possui uma sessão autenticada válida")]
#[given("que o usuário possui uma sessão autenticada válida")]
#[given("o usuário possui uma sessão autenticada válida")]
fn student_has_valid_session(world: &mut AppWorld) {
    world.institutional_auth.set_valid_session(None);
}

/// Executa a tentativa de autenticação institucional com a conta configurada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele realiza a autenticação com suas credenciais institucionais")]
#[when("realiza a autenticação com suas credenciais institucionais")]
#[when("o estudante realiza a autenticação com suas credenciais institucionais")]
#[when("ele conclui a validação de sua identidade institucional")]
#[when("conclui a validação de sua identidade institucional")]
#[when("o estudante conclui a validação de sua identidade institucional")]
#[when("ele tenta realizar o login institucional")]
#[when("tenta realizar o login institucional")]
#[when("o usuário tenta realizar o login institucional")]
#[when("ele tenta autenticar no CApp")]
#[when("tenta autenticar no CApp")]
#[when("o estudante tenta autenticar no CApp")]
#[when("ele tenta realizar o login institucional com essa conta")]
#[when("tenta realizar o login institucional com essa conta")]
#[when("o usuário tenta realizar o login institucional com essa conta")]
fn user_attempts_login(world: &mut AppWorld) {
    world.institutional_auth.authenticate();
}

/// Simula o cancelamento do login antes de sua conclusão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele cancela a autenticação antes de concluí-la")]
#[when("cancela a autenticação antes de concluí-la")]
#[when("o estudante cancela a autenticação antes de concluí-la")]
fn student_cancels_login(world: &mut AppWorld) {
    world.institutional_auth.cancel_login();
}

/// Tenta acessar um serviço restrito da comunidade acadêmica.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele tenta acessar um serviço restrito da comunidade acadêmica")]
#[when("tenta acessar um serviço restrito da comunidade acadêmica")]
#[when("o estudante tenta acessar um serviço restrito da comunidade acadêmica")]
#[when("o usuário tenta acessar um serviço restrito da comunidade acadêmica")]
fn user_accesses_restricted_service(world: &mut AppWorld) {
    world.institutional_auth.access_restricted_service();
}

/// Tenta submeter uma ação protegida que exige autenticação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele tenta realizar uma ação que exige autenticação")]
#[when("tenta realizar uma ação que exige autenticação")]
#[when("o estudante tenta realizar uma ação que exige autenticação")]
#[when("o usuário tenta realizar uma ação que exige autenticação")]
fn user_attempts_restricted_action(world: &mut AppWorld) {
    world.institutional_auth.perform_restricted_action();
}

/// Solicita o encerramento da sessão ativa no CApp.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele solicita sair do CApp")]
#[when("solicita sair do CApp")]
#[when("o estudante solicita sair do CApp")]
#[when("o usuário solicita sair do CApp")]
fn student_logs_out(world: &mut AppWorld) {
    world.institutional_auth.logout();
}

/// Confere se o acesso à plataforma foi concedido com sucesso.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o acesso não tiver sido concedido.
#[then("o acesso à plataforma é concedido com sucesso")]
#[then("o acesso a plataforma é concedido com sucesso")]
fn access_is_granted(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.access_granted,
        "o acesso à plataforma deveria ter sido concedido com sucesso"
    );
}

/// Confere se uma sessão autenticada foi estabelecida.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a sessão não estiver ativa e válida.
#[then("uma sessão autenticada é estabelecida para o estudante")]
#[then("o sistema estabelece a sessão autenticada do usuário")]
#[then("o sistema estabelece a sessão autenticada para o estudante")]
#[then("uma sessão autenticada é estabelecida para o usuário")]
#[then("uma sessão autenticada é estabelecida")]
fn session_is_established(world: &mut AppWorld) {
    assert!(
        matches!(world.institutional_auth.session, SessionState::Valid { .. }),
        "uma sessão autenticada válida deveria ter sido estabelecida"
    );
}

/// Confere se a identidade autenticada do usuário está disponível com o e-mail informado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `expected_email`: endereço de e-mail institucional esperado.
///
/// # Panics
///
/// Se o e-mail da identidade autenticada for diferente do esperado.
#[then(expr = "a identidade autenticada do usuário fica disponível com o e-mail {string}")]
#[then(expr = "a identidade autenticada do estudante fica disponível com o e-mail {string}")]
#[then(expr = "a identidade autenticada fica disponível com o e-mail {string}")]
fn authenticated_identity_is_available(world: &mut AppWorld, expected_email: String) {
    assert_eq!(
        world.institutional_auth.authenticated_email.as_deref(),
        Some(expected_email.as_str()),
        "o e-mail da identidade autenticada difere do esperado"
    );
    assert!(
        is_valid_unb_student_email(&expected_email),
        "o e-mail esperado deve cumprir o padrão institucional da UnB"
    );
}

/// Confere que o login institucional dispensou formulário manual de cadastro.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se tiver sido solicitado cadastro manual.
#[then("não é solicitado cadastro manual para entrar no CApp")]
#[then("o sistema não solicita cadastro manual para entrar no CApp")]
#[then("não solicita cadastro manual para entrar no CApp")]
fn manual_registration_is_not_requested(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.manual_registration_required,
        Some(false),
        "o cadastro manual não deveria ter sido solicitado no login institucional"
    );
}

/// Confere que a autenticação foi recusada por credenciais inválidas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a mensagem observada não indicar credenciais inválidas.
#[then("a autenticação é recusada indicando credenciais inválidas")]
#[then("a autenticação é recusada por credenciais inválidas")]
fn invalid_credentials_are_rejected(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.message,
        Some(AuthMessage::InvalidCredentials),
        "a autenticação deveria ser recusada indicando credenciais inválidas"
    );
    assert!(!world.institutional_auth.access_granted);
}

/// Confere mensagem amigável ao recusar conta institucional revogada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a mensagem observada não indicar falha na autenticação da conta.
#[then("o aplicativo informa que não foi possível autenticar a conta institucional")]
#[then("informa que não foi possível autenticar a conta institucional")]
fn account_rejection_is_reported(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.message,
        Some(AuthMessage::AccountRejected),
        "o aplicativo deveria informar que não foi possível autenticar a conta institucional"
    );
    assert!(!world.institutional_auth.access_granted);
}

/// Confere a mensagem amigável de exigência exclusiva de conta institucional da UnB.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a mensagem observada não indicar a exigência de conta da UnB.
#[then("o aplicativo informa que é necessário utilizar uma conta institucional da UnB")]
#[then("informa que é necessário utilizar uma conta institucional da UnB")]
fn unb_account_is_required(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.message,
        Some(AuthMessage::UnbAccountRequired),
        "o aplicativo deveria informar que é necessário utilizar conta institucional da UnB"
    );
    assert!(!world.institutional_auth.access_granted);
}

/// Confere mensagem amigável de indisponibilidade temporária do serviço de autenticação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a mensagem observada não indicar indisponibilidade temporária.
#[then("o aplicativo informa a indisponibilidade temporária da autenticação")]
#[then("informa a indisponibilidade temporária da autenticação")]
fn provider_unavailability_is_reported(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.message,
        Some(AuthMessage::Unavailable),
        "o aplicativo deveria informar a indisponibilidade temporária da autenticação"
    );
    assert!(!world.institutional_auth.access_granted);
}

/// Confere que nenhuma sessão autenticada foi criada e o acesso permanece bloqueado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se uma sessão foi criada ou o acesso não permanecer bloqueado.
#[then("nenhuma sessão autenticada é estabelecida")]
#[then("o acesso à plataforma permanece bloqueado")]
#[then("o acesso a plataforma permanece bloqueado")]
fn no_session_is_established(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.session,
        SessionState::Absent,
        "nenhuma sessão autenticada deveria ter sido estabelecida"
    );
    assert!(
        !world.institutional_auth.access_granted,
        "o acesso à plataforma deveria permanecer bloqueado"
    );
}

/// Confere o retorno do aplicativo à tela de entrada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a tela atual observada não for a tela de entrada.
#[then("o aplicativo retorna à tela de entrada")]
#[then("retorna à tela de entrada")]
#[then("o aplicativo retorna a tela de entrada")]
fn entry_screen_is_displayed(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.screen,
        ScreenState::Entry,
        "o aplicativo deveria ter retornado à tela de entrada"
    );
}

/// Confere que uma nova tentativa de login institucional está disponível.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se uma nova tentativa não estiver disponível ou a tela for diferente da entrada.
#[then("o estudante pode iniciar uma nova tentativa de login")]
#[then("ele pode iniciar uma nova tentativa de login")]
#[then("o usuário pode iniciar uma nova tentativa de login")]
#[then("pode iniciar uma nova tentativa de login")]
fn retry_is_available(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.retry_available,
        "o estudante deveria poder iniciar uma nova tentativa de login"
    );
    assert_eq!(
        world.institutional_auth.screen,
        ScreenState::Entry,
        "o aplicativo deveria estar na tela de entrada para uma nova tentativa"
    );
}

/// Confere que o acesso ao recurso restrito ou ação foi bloqueado por falta de autenticação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o acesso restrito não tiver sido bloqueado por ausência de autenticação.
#[then("o acesso ao recurso é bloqueado por ausência de autenticação")]
#[then("a operação é impedida por ausência de autenticação")]
#[then("o acesso ao recurso é bloqueado por falta de autenticação")]
#[then("a operação é impedida por falta de autenticação")]
fn unauthenticated_access_is_blocked(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.protected_access_blocked,
        "o acesso ao recurso restrito deveria ter sido bloqueado"
    );
    assert_eq!(
        world.institutional_auth.message,
        Some(AuthMessage::AuthenticationRequired),
        "a mensagem observada deveria indicar ausência de autenticação"
    );
}

/// Confere que o acesso foi bloqueado devido a uma sessão inválida ou expirada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o bloqueio por sessão sem validade não foi acionado.
#[then("o acesso ao recurso é bloqueado por falta de uma sessão válida")]
#[then("o acesso ao recurso é bloqueado por ausência de uma sessão válida")]
fn invalid_session_access_is_blocked(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.protected_access_blocked,
        "o acesso deveria ser bloqueado por falta de uma sessão válida"
    );
    assert_eq!(
        world.institutional_auth.message,
        Some(AuthMessage::InvalidSession),
        "a mensagem observada deveria indicar sessão inválida ou expirada"
    );
}

/// Confere que o aplicativo solicitou que o usuário realize autenticação institucional.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a autenticação institucional não tiver sido solicitada.
#[then("o aplicativo solicita que o usuário realize a autenticação institucional")]
#[then("solicita que o usuário realize a autenticação institucional")]
#[then("o aplicativo solicita que o estudante realize a autenticação institucional")]
#[then("solicita que o estudante realize a autenticação institucional")]
fn authentication_is_requested(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.login_requested,
        "o aplicativo deveria solicitar a autenticação institucional"
    );
}

/// Confere que a ação restrita solicitada não foi realizada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a ação protegida tiver sido realizada indevidamente.
#[then("a ação solicitada não é realizada")]
#[then("a operação solicitada não é realizada")]
fn restricted_action_is_not_performed(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_auth.action_performed,
        Some(false),
        "a ação solicitada não deveria ser realizada sem autenticação"
    );
}

/// Confere que a sessão do estudante foi encerrada no aplicativo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a sessão não tiver sido encerrada.
#[then("a sessão do estudante é encerrada no aplicativo")]
#[then("a sessão do usuário é encerrada no aplicativo")]
#[then("a sessão é encerrada no aplicativo")]
fn session_is_closed(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.logout_completed,
        "a sessão do estudante deveria ser encerrada no aplicativo"
    );
    assert_eq!(
        world.institutional_auth.session,
        SessionState::Absent,
        "nenhuma sessão deve permanecer ativa após o encerramento"
    );
}

/// Confere que o acesso a recursos restritos volta a exigir nova autenticação após logout.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se recursos restritos ainda puderem ser acessados sem nova autenticação.
#[then("o acesso a recursos restritos passa a exigir nova autenticação")]
#[then("o acesso a recursos protegidos passa a exigir nova autenticação")]
#[then("o acesso aos recursos restritos passa a exigir nova autenticação")]
fn protected_access_requires_new_login(world: &mut AppWorld) {
    assert!(
        world.institutional_auth.logout_completed,
        "a sessão anterior deve estar encerrada"
    );
    assert_eq!(
        world.institutional_auth.session,
        SessionState::Absent,
        "não deve haver sessão ativa disponível"
    );
    assert!(
        !world.institutional_auth.access_granted,
        "o acesso restrito deve exigir uma nova autenticação"
    );
}
