//! Implementação dos steps de BDD para autenticação institucional no CApp.
//!
//! Os passos operam sobre [`AppWorld`], interagindo com a fixture de autenticação
//! para verificar cenários de login com conta institucional ativa, tratamento de
//! credenciais inválidas ou revogadas e restrição de acesso a recursos protegidos
//! para usuários não autenticados, conforme a especificação em
//! `features/identidade_e_acesso/autenticacao_institucional.feature`.

use cucumber::{given, then, when};

use crate::support::{
    AppWorld,
    world::{
        DEFAULT_INSTITUTION_NAME, InstitutionStatus, InstitutionalAccount,
        InstitutionalAccountStatus, InstitutionalAuthError,
    },
};

/// Contextualiza uma instituição de ensino parceira ativa no CApp.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `institution_name`: nome da instituição parceira.
#[given(expr = "que a {string} é uma instituição de ensino ativa no CApp")]
#[given(expr = "a {string} é uma instituição de ensino ativa no CApp")]
fn institution_is_active(world: &mut AppWorld, institution_name: String) {
    world
        .authentication
        .register_institution(&institution_name, InstitutionStatus::Active);
}

/// Define que o estudante possui uma conta institucional ativa com credenciais válidas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: endereço de e-mail institucional do estudante.
#[given(expr = "que o estudante possui uma conta institucional ativa com o e-mail {string}")]
#[given(expr = "o estudante possui uma conta institucional ativa com o e-mail {string}")]
fn student_has_active_account(world: &mut AppWorld, email: String) {
    world.authentication.register_account(InstitutionalAccount {
        email: email.clone(),
        institution_name: DEFAULT_INSTITUTION_NAME.to_string(),
        status: InstitutionalAccountStatus::Active,
        credentials_valid: true,
    });
    world.authentication.current_target_email = Some(email);
}

/// Executa a tentativa de autenticação com as credenciais institucionais configuradas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum e-mail de teste tiver sido contextualizado previamente.
#[when("ele realiza a autenticação com suas credenciais institucionais")]
#[when("realiza a autenticação com suas credenciais institucionais")]
fn user_authenticates_with_credentials(world: &mut AppWorld) {
    let email = world
        .authentication
        .current_target_email
        .clone()
        .expect("e-mail institucional deve estar contextualizado");
    world.authentication.authenticate(&email);
}

/// Verifica se o acesso à plataforma foi concedido com sucesso.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o acesso não tiver sido concedido.
#[then("o acesso à plataforma é concedido com sucesso")]
#[then("o acesso à plataforma é concedido")]
#[then("concede o acesso à plataforma com sucesso")]
fn access_granted_successfully(world: &mut AppWorld) {
    assert!(
        world.authentication.access_granted,
        "o acesso à plataforma deveria ter sido concedido com sucesso"
    );
}

/// Verifica se uma sessão autenticada foi estabelecida para o estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhuma sessão ativa estiver presente no contexto.
#[then("uma sessão autenticada é estabelecida para o estudante")]
#[then("estabelece uma sessão autenticada para o estudante")]
#[then("é estabelecida uma sessão autenticada para o estudante")]
fn session_established_for_student(world: &mut AppWorld) {
    assert!(
        world.authentication.active_session.is_some(),
        "uma sessão autenticada deveria ter sido estabelecida para o estudante"
    );
}

/// Verifica se o perfil acadêmico do usuário reflete o e-mail institucional esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `expected_email`: endereço de e-mail institucional aguardado.
///
/// # Panics
///
/// Se a sessão não existir ou se o e-mail divergir do esperado.
#[then(expr = "o perfil acadêmico do usuário fica disponível com o e-mail {string}")]
#[then(expr = "fica disponível o perfil acadêmico do usuário com o e-mail {string}")]
#[then(expr = "o perfil acadêmico do estudante fica disponível com o e-mail {string}")]
fn profile_available_with_email(world: &mut AppWorld, expected_email: String) {
    let session = world
        .authentication
        .active_session
        .as_ref()
        .expect("uma sessão autenticada ativa deve estar presente");
    assert_eq!(
        session.profile.institutional_email, expected_email,
        "o e-mail do perfil acadêmico deve coincidir com o informado"
    );
}

