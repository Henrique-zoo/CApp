//! A organização por disciplina permite localizar materiais compatíveis com o contexto acadêmico.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: a disciplina {string} pertence à grade curricular do contexto acadêmico.
#[given(regex = r#"^a disciplina "([^"]*)" pertence à grade curricular do contexto acadêmico$"#)]
fn given_a_disciplina_valor_pertence_a_grade_curricular_do_contexto_academico(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a disciplina {string} pertence à grade curricular do contexto acadêmico");
}

/// Pendente: a grade curricular desse contexto possui a disciplina {string}.
#[given(regex = r#"^a grade curricular desse contexto possui a disciplina "([^"]*)"$"#)]
fn given_a_grade_curricular_desse_contexto_possui_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a grade curricular desse contexto possui a disciplina {string}");
}

/// Pendente: a grade curricular do contexto acadêmico possui as disciplinas {string}, {string} e {string}.
#[given(
    regex = r#"^a grade curricular do contexto acadêmico possui as disciplinas "([^"]*)", "([^"]*)" e "([^"]*)"$"#
)]
fn given_a_grade_curricular_do_contexto_academico_possui_as_disciplinas_valor_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "a grade curricular do contexto acadêmico possui as disciplinas {string}, {string} e {string}",
    );
}

/// Pendente: ambos estão associados à disciplina {string}.
#[given(regex = r#"^ambos estão associados à disciplina "([^"]*)"$"#)]
fn given_ambos_estao_associados_a_disciplina_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("ambos estão associados à disciplina {string}");
}

/// Pendente: existe a disciplina {string} em outro contexto acadêmico.
#[given(regex = r#"^existe a disciplina "([^"]*)" em outro contexto acadêmico$"#)]
fn given_existe_a_disciplina_valor_em_outro_contexto_academico(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("existe a disciplina {string} em outro contexto acadêmico");
}

/// Pendente: existe o material publicado {string} associado à disciplina {string}.
#[given(regex = r#"^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)"$"#)]
fn given_existe_o_material_publicado_valor_associado_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe o material publicado {string} associado à disciplina {string}");
}

/// Pendente: existe o material publicado {string} associado à disciplina {string} nesse CA.
#[given(
    regex = r#"^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)" nesse CA$"#
)]
fn given_existe_o_material_publicado_valor_associado_a_disciplina_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe o material publicado {string} associado à disciplina {string} nesse CA");
}

/// Pendente: existe o material publicado {string} associado à disciplina {string} no {string}.
#[given(
    regex = r#"^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)" no "([^"]*)"$"#
)]
fn given_existe_o_material_publicado_valor_associado_a_disciplina_valor_no_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "existe o material publicado {string} associado à disciplina {string} no {string}",
    );
}

/// Pendente: existe o material publicado {string} no formato PDF.
#[given(regex = r#"^existe o material publicado "([^"]*)" no formato PDF$"#)]
fn given_existe_o_material_publicado_valor_no_formato_pdf(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe o material publicado {string} no formato PDF");
}

/// Pendente: existe o material publicado {string} no formato de link.
#[given(regex = r#"^existe o material publicado "([^"]*)" no formato de link$"#)]
fn given_existe_o_material_publicado_valor_no_formato_de_link(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("existe o material publicado {string} no formato de link");
}

/// Pendente: existe o material {string} ainda não publicado.
#[given(regex = r#"^existe o material "([^"]*)" ainda não publicado$"#)]
fn given_existe_o_material_valor_ainda_nao_publicado(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe o material {string} ainda não publicado");
}

/// Pendente: existe um material publicado nesse CA.
#[given(regex = r#"^existe um material publicado nesse CA$"#)]
fn given_existe_um_material_publicado_nesse_ca(_world: &mut AppWorld) {
    pending_step("existe um material publicado nesse CA");
}

