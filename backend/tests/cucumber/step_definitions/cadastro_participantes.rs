//! A inscrição no programa de apadrinhamento identifica o papel de calouro ou veterano.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: o calouro ainda não está inscrito nesse programa.
#[given(regex = r#"^o calouro ainda não está inscrito nesse programa$"#)]
fn given_o_calouro_ainda_nao_esta_inscrito_nesse_programa(_world: &mut AppWorld) {
    pending_step("o calouro ainda não está inscrito nesse programa");
}

/// Pendente: o calouro está autenticado no CApp.
#[given(regex = r#"^o calouro está autenticado no CApp$"#)]
fn given_o_calouro_esta_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("o calouro está autenticado no CApp");
}

/// Pendente: o estudante ainda não está inscrito nesse programa.
#[given(regex = r#"^o estudante ainda não está inscrito nesse programa$"#)]
fn given_o_estudante_ainda_nao_esta_inscrito_nesse_programa(_world: &mut AppWorld) {
    pending_step("o estudante ainda não está inscrito nesse programa");
}

/// Pendente: o estudante ainda não está inscrito no programa {string}.
#[given(regex = r#"^o estudante ainda não está inscrito no programa "([^"]*)"$"#)]
fn given_o_estudante_ainda_nao_esta_inscrito_no_programa_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante ainda não está inscrito no programa {string}");
}

/// Pendente: o estudante está autenticado no CApp.
#[given(regex = r#"^o estudante está autenticado no CApp$"#)]
fn given_o_estudante_esta_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("o estudante está autenticado no CApp");
}

/// Pendente: o estudante está inscrito no programa {string} com o papel {string}.
#[given(regex = r#"^o estudante está inscrito no programa "([^"]*)" com o papel "([^"]*)"$"#)]
fn given_o_estudante_esta_inscrito_no_programa_valor_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante está inscrito no programa {string} com o papel {string}");
}

/// Pendente: o estudante já está inscrito nesse programa com o papel {string}.
#[given(regex = r#"^o estudante já está inscrito nesse programa com o papel "([^"]*)"$"#)]
fn given_o_estudante_ja_esta_inscrito_nesse_programa_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante já está inscrito nesse programa com o papel {string}");
}

/// Pendente: o estudante não está autenticado no CApp.
#[given(regex = r#"^o estudante não está autenticado no CApp$"#)]
fn given_o_estudante_nao_esta_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("o estudante não está autenticado no CApp");
}

/// Pendente: o programa {string} está aberto para inscrições.
#[given(regex = r#"^o programa "([^"]*)" está aberto para inscrições$"#)]
fn given_o_programa_valor_esta_aberto_para_inscricoes(_world: &mut AppWorld, _value_1: String) {
    pending_step("o programa {string} está aberto para inscrições");
}

/// Pendente: o veterano ainda não está inscrito nesse programa.
#[given(regex = r#"^o veterano ainda não está inscrito nesse programa$"#)]
fn given_o_veterano_ainda_nao_esta_inscrito_nesse_programa(_world: &mut AppWorld) {
    pending_step("o veterano ainda não está inscrito nesse programa");
}

/// Pendente: o veterano está autenticado no CApp.
#[given(regex = r#"^o veterano está autenticado no CApp$"#)]
fn given_o_veterano_esta_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("o veterano está autenticado no CApp");
}

/// Pendente: que o estudante está autenticado no CApp.
#[given(regex = r#"^que o estudante está autenticado no CApp$"#)]
fn given_que_o_estudante_esta_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("que o estudante está autenticado no CApp");
}

/// Pendente: que o programa de apadrinhamento ainda está em configuração.
#[given(regex = r#"^que o programa de apadrinhamento ainda está em configuração$"#)]
fn given_que_o_programa_de_apadrinhamento_ainda_esta_em_configuracao(_world: &mut AppWorld) {
    pending_step("que o programa de apadrinhamento ainda está em configuração");
}

/// Pendente: que o programa de apadrinhamento está aberto para inscrições.
#[given(regex = r#"^que o programa de apadrinhamento está aberto para inscrições$"#)]
fn given_que_o_programa_de_apadrinhamento_esta_aberto_para_inscricoes(_world: &mut AppWorld) {
    pending_step("que o programa de apadrinhamento está aberto para inscrições");
}

/// Pendente: que o programa de apadrinhamento está encerrado.
#[given(regex = r#"^que o programa de apadrinhamento está encerrado$"#)]
fn given_que_o_programa_de_apadrinhamento_esta_encerrado(_world: &mut AppWorld) {
    pending_step("que o programa de apadrinhamento está encerrado");
}

/// Pendente: a inscrição do calouro é registrada no programa.
#[then(regex = r#"^a inscrição do calouro é registrada no programa$"#)]
fn then_a_inscricao_do_calouro_e_registrada_no_programa(_world: &mut AppWorld) {
    pending_step("a inscrição do calouro é registrada no programa");
}

/// Pendente: a inscrição do veterano é registrada no programa.
#[then(regex = r#"^a inscrição do veterano é registrada no programa$"#)]
fn then_a_inscricao_do_veterano_e_registrada_no_programa(_world: &mut AppWorld) {
    pending_step("a inscrição do veterano é registrada no programa");
}

/// Pendente: a inscrição no programa {string} é registrada com o papel {string}.
#[then(regex = r#"^a inscrição no programa "([^"]*)" é registrada com o papel "([^"]*)"$"#)]
fn then_a_inscricao_no_programa_valor_e_registrada_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a inscrição no programa {string} é registrada com o papel {string}");
}

/// Pendente: a inscrição no programa {string} permanece com o papel {string}.
#[then(regex = r#"^a inscrição no programa "([^"]*)" permanece com o papel "([^"]*)"$"#)]
fn then_a_inscricao_no_programa_valor_permanece_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a inscrição no programa {string} permanece com o papel {string}");
}

