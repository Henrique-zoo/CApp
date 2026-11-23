//! A composição da gestão determina as permissões cumulativas de seus integrantes e precisa preservar seu histórico.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: ambos os cargos concedem a permissão {string}.
#[given(regex = r#"^ambos os cargos concedem a permissão "([^"]*)"$"#)]
fn given_ambos_os_cargos_concedem_a_permissao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("ambos os cargos concedem a permissão {string}");
}

/// Pendente: esse cargo concede a permissão {string}.
#[given(regex = r#"^esse cargo concede a permissão "([^"]*)"$"#)]
fn given_esse_cargo_concede_a_permissao_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("esse cargo concede a permissão {string}");
}

/// Pendente: esse cargo possui a permissão de {string}.
#[given(regex = r#"^esse cargo possui a permissão de "([^"]*)"$"#)]
fn given_esse_cargo_possui_a_permissao_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("esse cargo possui a permissão de {string}");
}

/// Pendente: exerce dois cargos que concedem a permissão {string}.
#[given(regex = r#"^exerce dois cargos que concedem a permissão "([^"]*)"$"#)]
fn given_exerce_dois_cargos_que_concedem_a_permissao_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("exerce dois cargos que concedem a permissão {string}");
}

/// Pendente: não integra a gestão de {string}.
#[given(regex = r#"^não integra a gestão de "([^"]*)"$"#)]
fn given_nao_integra_a_gestao_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não integra a gestão de {string}");
}

/// Pendente: o CA {string} não possui o cargo {string}.
#[given(regex = r#"^o CA "([^"]*)" não possui o cargo "([^"]*)"$"#)]
fn given_o_ca_valor_nao_possui_o_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o CA {string} não possui o cargo {string}");
}

/// Pendente: o cargo {string} concede a permissão {string}.
#[given(regex = r#"^o cargo "([^"]*)" concede a permissão "([^"]*)"$"#)]
fn given_o_cargo_valor_concede_a_permissao_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo {string} concede a permissão {string}");
}

/// Pendente: posteriormente teve o cargo alterado para {string} em {string}.
#[given(regex = r#"^posteriormente teve o cargo alterado para "([^"]*)" em "([^"]*)"$"#)]
fn given_posteriormente_teve_o_cargo_alterado_para_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("posteriormente teve o cargo alterado para {string} em {string}");
}

/// Pendente: que não existe perfil no CApp para o e-mail {string}.
#[given(regex = r#"^que não existe perfil no CApp para o e-mail "([^"]*)"$"#)]
fn given_que_nao_existe_perfil_no_capp_para_o_e_mail_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que não existe perfil no CApp para o e-mail {string}");
}

/// Pendente: que o cargo {string} possui a permissão de {string}.
#[given(regex = r#"^que o cargo "([^"]*)" possui a permissão de "([^"]*)"$"#)]
fn given_que_o_cargo_valor_possui_a_permissao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o cargo {string} possui a permissão de {string}");
}

/// Pendente: que o estudante {string} foi vinculado ao cargo {string} em {string}.
#[given(regex = r#"^que o estudante "([^"]*)" foi vinculado ao cargo "([^"]*)" em "([^"]*)"$"#)]
fn given_que_o_estudante_valor_foi_vinculado_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que o estudante {string} foi vinculado ao cargo {string} em {string}");
}

/// Pendente: que o histórico de {string} registra a vinculação de {string} ao cargo {string}.
#[given(
    regex = r#"^que o histórico de "([^"]*)" registra a vinculação de "([^"]*)" ao cargo "([^"]*)"$"#
)]
fn given_que_o_historico_de_valor_registra_a_vinculacao_de_valor_ao_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que o histórico de {string} registra a vinculação de {string} ao cargo {string}");
}

/// Pendente: que {string} exerce dois cargos em {string}.
#[given(regex = r#"^que "([^"]*)" exerce dois cargos em "([^"]*)"$"#)]
fn given_que_valor_exerce_dois_cargos_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que {string} exerce dois cargos em {string}");
}

