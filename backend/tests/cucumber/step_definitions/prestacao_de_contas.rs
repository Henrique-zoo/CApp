//! A prestação de contas permite consultar e registrar informações financeiras com as permissões do CA.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: existe um lançamento publicado com um documento financeiro anexado.
#[given(regex = r#"^existe um lançamento publicado com um documento financeiro anexado$"#)]
fn given_existe_um_lancamento_publicado_com_um_documento_financeiro_anexado(_world: &mut AppWorld) {
    pending_step("existe um lançamento publicado com um documento financeiro anexado");
}

/// Pendente: existem lançamentos publicados em dois CAs diferentes.
#[given(regex = r#"^existem lançamentos publicados em dois CAs diferentes$"#)]
fn given_existem_lancamentos_publicados_em_dois_cas_diferentes(_world: &mut AppWorld) {
    pending_step("existem lançamentos publicados em dois CAs diferentes");
}

/// Pendente: não possuo a permissão {string} nesse CA.
#[given(regex = r#"^não possuo a permissão "([^"]*)" nesse CA$"#)]
fn given_nao_possuo_a_permissao_valor_nesse_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("não possuo a permissão {string} nesse CA");
}

/// Pendente: o CA ativo não possui lançamentos financeiros publicados.
#[given(regex = r#"^o CA ativo não possui lançamentos financeiros publicados$"#)]
fn given_o_ca_ativo_nao_possui_lancamentos_financeiros_publicados(_world: &mut AppWorld) {
    pending_step("o CA ativo não possui lançamentos financeiros publicados");
}

/// Pendente: o CA ativo publicou receitas e despesas.
#[given(regex = r#"^o CA ativo publicou receitas e despesas$"#)]
fn given_o_ca_ativo_publicou_receitas_e_despesas(_world: &mut AppWorld) {
    pending_step("o CA ativo publicou receitas e despesas");
}

/// Pendente: o Centro Acadêmico não possui lançamentos financeiros publicados.
#[given(regex = r#"^o Centro Acadêmico não possui lançamentos financeiros publicados$"#)]
fn given_o_centro_academico_nao_possui_lancamentos_financeiros_publicados(_world: &mut AppWorld) {
    pending_step("o Centro Acadêmico não possui lançamentos financeiros publicados");
}

/// Pendente: que existe um lançamento de despesa publicado com o identificador {string} no valor de {string}.
#[given(
    regex = r#"^que existe um lançamento de despesa publicado com o identificador "([^"]*)" no valor de "([^"]*)"$"#
)]
fn given_que_existe_um_lancamento_de_despesa_publicado_com_o_identificador_valor_no_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "que existe um lançamento de despesa publicado com o identificador {string} no valor de {string}",
    );
}

/// Pendente: que existe um lançamento de receita publicado com o identificador {string} no valor de {string}.
#[given(
    regex = r#"^que existe um lançamento de receita publicado com o identificador "([^"]*)" no valor de "([^"]*)"$"#
)]
fn given_que_existe_um_lancamento_de_receita_publicado_com_o_identificador_valor_no_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "que existe um lançamento de receita publicado com o identificador {string} no valor de {string}",
    );
}

/// Pendente: que existe um lançamento publicado com comprovante {string} anexado.
#[given(regex = r#"^que existe um lançamento publicado com comprovante "([^"]*)" anexado$"#)]
fn given_que_existe_um_lancamento_publicado_com_comprovante_valor_anexado(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que existe um lançamento publicado com comprovante {string} anexado");
}

/// Pendente: que integro a gestão do CA ativo.
#[given(regex = r#"^que integro a gestão do CA ativo$"#)]
fn given_que_integro_a_gestao_do_ca_ativo(_world: &mut AppWorld) {
    pending_step("que integro a gestão do CA ativo");
}

/// Pendente: que o Centro Acadêmico não possui lançamentos financeiros publicados.
#[given(regex = r#"^que o Centro Acadêmico não possui lançamentos financeiros publicados$"#)]
fn given_que_o_centro_academico_nao_possui_lancamentos_financeiros_publicados(
    _world: &mut AppWorld,
) {
    pending_step("que o Centro Acadêmico não possui lançamentos financeiros publicados");
}

