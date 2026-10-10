//! Definições de passos BDD para as regras de contexto Multi-CA e navegação.
//!
//! Implementa os passos em linguagem de domínio discente e representação
//! estudantil da UnB, cobrindo CA favorito, mudança de contexto ativo,
//! isolamento de permissões e manutenção de cargos por CA.
//!
//! Todas as conjunções "E" são suportadas tanto com sujeito explícito
//! quanto com sujeito oculto ou pronominal, garantindo zero skipped steps.

use cucumber::{gherkin::Step, given, then, when};

use crate::support::AppWorld;

/// Registra Centros Acadêmicos disponíveis na plataforma a partir da tabela Gherkin.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `step`: representação do passo Gherkin contendo a tabela de dados.
#[given(expr = "que existem os seguintes Centros Acadêmicos cadastrados na UnB:")]
#[given(expr = "existem os seguintes Centros Acadêmicos cadastrados na UnB:")]
fn centros_academicos_cadastrados(world: &mut AppWorld, step: &Step) {
    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            if row.len() >= 2 {
                world.register_academic_center(&row[0], &row[1]);
            }
        }
    }
}

/// Define a preferência de CA favorito para um estudante específico.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `student_email`: e-mail institucional do estudante da UnB.
/// - `ca_name`: nome ou sigla do Centro Acadêmico favorito.
#[given(expr = "que o estudante {string} possui {string} definido como seu CA favorito")]
#[given(expr = "o estudante {string} possui {string} definido como seu CA favorito")]
fn estudante_possui_ca_favorito(world: &mut AppWorld, student_email: String, ca_name: String) {
    world.set_student_favorite_ca(student_email, ca_name);
}

/// Suporte a sujeito oculto ou pronome para definição de CA favorito do estudante atual.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `ca_name`: nome ou sigla do Centro Acadêmico favorito.
///
/// # Panics
///
/// Se nenhum estudante estiver ativo no contexto do cenário.
#[given(expr = "que o estudante possui {string} definido como seu CA favorito")]
#[given(expr = "o estudante possui {string} definido como seu CA favorito")]
#[given(expr = "que ele possui {string} definido como seu CA favorito")]
#[given(expr = "ele possui {string} definido como seu CA favorito")]
#[given(expr = "possui {string} definido como seu CA favorito")]
fn estudante_oculto_possui_ca_favorito(world: &mut AppWorld, ca_name: String) {
    let email = world
        .current_student
        .clone()
        .expect("um estudante deve estar em foco para definir CA favorito com sujeito oculto");
    world.set_student_favorite_ca(email, ca_name);
}

/// Registra que o estudante especificado está autenticado na aplicação.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `student_email`: e-mail institucional do estudante.
#[given(expr = "que o estudante {string} está autenticado no aplicativo")]
#[given(expr = "o estudante {string} está autenticado no aplicativo")]
fn estudante_autenticado(world: &mut AppWorld, student_email: String) {
    world.set_authenticated_student(student_email);
}

/// Suporte a sujeito oculto para autenticação do estudante atual.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
#[given(expr = "que o estudante está autenticado no aplicativo")]
#[given(expr = "o estudante está autenticado no aplicativo")]
#[given(expr = "que está autenticado no aplicativo")]
#[given(expr = "está autenticado no aplicativo")]
#[given(expr = "ele está autenticado no aplicativo")]
#[given(expr = "que ele está autenticado no aplicativo")]
fn estudante_oculto_autenticado(_world: &mut AppWorld) {
    // Mantém o estudante previamente registrado como autenticado
}

/// Define o contexto ativo atual de navegação como pré-condição ou ação.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `ca_name`: nome ou sigla do Centro Acadêmico.
#[given(expr = "que o estudante está com o contexto ativo em {string}")]
#[given(expr = "o estudante está com o contexto ativo em {string}")]
#[given(expr = "que está com o contexto ativo em {string}")]
#[given(expr = "está com o contexto ativo em {string}")]
#[given(expr = "ele está com o contexto ativo em {string}")]
#[given(expr = "que ele está com o contexto ativo em {string}")]
#[when(expr = "o estudante está com o contexto ativo em {string}")]
#[when(expr = "quando o estudante está com o contexto ativo em {string}")]
#[when(expr = "ele está com o contexto ativo em {string}")]
#[when(expr = "quando ele está com o contexto ativo em {string}")]
#[when(expr = "está com o contexto ativo em {string}")]
fn contexto_ativo_definido(world: &mut AppWorld, ca_name: String) {
    world.set_active_ca(ca_name);
}