/// Pendente: que {string} exerce o cargo {string} em {string}.
#[given(regex = r#"^que "([^"]*)" exerce o cargo "([^"]*)" em "([^"]*)"$"#)]
fn given_que_valor_exerce_o_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que {string} exerce o cargo {string} em {string}");
}

/// Pendente: que {string} exerce os cargos {string} e {string} em {string}.
#[given(regex = r#"^que "([^"]*)" exerce os cargos "([^"]*)" e "([^"]*)" em "([^"]*)"$"#)]
fn given_que_valor_exerce_os_cargos_valor_e_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("que {string} exerce os cargos {string} e {string} em {string}");
}

/// Pendente: que {string} é o único integrante autorizado a administrar a gestão de {string}.
#[given(
    regex = r#"^que "([^"]*)" é o único integrante autorizado a administrar a gestão de "([^"]*)"$"#
)]
fn given_que_valor_e_o_unico_integrante_autorizado_a_administrar_a_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que {string} é o único integrante autorizado a administrar a gestão de {string}");
}

/// Pendente: que {string} é o único integrante de {string} com a permissão de {string}.
#[given(
    regex = r#"^que "([^"]*)" é o único integrante de "([^"]*)" com a permissão de "([^"]*)"$"#
)]
fn given_que_valor_e_o_unico_integrante_de_valor_com_a_permissao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que {string} é o único integrante de {string} com a permissão de {string}");
}

/// Pendente: somente um de seus cargos concede a permissão {string}.
#[given(regex = r#"^somente um de seus cargos concede a permissão "([^"]*)"$"#)]
fn given_somente_um_de_seus_cargos_concede_a_permissao_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("somente um de seus cargos concede a permissão {string}");
}

/// Pendente: a composição da gestão de {string} permanece inalterada.
#[then(regex = r#"^a composição da gestão de "([^"]*)" permanece inalterada$"#)]
fn then_a_composicao_da_gestao_de_valor_permanece_inalterada(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a composição da gestão de {string} permanece inalterada");
}

/// Pendente: a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão.
#[then(
    regex = r#"^a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão$"#
)]
fn then_a_operacao_e_recusada_informando_que_a_gestao_precisa_manter_ao_menos_um_integrante_com_essa_permiss(
    _world: &mut AppWorld,
) {
    pending_step(
        "a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão",
    );
}

/// Pendente: a operação é recusada informando que o histórico administrativo não pode ser modificado.
#[then(
    regex = r#"^a operação é recusada informando que o histórico administrativo não pode ser modificado$"#
)]
fn then_a_operacao_e_recusada_informando_que_o_historico_administrativo_nao_pode_ser_modificado(
    _world: &mut AppWorld,
) {
    pending_step(
        "a operação é recusada informando que o histórico administrativo não pode ser modificado",
    );
}

/// Pendente: a operação é recusada informando que o integrante já exerce esse cargo.
#[then(regex = r#"^a operação é recusada informando que o integrante já exerce esse cargo$"#)]
fn then_a_operacao_e_recusada_informando_que_o_integrante_ja_exerce_esse_cargo(
    _world: &mut AppWorld,
) {
    pending_step("a operação é recusada informando que o integrante já exerce esse cargo");
}

/// Pendente: a operação é recusada informando que o usuário já exerce esse cargo.
#[then(regex = r#"^a operação é recusada informando que o usuário já exerce esse cargo$"#)]
fn then_a_operacao_e_recusada_informando_que_o_usuario_ja_exerce_esse_cargo(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o usuário já exerce esse cargo");
}

/// Pendente: a operação é recusada informando que o usuário não foi encontrado.
#[then(regex = r#"^a operação é recusada informando que o usuário não foi encontrado$"#)]
fn then_a_operacao_e_recusada_informando_que_o_usuario_nao_foi_encontrado(_world: &mut AppWorld) {
    pending_step("a operação é recusada informando que o usuário não foi encontrado");
}