/// Pendente: que o Centro Acadêmico possui lançamentos publicados contendo receitas de {string} e despesas de {string}.
#[given(
    regex = r#"^que o Centro Acadêmico possui lançamentos publicados contendo receitas de "([^"]*)" e despesas de "([^"]*)"$"#
)]
fn given_que_o_centro_academico_possui_lancamentos_publicados_contendo_receitas_de_valor_e_despesas_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "que o Centro Acadêmico possui lançamentos publicados contendo receitas de {string} e despesas de {string}",
    );
}

/// Pendente: que o usuário é um estudante autenticado vinculado a outro Centro Acadêmico.
#[given(regex = r#"^que o usuário é um estudante autenticado vinculado a outro Centro Acadêmico$"#)]
fn given_que_o_usuario_e_um_estudante_autenticado_vinculado_a_outro_centro_academico(
    _world: &mut AppWorld,
) {
    pending_step("que o usuário é um estudante autenticado vinculado a outro Centro Acadêmico");
}

/// Pendente: que o usuário é um estudante sem permissão financeira.
#[given(regex = r#"^que o usuário é um estudante sem permissão financeira$"#)]
fn given_que_o_usuario_e_um_estudante_sem_permissao_financeira(_world: &mut AppWorld) {
    pending_step("que o usuário é um estudante sem permissão financeira");
}

/// Pendente: que o usuário é um membro da gestão com permissão explícita de prestação de contas.
#[given(
    regex = r#"^que o usuário é um membro da gestão com permissão explícita de prestação de contas$"#
)]
fn given_que_o_usuario_e_um_membro_da_gestao_com_permissao_explicita_de_prestacao_de_contas(
    _world: &mut AppWorld,
) {
    pending_step(
        "que o usuário é um membro da gestão com permissão explícita de prestação de contas",
    );
}

/// Pendente: que o {string} é o CA ativo.
#[given(regex = r#"^que o "([^"]*)" é o CA ativo$"#)]
fn given_que_o_valor_e_o_ca_ativo(_world: &mut AppWorld, _value_1: String) {
    pending_step("que o {string} é o CA ativo");
}

/// Pendente: que possuo a permissão {string} somente em outro CA.
#[given(regex = r#"^que possuo a permissão "([^"]*)" somente em outro CA$"#)]
fn given_que_possuo_a_permissao_valor_somente_em_outro_ca(_world: &mut AppWorld, _value_1: String) {
    pending_step("que possuo a permissão {string} somente em outro CA");
}

/// Pendente: a alteração é bloqueada por falta de permissão administrativa.
#[then(regex = r#"^a alteração é bloqueada por falta de permissão administrativa$"#)]
fn then_a_alteracao_e_bloqueada_por_falta_de_permissao_administrativa(_world: &mut AppWorld) {
    pending_step("a alteração é bloqueada por falta de permissão administrativa");
}

/// Pendente: a consulta informa que não existem lançamentos publicados.
#[then(regex = r#"^a consulta informa que não existem lançamentos publicados$"#)]
fn then_a_consulta_informa_que_nao_existem_lancamentos_publicados(_world: &mut AppWorld) {
    pending_step("a consulta informa que não existem lançamentos publicados");
}

/// Pendente: a lista de lançamentos financeiros publicados está vazia.
#[then(regex = r#"^a lista de lançamentos financeiros publicados está vazia$"#)]
fn then_a_lista_de_lancamentos_financeiros_publicados_esta_vazia(_world: &mut AppWorld) {
    pending_step("a lista de lançamentos financeiros publicados está vazia");
}

/// Pendente: a operação de retificação é recusada informando que o lançamento não foi encontrado.
#[then(
    regex = r#"^a operação de retificação é recusada informando que o lançamento não foi encontrado$"#
)]
fn then_a_operacao_de_retificacao_e_recusada_informando_que_o_lancamento_nao_foi_encontrado(
    _world: &mut AppWorld,
) {
    pending_step(
        "a operação de retificação é recusada informando que o lançamento não foi encontrado",
    );
}

