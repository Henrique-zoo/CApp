//! A consulta da gestão vigente e de seus registros históricos permite conhecer a organização e a trajetória do CA.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: a exclusão desse cargo da estrutura atual do CA foi permitida.
#[given(regex = r#"^a exclusão desse cargo da estrutura atual do CA foi permitida$"#)]
fn given_a_exclusao_desse_cargo_da_estrutura_atual_do_ca_foi_permitida(_world: &mut AppWorld) {
    pending_step("a exclusão desse cargo da estrutura atual do CA foi permitida");
}

/// Pendente: a gestão possui {string} no cargo de {string}.
#[given(regex = r#"^a gestão possui "([^"]*)" no cargo de "([^"]*)"$"#)]
fn given_a_gestao_possui_valor_no_cargo_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a gestão possui {string} no cargo de {string}");
}

/// Pendente: esse cargo fez parte da gestão com mandato de {string} a {string}.
#[given(regex = r#"^esse cargo fez parte da gestão com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn given_esse_cargo_fez_parte_da_gestao_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("esse cargo fez parte da gestão com mandato de {string} a {string}");
}

/// Pendente: estou autenticado no CApp com uma conta de estudante.
#[given(regex = r#"^estou autenticado no CApp com uma conta de estudante$"#)]
fn given_estou_autenticado_no_capp_com_uma_conta_de_estudante(_world: &mut AppWorld) {
    pending_step("estou autenticado no CApp com uma conta de estudante");
}

/// Pendente: existe uma gestão registrada com mandato de {string} a {string}.
#[given(regex = r#"^existe uma gestão registrada com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn given_existe_uma_gestao_registrada_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe uma gestão registrada com mandato de {string} a {string}");
}

/// Pendente: existe uma gestão vigente nesse Centro Acadêmico.
#[given(regex = r#"^existe uma gestão vigente nesse Centro Acadêmico$"#)]
fn given_existe_uma_gestao_vigente_nesse_centro_academico(_world: &mut AppWorld) {
    pending_step("existe uma gestão vigente nesse Centro Acadêmico");
}

/// Pendente: existe uma gestão vigente no {string}.
#[given(regex = r#"^existe uma gestão vigente no "([^"]*)"$"#)]
fn given_existe_uma_gestao_vigente_no_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe uma gestão vigente no {string}");
}

/// Pendente: existe uma gestão vigente registrada.
#[given(regex = r#"^existe uma gestão vigente registrada$"#)]
fn given_existe_uma_gestao_vigente_registrada(_world: &mut AppWorld) {
    pending_step("existe uma gestão vigente registrada");
}

/// Pendente: existe uma gestão vigente registrada nesse Centro Acadêmico.
#[given(regex = r#"^existe uma gestão vigente registrada nesse Centro Acadêmico$"#)]
fn given_existe_uma_gestao_vigente_registrada_nesse_centro_academico(_world: &mut AppWorld) {
    pending_step("existe uma gestão vigente registrada nesse Centro Acadêmico");
}

/// Pendente: existe uma nova gestão vigente com mandato de {string} a {string}.
#[given(regex = r#"^existe uma nova gestão vigente com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn given_existe_uma_nova_gestao_vigente_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe uma nova gestão vigente com mandato de {string} a {string}");
}

/// Pendente: existem gestões anteriores registradas no CApp.
#[given(regex = r#"^existem gestões anteriores registradas no CApp$"#)]
fn given_existem_gestoes_anteriores_registradas_no_capp(_world: &mut AppWorld) {
    pending_step("existem gestões anteriores registradas no CApp");
}

/// Pendente: nenhuma gestão foi registrada nesse Centro Acadêmico desde sua adesão.
#[given(regex = r#"^nenhuma gestão foi registrada nesse Centro Acadêmico desde sua adesão$"#)]
fn given_nenhuma_gestao_foi_registrada_nesse_centro_academico_desde_sua_adesao(
    _world: &mut AppWorld,
) {
    pending_step("nenhuma gestão foi registrada nesse Centro Acadêmico desde sua adesão");
}

