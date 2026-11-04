//! A gestão de eventos precisa respeitar as permissões e a homologação, inclusive no ambiente físico do CA.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: existe em {string} o evento publicado {string}.
#[given(regex = r#"^existe em "([^"]*)" o evento publicado "([^"]*)"$"#)]
fn given_existe_em_valor_o_evento_publicado_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("existe em {string} o evento publicado {string}");
}

/// Pendente: o estudante {string} possui perfil no CApp e não integra a gestão de {string}.
#[given(
    regex = r#"^o estudante "([^"]*)" possui perfil no CApp e não integra a gestão de "([^"]*)"$"#
)]
fn given_o_estudante_valor_possui_perfil_no_capp_e_nao_integra_a_gestao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante {string} possui perfil no CApp e não integra a gestão de {string}");
}

/// Pendente: o local desse evento não é o ambiente físico do próprio CA.
#[given(regex = r#"^o local desse evento não é o ambiente físico do próprio CA$"#)]
fn given_o_local_desse_evento_nao_e_o_ambiente_fisico_do_proprio_ca(_world: &mut AppWorld) {
    pending_step("o local desse evento não é o ambiente físico do próprio CA");
}

/// Pendente: que a edição de um evento publicado em {string} o retirou da agenda pública.
#[given(
    regex = r#"^que a edição de um evento publicado em "([^"]*)" o retirou da agenda pública$"#
)]
fn given_que_a_edicao_de_um_evento_publicado_em_valor_o_retirou_da_agenda_publica(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que a edição de um evento publicado em {string} o retirou da agenda pública");
}

/// Pendente: que existe em {string} o evento em rascunho {string} com os seguintes dados:.
#[given(
    regex = r#"^que existe em "([^"]*)" o evento em rascunho "([^"]*)" com os seguintes dados:$"#
)]
fn given_que_existe_em_valor_o_evento_em_rascunho_valor_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe em {string} o evento em rascunho {string} com os seguintes dados:");
}

/// Pendente: que existe em {string} o evento em rascunho {string} criado por {string}.
#[given(regex = r#"^que existe em "([^"]*)" o evento em rascunho "([^"]*)" criado por "([^"]*)"$"#)]
fn given_que_existe_em_valor_o_evento_em_rascunho_valor_criado_por_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("que existe em {string} o evento em rascunho {string} criado por {string}");
}

/// Pendente: que existe em {string} o evento publicado {string} com os seguintes dados:.
#[given(
    regex = r#"^que existe em "([^"]*)" o evento publicado "([^"]*)" com os seguintes dados:$"#
)]
fn given_que_existe_em_valor_o_evento_publicado_valor_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe em {string} o evento publicado {string} com os seguintes dados:");
}

/// Pendente: que existe em {string} um evento no local {string} aguardando homologação.
#[given(regex = r#"^que existe em "([^"]*)" um evento no local "([^"]*)" aguardando homologação$"#)]
fn given_que_existe_em_valor_um_evento_no_local_valor_aguardando_homologacao(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que existe em {string} um evento no local {string} aguardando homologação");
}

/// Pendente: que existe em {string} um evento publicado em outro local.
#[given(regex = r#"^que existe em "([^"]*)" um evento publicado em outro local$"#)]
fn given_que_existe_em_valor_um_evento_publicado_em_outro_local(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que existe em {string} um evento publicado em outro local");
}

/// Pendente: que existe um evento em rascunho no local {string}.
#[given(regex = r#"^que existe um evento em rascunho no local "([^"]*)"$"#)]
fn given_que_existe_um_evento_em_rascunho_no_local_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("que existe um evento em rascunho no local {string}");
}

/// Pendente: que o gestor {string} possui publicação direta de eventos em {string}.
#[given(regex = r#"^que o gestor "([^"]*)" possui publicação direta de eventos em "([^"]*)"$"#)]
fn given_que_o_gestor_valor_possui_publicacao_direta_de_eventos_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o gestor {string} possui publicação direta de eventos em {string}");
}

/// Pendente: que o integrante possui a permissão {string} em {string}.
#[given(regex = r#"^que o integrante possui a permissão "([^"]*)" em "([^"]*)"$"#)]
fn given_que_o_integrante_possui_a_permissao_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("que o integrante possui a permissão {string} em {string}");
}

