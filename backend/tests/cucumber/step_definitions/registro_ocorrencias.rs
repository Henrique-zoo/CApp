//! O registro de ocorrências precisa distinguir seus alvos e respeitar as permissões específicas de cada CA.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: iniciei o registro de uma ocorrência para {string}.
#[given(regex = r#"^iniciei o registro de uma ocorrência para "([^"]*)"$"#)]
fn given_iniciei_o_registro_de_uma_ocorrencia_para_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("iniciei o registro de uma ocorrência para {string}");
}

/// Pendente: meu perfil não tem essa permissão no {string}.
#[given(regex = r#"^meu perfil não tem essa permissão no "([^"]*)"$"#)]
fn given_meu_perfil_nao_tem_essa_permissao_no_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("meu perfil não tem essa permissão no {string}");
}

/// Pendente: meu perfil não tem permissão para registrar ocorrências do próprio CA nesse CA.
#[given(
    regex = r#"^meu perfil não tem permissão para registrar ocorrências do próprio CA nesse CA$"#
)]
fn given_meu_perfil_nao_tem_permissao_para_registrar_ocorrencias_do_proprio_ca_nesse_ca(
    _world: &mut AppWorld,
) {
    pending_step("meu perfil não tem permissão para registrar ocorrências do próprio CA nesse CA");
}

/// Pendente: meu perfil não tem permissão para registrar ocorrências individuais nesse CA.
#[given(
    regex = r#"^meu perfil não tem permissão para registrar ocorrências individuais nesse CA$"#
)]
fn given_meu_perfil_nao_tem_permissao_para_registrar_ocorrencias_individuais_nesse_ca(
    _world: &mut AppWorld,
) {
    pending_step("meu perfil não tem permissão para registrar ocorrências individuais nesse CA");
}

/// Pendente: meu perfil tem permissão para registrar ocorrências do próprio CA nesse CA.
#[given(regex = r#"^meu perfil tem permissão para registrar ocorrências do próprio CA nesse CA$"#)]
fn given_meu_perfil_tem_permissao_para_registrar_ocorrencias_do_proprio_ca_nesse_ca(
    _world: &mut AppWorld,
) {
    pending_step("meu perfil tem permissão para registrar ocorrências do próprio CA nesse CA");
}

/// Pendente: meu perfil tem permissão para registrar ocorrências individuais nesse CA.
#[given(regex = r#"^meu perfil tem permissão para registrar ocorrências individuais nesse CA$"#)]
fn given_meu_perfil_tem_permissao_para_registrar_ocorrencias_individuais_nesse_ca(
    _world: &mut AppWorld,
) {
    pending_step("meu perfil tem permissão para registrar ocorrências individuais nesse CA");
}

/// Pendente: meu perfil tem permissão para registrar ocorrências individuais no {string}.
#[given(
    regex = r#"^meu perfil tem permissão para registrar ocorrências individuais no "([^"]*)"$"#
)]
fn given_meu_perfil_tem_permissao_para_registrar_ocorrencias_individuais_no_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("meu perfil tem permissão para registrar ocorrências individuais no {string}");
}

/// Pendente: o estudante {string} participa de outros CAs.
#[given(regex = r#"^o estudante "([^"]*)" participa de outros CAs$"#)]
fn given_o_estudante_valor_participa_de_outros_cas(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante {string} participa de outros CAs");
}

/// Pendente: o estudante {string} pode ser associado a uma ocorrência nesse CA.
#[given(regex = r#"^o estudante "([^"]*)" pode ser associado a uma ocorrência nesse CA$"#)]
fn given_o_estudante_valor_pode_ser_associado_a_uma_ocorrencia_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante {string} pode ser associado a uma ocorrência nesse CA");
}

/// Pendente: preenchi a descrição da ocorrência.
#[given(regex = r#"^preenchi a descrição da ocorrência$"#)]
fn given_preenchi_a_descricao_da_ocorrencia(_world: &mut AppWorld) {
    pending_step("preenchi a descrição da ocorrência");
}

