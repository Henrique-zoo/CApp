//! Steps pendentes da especificação de contexto Multi-CA (#38).
//!
//! Cada passo delega para `pending_step(..)`, aguardando a implementação
//! e automação da issue subsequente (#59).

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que existem os seguintes Centros Acadêmicos cadastrados na UnB:
#[given("que existem os seguintes Centros Acadêmicos cadastrados na UnB:")]
fn existem_centros_academicos_cadastrados(_world: &mut AppWorld) {
    pending_step("que existem os seguintes Centros Acadêmicos cadastrados na UnB:");
}

/// Pendente: que o estudante {string} possui {string} definido como seu CA favorito.
#[given(expr = "que o estudante {string} possui {string} definido como seu CA favorito")]
fn estudante_possui_ca_favorito(_world: &mut AppWorld, _email: String, _ca: String) {
    pending_step("que o estudante {string} possui {string} definido como seu CA favorito");
}

/// Pendente: o estudante abre o aplicativo CApp.
#[when("o estudante abre o aplicativo CApp")]
fn estudante_abre_aplicativo(_world: &mut AppWorld) {
    pending_step("o estudante abre o aplicativo CApp");
}

/// Pendente: o contexto ativo de navegação deve ser {string}.
#[then(expr = "o contexto ativo de navegação deve ser {string}")]
fn contexto_ativo_deve_ser(_world: &mut AppWorld, _ca: String) {
    pending_step("o contexto ativo de navegação deve ser {string}");
}

/// Pendente: o estudante visualiza as informações correspondentes a {string}.
#[then(expr = "o estudante visualiza as informações correspondentes a {string}")]
fn estudante_visualiza_informacoes(_world: &mut AppWorld, _ca: String) {
    pending_step("o estudante visualiza as informações correspondentes a {string}");
}

/// Pendente: o estudante não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito.
#[then(
    "o estudante não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito"
)]
fn estudante_nao_possui_privilegios_administrativos_por_favorito(_world: &mut AppWorld) {
    pending_step(
        "o estudante não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito",
    );
}

/// Pendente: está autenticado no aplicativo.
#[given("está autenticado no aplicativo")]
#[given("que está autenticado no aplicativo")]
fn esta_autenticado_no_aplicativo(_world: &mut AppWorld) {
    pending_step("está autenticado no aplicativo");
}

/// Pendente: está com o contexto ativo em {string}.
#[given(expr = "está com o contexto ativo em {string}")]
#[given(expr = "que está com o contexto ativo em {string}")]
#[when(expr = "o estudante está com o contexto ativo em {string}")]
fn contexto_ativo_em(_world: &mut AppWorld, _ca: String) {
    pending_step("está com o contexto ativo em {string}");
}

/// Pendente: o estudante altera o contexto ativo para {string}.
#[when(expr = "o estudante altera o contexto ativo para {string}")]
fn estudante_altera_contexto_ativo(_world: &mut AppWorld, _ca: String) {
    pending_step("o estudante altera o contexto ativo para {string}");
}

/// Pendente: a preferência de CA favorito do estudante permanece {string}.
#[then(expr = "a preferência de CA favorito do estudante permanece {string}")]
fn preferencia_ca_favorito_permanece(_world: &mut AppWorld, _ca: String) {
    pending_step("a preferência de CA favorito do estudante permanece {string}");
}

/// Pendente: que o usuário {string} é do curso {string}.
#[given(expr = "que o usuário {string} é do curso {string}")]
fn usuario_e_do_curso(_world: &mut AppWorld, _user: String, _curso: String) {
    pending_step("que o usuário {string} é do curso {string}");
}

/// Pendente: o Centro Acadêmico {string} representa os cursos {string}.
#[given(expr = "o Centro Acadêmico {string} representa os cursos {string}")]
fn ca_representa_cursos(_world: &mut AppWorld, _ca: String, _cursos: String) {
    pending_step("o Centro Acadêmico {string} representa os cursos {string}");
}

/// Pendente: o Centro Acadêmico {string} é o CA favorito do usuário {string}.
#[given(expr = "o Centro Acadêmico {string} é o CA favorito do usuário {string}")]
fn ca_e_favorito_do_usuario(_world: &mut AppWorld, _ca: String, _user: String) {
    pending_step("o Centro Acadêmico {string} é o CA favorito do usuário {string}");
}

/// Pendente: o estudante não deve ter acesso às funcionalidades exclusivas de membros nos Centros Acadêmicos {string} e {string}.
#[then(
    expr = "o estudante não deve ter acesso às funcionalidades exclusivas de membros nos Centros Acadêmicos {string} e {string}"
)]
fn estudante_nao_deve_ter_acesso_a_membros(_world: &mut AppWorld, _ca1: String, _ca2: String) {
    pending_step(
        "o estudante não deve ter acesso às funcionalidades exclusivas de membros nos Centros Acadêmicos {string} e {string}",
    );
}

/// Pendente: que o estudante {string} exerce o cargo de {string} em {string}.
#[given(expr = "que o estudante {string} exerce o cargo de {string} em {string}")]
fn estudante_exerce_cargo_especifico(
    _world: &mut AppWorld,
    _email: String,
    _cargo: String,
    _ca: String,
) {
    pending_step("que o estudante {string} exerce o cargo de {string} em {string}");
}

/// Pendente: o estudante exerce o cargo de {string} em {string}.
#[given(expr = "o estudante exerce o cargo de {string} em {string}")]
fn estudante_exerce_cargo_generico(_world: &mut AppWorld, _cargo: String, _ca: String) {
    pending_step("o estudante exerce o cargo de {string} em {string}");
}

/// Pendente: o cargo possui a permissão de {string}.
#[given(expr = "o cargo possui a permissão de {string}")]
fn cargo_possui_permissao(_world: &mut AppWorld, _permissao: String) {
    pending_step("o cargo possui a permissão de {string}");
}

/// Pendente: o estudante não possui cargo de gestão em {string}.
#[given(expr = "o estudante não possui cargo de gestão em {string}")]
fn estudante_nao_possui_cargo_gestao(_world: &mut AppWorld, _ca: String) {
    pending_step("o estudante não possui cargo de gestão em {string}");
}

/// Pendente: o estudante possui permissão para {string} no CA ativo.
#[then(expr = "o estudante possui permissão para {string} no CA ativo")]
fn estudante_possui_permissao_no_ca_ativo(_world: &mut AppWorld, _permissao: String) {
    pending_step("o estudante possui permissão para {string} no CA ativo");
}

/// Pendente: o estudante não possui permissão para {string} no CA ativo.
#[then(expr = "o estudante não possui permissão para {string} no CA ativo")]
#[then(expr = "não possui permissão para {string} no CA ativo")]
fn estudante_nao_possui_permissao_no_ca_ativo(_world: &mut AppWorld, _permissao: String) {
    pending_step("o estudante não possui permissão para {string} no CA ativo");
}

/// Pendente: atua estritamente como visitante no novo contexto.
#[then("atua estritamente como visitante no novo contexto")]
fn atua_estritamente_como_visitante(_world: &mut AppWorld) {
    pending_step("atua estritamente como visitante no novo contexto");
}