/// Contextualiza o primeiro acesso de um estudante com conta institucional ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: endereço de e-mail institucional do calouro/estudante.
#[given(
    expr = "que um estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa"
)]
#[given(
    expr = "um estudante com o e-mail {string} realiza o primeiro acesso com conta institucional ativa"
)]
fn student_first_access_with_active_account(world: &mut AppWorld, email: String) {
    world.authentication.register_account(InstitutionalAccount {
        email: email.clone(),
        institution_name: DEFAULT_INSTITUTION_NAME.to_string(),
        status: InstitutionalAccountStatus::Active,
        credentials_valid: true,
    });
    world.authentication.current_target_email = Some(email);
}

/// Conclui a validação de identidade institucional pelo estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum e-mail estiver contextualizado.
#[when("ele conclui a validação de sua identidade institucional")]
#[when("conclui a validação de sua identidade institucional")]
fn user_concludes_identity_validation(world: &mut AppWorld) {
    let email = world
        .authentication
        .current_target_email
        .clone()
        .expect("e-mail institucional deve estar contextualizado");
    world.authentication.authenticate(&email);
}

/// Verifica se o sistema estabeleceu a sessão autenticada do usuário.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a sessão do usuário não tiver sido estabelecida.
#[then("o sistema estabelece a sessão autenticada do usuário")]
#[then("estabelece a sessão autenticada do usuário")]
#[then("a sessão autenticada do usuário é estabelecida")]
fn system_establishes_user_session(world: &mut AppWorld) {
    assert!(
        world.authentication.active_session.is_some(),
        "o sistema deveria ter estabelecido a sessão autenticada do usuário"
    );
}

/// Confirma que a identidade institucional foi associada ao perfil do estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a sessão não existir ou se a identidade não estiver vinculada.
#[then("associa a identidade institucional ao novo perfil do estudante")]
#[then("o sistema associa a identidade institucional ao novo perfil do estudante")]
#[then("a identidade institucional é associada ao novo perfil do estudante")]
fn links_identity_to_profile(world: &mut AppWorld) {
    let session = world
        .authentication
        .active_session
        .as_ref()
        .expect("sessão autenticada ativa deve estar presente");
    assert!(
        session.profile.linked_identity,
        "a identidade institucional deveria estar vinculada ao novo perfil"
    );
}

/// Contextualiza credenciais institucionais inválidas fornecidas para um e-mail.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: e-mail informado no teste.
#[given(expr = "que um usuário possui credenciais institucionais inválidas para o e-mail {string}")]
#[given(expr = "um usuário possui credenciais institucionais inválidas para o e-mail {string}")]
fn user_has_invalid_credentials(world: &mut AppWorld, email: String) {
    world.authentication.register_account(InstitutionalAccount {
        email: email.clone(),
        institution_name: DEFAULT_INSTITUTION_NAME.to_string(),
        status: InstitutionalAccountStatus::Active,
        credentials_valid: false,
    });
    world.authentication.current_target_email = Some(email);
}

/// Tenta executar o login institucional com os dados preparados.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum e-mail estiver contextualizado.
#[when("ele tenta realizar o login institucional")]
#[when("tenta realizar o login institucional")]
fn user_tries_institutional_login(world: &mut AppWorld) {
    let email = world
        .authentication
        .current_target_email
        .clone()
        .expect("e-mail institucional deve estar contextualizado");
    world.authentication.authenticate(&email);
}

/// Verifica se a autenticação foi recusada indicando credenciais inválidas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a tentativa tiver sido aceita ou se o erro não indicar credenciais inválidas.
#[then("a autenticação é recusada indicando credenciais inválidas")]
#[then("recusa a autenticação indicando credenciais inválidas")]
fn auth_refused_invalid_credentials(world: &mut AppWorld) {
    assert!(
        !world.authentication.access_granted,
        "o acesso à plataforma não deveria ter sido concedido"
    );
    assert_eq!(
        world.authentication.last_error,
        Some(InstitutionalAuthError::InvalidCredentials),
        "o erro registrado deve indicar credenciais inválidas"
    );
}

/// Verifica se nenhuma sessão autenticada foi criada após tentativa inválida.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se alguma sessão tiver sido indevidamente criada.
#[then("nenhuma sessão autenticada é estabelecida")]
#[then("não estabelece nenhuma sessão autenticada")]
fn no_authenticated_session_established(world: &mut AppWorld) {
    assert!(
        world.authentication.active_session.is_none(),
        "nenhuma sessão autenticada deveria estar presente"
    );
}