/// Pendente: a agenda de eventos publicados de {string} permanece sem o evento {string}.
#[then(regex = r#"^a agenda de eventos publicados de "([^"]*)" permanece sem o evento "([^"]*)"$"#)]
fn then_a_agenda_de_eventos_publicados_de_valor_permanece_sem_o_evento_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a agenda de eventos publicados de {string} permanece sem o evento {string}");
}

/// Pendente: a criação não publica automaticamente o evento.
#[then(regex = r#"^a criação não publica automaticamente o evento$"#)]
fn then_a_criacao_nao_publica_automaticamente_o_evento(_world: &mut AppWorld) {
    pending_step("a criação não publica automaticamente o evento");
}

/// Pendente: a data de publicação do evento fica registrada.
#[then(regex = r#"^a data de publicação do evento fica registrada$"#)]
fn then_a_data_de_publicacao_do_evento_fica_registrada(_world: &mut AppWorld) {
    pending_step("a data de publicação do evento fica registrada");
}

/// Pendente: a operação é recusada informando que o horário de término deve ser posterior ao início.
#[then(
    regex = r#"^a operação é recusada informando que o horário de término deve ser posterior ao início$"#
)]
fn then_a_operacao_e_recusada_informando_que_o_horario_de_termino_deve_ser_posterior_ao_inicio(
    _world: &mut AppWorld,
) {
    pending_step(
        "a operação é recusada informando que o horário de término deve ser posterior ao início",
    );
}

/// Pendente: a operação é recusada informando que o término deve ser posterior ao início.
#[then(regex = r#"^a operação é recusada informando que o término deve ser posterior ao início$"#)]
fn then_a_operacao_e_recusada_informando_que_o_termino_deve_ser_posterior_ao_inicio(
    _world: &mut AppWorld,
) {
    pending_step("a operação é recusada informando que o término deve ser posterior ao início");
}

/// Pendente: a operação é recusada informando que {string} do evento é obrigatório.
#[then(regex = r#"^a operação é recusada informando que "([^"]*)" do evento é obrigatório$"#)]
fn then_a_operacao_e_recusada_informando_que_valor_do_evento_e_obrigatorio(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a operação é recusada informando que {string} do evento é obrigatório");
}

/// Pendente: a opção {string} permanece identificada como seu local.
#[then(regex = r#"^a opção "([^"]*)" permanece identificada como seu local$"#)]
fn then_a_opcao_valor_permanece_identificada_como_seu_local(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a opção {string} permanece identificada como seu local");
}

/// Pendente: nenhum horário de término é inventado pelo sistema.
#[then(regex = r#"^nenhum horário de término é inventado pelo sistema$"#)]
fn then_nenhum_horario_de_termino_e_inventado_pelo_sistema(_world: &mut AppWorld) {
    pending_step("nenhum horário de término é inventado pelo sistema");
}

/// Pendente: nenhum novo evento é registrado em {string}.
#[then(regex = r#"^nenhum novo evento é registrado em "([^"]*)"$"#)]
fn then_nenhum_novo_evento_e_registrado_em_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("nenhum novo evento é registrado em {string}");
}

/// Pendente: não aparece na agenda de eventos publicados de {string}.
#[then(regex = r#"^não aparece na agenda de eventos publicados de "([^"]*)"$"#)]
fn then_nao_aparece_na_agenda_de_eventos_publicados_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("não aparece na agenda de eventos publicados de {string}");
}

/// Pendente: o aplicativo identifica que a publicação exige homologação.
#[then(regex = r#"^o aplicativo identifica que a publicação exige homologação$"#)]
fn then_o_aplicativo_identifica_que_a_publicacao_exige_homologacao(_world: &mut AppWorld) {
    pending_step("o aplicativo identifica que a publicação exige homologação");
}

/// Pendente: o evento aguarda uma ação explícita de homologação.
#[then(regex = r#"^o evento aguarda uma ação explícita de homologação$"#)]
fn then_o_evento_aguarda_uma_acao_explicita_de_homologacao(_world: &mut AppWorld) {
    pending_step("o evento aguarda uma ação explícita de homologação");
}

/// Pendente: o evento aparece na agenda pública de {string}.
#[then(regex = r#"^o evento aparece na agenda pública de "([^"]*)"$"#)]
fn then_o_evento_aparece_na_agenda_publica_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o evento aparece na agenda pública de {string}");
}

