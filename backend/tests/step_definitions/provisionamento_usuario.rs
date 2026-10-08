//! Implementação dos steps de BDD para o provisionamento do usuário no CApp.
//!
//! Os passos operam sobre [`AppWorld`], interagindo com a fixture de provisionamento
//! de usuário para validar os cenários especificados em
//! `features/identidade_e_acesso/provisionamento_usuario.feature`:
//!
//! - Provisionamento automático de perfil no primeiro login com credencial institucional ativa.
//! - Associação unívoca entre a identidade institucional externa e o registro interno do usuário.
//! - Recuperação do perfil existente e atualização de último acesso para usuários já cadastrados.
//! - Prevenção de duplicidade e preservação do identificador interno em reautenticações.

use cucumber::{given, then, when};

use crate::support::{AppWorld, world::InstitutionStatus};

/// Contextualiza uma instituição de ensino parceira ativa na plataforma.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `institution_name`: nome da instituição de ensino parceira.
#[given(expr = "que a {string} é uma instituição de ensino ativa no CApp")]
#[given(expr = "a {string} é uma instituição de ensino ativa no CApp")]
fn institution_is_active(world: &mut AppWorld, institution_name: String) {
    world
        .user_provisioning
        .register_institution(&institution_name, InstitutionStatus::Active);
}

/// Define que o estudante calouro possui uma credencial institucional ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: endereço de e-mail institucional do estudante.
#[given(expr = "que o estudante calouro possui a credencial institucional ativa {string}")]
#[given(expr = "o estudante calouro possui a credencial institucional ativa {string}")]
fn freshman_has_active_credential(world: &mut AppWorld, email: String) {
    world.user_provisioning.current_email = Some(email);
}

/// Assegura que o estudante ainda não possui cadastro ou perfil prévio no aplicativo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o e-mail do estudante não estiver contextualizado ou se já houver cadastro prévio.
#[given(expr = "que o estudante ainda não possui cadastro no aplicativo")]
#[given(expr = "o estudante ainda não possui cadastro no aplicativo")]
#[given(expr = "ele ainda não possui cadastro no aplicativo")]
#[given(expr = "ainda não possui cadastro no aplicativo")]
fn student_has_no_prior_registration(world: &mut AppWorld) {
    let email = world
        .user_provisioning
        .current_email
        .as_deref()
        .expect("o e-mail do estudante deve estar contextualizado");
    assert!(
        world.user_provisioning.get_user_by_email(email).is_none(),
        "o estudante não deveria possuir cadastro prévio no aplicativo"
    );
}

/// Executa o primeiro login autenticado com a credencial institucional ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o e-mail institucional não tiver sido contextualizado previamente.
#[when("ele realiza o primeiro login com sucesso utilizando sua credencial institucional")]
#[when("realiza o primeiro login com sucesso utilizando sua credencial institucional")]
#[when("o estudante realiza o primeiro login com sucesso utilizando sua credencial institucional")]
fn first_login_with_institutional_credential(world: &mut AppWorld) {
    let email = world
        .user_provisioning
        .current_email
        .clone()
        .expect("o e-mail do estudante deve estar contextualizado");
    world.user_provisioning.provision_or_authenticate(&email);
}

/// Verifica se o perfil interno do usuário foi criado automaticamente pelo sistema.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o perfil interno não tiver sido criado ou se o identificador não foi gerado.
#[then("o sistema cria automaticamente o perfil interno do usuário")]
#[then("cria automaticamente o perfil interno do usuário")]
#[then("o perfil interno do usuário é criado automaticamente")]
fn system_creates_internal_user_profile(world: &mut AppWorld) {
    assert!(
        world.user_provisioning.was_user_created,
        "o perfil interno do usuário deveria ter sido criado automaticamente"
    );
    assert!(
        world.user_provisioning.current_user_id.is_some(),
        "um identificador único de usuário interno deveria ter sido gerado"
    );
}

/// Verifica se a conta interna foi ativada sem exigência de formulário manual de cadastro.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se for exigido formulário de cadastro ou se a conta interna não estiver ativa.
#[then("a conta interna do estudante é ativada sem exigir preenchimento de formulário de cadastro")]
#[then(
    "o sistema ativa a conta interna do estudante sem exigir preenchimento de formulário de cadastro"
)]
#[then("ativa a conta interna do estudante sem exigir preenchimento de formulário de cadastro")]
fn internal_account_activated_without_form(world: &mut AppWorld) {
    assert!(
        !world.user_provisioning.registration_form_required,
        "nenhum formulário de cadastro manual deveria ser exigido"
    );
    let user_id = world
        .user_provisioning
        .current_user_id
        .expect("o usuário interno deve existir após o provisionamento");
    let user = world
        .user_provisioning
        .get_user(&user_id)
        .expect("o registro do usuário deve estar presente na fixture");
    assert_eq!(
        user.status, "active",
        "a conta interna do estudante deveria estar com situação ativa"
    );
}