/// Pendente: preenchi o motivo da ocorrência.
#[given(regex = r#"^preenchi o motivo da ocorrência$"#)]
fn given_preenchi_o_motivo_da_ocorrencia(_world: &mut AppWorld) {
    pending_step("preenchi o motivo da ocorrência");
}

/// Pendente: que existe uma ocorrência registrada no {string}.
#[given(regex = r#"^que existe uma ocorrência registrada no "([^"]*)"$"#)]
fn given_que_existe_uma_ocorrencia_registrada_no_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("que existe uma ocorrência registrada no {string}");
}

/// Pendente: a ocorrência fica associada ao estudante {string}.
#[then(regex = r#"^a ocorrência fica associada ao estudante "([^"]*)"$"#)]
fn then_a_ocorrencia_fica_associada_ao_estudante_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a ocorrência fica associada ao estudante {string}");
}

/// Pendente: a ocorrência fica vinculada ao {string}.
#[then(regex = r#"^a ocorrência fica vinculada ao "([^"]*)"$"#)]
fn then_a_ocorrencia_fica_vinculada_ao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a ocorrência fica vinculada ao {string}");
}

/// Pendente: a ocorrência não exige associação a um estudante.
#[then(regex = r#"^a ocorrência não exige associação a um estudante$"#)]
fn then_a_ocorrencia_nao_exige_associacao_a_um_estudante(_world: &mut AppWorld) {
    pending_step("a ocorrência não exige associação a um estudante");
}

/// Pendente: a ocorrência não é criada.
#[then(regex = r#"^a ocorrência não é criada$"#)]
fn then_a_ocorrencia_nao_e_criada(_world: &mut AppWorld) {
    pending_step("a ocorrência não é criada");
}

/// Pendente: a ocorrência é registrada no {string}.
#[then(regex = r#"^a ocorrência é registrada no "([^"]*)"$"#)]
fn then_a_ocorrencia_e_registrada_no_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a ocorrência é registrada no {string}");
}

/// Pendente: o aplicativo informa que a descrição é obrigatória.
#[then(regex = r#"^o aplicativo informa que a descrição é obrigatória$"#)]
fn then_o_aplicativo_informa_que_a_descricao_e_obrigatoria(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que a descrição é obrigatória");
}

/// Pendente: o aplicativo informa que não tenho permissão para esse tipo de registro.
#[then(regex = r#"^o aplicativo informa que não tenho permissão para esse tipo de registro$"#)]
fn then_o_aplicativo_informa_que_nao_tenho_permissao_para_esse_tipo_de_registro(
    _world: &mut AppWorld,
) {
    pending_step("o aplicativo informa que não tenho permissão para esse tipo de registro");
}

/// Pendente: o aplicativo informa que não tenho permissão para registrar ocorrências nesse CA.
#[then(
    regex = r#"^o aplicativo informa que não tenho permissão para registrar ocorrências nesse CA$"#
)]
fn then_o_aplicativo_informa_que_nao_tenho_permissao_para_registrar_ocorrencias_nesse_ca(
    _world: &mut AppWorld,
) {
    pending_step(
        "o aplicativo informa que não tenho permissão para registrar ocorrências nesse CA",
    );
}

/// Pendente: o aplicativo informa que o motivo é obrigatório.
#[then(regex = r#"^o aplicativo informa que o motivo é obrigatório$"#)]
fn then_o_aplicativo_informa_que_o_motivo_e_obrigatorio(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que o motivo é obrigatório");
}

/// Pendente: o encerramento não remove o registro histórico.
#[then(regex = r#"^o encerramento não remove o registro histórico$"#)]
fn then_o_encerramento_nao_remove_o_registro_historico(_world: &mut AppWorld) {
    pending_step("o encerramento não remove o registro histórico");
}

/// Pendente: o próprio CA é identificado como alvo da ocorrência.
#[then(regex = r#"^o próprio CA é identificado como alvo da ocorrência$"#)]
fn then_o_proprio_ca_e_identificado_como_alvo_da_ocorrencia(_world: &mut AppWorld) {
    pending_step("o próprio CA é identificado como alvo da ocorrência");
}

