//! A consulta de notícias precisa refletir o contexto ativo, a publicação e a ordenação selecionada.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: escolhi a ordenação {string}.
#[given(regex = r#"^escolhi a ordenação "([^"]*)"$"#)]
fn given_escolhi_a_ordenacao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("escolhi a ordenação {string}");
}

/// Pendente: existe a notícia publicada {string} nesse CA.
#[given(regex = r#"^existe a notícia publicada "([^"]*)" nesse CA$"#)]
fn given_existe_a_noticia_publicada_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe a notícia publicada {string} nesse CA");
}

/// Pendente: existe a notícia publicada {string} no CA {string}.
#[given(regex = r#"^existe a notícia publicada "([^"]*)" no CA "([^"]*)"$"#)]
fn given_existe_a_noticia_publicada_valor_no_ca_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe a notícia publicada {string} no CA {string}");
}

/// Pendente: existe a notícia {string} arquivada nesse CA.
#[given(regex = r#"^existe a notícia "([^"]*)" arquivada nesse CA$"#)]
fn given_existe_a_noticia_valor_arquivada_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe a notícia {string} arquivada nesse CA");
}

/// Pendente: existe a notícia {string} em rascunho nesse CA.
#[given(regex = r#"^existe a notícia "([^"]*)" em rascunho nesse CA$"#)]
fn given_existe_a_noticia_valor_em_rascunho_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe a notícia {string} em rascunho nesse CA");
}

/// Pendente: existe a notícia {string} pendente de homologação nesse CA.
#[given(regex = r#"^existe a notícia "([^"]*)" pendente de homologação nesse CA$"#)]
fn given_existe_a_noticia_valor_pendente_de_homologacao_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("existe a notícia {string} pendente de homologação nesse CA");
}

/// Pendente: existe a notícia {string} rejeitada nesse CA.
#[given(regex = r#"^existe a notícia "([^"]*)" rejeitada nesse CA$"#)]
fn given_existe_a_noticia_valor_rejeitada_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe a notícia {string} rejeitada nesse CA");
}

/// Pendente: existem as notícias publicadas {string} e {string} nesse CA.
#[given(regex = r#"^existem as notícias publicadas "([^"]*)" e "([^"]*)" nesse CA$"#)]
fn given_existem_as_noticias_publicadas_valor_e_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existem as notícias publicadas {string} e {string} nesse CA");
}

/// Pendente: existem as seguintes notícias publicadas nesse CA:.
#[given(regex = r#"^existem as seguintes notícias publicadas nesse CA:$"#)]
fn given_existem_as_seguintes_noticias_publicadas_nesse_ca(_world: &mut AppWorld) {
    pending_step("existem as seguintes notícias publicadas nesse CA:");
}

/// Pendente: não existem notícias publicadas nesse CA.
#[given(regex = r#"^não existem notícias publicadas nesse CA$"#)]
fn given_nao_existem_noticias_publicadas_nesse_ca(_world: &mut AppWorld) {
    pending_step("não existem notícias publicadas nesse CA");
}

/// Pendente: o conteúdo da notícia é {string}.
#[given(regex = r#"^o conteúdo da notícia é "([^"]*)"$"#)]
fn given_o_conteudo_da_noticia_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o conteúdo da notícia é {string}");
}

/// Pendente: que o CA ativo possui as notícias publicadas {string} em {string} e {string} em {string}.
#[given(
    regex = r#"^que o CA ativo possui as notícias publicadas "([^"]*)" em "([^"]*)" e "([^"]*)" em "([^"]*)"$"#
)]
fn given_que_o_ca_ativo_possui_as_noticias_publicadas_valor_em_valor_e_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step(
        "que o CA ativo possui as notícias publicadas {string} em {string} e {string} em {string}",
    );
}

/// Pendente: a listagem apresenta {string} antes de {string}.
#[then(regex = r#"^a listagem apresenta "([^"]*)" antes de "([^"]*)"$"#)]
fn then_a_listagem_apresenta_valor_antes_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a listagem apresenta {string} antes de {string}");
}