/// Pendente: existem materiais publicados associados às disciplinas {string} e {string}.
#[given(
    regex = r#"^existem materiais publicados associados às disciplinas "([^"]*)" e "([^"]*)"$"#
)]
fn given_existem_materiais_publicados_associados_as_disciplinas_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existem materiais publicados associados às disciplinas {string} e {string}");
}

/// Pendente: existem os materiais publicados {string} e {string} associados à disciplina {string}.
#[given(
    regex = r#"^existem os materiais publicados "([^"]*)" e "([^"]*)" associados à disciplina "([^"]*)"$"#
)]
fn given_existem_os_materiais_publicados_valor_e_valor_associados_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "existem os materiais publicados {string} e {string} associados à disciplina {string}",
    );
}

/// Pendente: não existem disciplinas cadastradas na grade curricular do contexto acadêmico.
#[given(
    regex = r#"^não existem disciplinas cadastradas na grade curricular do contexto acadêmico$"#
)]
fn given_nao_existem_disciplinas_cadastradas_na_grade_curricular_do_contexto_academico(
    _world: &mut AppWorld,
) {
    pending_step("não existem disciplinas cadastradas na grade curricular do contexto acadêmico");
}

/// Pendente: não existem materiais publicados associados à disciplina {string}.
#[given(regex = r#"^não existem materiais publicados associados à disciplina "([^"]*)"$"#)]
fn given_nao_existem_materiais_publicados_associados_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("não existem materiais publicados associados à disciplina {string}");
}

/// Pendente: o estudante está consultando os materiais de {string}.
#[given(regex = r#"^o estudante está consultando os materiais de "([^"]*)"$"#)]
fn given_o_estudante_esta_consultando_os_materiais_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante está consultando os materiais de {string}");
}

/// Pendente: o integrante não possui a permissão {string} nesse CA.
#[given(regex = r#"^o integrante não possui a permissão "([^"]*)" nesse CA$"#)]
fn given_o_integrante_nao_possui_a_permissao_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o integrante não possui a permissão {string} nesse CA");
}

/// Pendente: o material publicado {string} está associado à disciplina {string}.
#[given(regex = r#"^o material publicado "([^"]*)" está associado à disciplina "([^"]*)"$"#)]
fn given_o_material_publicado_valor_esta_associado_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o material publicado {string} está associado à disciplina {string}");
}

/// Pendente: que o estudante está no {string}.
#[given(regex = r#"^que o estudante está no "([^"]*)"$"#)]
fn given_que_o_estudante_esta_no_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o estudante está no {string}");
}

/// Pendente: que o integrante possui a permissão {string} no CA ativo.
#[given(regex = r#"^que o integrante possui a permissão "([^"]*)" no CA ativo$"#)]
fn given_que_o_integrante_possui_a_permissao_valor_no_ca_ativo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o integrante possui a permissão {string} no CA ativo");
}

/// Pendente: a associação do material com a disciplina é registrada.
#[then(regex = r#"^a associação do material com a disciplina é registrada$"#)]
fn then_a_associacao_do_material_com_a_disciplina_e_registrada(_world: &mut AppWorld) {
    pending_step("a associação do material com a disciplina é registrada");
}

/// Pendente: a associação é recusada.
#[then(regex = r#"^a associação é recusada$"#)]
fn then_a_associacao_e_recusada(_world: &mut AppWorld) {
    pending_step("a associação é recusada");
}

/// Pendente: a associação é recusada por falta de permissão.
#[then(regex = r#"^a associação é recusada por falta de permissão$"#)]
fn then_a_associacao_e_recusada_por_falta_de_permissao(_world: &mut AppWorld) {
    pending_step("a associação é recusada por falta de permissão");
}

/// Pendente: a disciplina {string} não é apresentada.
#[then(regex = r#"^a disciplina "([^"]*)" não é apresentada$"#)]
fn then_a_disciplina_valor_nao_e_apresentada(_world: &mut AppWorld, _value_1: String) {
    pending_step("a disciplina {string} não é apresentada");
}