/// Pendente: não possuo permissões administrativas nesse Centro Acadêmico.
#[given(regex = r#"^não possuo permissões administrativas nesse Centro Acadêmico$"#)]
fn given_nao_possuo_permissoes_administrativas_nesse_centro_academico(_world: &mut AppWorld) {
    pending_step("não possuo permissões administrativas nesse Centro Acadêmico");
}

/// Pendente: o mandato dessa gestão começa em {string} e termina em {string}.
#[given(regex = r#"^o mandato dessa gestão começa em "([^"]*)" e termina em "([^"]*)"$"#)]
fn given_o_mandato_dessa_gestao_comeca_em_valor_e_termina_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o mandato dessa gestão começa em {string} e termina em {string}");
}

/// Pendente: o vínculo ativo de {string} com esse cargo foi encerrado.
#[given(regex = r#"^o vínculo ativo de "([^"]*)" com esse cargo foi encerrado$"#)]
fn given_o_vinculo_ativo_de_valor_com_esse_cargo_foi_encerrado(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o vínculo ativo de {string} com esse cargo foi encerrado");
}

/// Pendente: o {string} está selecionado como Centro Acadêmico ativo.
#[given(regex = r#"^o "([^"]*)" está selecionado como Centro Acadêmico ativo$"#)]
fn given_o_valor_esta_selecionado_como_centro_academico_ativo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o {string} está selecionado como Centro Acadêmico ativo");
}

/// Pendente: o {string} possui gestões registradas no CApp.
#[given(regex = r#"^o "([^"]*)" possui gestões registradas no CApp$"#)]
fn given_o_valor_possui_gestoes_registradas_no_capp(_world: &mut AppWorld, _value_1: String) {
    pending_step("o {string} possui gestões registradas no CApp");
}

/// Pendente: o {string} é o Centro Acadêmico ativo.
#[given(regex = r#"^o "([^"]*)" é o Centro Acadêmico ativo$"#)]
fn given_o_valor_e_o_centro_academico_ativo(_world: &mut AppWorld, _value_1: String) {
    pending_step("o {string} é o Centro Acadêmico ativo");
}

/// Pendente: o {string} é selecionado como contexto inicial.
#[given(regex = r#"^o "([^"]*)" é selecionado como contexto inicial$"#)]
fn given_o_valor_e_selecionado_como_contexto_inicial(_world: &mut AppWorld, _value_1: String) {
    pending_step("o {string} é selecionado como contexto inicial");
}

/// Pendente: que existe uma gestão encerrada com mandato de {string} a {string}.
#[given(regex = r#"^que existe uma gestão encerrada com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn given_que_existe_uma_gestao_encerrada_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe uma gestão encerrada com mandato de {string} a {string}");
}

/// Pendente: que existe uma gestão vigente com mandato de {string} a {string}.
#[given(regex = r#"^que existe uma gestão vigente com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn given_que_existe_uma_gestao_vigente_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe uma gestão vigente com mandato de {string} a {string}");
}

/// Pendente: que existe uma gestão vigente no {string}.
#[given(regex = r#"^que existe uma gestão vigente no "([^"]*)"$"#)]
fn given_que_existe_uma_gestao_vigente_no_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("que existe uma gestão vigente no {string}");
}

/// Pendente: que não faço parte da gestão do {string}.
#[given(regex = r#"^que não faço parte da gestão do "([^"]*)"$"#)]
fn given_que_nao_faco_parte_da_gestao_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("que não faço parte da gestão do {string}");
}

/// Pendente: que não possuo cargo nem permissão administrativa no {string}.
#[given(regex = r#"^que não possuo cargo nem permissão administrativa no "([^"]*)"$"#)]
fn given_que_nao_possuo_cargo_nem_permissao_administrativa_no_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que não possuo cargo nem permissão administrativa no {string}");
}

/// Pendente: que o {string} aderiu ao CApp.
#[given(regex = r#"^que o "([^"]*)" aderiu ao CApp$"#)]
fn given_que_o_valor_aderiu_ao_capp(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o {string} aderiu ao CApp");
}

/// Pendente: que o {string} aderiu ao CApp em {string}.
#[given(regex = r#"^que o "([^"]*)" aderiu ao CApp em "([^"]*)"$"#)]
fn given_que_o_valor_aderiu_ao_capp_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o {string} aderiu ao CApp em {string}");
}