/// Pendente: o evento fica pendente de homologação em {string}.
#[then(regex = r#"^o evento fica pendente de homologação em "([^"]*)"$"#)]
fn then_o_evento_fica_pendente_de_homologacao_em_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o evento fica pendente de homologação em {string}");
}

/// Pendente: o evento fica pendente de nova homologação.
#[then(regex = r#"^o evento fica pendente de nova homologação$"#)]
fn then_o_evento_fica_pendente_de_nova_homologacao(_world: &mut AppWorld) {
    pending_step("o evento fica pendente de nova homologação");
}

/// Pendente: o evento fica visível na agenda de eventos publicados de {string} sem exigir homologação.
#[then(
    regex = r#"^o evento fica visível na agenda de eventos publicados de "([^"]*)" sem exigir homologação$"#
)]
fn then_o_evento_fica_visivel_na_agenda_de_eventos_publicados_de_valor_sem_exigir_homologacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "o evento fica visível na agenda de eventos publicados de {string} sem exigir homologação",
    );
}

/// Pendente: o evento não aparece na agenda de eventos publicados de {string}.
#[then(regex = r#"^o evento não aparece na agenda de eventos publicados de "([^"]*)"$"#)]
fn then_o_evento_nao_aparece_na_agenda_de_eventos_publicados_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o evento não aparece na agenda de eventos publicados de {string}");
}

/// Pendente: o evento não aparece na agenda pública antes da aprovação.
#[then(regex = r#"^o evento não aparece na agenda pública antes da aprovação$"#)]
fn then_o_evento_nao_aparece_na_agenda_publica_antes_da_aprovacao(_world: &mut AppWorld) {
    pending_step("o evento não aparece na agenda pública antes da aprovação");
}

/// Pendente: o evento não é publicado na agenda de {string}.
#[then(regex = r#"^o evento não é publicado na agenda de "([^"]*)"$"#)]
fn then_o_evento_nao_e_publicado_na_agenda_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o evento não é publicado na agenda de {string}");
}

/// Pendente: o evento passa a apresentar em {string} os dados atualizados:.
#[then(regex = r#"^o evento passa a apresentar em "([^"]*)" os dados atualizados:$"#)]
fn then_o_evento_passa_a_apresentar_em_valor_os_dados_atualizados(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o evento passa a apresentar em {string} os dados atualizados:");
}

/// Pendente: o evento passa a ficar pendente de homologação.
#[then(regex = r#"^o evento passa a ficar pendente de homologação$"#)]
fn then_o_evento_passa_a_ficar_pendente_de_homologacao(_world: &mut AppWorld) {
    pending_step("o evento passa a ficar pendente de homologação");
}

/// Pendente: o evento passa a ficar pendente de nova homologação em {string}.
#[then(regex = r#"^o evento passa a ficar pendente de nova homologação em "([^"]*)"$"#)]
fn then_o_evento_passa_a_ficar_pendente_de_nova_homologacao_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o evento passa a ficar pendente de nova homologação em {string}");
}

/// Pendente: o evento passa a figurar na agenda de eventos publicados de {string}.
#[then(regex = r#"^o evento passa a figurar na agenda de eventos publicados de "([^"]*)"$"#)]
fn then_o_evento_passa_a_figurar_na_agenda_de_eventos_publicados_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o evento passa a figurar na agenda de eventos publicados de {string}");
}

/// Pendente: o evento registra {string} como autor responsável em {string}.
#[then(regex = r#"^o evento registra "([^"]*)" como autor responsável em "([^"]*)"$"#)]
fn then_o_evento_registra_valor_como_autor_responsavel_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento registra {string} como autor responsável em {string}");
}

/// Pendente: o evento retorna à agenda pública com os dados homologados.
#[then(regex = r#"^o evento retorna à agenda pública com os dados homologados$"#)]
fn then_o_evento_retorna_a_agenda_publica_com_os_dados_homologados(_world: &mut AppWorld) {
    pending_step("o evento retorna à agenda pública com os dados homologados");
}

/// Pendente: o evento {string} fica registrado como rascunho em {string}.
#[then(regex = r#"^o evento "([^"]*)" fica registrado como rascunho em "([^"]*)"$"#)]
fn then_o_evento_valor_fica_registrado_como_rascunho_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} fica registrado como rascunho em {string}");
}