/// Pendente: o registro contém motivo, descrição, data e situação da ocorrência.
#[then(regex = r#"^o registro contém motivo, descrição, data e situação da ocorrência$"#)]
fn then_o_registro_contem_motivo_descricao_data_e_situacao_da_ocorrencia(_world: &mut AppWorld) {
    pending_step("o registro contém motivo, descrição, data e situação da ocorrência");
}

/// Pendente: o registro da ocorrência é preservado.
#[then(regex = r#"^o registro da ocorrência é preservado$"#)]
fn then_o_registro_da_ocorrencia_e_preservado(_world: &mut AppWorld) {
    pending_step("o registro da ocorrência é preservado");
}

/// Pendente: o registro não impõe banimento global ao estudante.
#[then(regex = r#"^o registro não impõe banimento global ao estudante$"#)]
fn then_o_registro_nao_impoe_banimento_global_ao_estudante(_world: &mut AppWorld) {
    pending_step("o registro não impõe banimento global ao estudante");
}

/// Pendente: {string} é identificado como responsável pelo registro.
#[then(regex = r#"^"([^"]*)" é identificado como responsável pelo registro$"#)]
fn then_valor_e_identificado_como_responsavel_pelo_registro(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} é identificado como responsável pelo registro");
}

/// Pendente: a ocorrência é encerrada.
#[when(regex = r#"^a ocorrência é encerrada$"#)]
fn when_a_ocorrencia_e_encerrada(_world: &mut AppWorld) {
    pending_step("a ocorrência é encerrada");
}

/// Pendente: registro uma ocorrência contra o próprio CA com motivo {string} e descrição {string}.
#[when(
    regex = r#"^registro uma ocorrência contra o próprio CA com motivo "([^"]*)" e descrição "([^"]*)"$"#
)]
fn when_registro_uma_ocorrencia_contra_o_proprio_ca_com_motivo_valor_e_descricao_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "registro uma ocorrência contra o próprio CA com motivo {string} e descrição {string}",
    );
}

/// Pendente: registro uma ocorrência individual para {string} com motivo {string} e descrição {string}.
#[when(
    regex = r#"^registro uma ocorrência individual para "([^"]*)" com motivo "([^"]*)" e descrição "([^"]*)"$"#
)]
fn when_registro_uma_ocorrencia_individual_para_valor_com_motivo_valor_e_descricao_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "registro uma ocorrência individual para {string} com motivo {string} e descrição {string}",
    );
}

/// Pendente: registro uma ocorrência individual para {string} nesse CA.
#[when(regex = r#"^registro uma ocorrência individual para "([^"]*)" nesse CA$"#)]
fn when_registro_uma_ocorrencia_individual_para_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("registro uma ocorrência individual para {string} nesse CA");
}

/// Pendente: tento registrar uma ocorrência contra o próprio CA.
#[when(regex = r#"^tento registrar uma ocorrência contra o próprio CA$"#)]
fn when_tento_registrar_uma_ocorrencia_contra_o_proprio_ca(_world: &mut AppWorld) {
    pending_step("tento registrar uma ocorrência contra o próprio CA");
}

/// Pendente: tento registrar uma ocorrência individual para {string}.
#[when(regex = r#"^tento registrar uma ocorrência individual para "([^"]*)"$"#)]
fn when_tento_registrar_uma_ocorrencia_individual_para_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("tento registrar uma ocorrência individual para {string}");
}

/// Pendente: tento salvar a ocorrência sem informar a descrição.
#[when(regex = r#"^tento salvar a ocorrência sem informar a descrição$"#)]
fn when_tento_salvar_a_ocorrencia_sem_informar_a_descricao(_world: &mut AppWorld) {
    pending_step("tento salvar a ocorrência sem informar a descrição");
}

/// Pendente: tento salvar a ocorrência sem informar o motivo.
#[when(regex = r#"^tento salvar a ocorrência sem informar o motivo$"#)]
fn when_tento_salvar_a_ocorrencia_sem_informar_o_motivo(_world: &mut AppWorld) {
    pending_step("tento salvar a ocorrência sem informar o motivo");
}