/// Pendente: que o {string} não possui gestões registradas no CApp.
#[given(regex = r#"^que o "([^"]*)" não possui gestões registradas no CApp$"#)]
fn given_que_o_valor_nao_possui_gestoes_registradas_no_capp(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o {string} não possui gestões registradas no CApp");
}

/// Pendente: que o {string} não possui uma gestão vigente registrada no CApp.
#[given(regex = r#"^que o "([^"]*)" não possui uma gestão vigente registrada no CApp$"#)]
fn given_que_o_valor_nao_possui_uma_gestao_vigente_registrada_no_capp(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o {string} não possui uma gestão vigente registrada no CApp");
}

/// Pendente: que o {string} é o CA favorito da minha conta.
#[given(regex = r#"^que o "([^"]*)" é o CA favorito da minha conta$"#)]
fn given_que_o_valor_e_o_ca_favorito_da_minha_conta(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o {string} é o CA favorito da minha conta");
}

/// Pendente: que {string} ocupou o cargo personalizado de {string}.
#[given(regex = r#"^que "([^"]*)" ocupou o cargo personalizado de "([^"]*)"$"#)]
fn given_que_valor_ocupou_o_cargo_personalizado_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que {string} ocupou o cargo personalizado de {string}");
}

/// Pendente: {string} integra a gestão de {string} como {string}.
#[given(regex = r#"^"([^"]*)" integra a gestão de "([^"]*)" como "([^"]*)"$"#)]
fn given_valor_integra_a_gestao_de_valor_como_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} integra a gestão de {string} como {string}");
}

/// Pendente: {string} integra a gestão do {string}.
#[given(regex = r#"^"([^"]*)" integra a gestão do "([^"]*)"$"#)]
fn given_valor_integra_a_gestao_do_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} integra a gestão do {string}");
}

/// Pendente: {string} integrou a gestão de {string} como {string}.
#[given(regex = r#"^"([^"]*)" integrou a gestão de "([^"]*)" como "([^"]*)"$"#)]
fn given_valor_integrou_a_gestao_de_valor_como_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} integrou a gestão de {string} como {string}");
}

/// Pendente: {string} integrou essa gestão no cargo de {string}.
#[given(regex = r#"^"([^"]*)" integrou essa gestão no cargo de "([^"]*)"$"#)]
fn given_valor_integrou_essa_gestao_no_cargo_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} integrou essa gestão no cargo de {string}");
}

/// Pendente: {string} ocupou o cargo de {string} nessa gestão.
#[given(regex = r#"^"([^"]*)" ocupou o cargo de "([^"]*)" nessa gestão$"#)]
fn given_valor_ocupou_o_cargo_de_valor_nessa_gestao(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("{string} ocupou o cargo de {string} nessa gestão");
}

/// Pendente: a ausência de permissão administrativa não impede a consulta.
#[then(regex = r#"^a ausência de permissão administrativa não impede a consulta$"#)]
fn then_a_ausencia_de_permissao_administrativa_nao_impede_a_consulta(_world: &mut AppWorld) {
    pending_step("a ausência de permissão administrativa não impede a consulta");
}

/// Pendente: a composição da gestão anterior não é substituída pela nova composição.
#[then(regex = r#"^a composição da gestão anterior não é substituída pela nova composição$"#)]
fn then_a_composicao_da_gestao_anterior_nao_e_substituida_pela_nova_composicao(
    _world: &mut AppWorld,
) {
    pending_step("a composição da gestão anterior não é substituída pela nova composição");
}

/// Pendente: a consulta não exige uma permissão administrativa.
#[then(regex = r#"^a consulta não exige uma permissão administrativa$"#)]
fn then_a_consulta_nao_exige_uma_permissao_administrativa(_world: &mut AppWorld) {
    pending_step("a consulta não exige uma permissão administrativa");
}

/// Pendente: a consulta utiliza o contexto do Centro Acadêmico selecionado.
#[then(regex = r#"^a consulta utiliza o contexto do Centro Acadêmico selecionado$"#)]
fn then_a_consulta_utiliza_o_contexto_do_centro_academico_selecionado(_world: &mut AppWorld) {
    pending_step("a consulta utiliza o contexto do Centro Acadêmico selecionado");
}