/// Pendente: o evento {string} fica visível na agenda de eventos publicados de {string}.
#[then(regex = r#"^o evento "([^"]*)" fica visível na agenda de eventos publicados de "([^"]*)"$"#)]
fn then_o_evento_valor_fica_visivel_na_agenda_de_eventos_publicados_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} fica visível na agenda de eventos publicados de {string}");
}

/// Pendente: o evento {string} mantém a situação de rascunho em {string}.
#[then(regex = r#"^o evento "([^"]*)" mantém a situação de rascunho em "([^"]*)"$"#)]
fn then_o_evento_valor_mantem_a_situacao_de_rascunho_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} mantém a situação de rascunho em {string}");
}

/// Pendente: o evento {string} não é publicado na agenda de {string}.
#[then(regex = r#"^o evento "([^"]*)" não é publicado na agenda de "([^"]*)"$"#)]
fn then_o_evento_valor_nao_e_publicado_na_agenda_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} não é publicado na agenda de {string}");
}

/// Pendente: o evento {string} passa a estar publicado em {string}.
#[then(regex = r#"^o evento "([^"]*)" passa a estar publicado em "([^"]*)"$"#)]
fn then_o_evento_valor_passa_a_estar_publicado_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} passa a estar publicado em {string}");
}

/// Pendente: o evento {string} permanece como rascunho em {string}.
#[then(regex = r#"^o evento "([^"]*)" permanece como rascunho em "([^"]*)"$"#)]
fn then_o_evento_valor_permanece_como_rascunho_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} permanece como rascunho em {string}");
}

/// Pendente: o evento {string} permanece inalterado em {string}.
#[then(regex = r#"^o evento "([^"]*)" permanece inalterado em "([^"]*)"$"#)]
fn then_o_evento_valor_permanece_inalterado_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o evento {string} permanece inalterado em {string}");
}

/// Pendente: o evento é publicado na agenda de {string}.
#[then(regex = r#"^o evento é publicado na agenda de "([^"]*)"$"#)]
fn then_o_evento_e_publicado_na_agenda_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o evento é publicado na agenda de {string}");
}

/// Pendente: o evento é registrado sem horário de término.
#[then(regex = r#"^o evento é registrado sem horário de término$"#)]
fn then_o_evento_e_registrado_sem_horario_de_termino(_world: &mut AppWorld) {
    pending_step("o evento é registrado sem horário de término");
}

/// Pendente: o evento é retirado da agenda pública enquanto aguarda a homologação.
#[then(regex = r#"^o evento é retirado da agenda pública enquanto aguarda a homologação$"#)]
fn then_o_evento_e_retirado_da_agenda_publica_enquanto_aguarda_a_homologacao(
    _world: &mut AppWorld,
) {
    pending_step("o evento é retirado da agenda pública enquanto aguarda a homologação");
}

/// Pendente: o local do evento fica associado ao espaço do CA {string}.
#[then(regex = r#"^o local do evento fica associado ao espaço do CA "([^"]*)"$"#)]
fn then_o_local_do_evento_fica_associado_ao_espaco_do_ca_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o local do evento fica associado ao espaço do CA {string}");
}

/// Pendente: o proponente pode identificar que a publicação foi rejeitada.
#[then(regex = r#"^o proponente pode identificar que a publicação foi rejeitada$"#)]
fn then_o_proponente_pode_identificar_que_a_publicacao_foi_rejeitada(_world: &mut AppWorld) {
    pending_step("o proponente pode identificar que a publicação foi rejeitada");
}

/// Pendente: permanece fora da agenda de eventos publicados de {string}.
#[then(regex = r#"^permanece fora da agenda de eventos publicados de "([^"]*)"$"#)]
fn then_permanece_fora_da_agenda_de_eventos_publicados_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("permanece fora da agenda de eventos publicados de {string}");
}

/// Pendente: permanece fora da agenda pública.
#[then(regex = r#"^permanece fora da agenda pública$"#)]
fn then_permanece_fora_da_agenda_publica(_world: &mut AppWorld) {
    pending_step("permanece fora da agenda pública");
}

