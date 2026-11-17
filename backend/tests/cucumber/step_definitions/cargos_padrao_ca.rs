//! O cadastro de um CA precisa entregar sua estrutura inicial de cargos e preservar a presidência.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: esse cargo possui a permissão {string}.
#[given(regex = r#"^esse cargo possui a permissão "([^"]*)"$"#)]
fn given_esse_cargo_possui_a_permissao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("esse cargo possui a permissão {string}");
}

/// Pendente: o administrador {string} administra a plataforma.
#[given(regex = r#"^o administrador "([^"]*)" administra a plataforma$"#)]
fn given_o_administrador_valor_administra_a_plataforma(_world: &mut AppWorld, _value_1: String) {
    pending_step("o administrador {string} administra a plataforma");
}

/// Pendente: o estudante {string} exerce o cargo {string} em {string}.
#[given(regex = r#"^o estudante "([^"]*)" exerce o cargo "([^"]*)" em "([^"]*)"$"#)]
fn given_o_estudante_valor_exerce_o_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o estudante {string} exerce o cargo {string} em {string}");
}

/// Pendente: o gestor {string} remove {string} da gestão de {string}.
#[given(regex = r#"^o gestor "([^"]*)" remove "([^"]*)" da gestão de "([^"]*)"$"#)]
fn given_o_gestor_valor_remove_valor_da_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} remove {string} da gestão de {string}");
}

/// Pendente: que o estudante informa a si mesmo como Presidente da gestão inicial de {string}.
#[given(
    regex = r#"^que o estudante informa a si mesmo como Presidente da gestão inicial de "([^"]*)"$"#
)]
fn given_que_o_estudante_informa_a_si_mesmo_como_presidente_da_gestao_inicial_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "que o estudante informa a si mesmo como Presidente da gestão inicial de {string}",
    );
}

/// Pendente: que o histórico de {string} registra a passagem de {string} pelo cargo {string}.
#[given(
    regex = r#"^que o histórico de "([^"]*)" registra a passagem de "([^"]*)" pelo cargo "([^"]*)"$"#
)]
fn given_que_o_historico_de_valor_registra_a_passagem_de_valor_pelo_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que o histórico de {string} registra a passagem de {string} pelo cargo {string}");
}

/// Pendente: que {string} foi cadastrado com {string} como Presidente da gestão inicial.
#[given(
    regex = r#"^que "([^"]*)" foi cadastrado com "([^"]*)" como Presidente da gestão inicial$"#
)]
fn given_que_valor_foi_cadastrado_com_valor_como_presidente_da_gestao_inicial(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que {string} foi cadastrado com {string} como Presidente da gestão inicial");
}

/// Pendente: nenhum deles possui permissões administrativas no outro CA.
#[then(regex = r#"^nenhum deles possui permissões administrativas no outro CA$"#)]
fn then_nenhum_deles_possui_permissoes_administrativas_no_outro_ca(_world: &mut AppWorld) {
    pending_step("nenhum deles possui permissões administrativas no outro CA");
}

/// Pendente: o CA {string} continua possuindo os cargos {string}, {string} e {string}.
#[then(regex = r#"^o CA "([^"]*)" continua possuindo os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$"#)]
fn then_o_ca_valor_continua_possuindo_os_cargos_valor_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o CA {string} continua possuindo os cargos {string}, {string} e {string}");
}

/// Pendente: o CA {string} passa a possuir exatamente os cargos {string}, {string} e {string}.
#[then(
    regex = r#"^o CA "([^"]*)" passa a possuir exatamente os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$"#
)]
fn then_o_ca_valor_passa_a_possuir_exatamente_os_cargos_valor_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step(
        "o CA {string} passa a possuir exatamente os cargos {string}, {string} e {string}",
    );
}

/// Pendente: o CA {string} permanece com um único conjunto de cargos.
#[then(regex = r#"^o CA "([^"]*)" permanece com um único conjunto de cargos$"#)]
fn then_o_ca_valor_permanece_com_um_unico_conjunto_de_cargos(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o CA {string} permanece com um único conjunto de cargos");
}

/// Pendente: o cadastro é recusado informando que o CA já existe.
#[then(regex = r#"^o cadastro é recusado informando que o CA já existe$"#)]
fn then_o_cadastro_e_recusado_informando_que_o_ca_ja_existe(_world: &mut AppWorld) {
    pending_step("o cadastro é recusado informando que o CA já existe");
}

/// Pendente: o cargo {string} apresenta o responsável informado na criação.
#[then(regex = r#"^o cargo "([^"]*)" apresenta o responsável informado na criação$"#)]
fn then_o_cargo_valor_apresenta_o_responsavel_informado_na_criacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o cargo {string} apresenta o responsável informado na criação");
}