/// Pendente: a consulta é disponibilizada normalmente.
#[then(regex = r#"^a consulta é disponibilizada normalmente$"#)]
fn then_a_consulta_e_disponibilizada_normalmente(_world: &mut AppWorld) {
    pending_step("a consulta é disponibilizada normalmente");
}

/// Pendente: a exclusão do cargo da estrutura atual não apaga seu registro histórico.
#[then(regex = r#"^a exclusão do cargo da estrutura atual não apaga seu registro histórico$"#)]
fn then_a_exclusao_do_cargo_da_estrutura_atual_nao_apaga_seu_registro_historico(
    _world: &mut AppWorld,
) {
    pending_step("a exclusão do cargo da estrutura atual não apaga seu registro histórico");
}

/// Pendente: a gestão de {string} continua disponível no histórico.
#[then(regex = r#"^a gestão de "([^"]*)" continua disponível no histórico$"#)]
fn then_a_gestao_de_valor_continua_disponivel_no_historico(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a gestão de {string} continua disponível no histórico");
}

/// Pendente: a gestão de {string} é apresentada como a gestão vigente.
#[then(regex = r#"^a gestão de "([^"]*)" é apresentada como a gestão vigente$"#)]
fn then_a_gestao_de_valor_e_apresentada_como_a_gestao_vigente(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a gestão de {string} é apresentada como a gestão vigente");
}

/// Pendente: consigo identificar a participação histórica de cada integrante.
#[then(regex = r#"^consigo identificar a participação histórica de cada integrante$"#)]
fn then_consigo_identificar_a_participacao_historica_de_cada_integrante(_world: &mut AppWorld) {
    pending_step("consigo identificar a participação histórica de cada integrante");
}

/// Pendente: consigo identificar o período de cada gestão registrada.
#[then(regex = r#"^consigo identificar o período de cada gestão registrada$"#)]
fn then_consigo_identificar_o_periodo_de_cada_gestao_registrada(_world: &mut AppWorld) {
    pending_step("consigo identificar o período de cada gestão registrada");
}

/// Pendente: consigo identificar os respectivos períodos de mandato.
#[then(regex = r#"^consigo identificar os respectivos períodos de mandato$"#)]
fn then_consigo_identificar_os_respectivos_periodos_de_mandato(_world: &mut AppWorld) {
    pending_step("consigo identificar os respectivos períodos de mandato");
}

/// Pendente: consigo identificar {string} como {string}.
#[then(regex = r#"^consigo identificar "([^"]*)" como "([^"]*)"$"#)]
fn then_consigo_identificar_valor_como_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("consigo identificar {string} como {string}");
}

/// Pendente: consigo visualizar a gestão vigente.
#[then(regex = r#"^consigo visualizar a gestão vigente$"#)]
fn then_consigo_visualizar_a_gestao_vigente(_world: &mut AppWorld) {
    pending_step("consigo visualizar a gestão vigente");
}

/// Pendente: consigo visualizar as datas de início e término do mandato.
#[then(regex = r#"^consigo visualizar as datas de início e término do mandato$"#)]
fn then_consigo_visualizar_as_datas_de_inicio_e_termino_do_mandato(_world: &mut AppWorld) {
    pending_step("consigo visualizar as datas de início e término do mandato");
}

/// Pendente: consigo visualizar as gestões registradas.
#[then(regex = r#"^consigo visualizar as gestões registradas$"#)]
fn then_consigo_visualizar_as_gestoes_registradas(_world: &mut AppWorld) {
    pending_step("consigo visualizar as gestões registradas");
}

/// Pendente: consigo visualizar os cargos ocupados por cada integrante.
#[then(regex = r#"^consigo visualizar os cargos ocupados por cada integrante$"#)]
fn then_consigo_visualizar_os_cargos_ocupados_por_cada_integrante(_world: &mut AppWorld) {
    pending_step("consigo visualizar os cargos ocupados por cada integrante");
}

