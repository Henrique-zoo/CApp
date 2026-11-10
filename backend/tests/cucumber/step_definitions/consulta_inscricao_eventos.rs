//! A agenda apresenta eventos publicados e permite ao estudante administrar sua própria inscrição.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: ainda não estou inscrito nesse evento.
#[given(regex = r#"^ainda não estou inscrito nesse evento$"#)]
fn given_ainda_nao_estou_inscrito_nesse_evento(_world: &mut AppWorld) {
    pending_step("ainda não estou inscrito nesse evento");
}

/// Pendente: estou consultando a agenda pública de eventos.
#[given(regex = r#"^estou consultando a agenda pública de eventos$"#)]
fn given_estou_consultando_a_agenda_publica_de_eventos(_world: &mut AppWorld) {
    pending_step("estou consultando a agenda pública de eventos");
}

/// Pendente: estou inscrito no evento publicado {string}.
#[given(regex = r#"^estou inscrito no evento publicado "([^"]*)"$"#)]
fn given_estou_inscrito_no_evento_publicado_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("estou inscrito no evento publicado {string}");
}

/// Pendente: existe o evento publicado {string} nesse CA.
#[given(regex = r#"^existe o evento publicado "([^"]*)" nesse CA$"#)]
fn given_existe_o_evento_publicado_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe o evento publicado {string} nesse CA");
}

/// Pendente: existe o evento publicado {string} no CA {string}.
#[given(regex = r#"^existe o evento publicado "([^"]*)" no CA "([^"]*)"$"#)]
fn given_existe_o_evento_publicado_valor_no_ca_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe o evento publicado {string} no CA {string}");
}

/// Pendente: existe o evento {string} não publicado nesse CA.
#[given(regex = r#"^existe o evento "([^"]*)" não publicado nesse CA$"#)]
fn given_existe_o_evento_valor_nao_publicado_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe o evento {string} não publicado nesse CA");
}

/// Pendente: existe o evento {string} proposto e ainda não homologado nesse CA.
#[given(regex = r#"^existe o evento "([^"]*)" proposto e ainda não homologado nesse CA$"#)]
fn given_existe_o_evento_valor_proposto_e_ainda_nao_homologado_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("existe o evento {string} proposto e ainda não homologado nesse CA");
}

/// Pendente: existe o evento {string} proposto e ainda não homologado no CA {string}.
#[given(regex = r#"^existe o evento "([^"]*)" proposto e ainda não homologado no CA "([^"]*)"$"#)]
fn given_existe_o_evento_valor_proposto_e_ainda_nao_homologado_no_ca_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe o evento {string} proposto e ainda não homologado no CA {string}");
}

/// Pendente: existem os eventos publicados {string} e {string} nesse CA.
#[given(regex = r#"^existem os eventos publicados "([^"]*)" e "([^"]*)" nesse CA$"#)]
fn given_existem_os_eventos_publicados_valor_e_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existem os eventos publicados {string} e {string} nesse CA");
}

/// Pendente: já estou inscrito nesse evento.
#[given(regex = r#"^já estou inscrito nesse evento$"#)]
fn given_ja_estou_inscrito_nesse_evento(_world: &mut AppWorld) {
    pending_step("já estou inscrito nesse evento");
}

/// Pendente: não estou inscrito no evento publicado {string}.
#[given(regex = r#"^não estou inscrito no evento publicado "([^"]*)"$"#)]
fn given_nao_estou_inscrito_no_evento_publicado_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não estou inscrito no evento publicado {string}");
}

/// Pendente: não existem eventos publicados nesse CA.
#[given(regex = r#"^não existem eventos publicados nesse CA$"#)]
fn given_nao_existem_eventos_publicados_nesse_ca(_world: &mut AppWorld) {
    pending_step("não existem eventos publicados nesse CA");
}

/// Pendente: o evento contém a informação {string}.
#[given(regex = r#"^o evento contém a informação "([^"]*)"$"#)]
fn given_o_evento_contem_a_informacao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o evento contém a informação {string}");
}

/// Pendente: que não foi possível obter os eventos publicados do CA ativo.
#[given(regex = r#"^que não foi possível obter os eventos publicados do CA ativo$"#)]
fn given_que_nao_foi_possivel_obter_os_eventos_publicados_do_ca_ativo(_world: &mut AppWorld) {
    pending_step("que não foi possível obter os eventos publicados do CA ativo");
}

/// Pendente: a listagem informa que não há eventos publicados.
#[then(regex = r#"^a listagem informa que não há eventos publicados$"#)]
fn then_a_listagem_informa_que_nao_ha_eventos_publicados(_world: &mut AppWorld) {
    pending_step("a listagem informa que não há eventos publicados");
}

/// Pendente: minha inscrição continua única.
#[then(regex = r#"^minha inscrição continua única$"#)]
fn then_minha_inscricao_continua_unica(_world: &mut AppWorld) {
    pending_step("minha inscrição continua única");
}

