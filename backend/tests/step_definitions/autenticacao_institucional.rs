//! Steps pendentes da especificação de autenticação institucional (#3).
//!
//! Os padrões Gherkin estão registrados, mas a automação ainda não foi
//! implementada. Cada step falha explicitamente com uma mensagem de pendência;
//! nenhum resultado de autenticação é simulado ou aprovado antecipadamente.
//! A implementação posterior (#11) deverá preparar os contextos, executar o
//! comportamento real do backend e verificar seus resultados.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que o usuário não está autenticado no aplicativo.
#[given("que o usuário não está autenticado no aplicativo")]
fn user_is_not_authenticated(_world: &mut AppWorld) {
    pending_step("que o usuário não está autenticado no aplicativo");
}

/// Pendente: que o estudante possui uma conta institucional ativa da UnB com o e-mail {string}.
#[given(expr = "que o estudante possui uma conta institucional ativa da UnB com o e-mail {string}")]
#[given(
    expr = "que um estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa da UnB"
)]
fn student_has_unb_account(_world: &mut AppWorld, _email: String) {
    pending_step(
        "que o estudante possui uma conta institucional ativa da UnB com o e-mail {string}",
    );
}

/// Pendente: que um usuário possui credenciais institucionais inválidas para o e-mail {string}.
#[given(expr = "que um usuário possui credenciais institucionais inválidas para o e-mail {string}")]
fn user_has_invalid_credentials(_world: &mut AppWorld, _email: String) {
    pending_step(
        "que um usuário possui credenciais institucionais inválidas para o e-mail {string}",
    );
}

/// Pendente: que o estudante possui uma conta institucional com o e-mail {string} revogada pela UnB.
#[given(
    expr = "que o estudante possui uma conta institucional com o e-mail {string} revogada pela UnB"
)]
fn student_account_revoked(_world: &mut AppWorld, _email: String) {
    pending_step(
        "que o estudante possui uma conta institucional com o e-mail {string} revogada pela UnB",
    );
}

/// Pendente: o serviço de autenticação recusa o acesso dessa conta.
#[given("o serviço de autenticação recusa o acesso dessa conta")]
fn provider_rejects_account(_world: &mut AppWorld) {
    pending_step("o serviço de autenticação recusa o acesso dessa conta");
}

/// Pendente: que o usuário possui uma conta pessoal Microsoft válida.
#[given("que o usuário possui uma conta pessoal Microsoft válida")]
fn user_has_personal_account(_world: &mut AppWorld) {
    pending_step("que o usuário possui uma conta pessoal Microsoft válida");
}

/// Pendente: que o usuário possui uma conta institucional Microsoft válida de outra universidade.
#[given("que o usuário possui uma conta institucional Microsoft válida de outra universidade")]
fn user_has_other_university_account(_world: &mut AppWorld) {
    pending_step(
        "que o usuário possui uma conta institucional Microsoft válida de outra universidade",
    );
}

/// Pendente: o serviço de autenticação confirma que a conta pertence a essa outra universidade.
#[given("o serviço de autenticação confirma que a conta pertence a essa outra universidade")]
fn provider_confirms_other_university(_world: &mut AppWorld) {
    pending_step(
        "o serviço de autenticação confirma que a conta pertence a essa outra universidade",
    );
}

/// Pendente: que o estudante iniciou o login institucional.
#[given("que o estudante iniciou o login institucional")]
fn student_started_login(_world: &mut AppWorld) {
    pending_step("que o estudante iniciou o login institucional");
}

/// Pendente: o serviço de autenticação está temporariamente indisponível.
#[given("o serviço de autenticação está temporariamente indisponível")]
fn provider_is_unavailable(_world: &mut AppWorld) {
    pending_step("o serviço de autenticação está temporariamente indisponível");
}

/// Pendente: que o estudante possui uma sessão {string}.
#[given(expr = "que o estudante possui uma sessão {string}")]
fn student_has_unusable_session(_world: &mut AppWorld, _status: String) {
    pending_step("que o estudante possui uma sessão {string}");
}

/// Pendente: que o estudante possui uma sessão autenticada válida.
#[given("que o estudante possui uma sessão autenticada válida")]
fn student_has_valid_session(_world: &mut AppWorld) {
    pending_step("que o estudante possui uma sessão autenticada válida");
}

/// Pendente: ele realiza a autenticação com suas credenciais institucionais.
#[when("ele realiza a autenticação com suas credenciais institucionais")]
#[when("ele conclui a validação de sua identidade institucional")]
#[when("ele tenta realizar o login institucional")]
#[when("ele tenta autenticar no CApp")]
#[when("ele tenta realizar o login institucional com essa conta")]
fn user_attempts_login(_world: &mut AppWorld) {
    pending_step("ele realiza a autenticação com suas credenciais institucionais");
}

/// Pendente: ele cancela a autenticação antes de concluí-la.
#[when("ele cancela a autenticação antes de concluí-la")]
fn student_cancels_login(_world: &mut AppWorld) {
    pending_step("ele cancela a autenticação antes de concluí-la");
}

/// Pendente: ele tenta acessar um serviço restrito da comunidade acadêmica.
#[when("ele tenta acessar um serviço restrito da comunidade acadêmica")]
fn user_accesses_restricted_service(_world: &mut AppWorld) {
    pending_step("ele tenta acessar um serviço restrito da comunidade acadêmica");
}