/// Pendente: a prestação de contas do CA ativo permanece inalterada.
#[then(regex = r#"^a prestação de contas do CA ativo permanece inalterada$"#)]
fn then_a_prestacao_de_contas_do_ca_ativo_permanece_inalterada(_world: &mut AppWorld) {
    pending_step("a prestação de contas do CA ativo permanece inalterada");
}

/// Pendente: a publicação é recusada por dados inválidos do lançamento.
#[then(regex = r#"^a publicação é recusada por dados inválidos do lançamento$"#)]
fn then_a_publicacao_e_recusada_por_dados_invalidos_do_lancamento(_world: &mut AppWorld) {
    pending_step("a publicação é recusada por dados inválidos do lançamento");
}

/// Pendente: a publicação é recusada por falta de permissão.
#[then(regex = r#"^a publicação é recusada por falta de permissão$"#)]
fn then_a_publicacao_e_recusada_por_falta_de_permissao(_world: &mut AppWorld) {
    pending_step("a publicação é recusada por falta de permissão");
}

/// Pendente: a retificação é recusada por ausência de justificativa obrigatória.
#[then(regex = r#"^a retificação é recusada por ausência de justificativa obrigatória$"#)]
fn then_a_retificacao_e_recusada_por_ausencia_de_justificativa_obrigatoria(_world: &mut AppWorld) {
    pending_step("a retificação é recusada por ausência de justificativa obrigatória");
}

/// Pendente: as despesas publicadas ficam disponíveis para consulta.
#[then(regex = r#"^as despesas publicadas ficam disponíveis para consulta$"#)]
fn then_as_despesas_publicadas_ficam_disponiveis_para_consulta(_world: &mut AppWorld) {
    pending_step("as despesas publicadas ficam disponíveis para consulta");
}

/// Pendente: ele visualiza o total de despesas acumuladas de {string}.
#[then(regex = r#"^ele visualiza o total de despesas acumuladas de "([^"]*)"$"#)]
fn then_ele_visualiza_o_total_de_despesas_acumuladas_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("ele visualiza o total de despesas acumuladas de {string}");
}

/// Pendente: ele visualiza o total de receitas acumuladas de {string}.
#[then(regex = r#"^ele visualiza o total de receitas acumuladas de "([^"]*)"$"#)]
fn then_ele_visualiza_o_total_de_receitas_acumuladas_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("ele visualiza o total de receitas acumuladas de {string}");
}

/// Pendente: nenhum lançamento financeiro é criado.
#[then(regex = r#"^nenhum lançamento financeiro é criado$"#)]
fn then_nenhum_lancamento_financeiro_e_criado(_world: &mut AppWorld) {
    pending_step("nenhum lançamento financeiro é criado");
}

/// Pendente: nenhum lançamento é incluído nos registros contábeis do Centro Acadêmico.
#[then(regex = r#"^nenhum lançamento é incluído nos registros contábeis do Centro Acadêmico$"#)]
fn then_nenhum_lancamento_e_incluido_nos_registros_contabeis_do_centro_academico(
    _world: &mut AppWorld,
) {
    pending_step("nenhum lançamento é incluído nos registros contábeis do Centro Acadêmico");
}

/// Pendente: o comprovante anexado é disponibilizado para visualização pública e download.
#[then(regex = r#"^o comprovante anexado é disponibilizado para visualização pública e download$"#)]
fn then_o_comprovante_anexado_e_disponibilizado_para_visualizacao_publica_e_download(
    _world: &mut AppWorld,
) {
    pending_step("o comprovante anexado é disponibilizado para visualização pública e download");
}

/// Pendente: o documento fica disponível para consulta.
#[then(regex = r#"^o documento fica disponível para consulta$"#)]
fn then_o_documento_fica_disponivel_para_consulta(_world: &mut AppWorld) {
    pending_step("o documento fica disponível para consulta");
}