/// Pendente: minha inscrição no evento deixa de estar ativa.
#[then(regex = r#"^minha inscrição no evento deixa de estar ativa$"#)]
fn then_minha_inscricao_no_evento_deixa_de_estar_ativa(_world: &mut AppWorld) {
    pending_step("minha inscrição no evento deixa de estar ativa");
}

/// Pendente: minha inscrição no evento é registrada.
#[then(regex = r#"^minha inscrição no evento é registrada$"#)]
fn then_minha_inscricao_no_evento_e_registrada(_world: &mut AppWorld) {
    pending_step("minha inscrição no evento é registrada");
}

/// Pendente: minha inscrição não é registrada.
#[then(regex = r#"^minha inscrição não é registrada$"#)]
fn then_minha_inscricao_nao_e_registrada(_world: &mut AppWorld) {
    pending_step("minha inscrição não é registrada");
}

/// Pendente: nenhuma inscrição é alterada.
#[then(regex = r#"^nenhuma inscrição é alterada$"#)]
fn then_nenhuma_inscricao_e_alterada(_world: &mut AppWorld) {
    pending_step("nenhuma inscrição é alterada");
}

/// Pendente: o aplicativo informa que a inscrição foi realizada.
#[then(regex = r#"^o aplicativo informa que a inscrição foi realizada$"#)]
fn then_o_aplicativo_informa_que_a_inscricao_foi_realizada(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que a inscrição foi realizada");
}

/// Pendente: o aplicativo informa que já estou inscrito nesse evento.
#[then(regex = r#"^o aplicativo informa que já estou inscrito nesse evento$"#)]
fn then_o_aplicativo_informa_que_ja_estou_inscrito_nesse_evento(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que já estou inscrito nesse evento");
}

/// Pendente: o aplicativo informa que não foi possível carregar os eventos.
#[then(regex = r#"^o aplicativo informa que não foi possível carregar os eventos$"#)]
fn then_o_aplicativo_informa_que_nao_foi_possivel_carregar_os_eventos(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que não foi possível carregar os eventos");
}

/// Pendente: o aplicativo informa que não há inscrição ativa para cancelar.
#[then(regex = r#"^o aplicativo informa que não há inscrição ativa para cancelar$"#)]
fn then_o_aplicativo_informa_que_nao_ha_inscricao_ativa_para_cancelar(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que não há inscrição ativa para cancelar");
}

/// Pendente: o aplicativo informa que o cancelamento foi realizado.
#[then(regex = r#"^o aplicativo informa que o cancelamento foi realizado$"#)]
fn then_o_aplicativo_informa_que_o_cancelamento_foi_realizado(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que o cancelamento foi realizado");
}

/// Pendente: o aplicativo informa que o evento não está disponível para inscrição.
#[then(regex = r#"^o aplicativo informa que o evento não está disponível para inscrição$"#)]
fn then_o_aplicativo_informa_que_o_evento_nao_esta_disponivel_para_inscricao(
    _world: &mut AppWorld,
) {
    pending_step("o aplicativo informa que o evento não está disponível para inscrição");
}

/// Pendente: os detalhes do evento não são disponibilizados.
#[then(regex = r#"^os detalhes do evento não são disponibilizados$"#)]
fn then_os_detalhes_do_evento_nao_sao_disponibilizados(_world: &mut AppWorld) {
    pending_step("os detalhes do evento não são disponibilizados");
}

/// Pendente: vejo a informação {string}.
#[then(regex = r#"^vejo a informação "([^"]*)"$"#)]
fn then_vejo_a_informacao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo a informação {string}");
}

/// Pendente: consulto a lista de eventos.
#[when(regex = r#"^consulto a lista de eventos$"#)]
fn when_consulto_a_lista_de_eventos(_world: &mut AppWorld) {
    pending_step("consulto a lista de eventos");
}

/// Pendente: consulto novamente a lista de eventos.
#[when(regex = r#"^consulto novamente a lista de eventos$"#)]
fn when_consulto_novamente_a_lista_de_eventos(_world: &mut AppWorld) {
    pending_step("consulto novamente a lista de eventos");
}

/// Pendente: consulto os detalhes do evento {string}.
#[when(regex = r#"^consulto os detalhes do evento "([^"]*)"$"#)]
fn when_consulto_os_detalhes_do_evento_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("consulto os detalhes do evento {string}");
}

/// Pendente: solicito cancelar minha inscrição no evento {string}.
#[when(regex = r#"^solicito cancelar minha inscrição no evento "([^"]*)"$"#)]
fn when_solicito_cancelar_minha_inscricao_no_evento_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("solicito cancelar minha inscrição no evento {string}");
}

/// Pendente: solicito novamente minha inscrição no evento {string}.
#[when(regex = r#"^solicito novamente minha inscrição no evento "([^"]*)"$"#)]
fn when_solicito_novamente_minha_inscricao_no_evento_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("solicito novamente minha inscrição no evento {string}");
}

/// Pendente: tento consultar os detalhes do evento {string}.
#[when(regex = r#"^tento consultar os detalhes do evento "([^"]*)"$"#)]
fn when_tento_consultar_os_detalhes_do_evento_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("tento consultar os detalhes do evento {string}");
}