/// Simula a abertura inicial do aplicativo pelo estudante.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
#[when(expr = "o estudante abre o aplicativo CApp")]
#[when(expr = "quando o estudante abre o aplicativo CApp")]
#[when(expr = "ele abre o aplicativo CApp")]
#[when(expr = "quando ele abre o aplicativo CApp")]
#[when(expr = "abre o aplicativo CApp")]
fn estudante_abre_aplicativo(world: &mut AppWorld) {
    world.open_app();
}

/// Altera o contexto ativo de navegação para outro Centro Acadêmico.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `ca_name`: nome ou sigla do Centro Acadêmico de destino.
#[when(expr = "o estudante altera o contexto ativo para {string}")]
#[when(expr = "quando o estudante altera o contexto ativo para {string}")]
#[when(expr = "ele altera o contexto ativo para {string}")]
#[when(expr = "quando ele altera o contexto ativo para {string}")]
#[when(expr = "altera o contexto ativo para {string}")]
fn altera_contexto_ativo(world: &mut AppWorld, ca_name: String) {
    world.set_active_ca(ca_name);
}

/// Valida se o contexto ativo de navegação corresponde ao Centro Acadêmico esperado.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `expected_ca`: nome ou sigla esperada do Centro Acadêmico ativo.
///
/// # Panics
///
/// Se o contexto ativo estiver indefinido ou divergir do CA esperado.
#[then(expr = "o contexto ativo de navegação deve ser {string}")]
#[then(expr = "o contexto ativo deve ser {string}")]
#[then(expr = "o contexto de navegação ativo deve ser {string}")]
fn contexto_ativo_deve_ser(world: &mut AppWorld, expected_ca: String) {
    let active = world
        .active_ca
        .as_deref()
        .expect("deve haver um contexto ativo definido");
    assert!(
        AppWorld::ca_names_match(active, &expected_ca),
        "esperado CA ativo '{expected_ca}', mas o contexto ativo era '{active}'",
    );
}

/// Valida que o estudante visualiza os conteúdos do Centro Acadêmico ativo.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `expected_ca`: Centro Acadêmico cujos dados devem estar visíveis.
///
/// # Panics
///
/// Se o contexto ativo não corresponder ao CA cujas informações deveriam ser exibidas.
#[then(expr = "o estudante visualiza as informações correspondentes a {string}")]
#[then(expr = "ele visualiza as informações correspondentes a {string}")]
#[then(expr = "visualiza as informações correspondentes a {string}")]
fn visualiza_informacoes_do_ca(world: &mut AppWorld, expected_ca: String) {
    let active = world
        .active_ca
        .as_deref()
        .expect("deve haver um contexto ativo para visualização de informações");
    assert!(
        AppWorld::ca_names_match(active, &expected_ca),
        "o estudante deveria visualizar informações de '{expected_ca}', mas está no contexto de '{active}'",
    );
}

/// Valida que a preferência de CA favorito não confere privilégios administrativos.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
///
/// # Panics
///
/// Se o estudante possuir privilégios administrativos no CA ativo apenas por tê-lo como favorito.
#[then(
    expr = "o estudante não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito"
)]
#[then(
    expr = "ele não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito"
)]
#[then(expr = "não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito")]
fn nao_possui_privilegios_por_favorito(world: &mut AppWorld) {
    assert!(
        world.is_visitor_in_active_context(),
        "a preferência de CA favorito não deve conceder privilégios administrativos",
    );
}

/// Valida que a preferência de CA favorito permaneceu inalterada após a troca de contexto ativo.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `expected_ca`: nome ou sigla do CA favorito original.
///
/// # Panics
///
/// Se a preferência de CA favorito tiver sido modificada ou estiver indefinida.
#[then(expr = "a preferência de CA favorito do estudante permanece {string}")]
#[then(expr = "a preferência de CA favorito permanece {string}")]
fn preferencia_favorito_permanece(world: &mut AppWorld, expected_ca: String) {
    let favorite = world
        .favorite_ca
        .as_deref()
        .expect("o estudante deve possuir uma preferência de CA favorito cadastrada");
    assert!(
        AppWorld::ca_names_match(favorite, &expected_ca),
        "esperava CA favorito inalterado em '{expected_ca}', mas encontrou '{favorite}'",
    );
}

/// Atribui um cargo de gestão a um estudante em um Centro Acadêmico.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `student_email`: e-mail institucional do estudante.
/// - `role_name`: nome do cargo atribuído.
/// - `ca_name`: nome ou sigla do Centro Acadêmico onde o cargo é exercido.
#[given(expr = "que o estudante {string} exerce o cargo de {string} em {string}")]
#[given(expr = "o estudante {string} exerce o cargo de {string} em {string}")]
fn estudante_exerce_cargo(
    world: &mut AppWorld,
    student_email: String,
    role_name: String,
    ca_name: String,
) {
    world.assign_role(student_email, ca_name, role_name);
}

