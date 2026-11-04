//! Steps compartilhados entre funcionalidades dos cenários de produto.
//!
//! O vocabulário reutilizado precisa de um registro único para evitar definições
//! ambíguas. Este módulo registra essas expressões na coleção do Cucumber usada
//! pela suíte do backend; os módulos de cada funcionalidade mantêm seus passos
//! específicos.
//!
//! Os passos ainda sem automação chamam `pending_step` e falham explicitamente se
//! executados. Ao implementar seu comportamento real, o registro permanece neste
//! módulo e as tags de pendência são removidas dos cenários já automatizados.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: estou autenticado como {string}.
#[given(regex = r#"^estou autenticado como "([^"]*)"$"#)]
fn given_estou_autenticado_como_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("estou autenticado como {string}");
}

/// Pendente: existe o material publicado {string} nesse CA.
#[given(regex = r#"^existe o material publicado "([^"]*)" nesse CA$"#)]
fn given_existe_o_material_publicado_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe o material publicado {string} nesse CA");
}

/// Pendente: não recebeu uma concessão separada de {string}.
#[given(regex = r#"^não recebeu uma concessão separada de "([^"]*)"$"#)]
fn given_nao_recebeu_uma_concessao_separada_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não recebeu uma concessão separada de {string}");
}

/// Pendente: o CA {string} possui os cargos {string}, {string} e {string}.
#[given(regex = r#"^o CA "([^"]*)" possui os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$"#)]
fn given_o_ca_valor_possui_os_cargos_valor_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o CA {string} possui os cargos {string}, {string} e {string}");
}

/// Pendente: o Centro Acadêmico {string} está cadastrado na UnB.
#[given(regex = r#"^o Centro Acadêmico "([^"]*)" está cadastrado na UnB$"#)]
fn given_o_centro_academico_valor_esta_cadastrado_na_unb(_world: &mut AppWorld, _value_1: String) {
    pending_step("o Centro Acadêmico {string} está cadastrado na UnB");
}

/// Pendente: o cargo {string} de {string} não possui integrante vinculado.
#[given(regex = r#"^o cargo "([^"]*)" de "([^"]*)" não possui integrante vinculado$"#)]
fn given_o_cargo_valor_de_valor_nao_possui_integrante_vinculado(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo {string} de {string} não possui integrante vinculado");
}

/// Pendente: o cargo {string} não possui a permissão de {string}.
#[given(regex = r#"^o cargo "([^"]*)" não possui a permissão de "([^"]*)"$"#)]
fn given_o_cargo_valor_nao_possui_a_permissao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo {string} não possui a permissão de {string}");
}

/// Pendente: o cargo {string} possui a permissão de {string}.
#[given(regex = r#"^o cargo "([^"]*)" possui a permissão de "([^"]*)"$"#)]
fn given_o_cargo_valor_possui_a_permissao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo {string} possui a permissão de {string}");
}

/// Pendente: o estudante {string} exerce o cargo de {string} em {string}.
#[given(regex = r#"^o estudante "([^"]*)" exerce o cargo de "([^"]*)" em "([^"]*)"$"#)]
fn given_o_estudante_valor_exerce_o_cargo_de_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o estudante {string} exerce o cargo de {string} em {string}");
}

/// Pendente: o estudante {string} não possui cargo de gestão em {string}.
#[given(regex = r#"^o estudante "([^"]*)" não possui cargo de gestão em "([^"]*)"$"#)]
fn given_o_estudante_valor_nao_possui_cargo_de_gestao_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante {string} não possui cargo de gestão em {string}");
}

/// Pendente: o estudante {string} possui perfil no CApp.
#[given(regex = r#"^o estudante "([^"]*)" possui perfil no CApp$"#)]
fn given_o_estudante_valor_possui_perfil_no_capp(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante {string} possui perfil no CApp");
}

/// Pendente: o integrante possui a permissão {string} nesse CA.
#[given(regex = r#"^o integrante possui a permissão "([^"]*)" nesse CA$"#)]
fn given_o_integrante_possui_a_permissao_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("o integrante possui a permissão {string} nesse CA");
}

/// Pendente: possuo a permissão {string} nesse CA.
#[given(regex = r#"^possuo a permissão "([^"]*)" nesse CA$"#)]
fn given_possuo_a_permissao_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("possuo a permissão {string} nesse CA");
}

/// Pendente: que estou autenticado no CApp.
#[given(regex = r#"^que estou autenticado no CApp$"#)]
fn given_que_estou_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("que estou autenticado no CApp");
}

/// Pendente: que o CA ativo é {string}.
#[given(regex = r#"^que o CA ativo é "([^"]*)"$"#)]
fn given_que_o_ca_ativo_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o CA ativo é {string}");
}

/// Pendente: que o Centro Acadêmico {string} está cadastrado na UnB.
#[given(regex = r#"^que o Centro Acadêmico "([^"]*)" está cadastrado na UnB$"#)]
fn given_que_o_centro_academico_valor_esta_cadastrado_na_unb(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o Centro Acadêmico {string} está cadastrado na UnB");
}

/// Pendente: que o cargo {string} de {string} não possui integrante vinculado.
#[given(regex = r#"^que o cargo "([^"]*)" de "([^"]*)" não possui integrante vinculado$"#)]
fn given_que_o_cargo_valor_de_valor_nao_possui_integrante_vinculado(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o cargo {string} de {string} não possui integrante vinculado");
}

/// Pendente: que o estudante {string} possui perfil no CApp.
#[given(regex = r#"^que o estudante "([^"]*)" possui perfil no CApp$"#)]
fn given_que_o_estudante_valor_possui_perfil_no_capp(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o estudante {string} possui perfil no CApp");
}

