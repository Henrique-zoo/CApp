//! Os materiais publicados precisam permanecer acessíveis no contexto acadêmico e de CA selecionado.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: a descrição do material é {string}.
#[given(regex = r#"^a descrição do material é "([^"]*)"$"#)]
fn given_a_descricao_do_material_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a descrição do material é {string}");
}

/// Pendente: existe o material publicado {string} no CA {string}.
#[given(regex = r#"^existe o material publicado "([^"]*)" no CA "([^"]*)"$"#)]
fn given_existe_o_material_publicado_valor_no_ca_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe o material publicado {string} no CA {string}");
}

/// Pendente: existe o material {string} não publicado nesse CA.
#[given(regex = r#"^existe o material "([^"]*)" não publicado nesse CA$"#)]
fn given_existe_o_material_valor_nao_publicado_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe o material {string} não publicado nesse CA");
}

/// Pendente: existem os materiais publicados {string} e {string} nesse CA.
#[given(regex = r#"^existem os materiais publicados "([^"]*)" e "([^"]*)" nesse CA$"#)]
fn given_existem_os_materiais_publicados_valor_e_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existem os materiais publicados {string} e {string} nesse CA");
}

/// Pendente: não existem materiais publicados nesse CA.
#[given(regex = r#"^não existem materiais publicados nesse CA$"#)]
fn given_nao_existem_materiais_publicados_nesse_ca(_world: &mut AppWorld) {
    pending_step("não existem materiais publicados nesse CA");
}

/// Pendente: o material possui conteúdo disponível no formato {string}.
#[given(regex = r#"^o material possui conteúdo disponível no formato "([^"]*)"$"#)]
fn given_o_material_possui_conteudo_disponivel_no_formato_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o material possui conteúdo disponível no formato {string}");
}

/// Pendente: que não foi possível obter os materiais publicados do CA ativo.
#[given(regex = r#"^que não foi possível obter os materiais publicados do CA ativo$"#)]
fn given_que_nao_foi_possivel_obter_os_materiais_publicados_do_ca_ativo(_world: &mut AppWorld) {
    pending_step("que não foi possível obter os materiais publicados do CA ativo");
}

/// Pendente: a listagem informa que não há materiais publicados.
#[then(regex = r#"^a listagem informa que não há materiais publicados$"#)]
fn then_a_listagem_informa_que_nao_ha_materiais_publicados(_world: &mut AppWorld) {
    pending_step("a listagem informa que não há materiais publicados");
}

/// Pendente: o aplicativo informa que não foi possível carregar os materiais.
#[then(regex = r#"^o aplicativo informa que não foi possível carregar os materiais$"#)]
fn then_o_aplicativo_informa_que_nao_foi_possivel_carregar_os_materiais(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que não foi possível carregar os materiais");
}

/// Pendente: o conteúdo do material não é disponibilizado.
#[then(regex = r#"^o conteúdo do material não é disponibilizado$"#)]
fn then_o_conteudo_do_material_nao_e_disponibilizado(_world: &mut AppWorld) {
    pending_step("o conteúdo do material não é disponibilizado");
}

/// Pendente: o conteúdo publicado é disponibilizado no formato {string}.
#[then(regex = r#"^o conteúdo publicado é disponibilizado no formato "([^"]*)"$"#)]
fn then_o_conteudo_publicado_e_disponibilizado_no_formato_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o conteúdo publicado é disponibilizado no formato {string}");
}

/// Pendente: vejo a descrição {string}.
#[then(regex = r#"^vejo a descrição "([^"]*)"$"#)]
fn then_vejo_a_descricao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo a descrição {string}");
}

/// Pendente: acesso o conteúdo do material {string}.
#[when(regex = r#"^acesso o conteúdo do material "([^"]*)"$"#)]
fn when_acesso_o_conteudo_do_material_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("acesso o conteúdo do material {string}");
}

/// Pendente: consulto a lista de materiais.
#[when(regex = r#"^consulto a lista de materiais$"#)]
fn when_consulto_a_lista_de_materiais(_world: &mut AppWorld) {
    pending_step("consulto a lista de materiais");
}

/// Pendente: consulto novamente a lista de materiais.
#[when(regex = r#"^consulto novamente a lista de materiais$"#)]
fn when_consulto_novamente_a_lista_de_materiais(_world: &mut AppWorld) {
    pending_step("consulto novamente a lista de materiais");
}

/// Pendente: consulto os detalhes do material {string}.
#[when(regex = r#"^consulto os detalhes do material "([^"]*)"$"#)]
fn when_consulto_os_detalhes_do_material_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("consulto os detalhes do material {string}");
}

/// Pendente: tento acessar o conteúdo do material {string}.
#[when(regex = r#"^tento acessar o conteúdo do material "([^"]*)"$"#)]
fn when_tento_acessar_o_conteudo_do_material_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("tento acessar o conteúdo do material {string}");
}