/// Verifica se o perfil da aplicação foi associado ao e-mail institucional esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `expected_email`: endereço de e-mail institucional esperado.
///
/// # Panics
///
/// Se o e-mail registrado no perfil divergir do e-mail esperado.
#[then(expr = "o perfil da aplicação fica associado ao e-mail institucional {string}")]
#[then(expr = "o sistema associa o perfil da aplicação ao e-mail institucional {string}")]
#[then(expr = "associa o perfil da aplicação ao e-mail institucional {string}")]
#[then(expr = "o perfil interno fica associado ao e-mail institucional {string}")]
fn profile_associated_with_email(world: &mut AppWorld, expected_email: String) {
    let user_id = world
        .user_provisioning
        .current_user_id
        .expect("o usuário interno deve existir");
    let user = world
        .user_provisioning
        .get_user(&user_id)
        .expect("o registro do usuário deve estar presente");
    assert_eq!(
        user.institutional_email, expected_email,
        "o e-mail institucional do perfil deve coincidir com o informado"
    );
}

/// Define que o estudante concluiu com sucesso a validação da identidade externa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: endereço de e-mail institucional validado externamente.
#[given(expr = "que um estudante com e-mail institucional {string} conclui a validação externa")]
#[given(expr = "um estudante com e-mail institucional {string} conclui a validação externa")]
#[given(expr = "que o estudante com e-mail institucional {string} conclui a validação externa")]
fn student_completes_external_validation(world: &mut AppWorld, email: String) {
    world.user_provisioning.current_email = Some(email);
}

/// Dispara o processamento de provisionamento do perfil do usuário na aplicação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum e-mail institucional tiver sido contextualizado.
#[when("o sistema processa o provisionamento do perfil do usuário")]
#[when("processa o provisionamento do perfil do usuário")]
#[when("o provisionamento do perfil do usuário é processado")]
fn process_user_provisioning(world: &mut AppWorld) {
    let email = world
        .user_provisioning
        .current_email
        .clone()
        .expect("o e-mail institucional deve estar contextualizado");
    world.user_provisioning.provision_or_authenticate(&email);
}

/// Assegura que uma identidade institucional externa unívoca foi associada ao registro interno.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o usuário interno ou a identidade externa associada não existirem.
#[then("uma identidade institucional externa unívoca é associada ao registro interno do estudante")]
#[then(
    "o sistema associa uma identidade institucional externa unívoca ao registro interno do estudante"
)]
#[then("associa uma identidade institucional externa unívoca ao registro interno do estudante")]
fn unique_external_identity_associated(world: &mut AppWorld) {
    let user_id = world
        .user_provisioning
        .current_user_id
        .expect("o usuário interno deve existir");
    let identity = world
        .user_provisioning
        .get_identity_for_user(&user_id)
        .expect("uma identidade externa deve estar vinculada ao usuário");
    assert_eq!(
        identity.user_id, user_id,
        "a identidade externa deve referenciar o usuário interno correto"
    );
}

/// Verifica que nenhum outro usuário interno pode ser vinculado à mesma identidade externa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a identidade não for encontrada ou se houver mais de um usuário vinculado a ela.
#[then("nenhum outro usuário interno pode ser vinculado à mesma identidade institucional externa")]
#[then(
    "o sistema impede que outro usuário interno seja vinculado à mesma identidade institucional externa"
)]
#[then("impede que outro usuário interno seja vinculado à mesma identidade institucional externa")]
fn no_other_user_linked_to_same_external_identity(world: &mut AppWorld) {
    let user_id = world
        .user_provisioning
        .current_user_id
        .expect("o usuário interno deve existir");
    let identity = world
        .user_provisioning
        .get_identity_for_user(&user_id)
        .expect("a identidade externa deve existir");
    let linked_count = world
        .user_provisioning
        .count_users_for_identity(&identity.external_subject);
    assert_eq!(
        linked_count, 1,
        "exatamente um usuário interno deve estar vinculado à identidade externa"
    );
}

/// Define que o estudante veterano já possui um perfil cadastrado no aplicativo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: endereço de e-mail institucional do estudante veterano.
#[given(
    expr = "que o estudante veterano com e-mail {string} já possui perfil cadastrado no aplicativo"
)]
#[given(
    expr = "o estudante veterano com e-mail {string} já possui perfil cadastrado no aplicativo"
)]
#[given(expr = "que o estudante com e-mail {string} já possui perfil cadastrado no aplicativo")]
fn veteran_already_registered(world: &mut AppWorld, email: String) {
    let user_id = world
        .user_provisioning
        .register_existing_user(&email, "Estudante Veterano");
    world.user_provisioning.initial_user_id = Some(user_id);
    world.user_provisioning.current_email = Some(email);
}

/// Executa o login no aplicativo com a credencial institucional de usuário existente.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o e-mail institucional não estiver contextualizado.
#[when("ele realiza login no aplicativo utilizando sua credencial institucional")]
#[when("realiza login no aplicativo utilizando sua credencial institucional")]
#[when("o estudante realiza login no aplicativo utilizando sua credencial institucional")]
fn user_logs_in_with_institutional_credential(world: &mut AppWorld) {
    let email = world
        .user_provisioning
        .current_email
        .clone()
        .expect("o e-mail do estudante deve estar contextualizado");
    world.user_provisioning.provision_or_authenticate(&email);
}

