//! A consulta ao FAQ precisa encontrar respostas cadastradas no CA ativo.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: existe a pergunta frequente {string} com a resposta {string}.
#[given(regex = r#"^existe a pergunta frequente "([^"]*)" com a resposta "([^"]*)"$"#)]
fn given_existe_a_pergunta_frequente_valor_com_a_resposta_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe a pergunta frequente {string} com a resposta {string}");
}

/// Pendente: existe a pergunta frequente {string} nesse CA.
#[given(regex = r#"^existe a pergunta frequente "([^"]*)" nesse CA$"#)]
fn given_existe_a_pergunta_frequente_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe a pergunta frequente {string} nesse CA");
}

/// Pendente: existe a pergunta frequente {string} somente no {string}.
#[given(regex = r#"^existe a pergunta frequente "([^"]*)" somente no "([^"]*)"$"#)]
fn given_existe_a_pergunta_frequente_valor_somente_no_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe a pergunta frequente {string} somente no {string}");
}

/// Pendente: existe uma pergunta frequente que corresponde a {string} somente no {string}.
#[given(
    regex = r#"^existe uma pergunta frequente que corresponde a "([^"]*)" somente no "([^"]*)"$"#
)]
fn given_existe_uma_pergunta_frequente_que_corresponde_a_valor_somente_no_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe uma pergunta frequente que corresponde a {string} somente no {string}");
}

/// Pendente: existem perguntas frequentes que correspondem a {string} nesse CA.
#[given(regex = r#"^existem perguntas frequentes que correspondem a "([^"]*)" nesse CA$"#)]
fn given_existem_perguntas_frequentes_que_correspondem_a_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("existem perguntas frequentes que correspondem a {string} nesse CA");
}

/// Pendente: existem perguntas frequentes que não correspondem a {string} nesse CA.
#[given(regex = r#"^existem perguntas frequentes que não correspondem a "([^"]*)" nesse CA$"#)]
fn given_existem_perguntas_frequentes_que_nao_correspondem_a_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("existem perguntas frequentes que não correspondem a {string} nesse CA");
}

/// Pendente: nenhuma pergunta frequente desse CA corresponde a {string}.
#[given(regex = r#"^nenhuma pergunta frequente desse CA corresponde a "([^"]*)"$"#)]
fn given_nenhuma_pergunta_frequente_desse_ca_corresponde_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("nenhuma pergunta frequente desse CA corresponde a {string}");
}

/// Pendente: não existem perguntas frequentes disponíveis nesse CA.
#[given(regex = r#"^não existem perguntas frequentes disponíveis nesse CA$"#)]
fn given_nao_existem_perguntas_frequentes_disponiveis_nesse_ca(_world: &mut AppWorld) {
    pending_step("não existem perguntas frequentes disponíveis nesse CA");
}

/// Pendente: nenhuma pergunta frequente é apresentada.
#[then(regex = r#"^nenhuma pergunta frequente é apresentada$"#)]
fn then_nenhuma_pergunta_frequente_e_apresentada(_world: &mut AppWorld) {
    pending_step("nenhuma pergunta frequente é apresentada");
}

/// Pendente: não vejo a pergunta do {string}.
#[then(regex = r#"^não vejo a pergunta do "([^"]*)"$"#)]
fn then_nao_vejo_a_pergunta_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não vejo a pergunta do {string}");
}

/// Pendente: não vejo a pergunta {string}.
#[then(regex = r#"^não vejo a pergunta "([^"]*)"$"#)]
fn then_nao_vejo_a_pergunta_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não vejo a pergunta {string}");
}

/// Pendente: não vejo perguntas correspondentes à busca.
#[then(regex = r#"^não vejo perguntas correspondentes à busca$"#)]
fn then_nao_vejo_perguntas_correspondentes_a_busca(_world: &mut AppWorld) {
    pending_step("não vejo perguntas correspondentes à busca");
}

/// Pendente: não vejo perguntas correspondentes à busca no CA ativo.
#[then(regex = r#"^não vejo perguntas correspondentes à busca no CA ativo$"#)]
fn then_nao_vejo_perguntas_correspondentes_a_busca_no_ca_ativo(_world: &mut AppWorld) {
    pending_step("não vejo perguntas correspondentes à busca no CA ativo");
}

/// Pendente: não vejo perguntas que não correspondem à busca.
#[then(regex = r#"^não vejo perguntas que não correspondem à busca$"#)]
fn then_nao_vejo_perguntas_que_nao_correspondem_a_busca(_world: &mut AppWorld) {
    pending_step("não vejo perguntas que não correspondem à busca");
}

/// Pendente: vejo a pergunta {string}.
#[then(regex = r#"^vejo a pergunta "([^"]*)"$"#)]
fn then_vejo_a_pergunta_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo a pergunta {string}");
}

/// Pendente: vejo a resposta {string}.
#[then(regex = r#"^vejo a resposta "([^"]*)"$"#)]
fn then_vejo_a_resposta_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo a resposta {string}");
}

/// Pendente: vejo as perguntas correspondentes a {string}.
#[then(regex = r#"^vejo as perguntas correspondentes a "([^"]*)"$"#)]
fn then_vejo_as_perguntas_correspondentes_a_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo as perguntas correspondentes a {string}");
}

/// Pendente: busco no FAQ por {string}.
#[when(regex = r#"^busco no FAQ por "([^"]*)"$"#)]
fn when_busco_no_faq_por_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("busco no FAQ por {string}");
}

/// Pendente: consulto o FAQ.
#[when(regex = r#"^consulto o FAQ$"#)]
fn when_consulto_o_faq(_world: &mut AppWorld) {
    pending_step("consulto o FAQ");
}

/// Pendente: seleciono a pergunta {string}.
#[when(regex = r#"^seleciono a pergunta "([^"]*)"$"#)]
fn when_seleciono_a_pergunta_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciono a pergunta {string}");
}

/// Pendente: seleciono o {string} como CA ativo.
#[when(regex = r#"^seleciono o "([^"]*)" como CA ativo$"#)]
fn when_seleciono_o_valor_como_ca_ativo(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciono o {string} como CA ativo");
}