/// Pendente: consigo visualizar os integrantes da gestão vigente.
#[then(regex = r#"^consigo visualizar os integrantes da gestão vigente$"#)]
fn then_consigo_visualizar_os_integrantes_da_gestao_vigente(_world: &mut AppWorld) {
    pending_step("consigo visualizar os integrantes da gestão vigente");
}

/// Pendente: encontro a gestão com mandato de {string} a {string}.
#[then(regex = r#"^encontro a gestão com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn then_encontro_a_gestao_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("encontro a gestão com mandato de {string} a {string}");
}

/// Pendente: não apresenta as gestões do {string}.
#[then(regex = r#"^não apresenta as gestões do "([^"]*)"$"#)]
fn then_nao_apresenta_as_gestoes_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("não apresenta as gestões do {string}");
}

/// Pendente: não apresenta cargos atuais sem registro correspondente.
#[then(regex = r#"^não apresenta cargos atuais sem registro correspondente$"#)]
fn then_nao_apresenta_cargos_atuais_sem_registro_correspondente(_world: &mut AppWorld) {
    pending_step("não apresenta cargos atuais sem registro correspondente");
}

/// Pendente: não apresenta cargos sem associação com uma gestão registrada.
#[then(regex = r#"^não apresenta cargos sem associação com uma gestão registrada$"#)]
fn then_nao_apresenta_cargos_sem_associacao_com_uma_gestao_registrada(_world: &mut AppWorld) {
    pending_step("não apresenta cargos sem associação com uma gestão registrada");
}

/// Pendente: não apresenta integrantes como membros de uma gestão vigente.
#[then(regex = r#"^não apresenta integrantes como membros de uma gestão vigente$"#)]
fn then_nao_apresenta_integrantes_como_membros_de_uma_gestao_vigente(_world: &mut AppWorld) {
    pending_step("não apresenta integrantes como membros de uma gestão vigente");
}

/// Pendente: não apresenta integrantes sem participação histórica registrada.
#[then(regex = r#"^não apresenta integrantes sem participação histórica registrada$"#)]
fn then_nao_apresenta_integrantes_sem_participacao_historica_registrada(_world: &mut AppWorld) {
    pending_step("não apresenta integrantes sem participação histórica registrada");
}

/// Pendente: não são apresentadas informações exclusivas do {string}.
#[then(regex = r#"^não são apresentadas informações exclusivas do "([^"]*)"$"#)]
fn then_nao_sao_apresentadas_informacoes_exclusivas_do_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("não são apresentadas informações exclusivas do {string}");
}

/// Pendente: não utiliza registros de outro Centro Acadêmico para preencher o histórico do CA ativo.
#[then(
    regex = r#"^não utiliza registros de outro Centro Acadêmico para preencher o histórico do CA ativo$"#
)]
fn then_nao_utiliza_registros_de_outro_centro_academico_para_preencher_o_historico_do_ca_ativo(
    _world: &mut AppWorld,
) {
    pending_step(
        "não utiliza registros de outro Centro Acadêmico para preencher o histórico do CA ativo",
    );
}

/// Pendente: não visualizo {string} como integrante da gestão do CA ativo.
#[then(regex = r#"^não visualizo "([^"]*)" como integrante da gestão do CA ativo$"#)]
fn then_nao_visualizo_valor_como_integrante_da_gestao_do_ca_ativo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("não visualizo {string} como integrante da gestão do CA ativo");
}

/// Pendente: o aplicativo informa que não existe gestão vigente registrada.
#[then(regex = r#"^o aplicativo informa que não existe gestão vigente registrada$"#)]
fn then_o_aplicativo_informa_que_nao_existe_gestao_vigente_registrada(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que não existe gestão vigente registrada");
}

/// Pendente: o aplicativo informa que não existem gestões anteriores registradas.
#[then(regex = r#"^o aplicativo informa que não existem gestões anteriores registradas$"#)]
fn then_o_aplicativo_informa_que_nao_existem_gestoes_anteriores_registradas(_world: &mut AppWorld) {
    pending_step("o aplicativo informa que não existem gestões anteriores registradas");
}

/// Pendente: o aplicativo informa que não existem gestões registradas para o {string}.
#[then(regex = r#"^o aplicativo informa que não existem gestões registradas para o "([^"]*)"$"#)]
fn then_o_aplicativo_informa_que_nao_existem_gestoes_registradas_para_o_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o aplicativo informa que não existem gestões registradas para o {string}");
}