/// Pendente: o histórico de cancelamento registra a justificativa, data e autor.
#[then(regex = r#"^o histórico de cancelamento registra a justificativa, data e autor$"#)]
fn then_o_historico_de_cancelamento_registra_a_justificativa_data_e_autor(_world: &mut AppWorld) {
    pending_step("o histórico de cancelamento registra a justificativa, data e autor");
}

/// Pendente: o histórico de retificações registra a alteração com justificativa, data e autor.
#[then(
    regex = r#"^o histórico de retificações registra a alteração com justificativa, data e autor$"#
)]
fn then_o_historico_de_retificacoes_registra_a_alteracao_com_justificativa_data_e_autor(
    _world: &mut AppWorld,
) {
    pending_step(
        "o histórico de retificações registra a alteração com justificativa, data e autor",
    );
}

/// Pendente: o lançamento fica disponível na prestação de contas do CA ativo.
#[then(regex = r#"^o lançamento fica disponível na prestação de contas do CA ativo$"#)]
fn then_o_lancamento_fica_disponivel_na_prestacao_de_contas_do_ca_ativo(_world: &mut AppWorld) {
    pending_step("o lançamento fica disponível na prestação de contas do CA ativo");
}

/// Pendente: o lançamento financeiro fica salvo como rascunho.
#[then(regex = r#"^o lançamento financeiro fica salvo como rascunho$"#)]
fn then_o_lancamento_financeiro_fica_salvo_como_rascunho(_world: &mut AppWorld) {
    pending_step("o lançamento financeiro fica salvo como rascunho");
}

/// Pendente: o lançamento {string} deixa de compor o saldo consolidado.
#[then(regex = r#"^o lançamento "([^"]*)" deixa de compor o saldo consolidado$"#)]
fn then_o_lancamento_valor_deixa_de_compor_o_saldo_consolidado(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o lançamento {string} deixa de compor o saldo consolidado");
}

/// Pendente: o lançamento {string} exibe os metadados retificados.
#[then(regex = r#"^o lançamento "([^"]*)" exibe os metadados retificados$"#)]
fn then_o_lancamento_valor_exibe_os_metadados_retificados(_world: &mut AppWorld, _value_1: String) {
    pending_step("o lançamento {string} exibe os metadados retificados");
}

/// Pendente: o lançamento {string} fica cancelado.
#[then(regex = r#"^o lançamento "([^"]*)" fica cancelado$"#)]
fn then_o_lancamento_valor_fica_cancelado(_world: &mut AppWorld, _value_1: String) {
    pending_step("o lançamento {string} fica cancelado");
}

/// Pendente: o lançamento {string} é atualizado para o valor de {string}.
#[then(regex = r#"^o lançamento "([^"]*)" é atualizado para o valor de "([^"]*)"$"#)]
fn then_o_lancamento_valor_e_atualizado_para_o_valor_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o lançamento {string} é atualizado para o valor de {string}");
}

/// Pendente: o saldo consolidado do Centro Acadêmico reflete a despesa retificada de {string}.
#[then(
    regex = r#"^o saldo consolidado do Centro Acadêmico reflete a despesa retificada de "([^"]*)"$"#
)]
fn then_o_saldo_consolidado_do_centro_academico_reflete_a_despesa_retificada_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "o saldo consolidado do Centro Acadêmico reflete a despesa retificada de {string}",
    );
}

/// Pendente: o valor do lançamento {string} permanece {string}.
#[then(regex = r#"^o valor do lançamento "([^"]*)" permanece "([^"]*)"$"#)]
fn then_o_valor_do_lancamento_valor_permanece_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o valor do lançamento {string} permanece {string}");
}

/// Pendente: seu tipo, descrição, valor e documento ficam associados ao lançamento.
#[then(regex = r#"^seu tipo, descrição, valor e documento ficam associados ao lançamento$"#)]
fn then_seu_tipo_descricao_valor_e_documento_ficam_associados_ao_lancamento(_world: &mut AppWorld) {
    pending_step("seu tipo, descrição, valor e documento ficam associados ao lançamento");
}