/// Pendente: sai da agenda pública até que a alteração seja aprovada.
#[then(regex = r#"^sai da agenda pública até que a alteração seja aprovada$"#)]
fn then_sai_da_agenda_publica_ate_que_a_alteracao_seja_aprovada(_world: &mut AppWorld) {
    pending_step("sai da agenda pública até que a alteração seja aprovada");
}

/// Pendente: aprova a alteração do evento.
#[when(regex = r#"^aprova a alteração do evento$"#)]
fn when_aprova_a_alteracao_do_evento(_world: &mut AppWorld) {
    pending_step("aprova a alteração do evento");
}

/// Pendente: aprova a publicação do evento.
#[when(regex = r#"^aprova a publicação do evento$"#)]
fn when_aprova_a_publicacao_do_evento(_world: &mut AppWorld) {
    pending_step("aprova a publicação do evento");
}

/// Pendente: cria e publica diretamente um evento válido em um local diferente do ambiente físico do próprio CA.
#[when(
    regex = r#"^cria e publica diretamente um evento válido em um local diferente do ambiente físico do próprio CA$"#
)]
fn when_cria_e_publica_diretamente_um_evento_valido_em_um_local_diferente_do_ambiente_fisico_do_proprio_ca(
    _world: &mut AppWorld,
) {
    pending_step(
        "cria e publica diretamente um evento válido em um local diferente do ambiente físico do próprio CA",
    );
}

/// Pendente: cria um evento válido com o local {string} e solicita a publicação.
#[when(regex = r#"^cria um evento válido com o local "([^"]*)" e solicita a publicação$"#)]
fn when_cria_um_evento_valido_com_o_local_valor_e_solicita_a_publicacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("cria um evento válido com o local {string} e solicita a publicação");
}

/// Pendente: cria um evento válido com o local {string} e solicita a publicação direta.
#[when(regex = r#"^cria um evento válido com o local "([^"]*)" e solicita a publicação direta$"#)]
fn when_cria_um_evento_valido_com_o_local_valor_e_solicita_a_publicacao_direta(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("cria um evento válido com o local {string} e solicita a publicação direta");
}

/// Pendente: o estudante {string} tenta criar e publicar diretamente em {string} um evento com os seguintes dados:.
#[when(
    regex = r#"^o estudante "([^"]*)" tenta criar e publicar diretamente em "([^"]*)" um evento com os seguintes dados:$"#
)]
fn when_o_estudante_valor_tenta_criar_e_publicar_diretamente_em_valor_um_evento_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o estudante {string} tenta criar e publicar diretamente em {string} um evento com os seguintes dados:",
    );
}

/// Pendente: o gestor com publicação direta solicita a publicação desse rascunho em {string}.
#[when(
    regex = r#"^o gestor com publicação direta solicita a publicação desse rascunho em "([^"]*)"$"#
)]
fn when_o_gestor_com_publicacao_direta_solicita_a_publicacao_desse_rascunho_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o gestor com publicação direta solicita a publicação desse rascunho em {string}");
}

/// Pendente: o gestor {string} altera o local para {string}.
#[when(regex = r#"^o gestor "([^"]*)" altera o local para "([^"]*)"$"#)]
fn when_o_gestor_valor_altera_o_local_para_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o gestor {string} altera o local para {string}");
}

/// Pendente: o gestor {string} cria como rascunho em {string} um evento com os seguintes dados:.
#[when(
    regex = r#"^o gestor "([^"]*)" cria como rascunho em "([^"]*)" um evento com os seguintes dados:$"#
)]
fn when_o_gestor_valor_cria_como_rascunho_em_valor_um_evento_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o gestor {string} cria como rascunho em {string} um evento com os seguintes dados:",
    );
}

/// Pendente: o gestor {string} cria e publica diretamente em {string} um evento com os seguintes dados:.
#[when(
    regex = r#"^o gestor "([^"]*)" cria e publica diretamente em "([^"]*)" um evento com os seguintes dados:$"#
)]
fn when_o_gestor_valor_cria_e_publica_diretamente_em_valor_um_evento_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o gestor {string} cria e publica diretamente em {string} um evento com os seguintes dados:",
    );
}

