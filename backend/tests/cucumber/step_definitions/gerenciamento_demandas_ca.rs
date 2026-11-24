//! O gerenciamento de demandas permite à gestão do CA analisar solicitações e atualizar seus status.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que existem as seguintes demandas no fórum institucional do curso:.
#[given(regex = r#"^que existem as seguintes demandas no fórum institucional do curso:$"#)]
fn given_que_existem_as_seguintes_demandas_no_forum_institucional_do_curso(_world: &mut AppWorld) {
    pending_step("que existem as seguintes demandas no fórum institucional do curso:");
}

/// Pendente: que existem as seguintes demandas no fórum do espaço físico:.
#[given(regex = r#"^que existem as seguintes demandas no fórum do espaço físico:$"#)]
fn given_que_existem_as_seguintes_demandas_no_forum_do_espaco_fisico(_world: &mut AppWorld) {
    pending_step("que existem as seguintes demandas no fórum do espaço físico:");
}

/// Pendente: existe a demanda institucional {string} associada somente ao {string}.
#[given(regex = r#"^existe a demanda institucional "([^"]*)" associada somente ao "([^"]*)"$"#)]
fn given_existe_a_demanda_institucional_valor_associada_somente_ao_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe a demanda institucional {string} associada somente ao {string}");
}

/// Pendente: que existe a demanda institucional {string} com o status {string}.
#[given(regex = r#"^que existe a demanda institucional "([^"]*)" com o status "([^"]*)"$"#)]
fn given_que_existe_a_demanda_institucional_valor_com_o_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe a demanda institucional {string} com o status {string}");
}

/// Pendente: que a demanda institucional {string} possui o status {string}.
#[given(regex = r#"^que a demanda institucional "([^"]*)" possui o status "([^"]*)"$"#)]
fn given_que_a_demanda_institucional_valor_possui_o_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que a demanda institucional {string} possui o status {string}");
}

/// Pendente: que existe a demanda {string} sobre o espaço físico com o status {string}.
#[given(regex = r#"^que existe a demanda "([^"]*)" sobre o espaço físico com o status "([^"]*)"$"#)]
fn given_que_existe_a_demanda_valor_sobre_o_espaco_fisico_com_o_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe a demanda {string} sobre o espaço físico com o status {string}");
}

/// Pendente: existe a demanda institucional {string} com o status {string}.
#[given(regex = r#"^existe a demanda institucional "([^"]*)" com o status "([^"]*)"$"#)]
fn given_existe_a_demanda_institucional_valor_com_o_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe a demanda institucional {string} com o status {string}");
}

/// Pendente: existe a demanda institucional {string} no {string} com o status {string}.
#[given(
    regex = r#"^existe a demanda institucional "([^"]*)" no "([^"]*)" com o status "([^"]*)"$"#
)]
fn given_existe_a_demanda_institucional_valor_no_valor_com_o_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("existe a demanda institucional {string} no {string} com o status {string}");
}

/// Pendente: consulto administrativamente as demandas do fórum institucional.
#[when(regex = r#"^consulto administrativamente as demandas do fórum institucional$"#)]
fn when_consulto_administrativamente_as_demandas_do_forum_institucional(_world: &mut AppWorld) {
    pending_step("consulto administrativamente as demandas do fórum institucional");
}

/// Pendente: consulto administrativamente as demandas do espaço físico.
#[when(regex = r#"^consulto administrativamente as demandas do espaço físico$"#)]
fn when_consulto_administrativamente_as_demandas_do_espaco_fisico(_world: &mut AppWorld) {
    pending_step("consulto administrativamente as demandas do espaço físico");
}

/// Pendente: filtro as demandas institucionais pelo status {string}.
#[when(regex = r#"^filtro as demandas institucionais pelo status "([^"]*)"$"#)]
fn when_filtro_as_demandas_institucionais_pelo_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("filtro as demandas institucionais pelo status {string}");
}

/// Pendente: atualizo o status da demanda {string} para {string} com a justificativa {string}.
#[when(
    regex = r#"^atualizo o status da demanda "([^"]*)" para "([^"]*)" com a justificativa "([^"]*)"$"#
)]
fn when_atualizo_o_status_da_demanda_valor_para_valor_com_a_justificativa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "atualizo o status da demanda {string} para {string} com a justificativa {string}",
    );
}

/// Pendente: tento atualizar o status da demanda {string} para {string} com a justificativa {string}.
#[when(
    regex = r#"^tento atualizar o status da demanda "([^"]*)" para "([^"]*)" com a justificativa "([^"]*)"$"#
)]
fn when_tento_atualizar_o_status_da_demanda_valor_para_valor_com_a_justificativa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "tento atualizar o status da demanda {string} para {string} com a justificativa {string}",
    );
}