/// Pendente: a operação é recusada informando que o usuário não integra a gestão do CA.
#[then(regex = r#"^a operação é recusada informando que o usuário não integra a gestão do CA$"#)]
fn then_a_operacao_e_recusada_informando_que_o_usuario_nao_integra_a_gestao_do_ca(
    _world: &mut AppWorld,
) {
    pending_step("a operação é recusada informando que o usuário não integra a gestão do CA");
}

/// Pendente: a remoção do vínculo é permitida.
#[then(regex = r#"^a remoção do vínculo é permitida$"#)]
fn then_a_remocao_do_vinculo_e_permitida(_world: &mut AppWorld) {
    pending_step("a remoção do vínculo é permitida");
}

/// Pendente: nenhum novo registro é adicionado ao histórico administrativo de {string}.
#[then(regex = r#"^nenhum novo registro é adicionado ao histórico administrativo de "([^"]*)"$"#)]
fn then_nenhum_novo_registro_e_adicionado_ao_historico_administrativo_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("nenhum novo registro é adicionado ao histórico administrativo de {string}");
}

/// Pendente: o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}.
#[then(
    regex = r#"^o histórico administrativo de "([^"]*)" apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de "([^"]*)"$"#
)]
fn then_o_historico_administrativo_de_valor_apresenta_em_ordem_cronologica_a_vinculacao_a_alteracao_e_a_remo(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o histórico administrativo de {string} apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}.
#[then(
    regex = r#"^o histórico administrativo de "([^"]*)" registra a alteração de "([^"]*)" do cargo "([^"]*)" para "([^"]*)"$"#
)]
fn then_o_historico_administrativo_de_valor_registra_a_alteracao_de_valor_do_cargo_valor_para_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a alteração de {string} do cargo {string} para {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a nova vinculação.
#[then(regex = r#"^o histórico administrativo de "([^"]*)" registra a nova vinculação$"#)]
fn then_o_historico_administrativo_de_valor_registra_a_nova_vinculacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o histórico administrativo de {string} registra a nova vinculação");
}

/// Pendente: o histórico administrativo de {string} registra a remoção de {string} do cargo {string}.
#[then(
    regex = r#"^o histórico administrativo de "([^"]*)" registra a remoção de "([^"]*)" do cargo "([^"]*)"$"#
)]
fn then_o_historico_administrativo_de_valor_registra_a_remocao_de_valor_do_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a remoção de {string} do cargo {string}",
    );
}

/// Pendente: o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}.
#[then(
    regex = r#"^o histórico administrativo de "([^"]*)" registra a vinculação de "([^"]*)" ao cargo "([^"]*)"$"#
)]
fn then_o_historico_administrativo_de_valor_registra_a_vinculacao_de_valor_ao_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "o histórico administrativo de {string} registra a vinculação de {string} ao cargo {string}",
    );
}

/// Pendente: o histórico preserva a participação nos dois cargos.
#[then(regex = r#"^o histórico preserva a participação nos dois cargos$"#)]
fn then_o_historico_preserva_a_participacao_nos_dois_cargos(_world: &mut AppWorld) {
    pending_step("o histórico preserva a participação nos dois cargos");
}

/// Pendente: o integrante continua podendo publicar notícias diretamente em {string}.
#[then(regex = r#"^o integrante continua podendo publicar notícias diretamente em "([^"]*)"$"#)]
fn then_o_integrante_continua_podendo_publicar_noticias_diretamente_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o integrante continua podendo publicar notícias diretamente em {string}");
}

/// Pendente: o integrante permanece no cargo {string} em {string}.
#[then(regex = r#"^o integrante permanece no cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_o_integrante_permanece_no_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o integrante permanece no cargo {string} em {string}");
}