/// Pendente: todos os metadados como categoria, data e justificativa são exibidos com transparência.
#[then(
    regex = r#"^todos os metadados como categoria, data e justificativa são exibidos com transparência$"#
)]
fn then_todos_os_metadados_como_categoria_data_e_justificativa_sao_exibidos_com_transparencia(
    _world: &mut AppWorld,
) {
    pending_step(
        "todos os metadados como categoria, data e justificativa são exibidos com transparência",
    );
}

/// Pendente: visualiza o saldo atual consolidado de {string}.
#[then(regex = r#"^visualiza o saldo atual consolidado de "([^"]*)"$"#)]
fn then_visualiza_o_saldo_atual_consolidado_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("visualiza o saldo atual consolidado de {string}");
}

/// Pendente: visualiza o total de despesas acumuladas de {string}.
#[then(regex = r#"^visualiza o total de despesas acumuladas de "([^"]*)"$"#)]
fn then_visualiza_o_total_de_despesas_acumuladas_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("visualiza o total de despesas acumuladas de {string}");
}

/// Pendente: visualizo os lançamentos publicados com seus tipos, descrições e valores.
#[then(regex = r#"^visualizo os lançamentos publicados com seus tipos, descrições e valores$"#)]
fn then_visualizo_os_lancamentos_publicados_com_seus_tipos_descricoes_e_valores(
    _world: &mut AppWorld,
) {
    pending_step("visualizo os lançamentos publicados com seus tipos, descrições e valores");
}

/// Pendente: visualizo somente os lançamentos publicados pelo novo CA ativo.
#[then(regex = r#"^visualizo somente os lançamentos publicados pelo novo CA ativo$"#)]
fn then_visualizo_somente_os_lancamentos_publicados_pelo_novo_ca_ativo(_world: &mut AppWorld) {
    pending_step("visualizo somente os lançamentos publicados pelo novo CA ativo");
}

/// Pendente: altero o CA ativo e consulto sua prestação de contas.
#[when(regex = r#"^altero o CA ativo e consulto sua prestação de contas$"#)]
fn when_altero_o_ca_ativo_e_consulto_sua_prestacao_de_contas(_world: &mut AppWorld) {
    pending_step("altero o CA ativo e consulto sua prestação de contas");
}

/// Pendente: consulto o documento desse lançamento.
#[when(regex = r#"^consulto o documento desse lançamento$"#)]
fn when_consulto_o_documento_desse_lancamento(_world: &mut AppWorld) {
    pending_step("consulto o documento desse lançamento");
}

/// Pendente: consulto sua prestação de contas.
#[when(regex = r#"^consulto sua prestação de contas$"#)]
fn when_consulto_sua_prestacao_de_contas(_world: &mut AppWorld) {
    pending_step("consulto sua prestação de contas");
}

/// Pendente: ele cadastra um lançamento financeiro manual com os seguintes dados:.
#[when(regex = r#"^ele cadastra um lançamento financeiro manual com os seguintes dados:$"#)]
fn when_ele_cadastra_um_lancamento_financeiro_manual_com_os_seguintes_dados(_world: &mut AppWorld) {
    pending_step("ele cadastra um lançamento financeiro manual com os seguintes dados:");
}

/// Pendente: ele cancela o lançamento {string} com a justificativa {string}.
#[when(regex = r#"^ele cancela o lançamento "([^"]*)" com a justificativa "([^"]*)"$"#)]
fn when_ele_cancela_o_lancamento_valor_com_a_justificativa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele cancela o lançamento {string} com a justificativa {string}");
}

/// Pendente: ele consulta o demonstrativo financeiro do {string}.
#[when(regex = r#"^ele consulta o demonstrativo financeiro do "([^"]*)"$"#)]
fn when_ele_consulta_o_demonstrativo_financeiro_do_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("ele consulta o demonstrativo financeiro do {string}");
}

/// Pendente: ele retifica o lançamento {string} alterando o valor para {string} com a justificativa {string}.
#[when(
    regex = r#"^ele retifica o lançamento "([^"]*)" alterando o valor para "([^"]*)" com a justificativa "([^"]*)"$"#
)]
fn when_ele_retifica_o_lancamento_valor_alterando_o_valor_para_valor_com_a_justificativa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "ele retifica o lançamento {string} alterando o valor para {string} com a justificativa {string}",
    );
}