/// Verifica se o sistema apenas recuperou o perfil existente do estudante sem recriá-lo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o usuário tiver sido recriado ou não tiver sido recuperado.
#[then("o sistema apenas recupera o perfil existente do estudante")]
#[then("apenas recupera o perfil existente do estudante")]
#[then("o perfil existente do estudante é recuperado")]
fn system_only_recovers_existing_profile(world: &mut AppWorld) {
    assert!(
        world.user_provisioning.was_user_recovered,
        "o perfil existente do estudante deveria ter sido recuperado"
    );
    assert!(
        !world.user_provisioning.was_user_created,
        "nenhum novo perfil deveria ter sido criado para usuário existente"
    );
}

/// Assegura que nenhum novo registro de usuário foi inserido no sistema.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se a quantidade de usuários após o login for diferente da quantidade anterior.
#[then("nenhum novo registro de usuário é criado no aplicativo")]
#[then("o sistema não cria nenhum novo registro de usuário no aplicativo")]
#[then("não cria nenhum novo registro de usuário no aplicativo")]
fn no_new_user_record_created(world: &mut AppWorld) {
    assert_eq!(
        world.user_provisioning.user_count_before, world.user_provisioning.user_count_after,
        "a contagem de usuários não deve aumentar após login de usuário já cadastrado"
    );
}

/// Assegura que a data do último acesso da identidade institucional foi devidamente atualizada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se as marcas temporais de login não estiverem presentes ou se o instante posterior não for maior.
#[then("a data do último acesso da identidade institucional é atualizada")]
#[then("o sistema atualiza a data do último acesso da identidade institucional")]
#[then("atualiza a data do último acesso da identidade institucional")]
fn last_login_date_is_updated(world: &mut AppWorld) {
    let before = world
        .user_provisioning
        .last_login_before
        .expect("instante anterior de login deve estar registrado");
    let after = world
        .user_provisioning
        .last_login_after
        .expect("novo instante de login deve estar registrado");
    assert!(
        after > before,
        "o instante do último acesso ({after}) deveria ser posterior ao anterior ({before})"
    );
}

/// Define que o estudante veterano possui um identificador interno único estabelecido.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `email`: endereço de e-mail institucional do estudante veterano.
#[given(
    expr = "que o estudante veterano com e-mail {string} possui um identificador interno único"
)]
#[given(expr = "o estudante veterano com e-mail {string} possui um identificador interno único")]
#[given(expr = "que o estudante com e-mail {string} possui um identificador interno único")]
fn veteran_has_unique_internal_id(world: &mut AppWorld, email: String) {
    let user_id = world
        .user_provisioning
        .register_existing_user(&email, "Estudante Veterano");
    world.user_provisioning.initial_user_id = Some(user_id);
    world.user_provisioning.current_email = Some(email);
}

/// Executa múltiplas reautenticações consecutivas para a mesma credencial institucional.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o e-mail do estudante não estiver contextualizado.
#[when("ele se reautentica no aplicativo consecutivas vezes com a mesma credencial institucional")]
#[when("se reautentica no aplicativo consecutivas vezes com a mesma credencial institucional")]
#[when(
    "o estudante se reautentica no aplicativo consecutivas vezes com a mesma credencial institucional"
)]
fn user_reauthenticates_multiple_times(world: &mut AppWorld) {
    let email = world
        .user_provisioning
        .current_email
        .clone()
        .expect("o e-mail do estudante deve estar contextualizado");
    world.user_provisioning.reauthenticate_multiple(&email, 3);
}

/// Assegura que o identificador interno original do estudante permaneceu idêntico após reautenticações.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o identificador inicial ou corrente não existirem ou forem diferentes.
#[then("o identificador interno original do estudante permanece inalterado")]
#[then("o sistema mantém inalterado o identificador interno original do estudante")]
#[then("mantém inalterado o identificador interno original do estudante")]
fn original_internal_id_remains_unchanged(world: &mut AppWorld) {
    let initial_id = world
        .user_provisioning
        .initial_user_id
        .expect("o identificador inicial deve estar registrado");
    let current_id = world
        .user_provisioning
        .current_user_id
        .expect("o identificador corrente deve estar presente");
    assert_eq!(
        initial_id, current_id,
        "o identificador interno original deve permanecer idêntico após reautenticações"
    );
}

/// Verifica que o sistema preservou exatamente um único registro de perfil para o estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o e-mail não estiver presente ou se a contagem de registros de perfil for diferente de 1.
#[then("o sistema preserva um único registro de perfil para o estudante")]
#[then("preserva um único registro de perfil para o estudante")]
#[then("um único registro de perfil é preservado para o estudante")]
fn system_preserves_single_profile_record(world: &mut AppWorld) {
    let email = world
        .user_provisioning
        .current_email
        .as_deref()
        .expect("o e-mail deve estar presente");
    let count = world.user_provisioning.count_users_with_email(email);
    assert_eq!(
        count, 1,
        "deve existir exatamente um único registro de perfil para o estudante"
    );
}