/// Pendente: a disciplina {string} é apresentada.
#[then(regex = r#"^a disciplina "([^"]*)" é apresentada$"#)]
fn then_a_disciplina_valor_e_apresentada(_world: &mut AppWorld, _value_1: String) {
    pending_step("a disciplina {string} é apresentada");
}

/// Pendente: a organização anterior do material permanece inalterada.
#[then(regex = r#"^a organização anterior do material permanece inalterada$"#)]
fn then_a_organizacao_anterior_do_material_permanece_inalterada(_world: &mut AppWorld) {
    pending_step("a organização anterior do material permanece inalterada");
}

/// Pendente: a organização corresponde ao contexto do {string}.
#[then(regex = r#"^a organização corresponde ao contexto do "([^"]*)"$"#)]
fn then_a_organizacao_corresponde_ao_contexto_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a organização corresponde ao contexto do {string}");
}

/// Pendente: ambos ficam disponíveis na organização da disciplina.
#[then(regex = r#"^ambos ficam disponíveis na organização da disciplina$"#)]
fn then_ambos_ficam_disponiveis_na_organizacao_da_disciplina(_world: &mut AppWorld) {
    pending_step("ambos ficam disponíveis na organização da disciplina");
}

/// Pendente: as disciplinas da grade curricular são apresentadas para consulta.
#[then(regex = r#"^as disciplinas da grade curricular são apresentadas para consulta$"#)]
fn then_as_disciplinas_da_grade_curricular_sao_apresentadas_para_consulta(_world: &mut AppWorld) {
    pending_step("as disciplinas da grade curricular são apresentadas para consulta");
}

/// Pendente: nenhum material é apresentado.
#[then(regex = r#"^nenhum material é apresentado$"#)]
fn then_nenhum_material_e_apresentado(_world: &mut AppWorld) {
    pending_step("nenhum material é apresentado");
}

/// Pendente: nenhuma disciplina é apresentada para consulta.
#[then(regex = r#"^nenhuma disciplina é apresentada para consulta$"#)]
fn then_nenhuma_disciplina_e_apresentada_para_consulta(_world: &mut AppWorld) {
    pending_step("nenhuma disciplina é apresentada para consulta");
}

/// Pendente: o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais.
#[then(
    regex = r#"^o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais$"#
)]
fn then_o_aplicativo_informa_que_nao_existem_disciplinas_disponiveis_para_a_organizacao_dos_materiais(
    _world: &mut AppWorld,
) {
    pending_step(
        "o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais",
    );
}

/// Pendente: o aplicativo informa que não existem materiais disponíveis para essa disciplina.
#[then(
    regex = r#"^o aplicativo informa que não existem materiais disponíveis para essa disciplina$"#
)]
fn then_o_aplicativo_informa_que_nao_existem_materiais_disponiveis_para_essa_disciplina(
    _world: &mut AppWorld,
) {
    pending_step("o aplicativo informa que não existem materiais disponíveis para essa disciplina");
}

/// Pendente: o estudante pode selecionar uma disciplina para visualizar seus materiais.
#[then(regex = r#"^o estudante pode selecionar uma disciplina para visualizar seus materiais$"#)]
fn then_o_estudante_pode_selecionar_uma_disciplina_para_visualizar_seus_materiais(
    _world: &mut AppWorld,
) {
    pending_step("o estudante pode selecionar uma disciplina para visualizar seus materiais");
}

/// Pendente: o material fica disponível para consulta nessa disciplina.
#[then(regex = r#"^o material fica disponível para consulta nessa disciplina$"#)]
fn then_o_material_fica_disponivel_para_consulta_nessa_disciplina(_world: &mut AppWorld) {
    pending_step("o material fica disponível para consulta nessa disciplina");
}