/// Pendente: o gestor {string} cria em {string} um evento com título, descrição, início e local válidos sem informar o término.
#[when(
    regex = r#"^o gestor "([^"]*)" cria em "([^"]*)" um evento com título, descrição, início e local válidos sem informar o término$"#
)]
fn when_o_gestor_valor_cria_em_valor_um_evento_com_titulo_descricao_inicio_e_local_validos_sem_informar_o_te(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o gestor {string} cria em {string} um evento com título, descrição, início e local válidos sem informar o término",
    );
}

/// Pendente: o gestor {string} edita o evento {string} em {string} para:.
#[when(regex = r#"^o gestor "([^"]*)" edita o evento "([^"]*)" em "([^"]*)" para:$"#)]
fn when_o_gestor_valor_edita_o_evento_valor_em_valor_para(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} edita o evento {string} em {string} para:");
}

/// Pendente: o gestor {string} publica diretamente o evento {string} em {string}.
#[when(regex = r#"^o gestor "([^"]*)" publica diretamente o evento "([^"]*)" em "([^"]*)"$"#)]
fn when_o_gestor_valor_publica_diretamente_o_evento_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} publica diretamente o evento {string} em {string}");
}

/// Pendente: o gestor {string} seleciona {string} como local de um evento válido.
#[when(regex = r#"^o gestor "([^"]*)" seleciona "([^"]*)" como local de um evento válido$"#)]
fn when_o_gestor_valor_seleciona_valor_como_local_de_um_evento_valido(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o gestor {string} seleciona {string} como local de um evento válido");
}

/// Pendente: o gestor {string} tenta criar em {string} um evento com início e término em {string}.
#[when(
    regex = r#"^o gestor "([^"]*)" tenta criar em "([^"]*)" um evento com início e término em "([^"]*)"$"#
)]
fn when_o_gestor_valor_tenta_criar_em_valor_um_evento_com_inicio_e_termino_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "o gestor {string} tenta criar em {string} um evento com início e término em {string}",
    );
}

/// Pendente: o gestor {string} tenta criar em {string} um evento com os seguintes dados:.
#[when(
    regex = r#"^o gestor "([^"]*)" tenta criar em "([^"]*)" um evento com os seguintes dados:$"#
)]
fn when_o_gestor_valor_tenta_criar_em_valor_um_evento_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o gestor {string} tenta criar em {string} um evento com os seguintes dados:");
}

/// Pendente: o gestor {string} tenta editar o evento {string} em {string}.
#[when(regex = r#"^o gestor "([^"]*)" tenta editar o evento "([^"]*)" em "([^"]*)"$"#)]
fn when_o_gestor_valor_tenta_editar_o_evento_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} tenta editar o evento {string} em {string}");
}

/// Pendente: o gestor {string} tenta editar o evento {string} em {string} para:.
#[when(regex = r#"^o gestor "([^"]*)" tenta editar o evento "([^"]*)" em "([^"]*)" para:$"#)]
fn when_o_gestor_valor_tenta_editar_o_evento_valor_em_valor_para(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("o gestor {string} tenta editar o evento {string} em {string} para:");
}

/// Pendente: rejeita a publicação do evento.
#[when(regex = r#"^rejeita a publicação do evento$"#)]
fn when_rejeita_a_publicacao_do_evento(_world: &mut AppWorld) {
    pending_step("rejeita a publicação do evento");
}

/// Pendente: {string} tenta criar e publicar diretamente em {string} um evento com os seguintes dados:.
#[when(
    regex = r#"^"([^"]*)" tenta criar e publicar diretamente em "([^"]*)" um evento com os seguintes dados:$"#
)]
fn when_valor_tenta_criar_e_publicar_diretamente_em_valor_um_evento_com_os_seguintes_dados(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "{string} tenta criar e publicar diretamente em {string} um evento com os seguintes dados:",
    );
}

/// Pendente: {string} tenta editar o evento {string} em {string} para:.
#[when(regex = r#"^"([^"]*)" tenta editar o evento "([^"]*)" em "([^"]*)" para:$"#)]
fn when_valor_tenta_editar_o_evento_valor_em_valor_para(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} tenta editar o evento {string} em {string} para:");
}

/// Pendente: {string} tenta publicar diretamente o evento {string} em {string}.
#[when(regex = r#"^"([^"]*)" tenta publicar diretamente o evento "([^"]*)" em "([^"]*)"$"#)]
fn when_valor_tenta_publicar_diretamente_o_evento_valor_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step("{string} tenta publicar diretamente o evento {string} em {string}");
}
