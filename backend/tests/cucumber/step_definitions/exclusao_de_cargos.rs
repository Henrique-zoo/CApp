//! A exclusão de cargos precisa respeitar os vínculos ativos e preservar as gestões anteriores.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: o CA {string} possui os cargos {string}, {string}, {string} e {string}.
#[given(regex = r#"^o CA "([^"]*)" possui os cargos "([^"]*)", "([^"]*)", "([^"]*)" e "([^"]*)"$"#)]
fn given_o_ca_valor_possui_os_cargos_valor_valor_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
    _value_5: String,
) {
    pending_step("o CA {string} possui os cargos {string}, {string}, {string} e {string}");
}

/// Pendente: o cargo {string} de {string} possui a permissão de {string}.
#[given(regex = r#"^o cargo "([^"]*)" de "([^"]*)" possui a permissão de "([^"]*)"$"#)]
fn given_o_cargo_valor_de_valor_possui_a_permissao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o cargo {string} de {string} possui a permissão de {string}");
}

/// Pendente: o gestor {string} altera o cargo de {string} para {string} em {string}.
#[given(regex = r#"^o gestor "([^"]*)" altera o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$"#)]
fn given_o_gestor_valor_altera_o_cargo_de_valor_para_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} altera o cargo de {string} para {string} em {string}");
}

/// Pendente: que o CA {string} não possui o cargo {string}.
#[given(regex = r#"^que o CA "([^"]*)" não possui o cargo "([^"]*)"$"#)]
fn given_que_o_ca_valor_nao_possui_o_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o CA {string} não possui o cargo {string}");
}

/// Pendente: que o CA {string} possui a gestão anterior de 2024 com {string} no cargo {string}.
#[given(
    regex = r#"^que o CA "([^"]*)" possui a gestão anterior de 2024 com "([^"]*)" no cargo "([^"]*)"$"#
)]
fn given_que_o_ca_valor_possui_a_gestao_anterior_de_2024_com_valor_no_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "que o CA {string} possui a gestão anterior de 2024 com {string} no cargo {string}",
    );
}

/// Pendente: que o estudante {string} não possui cargo de gestão em {string}.
#[given(regex = r#"^que o estudante "([^"]*)" não possui cargo de gestão em "([^"]*)"$"#)]
fn given_que_o_estudante_valor_nao_possui_cargo_de_gestao_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o estudante {string} não possui cargo de gestão em {string}");
}

/// Pendente: que o gestor {string} excluiu o cargo {string} de {string}.
#[given(regex = r#"^que o gestor "([^"]*)" excluiu o cargo "([^"]*)" de "([^"]*)"$"#)]
fn given_que_o_gestor_valor_excluiu_o_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que o gestor {string} excluiu o cargo {string} de {string}");
}

/// Pendente: a estrutura atual de cargos de {string} não lista o cargo {string}.
#[then(regex = r#"^a estrutura atual de cargos de "([^"]*)" não lista o cargo "([^"]*)"$"#)]
fn then_a_estrutura_atual_de_cargos_de_valor_nao_lista_o_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a estrutura atual de cargos de {string} não lista o cargo {string}");
}

/// Pendente: a estrutura de cargos de {string} permanece inalterada.
#[then(regex = r#"^a estrutura de cargos de "([^"]*)" permanece inalterada$"#)]
fn then_a_estrutura_de_cargos_de_valor_permanece_inalterada(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a estrutura de cargos de {string} permanece inalterada");
}

/// Pendente: a gestão anterior de 2024 continua exibindo {string} no cargo {string}.
#[then(regex = r#"^a gestão anterior de 2024 continua exibindo "([^"]*)" no cargo "([^"]*)"$"#)]
fn then_a_gestao_anterior_de_2024_continua_exibindo_valor_no_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a gestão anterior de 2024 continua exibindo {string} no cargo {string}");
}

/// Pendente: o CA {string} mantém os cargos {string}, {string} e {string}.
#[then(regex = r#"^o CA "([^"]*)" mantém os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$"#)]
fn then_o_ca_valor_mantem_os_cargos_valor_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o CA {string} mantém os cargos {string}, {string} e {string}");
}

/// Pendente: o cargo {string} de {string} mantém a permissão de {string}.
#[then(regex = r#"^o cargo "([^"]*)" de "([^"]*)" mantém a permissão de "([^"]*)"$"#)]
fn then_o_cargo_valor_de_valor_mantem_a_permissao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o cargo {string} de {string} mantém a permissão de {string}");
}

/// Pendente: o gestor {string} exclui o cargo {string} de {string}.
#[when(regex = r#"^o gestor "([^"]*)" exclui o cargo "([^"]*)" de "([^"]*)"$"#)]
fn when_o_gestor_valor_exclui_o_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} exclui o cargo {string} de {string}");
}

/// Pendente: o gestor {string} tenta excluir o cargo {string} de {string}.
#[when(regex = r#"^o gestor "([^"]*)" tenta excluir o cargo "([^"]*)" de "([^"]*)"$"#)]
fn when_o_gestor_valor_tenta_excluir_o_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} tenta excluir o cargo {string} de {string}");
}

/// Pendente: {string} tenta excluir o cargo {string} de {string}.
#[when(regex = r#"^"([^"]*)" tenta excluir o cargo "([^"]*)" de "([^"]*)"$"#)]
fn when_valor_tenta_excluir_o_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} tenta excluir o cargo {string} de {string}");
}