/// Pendente: o material não fica associado à disciplina incompatível.
#[then(regex = r#"^o material não fica associado à disciplina incompatível$"#)]
fn then_o_material_nao_fica_associado_a_disciplina_incompativel(_world: &mut AppWorld) {
    pending_step("o material não fica associado à disciplina incompatível");
}

/// Pendente: o material passa a pertencer à organização da disciplina {string}.
#[then(regex = r#"^o material passa a pertencer à organização da disciplina "([^"]*)"$"#)]
fn then_o_material_passa_a_pertencer_a_organizacao_da_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o material passa a pertencer à organização da disciplina {string}");
}

/// Pendente: o material selecionado fica disponível para acesso.
#[then(regex = r#"^o material selecionado fica disponível para acesso$"#)]
fn then_o_material_selecionado_fica_disponivel_para_acesso(_world: &mut AppWorld) {
    pending_step("o material selecionado fica disponível para acesso");
}

/// Pendente: o material {string} continua associado à disciplina.
#[then(regex = r#"^o material "([^"]*)" continua associado à disciplina$"#)]
fn then_o_material_valor_continua_associado_a_disciplina(_world: &mut AppWorld, _value_1: String) {
    pending_step("o material {string} continua associado à disciplina");
}

/// Pendente: o material {string} não é apresentado.
#[then(regex = r#"^o material "([^"]*)" não é apresentado$"#)]
fn then_o_material_valor_nao_e_apresentado(_world: &mut AppWorld, _value_1: String) {
    pending_step("o material {string} não é apresentado");
}

/// Pendente: o material {string} não é apresentado na consulta atual.
#[then(regex = r#"^o material "([^"]*)" não é apresentado na consulta atual$"#)]
fn then_o_material_valor_nao_e_apresentado_na_consulta_atual(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o material {string} não é apresentado na consulta atual");
}

/// Pendente: o material {string} não é apresentado nessa consulta.
#[then(regex = r#"^o material "([^"]*)" não é apresentado nessa consulta$"#)]
fn then_o_material_valor_nao_e_apresentado_nessa_consulta(_world: &mut AppWorld, _value_1: String) {
    pending_step("o material {string} não é apresentado nessa consulta");
}

/// Pendente: o material {string} é apresentado.
#[then(regex = r#"^o material "([^"]*)" é apresentado$"#)]
fn then_o_material_valor_e_apresentado(_world: &mut AppWorld, _value_1: String) {
    pending_step("o material {string} é apresentado");
}

/// Pendente: os materiais de {string} não são apresentados nessa consulta.
#[then(regex = r#"^os materiais de "([^"]*)" não são apresentados nessa consulta$"#)]
fn then_os_materiais_de_valor_nao_sao_apresentados_nessa_consulta(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("os materiais de {string} não são apresentados nessa consulta");
}

/// Pendente: permanece disponível para consulta.
#[then(regex = r#"^permanece disponível para consulta$"#)]
fn then_permanece_disponivel_para_consulta(_world: &mut AppWorld) {
    pending_step("permanece disponível para consulta");
}

/// Pendente: sua associação com a disciplina {string} é mantida.
#[then(regex = r#"^sua associação com a disciplina "([^"]*)" é mantida$"#)]
fn then_sua_associacao_com_a_disciplina_valor_e_mantida(_world: &mut AppWorld, _value_1: String) {
    pending_step("sua associação com a disciplina {string} é mantida");
}

/// Pendente: são apresentados os materiais associados à disciplina {string}.
#[then(regex = r#"^são apresentados os materiais associados à disciplina "([^"]*)"$"#)]
fn then_sao_apresentados_os_materiais_associados_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("são apresentados os materiais associados à disciplina {string}");
}

/// Pendente: acessa novamente a organização de materiais por disciplina.
#[when(regex = r#"^acessa novamente a organização de materiais por disciplina$"#)]
fn when_acessa_novamente_a_organizacao_de_materiais_por_disciplina(_world: &mut AppWorld) {
    pending_step("acessa novamente a organização de materiais por disciplina");
}