/// Pendente: a inscrição não é registrada.
#[then(regex = r#"^a inscrição não é registrada$"#)]
fn then_a_inscricao_nao_e_registrada(_world: &mut AppWorld) {
    pending_step("a inscrição não é registrada");
}

/// Pendente: a nova inscrição é recusada.
#[then(regex = r#"^a nova inscrição é recusada$"#)]
fn then_a_nova_inscricao_e_recusada(_world: &mut AppWorld) {
    pending_step("a nova inscrição é recusada");
}

/// Pendente: nenhum participante é inscrito nesse programa.
#[then(regex = r#"^nenhum participante é inscrito nesse programa$"#)]
fn then_nenhum_participante_e_inscrito_nesse_programa(_world: &mut AppWorld) {
    pending_step("nenhum participante é inscrito nesse programa");
}

/// Pendente: o calouro é informado de que o programa não está aberto para inscrições.
#[then(regex = r#"^o calouro é informado de que o programa não está aberto para inscrições$"#)]
fn then_o_calouro_e_informado_de_que_o_programa_nao_esta_aberto_para_inscricoes(
    _world: &mut AppWorld,
) {
    pending_step("o calouro é informado de que o programa não está aberto para inscrições");
}

/// Pendente: o estudante é informado de que a autenticação é obrigatória.
#[then(regex = r#"^o estudante é informado de que a autenticação é obrigatória$"#)]
fn then_o_estudante_e_informado_de_que_a_autenticacao_e_obrigatoria(_world: &mut AppWorld) {
    pending_step("o estudante é informado de que a autenticação é obrigatória");
}

/// Pendente: o estudante é informado de que precisa identificar o papel de calouro ou veterano.
#[then(
    regex = r#"^o estudante é informado de que precisa identificar o papel de calouro ou veterano$"#
)]
fn then_o_estudante_e_informado_de_que_precisa_identificar_o_papel_de_calouro_ou_veterano(
    _world: &mut AppWorld,
) {
    pending_step(
        "o estudante é informado de que precisa identificar o papel de calouro ou veterano",
    );
}

/// Pendente: o estudante permanece inscrito somente com o papel {string}.
#[then(regex = r#"^o estudante permanece inscrito somente com o papel "([^"]*)"$"#)]
fn then_o_estudante_permanece_inscrito_somente_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante permanece inscrito somente com o papel {string}");
}

/// Pendente: o papel desse participante é identificado como {string}.
#[then(regex = r#"^o papel desse participante é identificado como "([^"]*)"$"#)]
fn then_o_papel_desse_participante_e_identificado_como_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o papel desse participante é identificado como {string}");
}

/// Pendente: o veterano é informado de que o programa não está aberto para inscrições.
#[then(regex = r#"^o veterano é informado de que o programa não está aberto para inscrições$"#)]
fn then_o_veterano_e_informado_de_que_o_programa_nao_esta_aberto_para_inscricoes(
    _world: &mut AppWorld,
) {
    pending_step("o veterano é informado de que o programa não está aberto para inscrições");
}

/// Pendente: o calouro solicita a inscrição no programa com o papel {string}.
#[when(regex = r#"^o calouro solicita a inscrição no programa com o papel "([^"]*)"$"#)]
fn when_o_calouro_solicita_a_inscricao_no_programa_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o calouro solicita a inscrição no programa com o papel {string}");
}

/// Pendente: o estudante solicita a inscrição no programa {string} com o papel {string}.
#[when(regex = r#"^o estudante solicita a inscrição no programa "([^"]*)" com o papel "([^"]*)"$"#)]
fn when_o_estudante_solicita_a_inscricao_no_programa_valor_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante solicita a inscrição no programa {string} com o papel {string}");
}

/// Pendente: o estudante solicita a inscrição no programa com um papel diferente de {string} e {string}.
#[when(
    regex = r#"^o estudante solicita a inscrição no programa com um papel diferente de "([^"]*)" e "([^"]*)"$"#
)]
fn when_o_estudante_solicita_a_inscricao_no_programa_com_um_papel_diferente_de_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o estudante solicita a inscrição no programa com um papel diferente de {string} e {string}",
    );
}

/// Pendente: o estudante solicita a inscrição no programa sem informar um papel.
#[when(regex = r#"^o estudante solicita a inscrição no programa sem informar um papel$"#)]
fn when_o_estudante_solicita_a_inscricao_no_programa_sem_informar_um_papel(_world: &mut AppWorld) {
    pending_step("o estudante solicita a inscrição no programa sem informar um papel");
}

/// Pendente: o estudante solicita novamente a inscrição no programa com o papel {string}.
#[when(regex = r#"^o estudante solicita novamente a inscrição no programa com o papel "([^"]*)"$"#)]
fn when_o_estudante_solicita_novamente_a_inscricao_no_programa_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante solicita novamente a inscrição no programa com o papel {string}");
}

/// Pendente: o estudante tenta solicitar a inscrição no programa com o papel {string}.
#[when(regex = r#"^o estudante tenta solicitar a inscrição no programa com o papel "([^"]*)"$"#)]
fn when_o_estudante_tenta_solicitar_a_inscricao_no_programa_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante tenta solicitar a inscrição no programa com o papel {string}");
}

/// Pendente: o veterano solicita a inscrição no programa com o papel {string}.
#[when(regex = r#"^o veterano solicita a inscrição no programa com o papel "([^"]*)"$"#)]
fn when_o_veterano_solicita_a_inscricao_no_programa_com_o_papel_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o veterano solicita a inscrição no programa com o papel {string}");
}