/// Pendente: ele tenta realizar uma ação que exige autenticação.
#[when("ele tenta realizar uma ação que exige autenticação")]
fn user_attempts_restricted_action(_world: &mut AppWorld) {
    pending_step("ele tenta realizar uma ação que exige autenticação");
}

/// Pendente: ele solicita sair do CApp.
#[when("ele solicita sair do CApp")]
fn student_logs_out(_world: &mut AppWorld) {
    pending_step("ele solicita sair do CApp");
}

/// Pendente: o acesso à plataforma é concedido com sucesso.
#[then("o acesso à plataforma é concedido com sucesso")]
fn access_is_granted(_world: &mut AppWorld) {
    pending_step("o acesso à plataforma é concedido com sucesso");
}

/// Pendente: uma sessão autenticada é estabelecida para o estudante.
#[then("uma sessão autenticada é estabelecida para o estudante")]
#[then("o sistema estabelece a sessão autenticada do usuário")]
fn session_is_established(_world: &mut AppWorld) {
    pending_step("uma sessão autenticada é estabelecida para o estudante");
}

/// Pendente: a identidade autenticada do usuário fica disponível com o e-mail {string}.
#[then(expr = "a identidade autenticada do usuário fica disponível com o e-mail {string}")]
fn authenticated_identity_is_available(_world: &mut AppWorld, _email: String) {
    pending_step("a identidade autenticada do usuário fica disponível com o e-mail {string}");
}

/// Pendente: não é solicitado cadastro manual para entrar no CApp.
#[then("não é solicitado cadastro manual para entrar no CApp")]
fn manual_registration_is_not_requested(_world: &mut AppWorld) {
    pending_step("não é solicitado cadastro manual para entrar no CApp");
}

/// Pendente: a autenticação é recusada indicando credenciais inválidas.
#[then("a autenticação é recusada indicando credenciais inválidas")]
fn invalid_credentials_are_rejected(_world: &mut AppWorld) {
    pending_step("a autenticação é recusada indicando credenciais inválidas");
}

/// Pendente: o aplicativo informa que não foi possível autenticar a conta institucional.
#[then("o aplicativo informa que não foi possível autenticar a conta institucional")]
fn account_rejection_is_reported(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que não foi possível autenticar a conta institucional");
}

/// Pendente: o aplicativo informa que é necessário utilizar uma conta institucional da UnB.
#[then("o aplicativo informa que é necessário utilizar uma conta institucional da UnB")]
fn unb_account_is_required(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que é necessário utilizar uma conta institucional da UnB");
}

/// Pendente: o aplicativo informa a indisponibilidade temporária da autenticação.
#[then("o aplicativo informa a indisponibilidade temporária da autenticação")]
fn provider_unavailability_is_reported(_world: &mut AppWorld) {
    pending_step("o aplicativo informa a indisponibilidade temporária da autenticação");
}

/// Pendente: nenhuma sessão autenticada é estabelecida.
#[then("nenhuma sessão autenticada é estabelecida")]
#[then("o acesso à plataforma permanece bloqueado")]
fn no_session_is_established(_world: &mut AppWorld) {
    pending_step("nenhuma sessão autenticada é estabelecida");
}

/// Pendente: o aplicativo retorna à tela de entrada.
#[then("o aplicativo retorna à tela de entrada")]
fn entry_screen_is_displayed(_world: &mut AppWorld) {
    pending_step("o aplicativo retorna à tela de entrada");
}

/// Pendente: o estudante pode iniciar uma nova tentativa de login.
#[then("o estudante pode iniciar uma nova tentativa de login")]
fn retry_is_available(_world: &mut AppWorld) {
    pending_step("o estudante pode iniciar uma nova tentativa de login");
}

/// Pendente: o acesso ao recurso é bloqueado por ausência de autenticação.
#[then("o acesso ao recurso é bloqueado por ausência de autenticação")]
#[then("a operação é impedida por ausência de autenticação")]
fn unauthenticated_access_is_blocked(_world: &mut AppWorld) {
    pending_step("o acesso ao recurso é bloqueado por ausência de autenticação");
}

/// Pendente: o acesso ao recurso é bloqueado por falta de uma sessão válida.
#[then("o acesso ao recurso é bloqueado por falta de uma sessão válida")]
fn invalid_session_access_is_blocked(_world: &mut AppWorld) {
    pending_step("o acesso ao recurso é bloqueado por falta de uma sessão válida");
}

/// Pendente: o aplicativo solicita que o usuário realize a autenticação institucional.
#[then("o aplicativo solicita que o usuário realize a autenticação institucional")]
fn authentication_is_requested(_world: &mut AppWorld) {
    pending_step("o aplicativo solicita que o usuário realize a autenticação institucional");
}

/// Pendente: a ação solicitada não é realizada.
#[then("a ação solicitada não é realizada")]
fn restricted_action_is_not_performed(_world: &mut AppWorld) {
    pending_step("a ação solicitada não é realizada");
}

/// Pendente: a sessão do estudante é encerrada no aplicativo.
#[then("a sessão do estudante é encerrada no aplicativo")]
fn session_is_closed(_world: &mut AppWorld) {
    pending_step("a sessão do estudante é encerrada no aplicativo");
}

/// Pendente: o acesso a recursos restritos passa a exigir nova autenticação.
#[then("o acesso a recursos restritos passa a exigir nova autenticação")]
fn protected_access_requires_new_login(_world: &mut AppWorld) {
    pending_step("o acesso a recursos restritos passa a exigir nova autenticação");
}