/// Pendente: o integrante permanece nos dois cargos em {string}.
#[then(regex = r#"^o integrante permanece nos dois cargos em "([^"]*)"$"#)]
fn then_o_integrante_permanece_nos_dois_cargos_em_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o integrante permanece nos dois cargos em {string}");
}

/// Pendente: o registro identifica o gestor {string} como responsável e a data da alteração.
#[then(
    regex = r#"^o registro identifica o gestor "([^"]*)" como responsável e a data da alteração$"#
)]
fn then_o_registro_identifica_o_gestor_valor_como_responsavel_e_a_data_da_alteracao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o registro identifica o gestor {string} como responsável e a data da alteração");
}

/// Pendente: o registro identifica o gestor {string} como responsável e a data da remoção.
#[then(
    regex = r#"^o registro identifica o gestor "([^"]*)" como responsável e a data da remoção$"#
)]
fn then_o_registro_identifica_o_gestor_valor_como_responsavel_e_a_data_da_remocao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o registro identifica o gestor {string} como responsável e a data da remoção");
}

/// Pendente: o registro identifica o gestor {string} como responsável e a data da vinculação.
#[then(
    regex = r#"^o registro identifica o gestor "([^"]*)" como responsável e a data da vinculação$"#
)]
fn then_o_registro_identifica_o_gestor_valor_como_responsavel_e_a_data_da_vinculacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o registro identifica o gestor {string} como responsável e a data da vinculação");
}

/// Pendente: o registro permanece inalterado no histórico de {string}.
#[then(regex = r#"^o registro permanece inalterado no histórico de "([^"]*)"$"#)]
fn then_o_registro_permanece_inalterado_no_historico_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o registro permanece inalterado no histórico de {string}");
}

/// Pendente: os registros anteriores à remoção permanecem inalterados.
#[then(regex = r#"^os registros anteriores à remoção permanecem inalterados$"#)]
fn then_os_registros_anteriores_a_remocao_permanecem_inalterados(_world: &mut AppWorld) {
    pending_step("os registros anteriores à remoção permanecem inalterados");
}

/// Pendente: os registros da passagem anterior de {string} pela gestão permanecem disponíveis.
#[then(
    regex = r#"^os registros da passagem anterior de "([^"]*)" pela gestão permanecem disponíveis$"#
)]
fn then_os_registros_da_passagem_anterior_de_valor_pela_gestao_permanecem_disponiveis(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "os registros da passagem anterior de {string} pela gestão permanecem disponíveis",
    );
}

/// Pendente: perde apenas as permissões que não sejam concedidas por seus cargos restantes.
#[then(
    regex = r#"^perde apenas as permissões que não sejam concedidas por seus cargos restantes$"#
)]
fn then_perde_apenas_as_permissoes_que_nao_sejam_concedidas_por_seus_cargos_restantes(
    _world: &mut AppWorld,
) {
    pending_step("perde apenas as permissões que não sejam concedidas por seus cargos restantes");
}

/// Pendente: permanece autorizado a administrar a gestão pelo cargo restante.
#[then(regex = r#"^permanece autorizado a administrar a gestão pelo cargo restante$"#)]
fn then_permanece_autorizado_a_administrar_a_gestao_pelo_cargo_restante(_world: &mut AppWorld) {
    pending_step("permanece autorizado a administrar a gestão pelo cargo restante");
}

/// Pendente: possui as permissões de publicação de notícias e gerenciamento de eventos nesse CA.
#[then(
    regex = r#"^possui as permissões de publicação de notícias e gerenciamento de eventos nesse CA$"#
)]
fn then_possui_as_permissoes_de_publicacao_de_noticias_e_gerenciamento_de_eventos_nesse_ca(
    _world: &mut AppWorld,
) {
    pending_step(
        "possui as permissões de publicação de notícias e gerenciamento de eventos nesse CA",
    );
}

/// Pendente: todos os vínculos do integrante permanecem inalterados.
#[then(regex = r#"^todos os vínculos do integrante permanecem inalterados$"#)]
fn then_todos_os_vinculos_do_integrante_permanecem_inalterados(_world: &mut AppWorld) {
    pending_step("todos os vínculos do integrante permanecem inalterados");
}