/// Pendente: ele retifica o lançamento {string} com os seguintes metadados e a justificativa {string}:.
#[when(
    regex = r#"^ele retifica o lançamento "([^"]*)" com os seguintes metadados e a justificativa "([^"]*)":$"#
)]
fn when_ele_retifica_o_lancamento_valor_com_os_seguintes_metadados_e_a_justificativa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "ele retifica o lançamento {string} com os seguintes metadados e a justificativa {string}:",
    );
}

/// Pendente: ele salva um lançamento financeiro em rascunho com os seguintes dados:.
#[when(regex = r#"^ele salva um lançamento financeiro em rascunho com os seguintes dados:$"#)]
fn when_ele_salva_um_lancamento_financeiro_em_rascunho_com_os_seguintes_dados(
    _world: &mut AppWorld,
) {
    pending_step("ele salva um lançamento financeiro em rascunho com os seguintes dados:");
}

/// Pendente: ele tenta retificar o lançamento com identificador {string} que não existe.
#[when(regex = r#"^ele tenta retificar o lançamento com identificador "([^"]*)" que não existe$"#)]
fn when_ele_tenta_retificar_o_lancamento_com_identificador_valor_que_nao_existe(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("ele tenta retificar o lançamento com identificador {string} que não existe");
}

/// Pendente: ele tenta retificar o lançamento {string} alterando o valor para {string} sem justificativa.
#[when(
    regex = r#"^ele tenta retificar o lançamento "([^"]*)" alterando o valor para "([^"]*)" sem justificativa$"#
)]
fn when_ele_tenta_retificar_o_lancamento_valor_alterando_o_valor_para_valor_sem_justificativa(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "ele tenta retificar o lançamento {string} alterando o valor para {string} sem justificativa",
    );
}

/// Pendente: ele tenta retificar o valor do lançamento {string}.
#[when(regex = r#"^ele tenta retificar o valor do lançamento "([^"]*)"$"#)]
fn when_ele_tenta_retificar_o_valor_do_lancamento_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("ele tenta retificar o valor do lançamento {string}");
}

/// Pendente: qualquer estudante consulta o demonstrativo financeiro do Centro Acadêmico.
#[when(regex = r#"^qualquer estudante consulta o demonstrativo financeiro do Centro Acadêmico$"#)]
fn when_qualquer_estudante_consulta_o_demonstrativo_financeiro_do_centro_academico(
    _world: &mut AppWorld,
) {
    pending_step("qualquer estudante consulta o demonstrativo financeiro do Centro Acadêmico");
}

/// Pendente: registro uma {string} com descrição {string}, valor {string} e documento {string}.
#[when(
    regex = r#"^registro uma "([^"]*)" com descrição "([^"]*)", valor "([^"]*)" e documento "([^"]*)"$"#
)]
fn when_registro_uma_valor_com_descricao_valor_valor_valor_e_documento_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
    _value_4: String,
) {
    pending_step(
        "registro uma {string} com descrição {string}, valor {string} e documento {string}",
    );
}

/// Pendente: tento registrar um lançamento financeiro nesse CA.
#[when(regex = r#"^tento registrar um lançamento financeiro nesse CA$"#)]
fn when_tento_registrar_um_lancamento_financeiro_nesse_ca(_world: &mut AppWorld) {
    pending_step("tento registrar um lançamento financeiro nesse CA");
}

/// Pendente: tento registrar um lançamento financeiro no CA ativo.
#[when(regex = r#"^tento registrar um lançamento financeiro no CA ativo$"#)]
fn when_tento_registrar_um_lancamento_financeiro_no_ca_ativo(_world: &mut AppWorld) {
    pending_step("tento registrar um lançamento financeiro no CA ativo");
}

/// Pendente: um estudante consulta os detalhes desse lançamento.
#[when(regex = r#"^um estudante consulta os detalhes desse lançamento$"#)]
fn when_um_estudante_consulta_os_detalhes_desse_lancamento(_world: &mut AppWorld) {
    pending_step("um estudante consulta os detalhes desse lançamento");
}
