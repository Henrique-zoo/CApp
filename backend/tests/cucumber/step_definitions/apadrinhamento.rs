//! O pareamento manual precisa relacionar participantes válidos sob autorização do CA do programa.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: existe uma pessoa que não está inscrita no programa.
#[given(regex = r#"^existe uma pessoa que não está inscrita no programa$"#)]
fn given_existe_uma_pessoa_que_nao_esta_inscrita_no_programa(_world: &mut AppWorld) {
    pending_step("existe uma pessoa que não está inscrita no programa");
}

/// Pendente: existem participantes com inscrição válida no programa.
#[given(regex = r#"^existem participantes com inscrição válida no programa$"#)]
fn given_existem_participantes_com_inscricao_valida_no_programa(_world: &mut AppWorld) {
    pending_step("existem participantes com inscrição válida no programa");
}

/// Pendente: o gestor possui a permissão {string} no CA do programa.
#[given(regex = r#"^o gestor possui a permissão "([^"]*)" no CA do programa$"#)]
fn given_o_gestor_possui_a_permissao_valor_no_ca_do_programa(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o gestor possui a permissão {string} no CA do programa");
}

/// Pendente: o integrante da gestão não possui a permissão {string} nesse CA.
#[given(regex = r#"^o integrante da gestão não possui a permissão "([^"]*)" nesse CA$"#)]
fn given_o_integrante_da_gestao_nao_possui_a_permissao_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o integrante da gestão não possui a permissão {string} nesse CA");
}

/// Pendente: o integrante possui a permissão {string} somente em outro CA.
#[given(regex = r#"^o integrante possui a permissão "([^"]*)" somente em outro CA$"#)]
fn given_o_integrante_possui_a_permissao_valor_somente_em_outro_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o integrante possui a permissão {string} somente em outro CA");
}

/// Pendente: que o programa de apadrinhamento está configurado.
#[given(regex = r#"^que o programa de apadrinhamento está configurado$"#)]
fn given_que_o_programa_de_apadrinhamento_esta_configurado(_world: &mut AppWorld) {
    pending_step("que o programa de apadrinhamento está configurado");
}

/// Pendente: a associação entre os dois participantes é registrada.
#[then(regex = r#"^a associação entre os dois participantes é registrada$"#)]
fn then_a_associacao_entre_os_dois_participantes_e_registrada(_world: &mut AppWorld) {
    pending_step("a associação entre os dois participantes é registrada");
}

/// Pendente: a seleção é recusada.
#[then(regex = r#"^a seleção é recusada$"#)]
fn then_a_selecao_e_recusada(_world: &mut AppWorld) {
    pending_step("a seleção é recusada");
}

/// Pendente: nenhum pareamento é registrado.
#[then(regex = r#"^nenhum pareamento é registrado$"#)]
fn then_nenhum_pareamento_e_registrado(_world: &mut AppWorld) {
    pending_step("nenhum pareamento é registrado");
}

/// Pendente: nenhum pareamento é registrado com essa pessoa.
#[then(regex = r#"^nenhum pareamento é registrado com essa pessoa$"#)]
fn then_nenhum_pareamento_e_registrado_com_essa_pessoa(_world: &mut AppWorld) {
    pending_step("nenhum pareamento é registrado com essa pessoa");
}

/// Pendente: o gestor é informado de que precisa selecionar os dois participantes.
#[then(regex = r#"^o gestor é informado de que precisa selecionar os dois participantes$"#)]
fn then_o_gestor_e_informado_de_que_precisa_selecionar_os_dois_participantes(
    _world: &mut AppWorld,
) {
    pending_step("o gestor é informado de que precisa selecionar os dois participantes");
}

/// Pendente: o pareamento fica disponível na organização do programa.
#[then(regex = r#"^o pareamento fica disponível na organização do programa$"#)]
fn then_o_pareamento_fica_disponivel_na_organizacao_do_programa(_world: &mut AppWorld) {
    pending_step("o pareamento fica disponível na organização do programa");
}

/// Pendente: o pareamento não é registrado.
#[then(regex = r#"^o pareamento não é registrado$"#)]
fn then_o_pareamento_nao_e_registrado(_world: &mut AppWorld) {
    pending_step("o pareamento não é registrado");
}

/// Pendente: os pareamentos do programa permanecem inalterados.
#[then(regex = r#"^os pareamentos do programa permanecem inalterados$"#)]
fn then_os_pareamentos_do_programa_permanecem_inalterados(_world: &mut AppWorld) {
    pending_step("os pareamentos do programa permanecem inalterados");
}

/// Pendente: associa outro participante como afilhado.
#[when(regex = r#"^associa outro participante como afilhado$"#)]
fn when_associa_outro_participante_como_afilhado(_world: &mut AppWorld) {
    pending_step("associa outro participante como afilhado");
}

/// Pendente: ele seleciona um participante como padrinho.
#[when(regex = r#"^ele seleciona um participante como padrinho$"#)]
fn when_ele_seleciona_um_participante_como_padrinho(_world: &mut AppWorld) {
    pending_step("ele seleciona um participante como padrinho");
}

/// Pendente: ele tenta confirmar o pareamento sem selecionar um padrinho ou um afilhado.
#[when(regex = r#"^ele tenta confirmar o pareamento sem selecionar um padrinho ou um afilhado$"#)]
fn when_ele_tenta_confirmar_o_pareamento_sem_selecionar_um_padrinho_ou_um_afilhado(
    _world: &mut AppWorld,
) {
    pending_step("ele tenta confirmar o pareamento sem selecionar um padrinho ou um afilhado");
}

/// Pendente: ele tenta selecionar essa pessoa como padrinho ou afilhado.
#[when(regex = r#"^ele tenta selecionar essa pessoa como padrinho ou afilhado$"#)]
fn when_ele_tenta_selecionar_essa_pessoa_como_padrinho_ou_afilhado(_world: &mut AppWorld) {
    pending_step("ele tenta selecionar essa pessoa como padrinho ou afilhado");
}

/// Pendente: tenta confirmar um pareamento manual.
#[when(regex = r#"^tenta confirmar um pareamento manual$"#)]
fn when_tenta_confirmar_um_pareamento_manual(_world: &mut AppWorld) {
    pending_step("tenta confirmar um pareamento manual");
}

/// Pendente: tenta confirmar um pareamento manual neste programa.
#[when(regex = r#"^tenta confirmar um pareamento manual neste programa$"#)]
fn when_tenta_confirmar_um_pareamento_manual_neste_programa(_world: &mut AppWorld) {
    pending_step("tenta confirmar um pareamento manual neste programa");
}