/// Pendente: {string} deixa de possuir a permissão de {string} em {string}.
#[then(regex = r#"^"([^"]*)" deixa de possuir a permissão de "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_deixa_de_possuir_a_permissao_de_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} deixa de possuir a permissão de {string} em {string}");
}

/// Pendente: {string} deixa de possuir as permissões do cargo {string} em {string}.
#[then(regex = r#"^"([^"]*)" deixa de possuir as permissões do cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_deixa_de_possuir_as_permissoes_do_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} deixa de possuir as permissões do cargo {string} em {string}");
}

/// Pendente: {string} passa a exercer o cargo {string} em {string}.
#[then(regex = r#"^"([^"]*)" passa a exercer o cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_passa_a_exercer_o_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} passa a exercer o cargo {string} em {string}");
}

/// Pendente: {string} passa a integrar a gestão de {string} no cargo {string}.
#[then(regex = r#"^"([^"]*)" passa a integrar a gestão de "([^"]*)" no cargo "([^"]*)"$"#)]
fn then_valor_passa_a_integrar_a_gestao_de_valor_no_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} passa a integrar a gestão de {string} no cargo {string}");
}

/// Pendente: {string} passa a possuir a permissão de {string} em {string}.
#[then(regex = r#"^"([^"]*)" passa a possuir a permissão de "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_passa_a_possuir_a_permissao_de_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} passa a possuir a permissão de {string} em {string}");
}

/// Pendente: {string} passa a possuir as permissões do cargo {string} em {string}.
#[then(regex = r#"^"([^"]*)" passa a possuir as permissões do cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_passa_a_possuir_as_permissoes_do_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} passa a possuir as permissões do cargo {string} em {string}");
}

/// Pendente: {string} permanece sem integrar a gestão de {string}.
#[then(regex = r#"^"([^"]*)" permanece sem integrar a gestão de "([^"]*)"$"#)]
fn then_valor_permanece_sem_integrar_a_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} permanece sem integrar a gestão de {string}");
}

/// Pendente: {string} permanece somente no cargo {string} em {string}.
#[then(regex = r#"^"([^"]*)" permanece somente no cargo "([^"]*)" em "([^"]*)"$"#)]
fn then_valor_permanece_somente_no_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} permanece somente no cargo {string} em {string}");
}

/// Pendente: o gestor {string} altera o cargo de {string} para {string} em {string}.
#[when(regex = r#"^o gestor "([^"]*)" altera o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$"#)]
fn when_o_gestor_valor_altera_o_cargo_de_valor_para_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} altera o cargo de {string} para {string} em {string}");
}

/// Pendente: o gestor {string} remove somente seu vínculo com o cargo {string}.
#[when(regex = r#"^o gestor "([^"]*)" remove somente seu vínculo com o cargo "([^"]*)"$"#)]
fn when_o_gestor_valor_remove_somente_seu_vinculo_com_o_cargo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o gestor {string} remove somente seu vínculo com o cargo {string}");
}

/// Pendente: o gestor {string} remove somente um desses vínculos.
#[when(regex = r#"^o gestor "([^"]*)" remove somente um desses vínculos$"#)]
fn when_o_gestor_valor_remove_somente_um_desses_vinculos(_world: &mut AppWorld, _value_1: String) {
    pending_step("o gestor {string} remove somente um desses vínculos");
}

/// Pendente: o gestor {string} tenta alterar esse registro do histórico.
#[when(regex = r#"^o gestor "([^"]*)" tenta alterar esse registro do histórico$"#)]
fn when_o_gestor_valor_tenta_alterar_esse_registro_do_historico(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o gestor {string} tenta alterar esse registro do histórico");
}