/// Contextualiza conta institucional revogada ou inativa junto à instituição de ensino.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: e-mail da conta revogada.
#[given(
    expr = "que o estudante possui uma conta institucional com o e-mail {string} revogada pela instituição"
)]
#[given(
    expr = "o estudante possui uma conta institucional com o e-mail {string} revogada pela instituição"
)]
fn student_account_revoked(world: &mut AppWorld, email: String) {
    world.authentication.register_account(InstitutionalAccount {
        email: email.clone(),
        institution_name: DEFAULT_INSTITUTION_NAME.to_string(),
        status: InstitutionalAccountStatus::Revoked,
        credentials_valid: true,
    });
    world.authentication.current_target_email = Some(email);
}

/// Tenta autenticar no aplicativo CApp.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum e-mail estiver contextualizado.
#[when("ele tenta autenticar no CApp")]
#[when("tenta autenticar no CApp")]
fn user_tries_authenticate_in_capp(world: &mut AppWorld) {
    let email = world
        .authentication
        .current_target_email
        .clone()
        .expect("e-mail institucional deve estar contextualizado");
    world.authentication.authenticate(&email);
}

/// Verifica se a autenticação foi recusada informando que o vínculo está inativo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a autenticação tiver sido aceita ou se o erro divergir de vínculo inativo.
#[then("a autenticação é recusada informando que o vínculo institucional está inativo")]
#[then("recusa a autenticação informando que o vínculo institucional está inativo")]
fn auth_refused_inactive_enrollment(world: &mut AppWorld) {
    assert!(
        !world.authentication.access_granted,
        "o acesso à plataforma deveria ter sido recusado"
    );
    assert_eq!(
        world.authentication.last_error,
        Some(InstitutionalAuthError::AccountInactiveOrRevoked),
        "o erro registrado deve indicar conta ou vínculo inativo/revogado"
    );
}

/// Verifica se o acesso à plataforma permanece bloqueado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o acesso tiver sido concedido ou se uma sessão tiver sido criada.
#[then("o acesso à plataforma permanece bloqueado")]
#[then("permanece com o acesso à plataforma bloqueado")]
fn platform_access_remains_blocked(world: &mut AppWorld) {
    assert!(
        !world.authentication.access_granted,
        "o acesso deve permanecer bloqueado"
    );
    assert!(
        world.authentication.active_session.is_none(),
        "nenhuma sessão deve estar ativa"
    );
}

/// Contextualiza tentativa de autenticação com e-mail de instituição não parceira.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: e-mail informado na tentativa.
/// - `institution_name`: nome da instituição desconhecida.
#[given(expr = "que um estudante tenta autenticar com o e-mail {string} da {string}")]
#[given(expr = "um estudante tenta autenticar com o e-mail {string} da {string}")]
fn student_tries_unrecognized_institution_with_email(
    world: &mut AppWorld,
    email: String,
    institution_name: String,
) {
    world.authentication.register_account(InstitutionalAccount {
        email: email.clone(),
        institution_name,
        status: InstitutionalAccountStatus::Active,
        credentials_valid: true,
    });
    world.authentication.current_target_email = Some(email);
}

/// Contextualiza tentativa de autenticação com instituição não parceira genérica.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `institution_name`: nome da instituição desconhecida.
#[given(expr = "que um estudante tenta autenticar com uma conta institucional da {string}")]
#[given(expr = "um estudante tenta autenticar com uma conta institucional da {string}")]
fn student_tries_unrecognized_institution(world: &mut AppWorld, institution_name: String) {
    let email = "estudante@externa.edu.br".to_string();
    world.authentication.register_account(InstitutionalAccount {
        email: email.clone(),
        institution_name,
        status: InstitutionalAccountStatus::Active,
        credentials_valid: true,
    });
    world.authentication.current_target_email = Some(email);
}

/// Solicita validação da identidade institucional.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum e-mail estiver contextualizado.
#[when("ele solicita a validação da sua identidade institucional")]
#[when("solicita a validação da sua identidade institucional")]
fn user_requests_identity_validation(world: &mut AppWorld) {
    let email = world
        .authentication
        .current_target_email
        .clone()
        .expect("e-mail institucional deve estar contextualizado");
    world.authentication.authenticate(&email);
}