/// Pendente: a listagem informa que não há notícias publicadas.
#[then(regex = r#"^a listagem informa que não há notícias publicadas$"#)]
fn then_a_listagem_informa_que_nao_ha_noticias_publicadas(_world: &mut AppWorld) {
    pending_step("a listagem informa que não há notícias publicadas");
}

/// Pendente: a ordenação não altera quais notícias estão publicadas.
#[then(regex = r#"^a ordenação não altera quais notícias estão publicadas$"#)]
fn then_a_ordenacao_nao_altera_quais_noticias_estao_publicadas(_world: &mut AppWorld) {
    pending_step("a ordenação não altera quais notícias estão publicadas");
}

/// Pendente: as notícias aparecem nesta ordem:.
#[then(regex = r#"^as notícias aparecem nesta ordem:$"#)]
fn then_as_noticias_aparecem_nesta_ordem(_world: &mut AppWorld) {
    pending_step("as notícias aparecem nesta ordem:");
}

/// Pendente: não vejo notícias em rascunho, pendentes de homologação, rejeitadas ou arquivadas.
#[then(
    regex = r#"^não vejo notícias em rascunho, pendentes de homologação, rejeitadas ou arquivadas$"#
)]
fn then_nao_vejo_noticias_em_rascunho_pendentes_de_homologacao_rejeitadas_ou_arquivadas(
    _world: &mut AppWorld,
) {
    pending_step(
        "não vejo notícias em rascunho, pendentes de homologação, rejeitadas ou arquivadas",
    );
}

/// Pendente: o conteúdo da notícia não é disponibilizado.
#[then(regex = r#"^o conteúdo da notícia não é disponibilizado$"#)]
fn then_o_conteudo_da_noticia_nao_e_disponibilizado(_world: &mut AppWorld) {
    pending_step("o conteúdo da notícia não é disponibilizado");
}

/// Pendente: somente notícias disponíveis no contexto ativo são apresentadas.
#[then(regex = r#"^somente notícias disponíveis no contexto ativo são apresentadas$"#)]
fn then_somente_noticias_disponiveis_no_contexto_ativo_sao_apresentadas(_world: &mut AppWorld) {
    pending_step("somente notícias disponíveis no contexto ativo são apresentadas");
}

/// Pendente: vejo o conteúdo {string}.
#[then(regex = r#"^vejo o conteúdo "([^"]*)"$"#)]
fn then_vejo_o_conteudo_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo o conteúdo {string}");
}

/// Pendente: consulto a listagem de notícias.
#[when(regex = r#"^consulto a listagem de notícias$"#)]
fn when_consulto_a_listagem_de_noticias(_world: &mut AppWorld) {
    pending_step("consulto a listagem de notícias");
}

/// Pendente: consulto novamente a listagem de notícias.
#[when(regex = r#"^consulto novamente a listagem de notícias$"#)]
fn when_consulto_novamente_a_listagem_de_noticias(_world: &mut AppWorld) {
    pending_step("consulto novamente a listagem de notícias");
}

/// Pendente: consulto os detalhes da notícia {string}.
#[when(regex = r#"^consulto os detalhes da notícia "([^"]*)"$"#)]
fn when_consulto_os_detalhes_da_noticia_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("consulto os detalhes da notícia {string}");
}

/// Pendente: escolho a ordenação {string}.
#[when(regex = r#"^escolho a ordenação "([^"]*)"$"#)]
fn when_escolho_a_ordenacao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("escolho a ordenação {string}");
}

/// Pendente: tento consultar os detalhes da notícia {string}.
#[when(regex = r#"^tento consultar os detalhes da notícia "([^"]*)"$"#)]
fn when_tento_consultar_os_detalhes_da_noticia_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("tento consultar os detalhes da notícia {string}");
}