/// Pendente: o gestor {string} tenta alterar o cargo de {string} para {string} em {string}.
#[when(
    regex = r#"^o gestor "([^"]*)" tenta alterar o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$"#
)]
fn when_o_gestor_valor_tenta_alterar_o_cargo_de_valor_para_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} tenta alterar o cargo de {string} para {string} em {string}");
}

/// Pendente: o gestor {string} tenta excluir esse registro do histórico.
#[when(regex = r#"^o gestor "([^"]*)" tenta excluir esse registro do histórico$"#)]
fn when_o_gestor_valor_tenta_excluir_esse_registro_do_historico(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o gestor {string} tenta excluir esse registro do histórico");
}

/// Pendente: o gestor {string} tenta remover {string} da gestão de {string}.
#[when(regex = r#"^o gestor "([^"]*)" tenta remover "([^"]*)" da gestão de "([^"]*)"$"#)]
fn when_o_gestor_valor_tenta_remover_valor_da_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} tenta remover {string} da gestão de {string}");
}

/// Pendente: o gestor {string} tenta vincular {string} a um cargo em {string}.
#[when(regex = r#"^o gestor "([^"]*)" tenta vincular "([^"]*)" a um cargo em "([^"]*)"$"#)]
fn when_o_gestor_valor_tenta_vincular_valor_a_um_cargo_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} tenta vincular {string} a um cargo em {string}");
}

/// Pendente: o gestor {string} tenta vincular {string} novamente ao cargo {string} em {string}.
#[when(
    regex = r#"^o gestor "([^"]*)" tenta vincular "([^"]*)" novamente ao cargo "([^"]*)" em "([^"]*)"$"#
)]
fn when_o_gestor_valor_tenta_vincular_valor_novamente_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step(
        "o gestor {string} tenta vincular {string} novamente ao cargo {string} em {string}",
    );
}

/// Pendente: o gestor {string} vincula {string} ao cargo {string} em {string}.
#[when(regex = r#"^o gestor "([^"]*)" vincula "([^"]*)" ao cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_o_gestor_valor_vincula_valor_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} vincula {string} ao cargo {string} em {string}");
}

/// Pendente: o gestor {string} vincula {string} também ao cargo {string} em {string}.
#[when(regex = r#"^o gestor "([^"]*)" vincula "([^"]*)" também ao cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_o_gestor_valor_vincula_valor_tambem_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("o gestor {string} vincula {string} também ao cargo {string} em {string}");
}

/// Pendente: remove somente um desses vínculos.
#[when(regex = r#"^remove somente um desses vínculos$"#)]
fn when_remove_somente_um_desses_vinculos(_world: &mut AppWorld) {
    pending_step("remove somente um desses vínculos");
}

/// Pendente: tenta remover seu vínculo com esse cargo.
#[when(regex = r#"^tenta remover seu vínculo com esse cargo$"#)]
fn when_tenta_remover_seu_vinculo_com_esse_cargo(_world: &mut AppWorld) {
    pending_step("tenta remover seu vínculo com esse cargo");
}

/// Pendente: {string} tenta alterar o cargo de {string} para {string} em {string}.
#[when(regex = r#"^"([^"]*)" tenta alterar o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$"#)]
fn when_valor_tenta_alterar_o_cargo_de_valor_para_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("{string} tenta alterar o cargo de {string} para {string} em {string}");
}

/// Pendente: {string} tenta remover {string} da gestão de {string}.
#[when(regex = r#"^"([^"]*)" tenta remover "([^"]*)" da gestão de "([^"]*)"$"#)]
fn when_valor_tenta_remover_valor_da_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} tenta remover {string} da gestão de {string}");
}

/// Pendente: {string} tenta vincular {string} ao cargo {string} em {string}.
#[when(regex = r#"^"([^"]*)" tenta vincular "([^"]*)" ao cargo "([^"]*)" em "([^"]*)"$"#)]
fn when_valor_tenta_vincular_valor_ao_cargo_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step("{string} tenta vincular {string} ao cargo {string} em {string}");
}