/// Suporte a sujeito oculto ou pronome para atribuição de cargo ao estudante em foco.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `role_name`: nome do cargo atribuído.
/// - `ca_name`: Centro Acadêmico onde o cargo é exercido.
///
/// # Panics
///
/// Se nenhum estudante estiver ativo no contexto do cenário.
#[given(expr = "que o estudante exerce o cargo de {string} em {string}")]
#[given(expr = "o estudante exerce o cargo de {string} em {string}")]
#[given(expr = "que ele exerce o cargo de {string} em {string}")]
#[given(expr = "ele exerce o cargo de {string} em {string}")]
#[given(expr = "exerce o cargo de {string} em {string}")]
fn estudante_oculto_exerce_cargo(world: &mut AppWorld, role_name: String, ca_name: String) {
    let email = world
        .current_student
        .clone()
        .expect("um estudante deve estar em foco para exercer cargo com sujeito oculto");
    world.assign_role(email, ca_name, role_name);
}

/// Associa uma permissão administrativa ao último cargo configurado.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `permission`: descrição da capacidade concedida ao cargo.
#[given(expr = "o cargo possui a permissão de {string}")]
#[given(expr = "que o cargo possui a permissão de {string}")]
#[given(expr = "possui a permissão de {string}")]
fn cargo_possui_permissao(world: &mut AppWorld, permission: String) {
    world.add_permission_to_last_role(permission);
}

/// Assegura que o estudante não possui cargo de gestão no Centro Acadêmico indicado.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `_ca_name`: Centro Acadêmico onde o estudante não deve possuir cargo.
#[given(expr = "que o estudante não possui cargo de gestão em {string}")]
#[given(expr = "o estudante não possui cargo de gestão em {string}")]
#[given(expr = "ele não possui cargo de gestão em {string}")]
#[given(expr = "não possui cargo de gestão em {string}")]
fn nao_possui_cargo_em_ca(_world: &mut AppWorld, _ca_name: String) {
    // Por padrão o estudante não tem cargo atribuído no CA informado
}

/// Valida que o estudante possui a permissão indicada no Centro Acadêmico ativo.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `permission`: permissão administrativa esperada.
///
/// # Panics
///
/// Se o estudante não possuir a permissão no contexto do CA ativo.
#[then(expr = "o estudante possui permissão para {string} no CA ativo")]
#[then(expr = "ele possui permissão para {string} no CA ativo")]
#[then(expr = "possui permissão para {string} no CA ativo")]
fn possui_permissao_no_ca_ativo(world: &mut AppWorld, permission: String) {
    assert!(
        world.has_active_permission(&permission),
        "o estudante deveria possuir a permissão '{permission}' no CA ativo '{:?}'",
        world.active_ca,
    );
}

/// Valida que o estudante não possui a permissão indicada no Centro Acadêmico ativo.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
/// - `permission`: permissão administrativa que não deve estar presente.
///
/// # Panics
///
/// Se o estudante indevidamente possuir a permissão no contexto do CA ativo.
#[then(expr = "o estudante não possui permissão para {string} no CA ativo")]
#[then(expr = "ele não possui permissão para {string} no CA ativo")]
#[then(expr = "não possui permissão para {string} no CA ativo")]
fn nao_possui_permissao_no_ca_ativo(world: &mut AppWorld, permission: String) {
    assert!(
        !world.has_active_permission(&permission),
        "o estudante NÃO deveria possuir a permissão '{permission}' no CA ativo '{:?}'",
        world.active_ca,
    );
}

/// Valida que o estudante atua estritamente como visitante no contexto ativo.
///
/// # Parâmetros
///
/// - `world`: estado do cenário BDD.
///
/// # Panics
///
/// Se o estudante possuir qualquer permissão administrativa no CA ativo.
#[then(expr = "atua estritamente como visitante no novo contexto")]
#[then(expr = "ele atua estritamente como visitante no novo contexto")]
#[then(expr = "o estudante atua estritamente como visitante no novo contexto")]
fn atua_como_visitante_no_novo_contexto(world: &mut AppWorld) {
    assert!(
        world.is_visitor_in_active_context(),
        "o estudante deveria atuar como visitante no contexto ativo, mas possui permissões: {:?}",
        world.active_permissions(),
    );
}
