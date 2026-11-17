//! O fórum do espaço físico permite comunicar necessidades do CA sem exigir vínculo administrativo.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: não tenho vínculo de gestão com {string}.
#[given(regex = r#"^não tenho vínculo de gestão com "([^"]*)"$"#)]
fn given_nao_tenho_vinculo_de_gestao_com_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não tenho vínculo de gestão com {string}");
}

/// Pendente: a demanda fica associada somente ao {string}.
#[then(regex = r#"^a demanda fica associada somente ao "([^"]*)"$"#)]
fn then_a_demanda_fica_associada_somente_ao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a demanda fica associada somente ao {string}");
}

/// Pendente: a demanda não é criada.
#[then(regex = r#"^a demanda não é criada$"#)]
fn then_a_demanda_nao_e_criada(_world: &mut AppWorld) {
    pending_step("a demanda não é criada");
}

/// Pendente: a demanda não é encaminhada à coordenação do curso.
#[then(regex = r#"^a demanda não é encaminhada à coordenação do curso$"#)]
fn then_a_demanda_nao_e_encaminhada_a_coordenacao_do_curso(_world: &mut AppWorld) {
    pending_step("a demanda não é encaminhada à coordenação do curso");
}

/// Pendente: a demanda é criada no fórum do espaço físico do {string}.
#[then(regex = r#"^a demanda é criada no fórum do espaço físico do "([^"]*)"$"#)]
fn then_a_demanda_e_criada_no_forum_do_espaco_fisico_do_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a demanda é criada no fórum do espaço físico do {string}");
}

/// Pendente: o aplicativo informa que o conteúdo é obrigatório.
#[then(regex = r#"^o aplicativo informa que o conteúdo é obrigatório$"#)]
fn then_o_aplicativo_informa_que_o_conteudo_e_obrigatorio(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que o conteúdo é obrigatório");
}

/// Pendente: o autor da demanda é {string}.
#[then(regex = r#"^o autor da demanda é "([^"]*)"$"#)]
fn then_o_autor_da_demanda_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o autor da demanda é {string}");
}

/// Pendente: o conteúdo registrado é {string}.
#[then(regex = r#"^o conteúdo registrado é "([^"]*)"$"#)]
fn then_o_conteudo_registrado_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o conteúdo registrado é {string}");
}

/// Pendente: o status inicial da demanda é {string}.
#[then(regex = r#"^o status inicial da demanda é "([^"]*)"$"#)]
fn then_o_status_inicial_da_demanda_e_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o status inicial da demanda é {string}");
}

/// Pendente: registro uma demanda sobre o espaço físico com o conteúdo {string}.
#[when(regex = r#"^registro uma demanda sobre o espaço físico com o conteúdo "([^"]*)"$"#)]
fn when_registro_uma_demanda_sobre_o_espaco_fisico_com_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("registro uma demanda sobre o espaço físico com o conteúdo {string}");
}

/// Pendente: tento registrar uma demanda sobre o espaço físico sem informar o conteúdo.
#[when(regex = r#"^tento registrar uma demanda sobre o espaço físico sem informar o conteúdo$"#)]
fn when_tento_registrar_uma_demanda_sobre_o_espaco_fisico_sem_informar_o_conteudo(
    _world: &mut AppWorld,
) {
    pending_step("tento registrar uma demanda sobre o espaço físico sem informar o conteúdo");
}