/// Verifica se a autenticação foi recusada indicando instituição não reconhecida.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o acesso tiver sido liberado ou se o erro divergir de instituição não reconhecida.
#[then("a autenticação é recusada indicando instituição não reconhecida")]
#[then("recusa a autenticação indicando instituição não reconhecida")]
fn auth_refused_unknown_institution(world: &mut AppWorld) {
    assert!(
        !world.authentication.access_granted,
        "o acesso não deve ser concedido para instituição desconhecida"
    );
    assert_eq!(
        world.authentication.last_error,
        Some(InstitutionalAuthError::InstitutionNotRecognized),
        "o erro registrado deve indicar instituição não reconhecida"
    );
}

/// Verifica se nenhuma credencial de acesso foi gerada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se alguma credencial de acesso tiver sido gerada.
#[then("nenhuma credencial de acesso é gerada")]
#[then("não gera nenhuma credencial de acesso")]
fn no_access_credentials_generated(world: &mut AppWorld) {
    assert!(
        !world.authentication.access_credentials_generated,
        "nenhuma credencial de acesso deveria ter sido gerada"
    );
    assert!(
        world.authentication.active_session.is_none(),
        "nenhuma sessão deve estar ativa"
    );
}

/// Contextualiza que o usuário não está autenticado no aplicativo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário não está autenticado no aplicativo")]
#[given("o usuário não está autenticado no aplicativo")]
fn user_is_not_authenticated(world: &mut AppWorld) {
    world.authentication.active_session = None;
    world.authentication.access_granted = false;
}

/// Tenta acessar serviço ou funcionalidade restrita da comunidade acadêmica.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele tenta acessar um serviço restrito da comunidade acadêmica")]
#[when("tenta acessar um serviço restrito da comunidade acadêmica")]
fn user_tries_access_restricted_service(world: &mut AppWorld) {
    world.authentication.access_restricted_service();
}

/// Verifica se o acesso ao recurso foi bloqueado por ausência de autenticação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro registrado não indicar acesso não autenticado.
#[then("o acesso ao recurso é bloqueado por ausência de autenticação")]
#[then("bloqueia o acesso ao recurso por ausência de autenticação")]
fn access_blocked_due_to_unauthenticated(world: &mut AppWorld) {
    assert_eq!(
        world.authentication.last_error,
        Some(InstitutionalAuthError::UnauthenticatedAccess),
        "o acesso deveria ser bloqueado por falta de autenticação"
    );
}

/// Verifica se o aplicativo solicitou que o usuário realize autenticação institucional.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a solicitação de login não tiver sido disparada.
#[then("o aplicativo solicita que o usuário realize a autenticação institucional")]
#[then("solicita que o usuário realize a autenticação institucional")]
fn app_requests_institutional_authentication(world: &mut AppWorld) {
    assert!(
        world.authentication.login_requested,
        "o aplicativo deveria solicitar a autenticação institucional"
    );
}

/// Contextualiza visitante anônimo sem qualquer sessão estabelecida.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que um visitante anônimo não possui sessão de usuário estabelecida")]
#[given("um visitante anônimo não possui sessão de usuário estabelecida")]
fn anonymous_visitor_without_session(world: &mut AppWorld) {
    world.authentication.active_session = None;
    world.authentication.access_granted = false;
}

/// Tenta submeter uma proposta em consulta estudantil restrita sem credenciais.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("ele tenta submeter uma proposta em consulta estudantil restrita")]
#[when("tenta submeter uma proposta em consulta estudantil restrita")]
fn visitor_tries_submit_restricted_proposal(world: &mut AppWorld) {
    world.authentication.submit_restricted_action();
}

/// Verifica se a operação foi impedida exigindo credenciais ativas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a operação não tiver sido impedida com erro de autenticação.
#[then("a operação é impedida exigindo credenciais institucionais ativas")]
#[then("impede a operação exigindo credenciais institucionais ativas")]
fn operation_prevented_requiring_credentials(world: &mut AppWorld) {
    assert_eq!(
        world.authentication.last_error,
        Some(InstitutionalAuthError::UnauthenticatedAccess),
        "a operação protegida deveria ter sido impedida"
    );
}

/// Confirma que nenhum registro acadêmico foi persistido no sistema.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se qualquer registro tiver sido persistido indevidamente.
#[then("nenhum registro acadêmico é persistido no sistema")]
#[then("não persiste nenhum registro acadêmico no sistema")]
fn no_academic_record_persisted(world: &mut AppWorld) {
    assert_eq!(
        world.authentication.academic_records_persisted, 0,
        "nenhum registro acadêmico deveria ter sido persistido no sistema"
    );
}