/// Pendente: o cargo de {string} continua associado à participação histórica de {string}.
#[then(
    regex = r#"^o cargo de "([^"]*)" continua associado à participação histórica de "([^"]*)"$"#
)]
fn then_o_cargo_de_valor_continua_associado_a_participacao_historica_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo de {string} continua associado à participação histórica de {string}");
}

/// Pendente: o cargo {string} continua associado à participação histórica de {string}.
#[then(regex = r#"^o cargo "([^"]*)" continua associado à participação histórica de "([^"]*)"$"#)]
fn then_o_cargo_valor_continua_associado_a_participacao_historica_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o cargo {string} continua associado à participação histórica de {string}");
}

/// Pendente: os dados apresentados pertencem ao CA favorito selecionado como contexto inicial.
#[then(
    regex = r#"^os dados apresentados pertencem ao CA favorito selecionado como contexto inicial$"#
)]
fn then_os_dados_apresentados_pertencem_ao_ca_favorito_selecionado_como_contexto_inicial(
    _world: &mut AppWorld,
) {
    pending_step(
        "os dados apresentados pertencem ao CA favorito selecionado como contexto inicial",
    );
}

/// Pendente: visualizo a gestão do {string}.
#[then(regex = r#"^visualizo a gestão do "([^"]*)"$"#)]
fn then_visualizo_a_gestao_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("visualizo a gestão do {string}");
}

/// Pendente: visualizo as informações do {string}.
#[then(regex = r#"^visualizo as informações do "([^"]*)"$"#)]
fn then_visualizo_as_informacoes_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("visualizo as informações do {string}");
}

/// Pendente: visualizo o período de mandato dessa gestão.
#[then(regex = r#"^visualizo o período de mandato dessa gestão$"#)]
fn then_visualizo_o_periodo_de_mandato_dessa_gestao(_world: &mut AppWorld) {
    pending_step("visualizo o período de mandato dessa gestão");
}

/// Pendente: visualizo {string} como antiga {string}.
#[then(regex = r#"^visualizo "([^"]*)" como antiga "([^"]*)"$"#)]
fn then_visualizo_valor_como_antiga_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("visualizo {string} como antiga {string}");
}

/// Pendente: visualizo {string} como antigo {string}.
#[then(regex = r#"^visualizo "([^"]*)" como antigo "([^"]*)"$"#)]
fn then_visualizo_valor_como_antigo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("visualizo {string} como antigo {string}");
}

/// Pendente: visualizo {string} como integrante da gestão do CA ativo.
#[then(regex = r#"^visualizo "([^"]*)" como integrante da gestão do CA ativo$"#)]
fn then_visualizo_valor_como_integrante_da_gestao_do_ca_ativo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("visualizo {string} como integrante da gestão do CA ativo");
}

/// Pendente: {string} continua apresentado como integrante daquela gestão.
#[then(regex = r#"^"([^"]*)" continua apresentado como integrante daquela gestão$"#)]
fn then_valor_continua_apresentado_como_integrante_daquela_gestao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} continua apresentado como integrante daquela gestão");
}

/// Pendente: {string} continua associada ao cargo de {string} na gestão de {string}.
#[then(regex = r#"^"([^"]*)" continua associada ao cargo de "([^"]*)" na gestão de "([^"]*)"$"#)]
fn then_valor_continua_associada_ao_cargo_de_valor_na_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} continua associada ao cargo de {string} na gestão de {string}");
}

/// Pendente: {string} continua associado ao cargo de {string} na gestão de {string}.
#[then(regex = r#"^"([^"]*)" continua associado ao cargo de "([^"]*)" na gestão de "([^"]*)"$"#)]
fn then_valor_continua_associado_ao_cargo_de_valor_na_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} continua associado ao cargo de {string} na gestão de {string}");
}

/// Pendente: {string} continua registrada no histórico de participação dessa gestão.
#[then(regex = r#"^"([^"]*)" continua registrada no histórico de participação dessa gestão$"#)]
fn then_valor_continua_registrada_no_historico_de_participacao_dessa_gestao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} continua registrada no histórico de participação dessa gestão");
}