/// Pendente: que o histórico de {string} registra a remoção de {string} do cargo {string}.
#[given(
    regex = r#"^que o histórico de "([^"]*)" registra a remoção de "([^"]*)" do cargo "([^"]*)"$"#
)]
fn given_que_o_historico_de_valor_registra_a_remocao_de_valor_do_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que o histórico de {string} registra a remoção de {string} do cargo {string}");
}

/// Pendente: a operação é recusada informando que o cargo de Presidente não pode ser excluído.
#[then(
    regex = r#"^a operação é recusada informando que o cargo de Presidente não pode ser excluído$"#
)]
fn then_a_operacao_e_recusada_informando_que_o_cargo_de_presidente_nao_pode_ser_excluido(
    _world: &mut AppWorld,
) {
    pending_step(
        "a operação é recusada informando que o cargo de Presidente não pode ser excluído",
    );
}

/// Pendente: a operação é recusada informando que o cargo não existe no CA.
#[then(regex = r#"^a operação é recusada informando que o cargo não existe no CA$"#)]
fn then_a_operacao_e_recusada_informando_que_o_cargo_nao_existe_no_ca(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o cargo não existe no CA");
}

/// Pendente: a operação é recusada informando que o cargo possui integrante ativo.
#[then(regex = r#"^a operação é recusada informando que o cargo possui integrante ativo$"#)]
fn then_a_operacao_e_recusada_informando_que_o_cargo_possui_integrante_ativo(
    _world: &mut AppWorld,
) {
    pending_step("a operação é recusada informando que o cargo possui integrante ativo");
}

/// Pendente: a operação é recusada por falta de permissão.
#[then(regex = r#"^a operação é recusada por falta de permissão$"#)]
fn then_a_operacao_e_recusada_por_falta_de_permissao(_world: &mut AppWorld) {
    pending_step("a operação é recusada por falta de permissão");
}

/// Pendente: não vejo {string}.
#[then(regex = r#"^não vejo "([^"]*)"$"#)]
fn then_nao_vejo_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não vejo {string}");
}

/// Pendente: o CA {string} deixa de possuir o cargo {string}.
#[then(regex = r#"^o CA "([^"]*)" deixa de possuir o cargo "([^"]*)"$"#)]
fn then_o_ca_valor_deixa_de_possuir_o_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o CA {string} deixa de possuir o cargo {string}");
}

/// Pendente: o CA {string} mantém o cargo {string}.
#[then(regex = r#"^o CA "([^"]*)" mantém o cargo "([^"]*)"$"#)]
fn then_o_ca_valor_mantem_o_cargo_valor(_world: &mut AppWorld, _value_1: String, _value_2: String) {
    pending_step("o CA {string} mantém o cargo {string}");
}

/// Pendente: vejo o título {string}.
#[then(regex = r#"^vejo o título "([^"]*)"$"#)]
fn then_vejo_o_titulo_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo o título {string}");
}

/// Pendente: vejo {string}.
#[then(regex = r#"^vejo "([^"]*)"$"#)]
fn then_vejo_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo {string}");
}

/// Pendente: vejo {string} e {string}.
#[then(regex = r#"^vejo "([^"]*)" e "([^"]*)"$"#)]
fn then_vejo_valor_e_valor(_world: &mut AppWorld, _value_1: String, _value_2: String) {
    pending_step("vejo {string} e {string}");
}

/// Pendente: {string} continua podendo acessar {string} como estudante.
#[then(regex = r#"^"([^"]*)" continua podendo acessar "([^"]*)" como estudante$"#)]
fn then_valor_continua_podendo_acessar_valor_como_estudante(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} continua podendo acessar {string} como estudante");
}

/// Pendente: {string} deixa de integrar a gestão de {string}.
#[then(regex = r#"^"([^"]*)" deixa de integrar a gestão de "([^"]*)"$"#)]
fn then_valor_deixa_de_integrar_a_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} deixa de integrar a gestão de {string}");
}

/// Pendente: {string} não passa a integrar a gestão de {string}.
#[then(regex = r#"^"([^"]*)" não passa a integrar a gestão de "([^"]*)"$"#)]
fn then_valor_nao_passa_a_integrar_a_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} não passa a integrar a gestão de {string}");
}

/// Pendente: {string} permanece com a permissão de {string} em {string}.
#[then(regex = r#"^"([^"]*)" permanece com a permissão de "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_permanece_com_a_permissao_de_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} permanece com a permissão de {string} em {string}");
}

/// Pendente: {string} permanece no cargo {string} em {string}.
#[then(regex = r#"^"([^"]*)" permanece no cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_permanece_no_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} permanece no cargo {string} em {string}");
}

/// Pendente: altero o CA ativo para {string}.
#[when(regex = r#"^altero o CA ativo para "([^"]*)"$"#)]
fn when_altero_o_ca_ativo_para_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("altero o CA ativo para {string}");
}

/// Pendente: o gestor {string} remove {string} da gestão de {string}.
#[when(regex = r#"^o gestor "([^"]*)" remove "([^"]*)" da gestão de "([^"]*)"$"#)]
fn when_o_gestor_valor_remove_valor_da_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} remove {string} da gestão de {string}");
}

/// Pendente: o gestor {string} tenta vincular {string} ao cargo {string} em {string}.
#[when(regex = r#"^o gestor "([^"]*)" tenta vincular "([^"]*)" ao cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_o_gestor_valor_tenta_vincular_valor_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} tenta vincular {string} ao cargo {string} em {string}");
}

/// Pendente: solicito minha inscrição no evento {string}.
#[when(regex = r#"^solicito minha inscrição no evento "([^"]*)"$"#)]
fn when_solicito_minha_inscricao_no_evento_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("solicito minha inscrição no evento {string}");
}