/// Pendente: tento atualizar o status da demanda {string} para {string} sem informar justificativa.
#[when(
    regex = r#"^tento atualizar o status da demanda "([^"]*)" para "([^"]*)" sem informar justificativa$"#
)]
fn when_tento_atualizar_o_status_da_demanda_valor_para_valor_sem_informar_justificativa(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "tento atualizar o status da demanda {string} para {string} sem informar justificativa",
    );
}

/// Pendente: vejo a demanda {string} com o status {string} e autor {string}.
#[then(regex = r#"^vejo a demanda "([^"]*)" com o status "([^"]*)" e autor "([^"]*)"$"#)]
fn then_vejo_a_demanda_valor_com_o_status_valor_e_autor_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("vejo a demanda {string} com o status {string} e autor {string}");
}

/// Pendente: vejo a demanda {string} com o status {string}.
#[then(regex = r#"^vejo a demanda "([^"]*)" com o status "([^"]*)"$"#)]
fn then_vejo_a_demanda_valor_com_o_status_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("vejo a demanda {string} com o status {string}");
}

/// Pendente: vejo a demanda {string} na listagem.
#[then(regex = r#"^vejo a demanda "([^"]*)" na listagem$"#)]
fn then_vejo_a_demanda_valor_na_listagem(_world: &mut AppWorld, _value_1: String) {
    pending_step("vejo a demanda {string} na listagem");
}

/// Pendente: não vejo a demanda {string} na listagem.
#[then(regex = r#"^não vejo a demanda "([^"]*)" na listagem$"#)]
fn then_nao_vejo_a_demanda_valor_na_listagem(_world: &mut AppWorld, _value_1: String) {
    pending_step("não vejo a demanda {string} na listagem");
}

/// Pendente: a demanda {string} não é apresentada para a gestão do {string}.
#[then(regex = r#"^a demanda "([^"]*)" não é apresentada para a gestão do "([^"]*)"$"#)]
fn then_a_demanda_valor_nao_e_apresentada_para_a_gestao_do_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a demanda {string} não é apresentada para a gestão do {string}");
}

/// Pendente: o status atual da demanda {string} passa a ser {string}.
#[then(regex = r#"^o status atual da demanda "([^"]*)" passa a ser "([^"]*)"$"#)]
fn then_o_status_atual_da_demanda_valor_passa_a_ser_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o status atual da demanda {string} passa a ser {string}");
}

/// Pendente: o parecer registrado na demanda é {string}.
#[then(regex = r#"^o parecer registrado na demanda é "([^"]*)"$"#)]
fn then_o_parecer_registrado_na_demanda_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o parecer registrado na demanda é {string}");
}

/// Pendente: o histórico registra a alteração realizada por {string}.
#[then(regex = r#"^o histórico registra a alteração realizada por "([^"]*)"$"#)]
fn then_o_historico_registra_a_alteracao_realizada_por_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o histórico registra a alteração realizada por {string}");
}

/// Pendente: a demanda institucional registra o encaminhamento à coordenação do curso.
#[then(regex = r#"^a demanda institucional registra o encaminhamento à coordenação do curso$"#)]
fn then_a_demanda_institucional_registra_o_encaminhamento_a_coordenacao_do_curso(
    _world: &mut AppWorld,
) {
    pending_step("a demanda institucional registra o encaminhamento à coordenação do curso");
}

/// Pendente: a alteração de status é recusada informando que demandas de espaço físico não podem ser encaminhadas à coordenação.
#[then(
    regex = r#"^a alteração de status é recusada informando que demandas de espaço físico não podem ser encaminhadas à coordenação$"#
)]
fn then_a_alteracao_de_status_e_recusada_informando_que_demandas_de_espaco_fisico_nao_podem_ser_encaminhadas_a_coordenacao(
    _world: &mut AppWorld,
) {
    pending_step(
        "a alteração de status é recusada informando que demandas de espaço físico não podem ser encaminhadas à coordenação",
    );
}

/// Pendente: o status da demanda {string} permanece {string}.
#[then(regex = r#"^o status da demanda "([^"]*)" permanece "([^"]*)"$"#)]
fn then_o_status_da_demanda_valor_permanece_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o status da demanda {string} permanece {string}");
}

/// Pendente: a alteração de status é recusada informando transição inválida.
#[then(regex = r#"^a alteração de status é recusada informando transição inválida$"#)]
fn then_a_alteracao_de_status_e_recusada_informando_transicao_invalida(_world: &mut AppWorld) {
    pending_step("a alteração de status é recusada informando transição inválida");
}

/// Pendente: a alteração de status é recusada informando que a justificativa é obrigatória.
#[then(
    regex = r#"^a alteração de status é recusada informando que a justificativa é obrigatória$"#
)]
fn then_a_alteracao_de_status_e_recusada_informando_que_a_justificativa_e_obrigatoria(
    _world: &mut AppWorld,
) {
    pending_step("a alteração de status é recusada informando que a justificativa é obrigatória");
}