/// Pendente: {string} não é apresentada entre os integrantes atualmente vinculados à gestão.
#[then(
    regex = r#"^"([^"]*)" não é apresentada entre os integrantes atualmente vinculados à gestão$"#
)]
fn then_valor_nao_e_apresentada_entre_os_integrantes_atualmente_vinculados_a_gestao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} não é apresentada entre os integrantes atualmente vinculados à gestão");
}

/// Pendente: acesso a consulta de governança do Centro Acadêmico ativo.
#[when(regex = r#"^acesso a consulta de governança do Centro Acadêmico ativo$"#)]
fn when_acesso_a_consulta_de_governanca_do_centro_academico_ativo(_world: &mut AppWorld) {
    pending_step("acesso a consulta de governança do Centro Acadêmico ativo");
}

/// Pendente: acesso as informações de governança sem alterar o contexto ativo.
#[when(regex = r#"^acesso as informações de governança sem alterar o contexto ativo$"#)]
fn when_acesso_as_informacoes_de_governanca_sem_alterar_o_contexto_ativo(_world: &mut AppWorld) {
    pending_step("acesso as informações de governança sem alterar o contexto ativo");
}

/// Pendente: altero o Centro Acadêmico ativo para {string}.
#[when(regex = r#"^altero o Centro Acadêmico ativo para "([^"]*)"$"#)]
fn when_altero_o_centro_academico_ativo_para_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("altero o Centro Acadêmico ativo para {string}");
}

/// Pendente: consulto a composição histórica da gestão de {string}.
#[when(regex = r#"^consulto a composição histórica da gestão de "([^"]*)"$"#)]
fn when_consulto_a_composicao_historica_da_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("consulto a composição histórica da gestão de {string}");
}

/// Pendente: consulto a composição vigente e o histórico de participação dessa gestão.
#[when(regex = r#"^consulto a composição vigente e o histórico de participação dessa gestão$"#)]
fn when_consulto_a_composicao_vigente_e_o_historico_de_participacao_dessa_gestao(
    _world: &mut AppWorld,
) {
    pending_step("consulto a composição vigente e o histórico de participação dessa gestão");
}

/// Pendente: consulto as informações de governança.
#[when(regex = r#"^consulto as informações de governança$"#)]
fn when_consulto_as_informacoes_de_governanca(_world: &mut AppWorld) {
    pending_step("consulto as informações de governança");
}

/// Pendente: consulto as informações de governança do Centro Acadêmico ativo.
#[when(regex = r#"^consulto as informações de governança do Centro Acadêmico ativo$"#)]
fn when_consulto_as_informacoes_de_governanca_do_centro_academico_ativo(_world: &mut AppWorld) {
    pending_step("consulto as informações de governança do Centro Acadêmico ativo");
}

/// Pendente: consulto novamente as informações de governança.
#[when(regex = r#"^consulto novamente as informações de governança$"#)]
fn when_consulto_novamente_as_informacoes_de_governanca(_world: &mut AppWorld) {
    pending_step("consulto novamente as informações de governança");
}

/// Pendente: consulto o histórico de gestões.
#[when(regex = r#"^consulto o histórico de gestões$"#)]
fn when_consulto_o_historico_de_gestoes(_world: &mut AppWorld) {
    pending_step("consulto o histórico de gestões");
}

/// Pendente: consulto o histórico de gestões do Centro Acadêmico ativo.
#[when(regex = r#"^consulto o histórico de gestões do Centro Acadêmico ativo$"#)]
fn when_consulto_o_historico_de_gestoes_do_centro_academico_ativo(_world: &mut AppWorld) {
    pending_step("consulto o histórico de gestões do Centro Acadêmico ativo");
}

/// Pendente: consulto os detalhes da gestão com mandato de {string} a {string}.
#[when(regex = r#"^consulto os detalhes da gestão com mandato de "([^"]*)" a "([^"]*)"$"#)]
fn when_consulto_os_detalhes_da_gestao_com_mandato_de_valor_a_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("consulto os detalhes da gestão com mandato de {string} a {string}");
}