/// Pendente: consulta os materiais da disciplina {string}.
#[when(regex = r#"^consulta os materiais da disciplina "([^"]*)"$"#)]
fn when_consulta_os_materiais_da_disciplina_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("consulta os materiais da disciplina {string}");
}

/// Pendente: esse integrante associa {string} à disciplina {string}.
#[when(regex = r#"^esse integrante associa "([^"]*)" à disciplina "([^"]*)"$"#)]
fn when_esse_integrante_associa_valor_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("esse integrante associa {string} à disciplina {string}");
}

/// Pendente: esse integrante tenta associar {string} à disciplina {string}.
#[when(regex = r#"^esse integrante tenta associar "([^"]*)" à disciplina "([^"]*)"$"#)]
fn when_esse_integrante_tenta_associar_valor_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("esse integrante tenta associar {string} à disciplina {string}");
}

/// Pendente: o estudante acessa a organização de materiais por disciplina.
#[when(regex = r#"^o estudante acessa a organização de materiais por disciplina$"#)]
fn when_o_estudante_acessa_a_organizacao_de_materiais_por_disciplina(_world: &mut AppWorld) {
    pending_step("o estudante acessa a organização de materiais por disciplina");
}

/// Pendente: o estudante altera o CA ativo para {string}.
#[when(regex = r#"^o estudante altera o CA ativo para "([^"]*)"$"#)]
fn when_o_estudante_altera_o_ca_ativo_para_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante altera o CA ativo para {string}");
}

/// Pendente: o estudante consulta os materiais da disciplina {string}.
#[when(regex = r#"^o estudante consulta os materiais da disciplina "([^"]*)"$"#)]
fn when_o_estudante_consulta_os_materiais_da_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante consulta os materiais da disciplina {string}");
}

/// Pendente: o estudante retorna à lista de disciplinas.
#[when(regex = r#"^o estudante retorna à lista de disciplinas$"#)]
fn when_o_estudante_retorna_a_lista_de_disciplinas(_world: &mut AppWorld) {
    pending_step("o estudante retorna à lista de disciplinas");
}

/// Pendente: o estudante sai da organização de materiais.
#[when(regex = r#"^o estudante sai da organização de materiais$"#)]
fn when_o_estudante_sai_da_organizacao_de_materiais(_world: &mut AppWorld) {
    pending_step("o estudante sai da organização de materiais");
}

/// Pendente: o estudante seleciona a disciplina {string}.
#[when(regex = r#"^o estudante seleciona a disciplina "([^"]*)"$"#)]
fn when_o_estudante_seleciona_a_disciplina_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante seleciona a disciplina {string}");
}

/// Pendente: seleciona a disciplina {string}.
#[when(regex = r#"^seleciona a disciplina "([^"]*)"$"#)]
fn when_seleciona_a_disciplina_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciona a disciplina {string}");
}

/// Pendente: seleciona novamente a disciplina {string}.
#[when(regex = r#"^seleciona novamente a disciplina "([^"]*)"$"#)]
fn when_seleciona_novamente_a_disciplina_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciona novamente a disciplina {string}");
}

/// Pendente: seleciona o material {string}.
#[when(regex = r#"^seleciona o material "([^"]*)"$"#)]
fn when_seleciona_o_material_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciona o material {string}");
}

/// Pendente: seleciona {string}.
#[when(regex = r#"^seleciona "([^"]*)"$"#)]
fn when_seleciona_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciona {string}");
}

/// Pendente: tenta associar o material a uma disciplina.
#[when(regex = r#"^tenta associar o material a uma disciplina$"#)]
fn when_tenta_associar_o_material_a_uma_disciplina(_world: &mut AppWorld) {
    pending_step("tenta associar o material a uma disciplina");
}