/// Pendente: o cargo {string} de {string} não possui integrante vinculado.
#[then(regex = r#"^o cargo "([^"]*)" de "([^"]*)" não possui integrante vinculado$"#)]
fn then_o_cargo_valor_de_valor_nao_possui_integrante_vinculado(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo {string} de {string} não possui integrante vinculado");
}

/// Pendente: o cargo {string} fica vinculado ao responsável informado na gestão inicial.
#[then(regex = r#"^o cargo "([^"]*)" fica vinculado ao responsável informado na gestão inicial$"#)]
fn then_o_cargo_valor_fica_vinculado_ao_responsavel_informado_na_gestao_inicial(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o cargo {string} fica vinculado ao responsável informado na gestão inicial");
}

/// Pendente: os cargos {string} e {string} estão disponíveis para vinculação por usuários autorizados.
#[then(
    regex = r#"^os cargos "([^"]*)" e "([^"]*)" estão disponíveis para vinculação por usuários autorizados$"#
)]
fn then_os_cargos_valor_e_valor_estao_disponiveis_para_vinculacao_por_usuarios_autorizados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "os cargos {string} e {string} estão disponíveis para vinculação por usuários autorizados",
    );
}

/// Pendente: os cargos {string} e {string} ficam disponíveis para novas vinculações.
#[then(regex = r#"^os cargos "([^"]*)" e "([^"]*)" ficam disponíveis para novas vinculações$"#)]
fn then_os_cargos_valor_e_valor_ficam_disponiveis_para_novas_vinculacoes(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("os cargos {string} e {string} ficam disponíveis para novas vinculações");
}

/// Pendente: os registros anteriores do histórico administrativo de {string} permanecem disponíveis.
#[then(
    regex = r#"^os registros anteriores do histórico administrativo de "([^"]*)" permanecem disponíveis$"#
)]
fn then_os_registros_anteriores_do_historico_administrativo_de_valor_permanecem_disponiveis(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "os registros anteriores do histórico administrativo de {string} permanecem disponíveis",
    );
}

/// Pendente: {string} exerce o cargo {string} em {string}.
#[then(regex = r#"^"([^"]*)" exerce o cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_exerce_o_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} exerce o cargo {string} em {string}");
}

/// Pendente: o administrador {string} tenta excluir o cargo {string} em {string}.
#[when(regex = r#"^o administrador "([^"]*)" tenta excluir o cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_o_administrador_valor_tenta_excluir_o_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o administrador {string} tenta excluir o cargo {string} em {string}");
}

/// Pendente: o estudante {string} cadastra o Centro Acadêmico {string}.
#[when(regex = r#"^o estudante "([^"]*)" cadastra o Centro Acadêmico "([^"]*)"$"#)]
fn when_o_estudante_valor_cadastra_o_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante {string} cadastra o Centro Acadêmico {string}");
}

/// Pendente: o estudante {string} tenta cadastrar novamente o Centro Acadêmico {string}.
#[when(regex = r#"^o estudante "([^"]*)" tenta cadastrar novamente o Centro Acadêmico "([^"]*)"$"#)]
fn when_o_estudante_valor_tenta_cadastrar_novamente_o_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante {string} tenta cadastrar novamente o Centro Acadêmico {string}");
}

/// Pendente: o estudante {string} é vinculado ao cargo {string} em {string}.
#[when(regex = r#"^o estudante "([^"]*)" é vinculado ao cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_o_estudante_valor_e_vinculado_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o estudante {string} é vinculado ao cargo {string} em {string}");
}

/// Pendente: o gestor {string} tenta vincular {string} ao cargo {string} de {string}.
#[when(regex = r#"^o gestor "([^"]*)" tenta vincular "([^"]*)" ao cargo "([^"]*)" de "([^"]*)"$"#)]
fn when_o_gestor_valor_tenta_vincular_valor_ao_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} tenta vincular {string} ao cargo {string} de {string}");
}

/// Pendente: o responsável consulta a estrutura inicial da gestão de {string}.
#[when(regex = r#"^o responsável consulta a estrutura inicial da gestão de "([^"]*)"$"#)]
fn when_o_responsavel_consulta_a_estrutura_inicial_da_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o responsável consulta a estrutura inicial da gestão de {string}");
}

/// Pendente: {string} remove o cargo {string} de {string}.
#[when(regex = r#"^"([^"]*)" remove o cargo "([^"]*)" de "([^"]*)"$"#)]
fn when_valor_remove_o_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} remove o cargo {string} de {string}");
}

/// Pendente: {string} tenta excluir o cargo {string} em {string}.
#[when(regex = r#"^"([^"]*)" tenta excluir o cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_valor_tenta_excluir_o_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} tenta excluir o cargo {string} em {string}");
}

/// Pendente: {string} tenta remover o cargo {string} de {string}.
#[when(regex = r#"^"([^"]*)" tenta remover o cargo "([^"]*)" de "([^"]*)"$"#)]
fn when_valor_tenta_remover_o_cargo_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} tenta remover o cargo {string} de {string}");
}
