//! A votação precisa assegurar elegibilidade, participação única e as garantias da modalidade eleitoral.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: a eleição está no período de votação.
#[given(regex = r#"^a eleição está no período de votação$"#)]
fn given_a_eleicao_esta_no_periodo_de_votacao(_world: &mut AppWorld) {
    pending_step("a eleição está no período de votação");
}

/// Pendente: a eleição possui as chapas ativas {string} e {string}.
#[given(regex = r#"^a eleição possui as chapas ativas "([^"]*)" e "([^"]*)"$"#)]
fn given_a_eleicao_possui_as_chapas_ativas_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a eleição possui as chapas ativas {string} e {string}");
}

/// Pendente: as solicitações válidas de voto de {string} são assinadas com sua própria chave.
#[given(
    regex = r#"^as solicitações válidas de voto de "([^"]*)" são assinadas com sua própria chave$"#
)]
fn given_as_solicitacoes_validas_de_voto_de_valor_sao_assinadas_com_sua_propria_chave(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("as solicitações válidas de voto de {string} são assinadas com sua própria chave");
}

/// Pendente: diferentes eleitores já registraram suas cédulas.
#[given(regex = r#"^diferentes eleitores já registraram suas cédulas$"#)]
fn given_diferentes_eleitores_ja_registraram_suas_cedulas(_world: &mut AppWorld) {
    pending_step("diferentes eleitores já registraram suas cédulas");
}

/// Pendente: existe uma chapa inativa chamada {string}.
#[given(regex = r#"^existe uma chapa inativa chamada "([^"]*)"$"#)]
fn given_existe_uma_chapa_inativa_chamada_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("existe uma chapa inativa chamada {string}");
}

/// Pendente: o estudante {string} ainda não votou nessa eleição.
#[given(regex = r#"^o estudante "([^"]*)" ainda não votou nessa eleição$"#)]
fn given_o_estudante_valor_ainda_nao_votou_nessa_eleicao(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante {string} ainda não votou nessa eleição");
}

/// Pendente: o estudante {string} já participou dessa eleição.
#[given(regex = r#"^o estudante "([^"]*)" já participou dessa eleição$"#)]
fn given_o_estudante_valor_ja_participou_dessa_eleicao(_world: &mut AppWorld, _value_1: String) {
    pending_step("o estudante {string} já participou dessa eleição");
}

/// Pendente: o estudante {string} já registrou um voto na chapa {string}.
#[given(regex = r#"^o estudante "([^"]*)" já registrou um voto na chapa "([^"]*)"$"#)]
fn given_o_estudante_valor_ja_registrou_um_voto_na_chapa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o estudante {string} já registrou um voto na chapa {string}");
}

/// Pendente: o estudante {string} não possui vínculo acadêmico válido com o Centro Acadêmico {string}.
#[given(
    regex = r#"^o estudante "([^"]*)" não possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$"#
)]
fn given_o_estudante_valor_nao_possui_vinculo_academico_valido_com_o_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o estudante {string} não possui vínculo acadêmico válido com o Centro Acadêmico {string}",
    );
}

/// Pendente: o estudante {string} possui vínculo acadêmico válido com o Centro Acadêmico {string}.
#[given(
    regex = r#"^o estudante "([^"]*)" possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$"#
)]
fn given_o_estudante_valor_possui_vinculo_academico_valido_com_o_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o estudante {string} possui vínculo acadêmico válido com o Centro Acadêmico {string}",
    );
}

/// Pendente: o período de votação ainda não começou.
#[given(regex = r#"^o período de votação ainda não começou$"#)]
fn given_o_periodo_de_votacao_ainda_nao_comecou(_world: &mut AppWorld) {
    pending_step("o período de votação ainda não começou");
}

/// Pendente: o período de votação foi encerrado.
#[given(regex = r#"^o período de votação foi encerrado$"#)]
fn given_o_periodo_de_votacao_foi_encerrado(_world: &mut AppWorld) {
    pending_step("o período de votação foi encerrado");
}

/// Pendente: o quadro público contém a cédula de {string} em formato cifrado.
#[given(regex = r#"^o quadro público contém a cédula de "([^"]*)" em formato cifrado$"#)]
fn given_o_quadro_publico_contem_a_cedula_de_valor_em_formato_cifrado(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o quadro público contém a cédula de {string} em formato cifrado");
}

/// Pendente: que a eleição está configurada para votação identificada.
#[given(regex = r#"^que a eleição está configurada para votação identificada$"#)]
fn given_que_a_eleicao_esta_configurada_para_votacao_identificada(_world: &mut AppWorld) {
    pending_step("que a eleição está configurada para votação identificada");
}

/// Pendente: que a eleição está configurada para votação secreta.
#[given(regex = r#"^que a eleição está configurada para votação secreta$"#)]
fn given_que_a_eleicao_esta_configurada_para_votacao_secreta(_world: &mut AppWorld) {
    pending_step("que a eleição está configurada para votação secreta");
}

/// Pendente: que a eleição está no período de votação.
#[given(regex = r#"^que a eleição está no período de votação$"#)]
fn given_que_a_eleicao_esta_no_periodo_de_votacao(_world: &mut AppWorld) {
    pending_step("que a eleição está no período de votação");
}

/// Pendente: que existe uma eleição para o Centro Acadêmico {string}.
#[given(regex = r#"^que existe uma eleição para o Centro Acadêmico "([^"]*)"$"#)]
fn given_que_existe_uma_eleicao_para_o_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que existe uma eleição para o Centro Acadêmico {string}");
}

/// Pendente: que o estudante {string} possui vínculo acadêmico válido com o Centro Acadêmico {string}.
#[given(
    regex = r#"^que o estudante "([^"]*)" possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$"#
)]
fn given_que_o_estudante_valor_possui_vinculo_academico_valido_com_o_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "que o estudante {string} possui vínculo acadêmico válido com o Centro Acadêmico {string}",
    );
}

/// Pendente: {string} já registrou sua participação nessa eleição por votação secreta.
#[given(regex = r#"^"([^"]*)" já registrou sua participação nessa eleição por votação secreta$"#)]
fn given_valor_ja_registrou_sua_participacao_nessa_eleicao_por_votacao_secreta(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} já registrou sua participação nessa eleição por votação secreta");
}

/// Pendente: a chapa inativa {string} não é apresentada como opção.
#[then(regex = r#"^a chapa inativa "([^"]*)" não é apresentada como opção$"#)]
fn then_a_chapa_inativa_valor_nao_e_apresentada_como_opcao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a chapa inativa {string} não é apresentada como opção");
}

/// Pendente: a confirmação não revela a chapa escolhida.
#[then(regex = r#"^a confirmação não revela a chapa escolhida$"#)]
fn then_a_confirmacao_nao_revela_a_chapa_escolhida(_world: &mut AppWorld) {
    pending_step("a confirmação não revela a chapa escolhida");
}

/// Pendente: a cédula cifrada não revela a escolha em texto claro nos dados públicos.
#[then(regex = r#"^a cédula cifrada não revela a escolha em texto claro nos dados públicos$"#)]
fn then_a_cedula_cifrada_nao_revela_a_escolha_em_texto_claro_nos_dados_publicos(
    _world: &mut AppWorld,
) {
    pending_step("a cédula cifrada não revela a escolha em texto claro nos dados públicos");
}

/// Pendente: a cédula é aceita e o voto é confirmado.
#[then(regex = r#"^a cédula é aceita e o voto é confirmado$"#)]
fn then_a_cedula_e_aceita_e_o_voto_e_confirmado(_world: &mut AppWorld) {
    pending_step("a cédula é aceita e o voto é confirmado");
}

/// Pendente: a cédula é recusada.
#[then(regex = r#"^a cédula é recusada$"#)]
fn then_a_cedula_e_recusada(_world: &mut AppWorld) {
    pending_step("a cédula é recusada");
}

/// Pendente: a outra tentativa é recusada por participação já realizada.
#[then(regex = r#"^a outra tentativa é recusada por participação já realizada$"#)]
fn then_a_outra_tentativa_e_recusada_por_participacao_ja_realizada(_world: &mut AppWorld) {
    pending_step("a outra tentativa é recusada por participação já realizada");
}

/// Pendente: a participação de {string} continua limitada a um único voto nessa eleição.
#[then(regex = r#"^a participação de "([^"]*)" continua limitada a um único voto nessa eleição$"#)]
fn then_a_participacao_de_valor_continua_limitada_a_um_unico_voto_nessa_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a participação de {string} continua limitada a um único voto nessa eleição");
}

/// Pendente: a participação de {string} é marcada como realizada nessa eleição.
#[then(regex = r#"^a participação de "([^"]*)" é marcada como realizada nessa eleição$"#)]
fn then_a_participacao_de_valor_e_marcada_como_realizada_nessa_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a participação de {string} é marcada como realizada nessa eleição");
}

/// Pendente: a tentativa de voto é recusada.
#[then(regex = r#"^a tentativa de voto é recusada$"#)]
fn then_a_tentativa_de_voto_e_recusada(_world: &mut AppWorld) {
    pending_step("a tentativa de voto é recusada");
}

/// Pendente: a tentativa de voto é recusada por falta de elegibilidade.
#[then(regex = r#"^a tentativa de voto é recusada por falta de elegibilidade$"#)]
fn then_a_tentativa_de_voto_e_recusada_por_falta_de_elegibilidade(_world: &mut AppWorld) {
    pending_step("a tentativa de voto é recusada por falta de elegibilidade");
}

/// Pendente: a tentativa de voto é recusada porque a votação ainda não começou.
#[then(regex = r#"^a tentativa de voto é recusada porque a votação ainda não começou$"#)]
fn then_a_tentativa_de_voto_e_recusada_porque_a_votacao_ainda_nao_comecou(_world: &mut AppWorld) {
    pending_step("a tentativa de voto é recusada porque a votação ainda não começou");
}

/// Pendente: a tentativa de voto é recusada porque a votação foi encerrada.
#[then(regex = r#"^a tentativa de voto é recusada porque a votação foi encerrada$"#)]
fn then_a_tentativa_de_voto_e_recusada_porque_a_votacao_foi_encerrada(_world: &mut AppWorld) {
    pending_step("a tentativa de voto é recusada porque a votação foi encerrada");
}

/// Pendente: apenas um voto fica registrado para {string} nessa eleição.
#[then(regex = r#"^apenas um voto fica registrado para "([^"]*)" nessa eleição$"#)]
fn then_apenas_um_voto_fica_registrado_para_valor_nessa_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("apenas um voto fica registrado para {string} nessa eleição");
}

/// Pendente: apenas um voto é contabilizado para essa participação.
#[then(regex = r#"^apenas um voto é contabilizado para essa participação$"#)]
fn then_apenas_um_voto_e_contabilizado_para_essa_participacao(_world: &mut AppWorld) {
    pending_step("apenas um voto é contabilizado para essa participação");
}

/// Pendente: apenas uma das tentativas é aceita.
#[then(regex = r#"^apenas uma das tentativas é aceita$"#)]
fn then_apenas_uma_das_tentativas_e_aceita(_world: &mut AppWorld) {
    pending_step("apenas uma das tentativas é aceita");
}

/// Pendente: nenhum voto é registrado para o estudante {string}.
#[then(regex = r#"^nenhum voto é registrado para o estudante "([^"]*)"$"#)]
fn then_nenhum_voto_e_registrado_para_o_estudante_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("nenhum voto é registrado para o estudante {string}");
}

/// Pendente: nenhum voto é registrado para {string}.
#[then(regex = r#"^nenhum voto é registrado para "([^"]*)"$"#)]
fn then_nenhum_voto_e_registrado_para_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("nenhum voto é registrado para {string}");
}

/// Pendente: o registro identifica {string} como eleitora e {string} como sua escolha.
#[then(regex = r#"^o registro identifica "([^"]*)" como eleitora e "([^"]*)" como sua escolha$"#)]
fn then_o_registro_identifica_valor_como_eleitora_e_valor_como_sua_escolha(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("o registro identifica {string} como eleitora e {string} como sua escolha");
}

/// Pendente: o registro público contém a identificação do eleitor, a cédula cifrada e a assinatura do eleitor.
#[then(
    regex = r#"^o registro público contém a identificação do eleitor, a cédula cifrada e a assinatura do eleitor$"#
)]
fn then_o_registro_publico_contem_a_identificacao_do_eleitor_a_cedula_cifrada_e_a_assinatura_do_eleitor(
    _world: &mut AppWorld,
) {
    pending_step(
        "o registro público contém a identificação do eleitor, a cédula cifrada e a assinatura do eleitor",
    );
}

/// Pendente: o servidor fornece um recibo assinado da cédula recebida.
#[then(regex = r#"^o servidor fornece um recibo assinado da cédula recebida$"#)]
fn then_o_servidor_fornece_um_recibo_assinado_da_cedula_recebida(_world: &mut AppWorld) {
    pending_step("o servidor fornece um recibo assinado da cédula recebida");
}

/// Pendente: o voto original na chapa {string} permanece como o único voto de {string} nessa eleição.
#[then(
    regex = r#"^o voto original na chapa "([^"]*)" permanece como o único voto de "([^"]*)" nessa eleição$"#
)]
fn then_o_voto_original_na_chapa_valor_permanece_como_o_unico_voto_de_valor_nessa_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o voto original na chapa {string} permanece como o único voto de {string} nessa eleição",
    );
}

/// Pendente: o voto é confirmado.
#[then(regex = r#"^o voto é confirmado$"#)]
fn then_o_voto_e_confirmado(_world: &mut AppWorld) {
    pending_step("o voto é confirmado");
}

/// Pendente: são apresentadas as chapas ativas {string} e {string}.
#[then(regex = r#"^são apresentadas as chapas ativas "([^"]*)" e "([^"]*)"$"#)]
fn then_sao_apresentadas_as_chapas_ativas_valor_e_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("são apresentadas as chapas ativas {string} e {string}");
}

/// Pendente: {string} consegue verificar que sua cédula cifrada consta no quadro público.
#[then(regex = r#"^"([^"]*)" consegue verificar que sua cédula cifrada consta no quadro público$"#)]
fn then_valor_consegue_verificar_que_sua_cedula_cifrada_consta_no_quadro_publico(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} consegue verificar que sua cédula cifrada consta no quadro público");
}

/// Pendente: consulta as opções de voto.
#[when(regex = r#"^consulta as opções de voto$"#)]
fn when_consulta_as_opcoes_de_voto(_world: &mut AppWorld) {
    pending_step("consulta as opções de voto");
}

/// Pendente: registra um voto na chapa {string}.
#[when(regex = r#"^registra um voto na chapa "([^"]*)"$"#)]
fn when_registra_um_voto_na_chapa_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("registra um voto na chapa {string}");
}

/// Pendente: registra um voto secreto.
#[when(regex = r#"^registra um voto secreto$"#)]
fn when_registra_um_voto_secreto(_world: &mut AppWorld) {
    pending_step("registra um voto secreto");
}

/// Pendente: submete uma cédula sem assinatura digital válida.
#[when(regex = r#"^submete uma cédula sem assinatura digital válida$"#)]
fn when_submete_uma_cedula_sem_assinatura_digital_valida(_world: &mut AppWorld) {
    pending_step("submete uma cédula sem assinatura digital válida");
}

/// Pendente: são realizadas simultaneamente duas tentativas de voto, uma na chapa {string} e outra na chapa {string}.
#[when(
    regex = r#"^são realizadas simultaneamente duas tentativas de voto, uma na chapa "([^"]*)" e outra na chapa "([^"]*)"$"#
)]
fn when_sao_realizadas_simultaneamente_duas_tentativas_de_voto_uma_na_chapa_valor_e_outra_na_chapa_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "são realizadas simultaneamente duas tentativas de voto, uma na chapa {string} e outra na chapa {string}",
    );
}

/// Pendente: tenta registrar outro voto na chapa {string}.
#[when(regex = r#"^tenta registrar outro voto na chapa "([^"]*)"$"#)]
fn when_tenta_registrar_outro_voto_na_chapa_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("tenta registrar outro voto na chapa {string}");
}

/// Pendente: tenta registrar um voto na chapa {string}.
#[when(regex = r#"^tenta registrar um voto na chapa "([^"]*)"$"#)]
fn when_tenta_registrar_um_voto_na_chapa_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("tenta registrar um voto na chapa {string}");
}

/// Pendente: uma pessoa consulta os dados públicos da eleição.
#[when(regex = r#"^uma pessoa consulta os dados públicos da eleição$"#)]
fn when_uma_pessoa_consulta_os_dados_publicos_da_eleicao(_world: &mut AppWorld) {
    pending_step("uma pessoa consulta os dados públicos da eleição");
}

/// Pendente: uma pessoa observa a ordem e o momento de submissão das cédulas.
#[when(regex = r#"^uma pessoa observa a ordem e o momento de submissão das cédulas$"#)]
fn when_uma_pessoa_observa_a_ordem_e_o_momento_de_submissao_das_cedulas(_world: &mut AppWorld) {
    pending_step("uma pessoa observa a ordem e o momento de submissão das cédulas");
}

/// Pendente: "{string}" está autenticada pela sua conta Firebase.
#[given(regex = r#"^"([^"]*)" está autenticada pela sua conta Firebase$"#)]
fn given_valor_esta_autenticada_pela_sua_conta_firebase(_world: &mut AppWorld, _value_1: String) {
    pending_step("\"{string}\" está autenticada pela sua conta Firebase");
}

/// Pendente: "{string}" não possui inscrição ou chave prévia nessa eleição.
#[given(regex = r#"^"([^"]*)" não possui inscrição ou chave prévia nessa eleição$"#)]
fn given_valor_nao_possui_inscricao_ou_chave_previa_nessa_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("\"{string}\" não possui inscrição ou chave prévia nessa eleição");
}

/// Pendente: registra seu primeiro voto na chapa "{string}".
#[when(regex = r#"^registra seu primeiro voto na chapa "([^"]*)"$"#)]
fn when_registra_seu_primeiro_voto_na_chapa_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("registra seu primeiro voto na chapa \"{string}\"");
}

/// Pendente: a chave pública enviada na cédula é vinculada à conta autenticada de "{string}".
#[then(
    regex = r#"^a chave pública enviada na cédula é vinculada à conta autenticada de "([^"]*)"$"#
)]
fn then_a_chave_publica_enviada_na_cedula_e_vinculada_a_conta_autenticada_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "a chave pública enviada na cédula é vinculada à conta autenticada de \"{string}\"",
    );
}

/// Pendente: a chave privada permanece no dispositivo de "{string}".
#[then(regex = r#"^a chave privada permanece no dispositivo de "([^"]*)"$"#)]
fn then_a_chave_privada_permanece_no_dispositivo_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("a chave privada permanece no dispositivo de \"{string}\"");
}

/// Pendente: uma nova verificação institucional de "{string}" informa outro curso.
#[given(regex = r#"^uma nova verificação institucional de "([^"]*)" informa outro curso$"#)]
fn given_uma_nova_verificacao_institucional_de_valor_informa_outro_curso(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("uma nova verificação institucional de \"{string}\" informa outro curso");
}

/// Pendente: o estudante "{string}" está autenticado e atende aos critérios de elegibilidade.
#[given(
    regex = r#"^o estudante "([^"]*)" está autenticado e atende aos critérios de elegibilidade$"#
)]
fn given_o_estudante_valor_esta_autenticado_e_atende_aos_criterios_de_elegibilidade(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "o estudante \"{string}\" está autenticado e atende aos critérios de elegibilidade",
    );
}

/// Pendente: "{string}" está autenticada e atende aos critérios de elegibilidade.
#[given(regex = r#"^"([^"]*)" está autenticada e atende aos critérios de elegibilidade$"#)]
fn given_valor_esta_autenticada_e_atende_aos_criterios_de_elegibilidade(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("\"{string}\" está autenticada e atende aos critérios de elegibilidade");
}

/// Pendente: "{string}" envia uma cédula com o identificador eleitoral de "{string}".
#[when(regex = r#"^"([^"]*)" envia uma cédula com o identificador eleitoral de "([^"]*)"$"#)]
fn when_valor_envia_uma_cedula_com_o_identificador_eleitoral_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("\"{string}\" envia uma cédula com o identificador eleitoral de \"{string}\"");
}

/// Pendente: a cédula é recusada porque seu eleitor diverge da conta autenticada.
#[then(regex = r#"^a cédula é recusada porque seu eleitor diverge da conta autenticada$"#)]
fn then_a_cedula_e_recusada_porque_seu_eleitor_diverge_da_conta_autenticada(_world: &mut AppWorld) {
    pending_step("a cédula é recusada porque seu eleitor diverge da conta autenticada");
}

/// Pendente: submete uma cédula assinada com sua chave local, indicando a chapa "{string}".
#[when(regex = r#"^submete uma cédula assinada com sua chave local, indicando a chapa "([^"]*)"$"#)]
fn when_submete_uma_cedula_assinada_com_sua_chave_local_indicando_a_chapa_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("submete uma cédula assinada com sua chave local, indicando a chapa \"{string}\"");
}

/// Pendente: a assinatura digital é validada com a chave pública enviada na cédula de "{string}".
#[then(
    regex = r#"^a assinatura digital é validada com a chave pública enviada na cédula de "([^"]*)"$"#
)]
fn then_a_assinatura_digital_e_validada_com_a_chave_publica_enviada_na_cedula_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "a assinatura digital é validada com a chave pública enviada na cédula de \"{string}\"",
    );
}

/// Pendente: a publicação não expõe escolha em claro nem aleatoriedade da cifragem.
#[then(regex = r#"^a publicação não expõe escolha em claro nem aleatoriedade da cifragem$"#)]
fn then_a_publicacao_nao_expoe_escolha_em_claro_nem_aleatoriedade_da_cifragem(
    _world: &mut AppWorld,
) {
    pending_step("a publicação não expõe escolha em claro nem aleatoriedade da cifragem");
}

/// Pendente: os dados não publicam a escolha em claro de "{string}".
#[then(regex = r#"^os dados não publicam a escolha em claro de "([^"]*)"$"#)]
fn then_os_dados_nao_publicam_a_escolha_em_claro_de_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("os dados não publicam a escolha em claro de \"{string}\"");
}

/// Pendente: "{string}" já participou dessa eleição.
#[given(regex = r#"^"([^"]*)" já participou dessa eleição$"#)]
fn given_valor_ja_participou_dessa_eleicao(_world: &mut AppWorld, _value_1: String) {
    pending_step("\"{string}\" já participou dessa eleição");
}

/// Pendente: "{string}" tenta votar em outro dispositivo com um novo par de chaves.
#[when(regex = r#"^"([^"]*)" tenta votar em outro dispositivo com um novo par de chaves$"#)]
fn when_valor_tenta_votar_em_outro_dispositivo_com_um_novo_par_de_chaves(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("\"{string}\" tenta votar em outro dispositivo com um novo par de chaves");
}

/// Pendente: que "{string}" já participou dessa eleição.
#[given(regex = r#"^que "([^"]*)" já participou dessa eleição$"#)]
fn given_que_valor_ja_participou_dessa_eleicao(_world: &mut AppWorld, _value_1: String) {
    pending_step("que \"{string}\" já participou dessa eleição");
}

/// Pendente: "{string}" está autenticada em outro dispositivo sem a chave privada anterior.
#[given(
    regex = r#"^"([^"]*)" está autenticada em outro dispositivo sem a chave privada anterior$"#
)]
fn given_valor_esta_autenticada_em_outro_dispositivo_sem_a_chave_privada_anterior(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("\"{string}\" está autenticada em outro dispositivo sem a chave privada anterior");
}

/// Pendente: solicita seu recibo de participação.
#[when(regex = r#"^solicita seu recibo de participação$"#)]
fn when_solicita_seu_recibo_de_participacao(_world: &mut AppWorld) {
    pending_step("solicita seu recibo de participação");
}

/// Pendente: recebe o recibo do voto já aceito.
#[then(regex = r#"^recebe o recibo do voto já aceito$"#)]
fn then_recebe_o_recibo_do_voto_ja_aceito(_world: &mut AppWorld) {
    pending_step("recebe o recibo do voto já aceito");
}

/// Pendente: nenhum novo voto é registrado.
#[then(regex = r#"^nenhum novo voto é registrado$"#)]
fn then_nenhum_novo_voto_e_registrado(_world: &mut AppWorld) {
    pending_step("nenhum novo voto é registrado");
}

/// Pendente: que o voto de "{string}" foi aceito mas a confirmação não chegou ao dispositivo.
#[given(
    regex = r#"^que o voto de "([^"]*)" foi aceito mas a confirmação não chegou ao dispositivo$"#
)]
fn given_que_o_voto_de_valor_foi_aceito_mas_a_confirmacao_nao_chegou_ao_dispositivo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "que o voto de \"{string}\" foi aceito mas a confirmação não chegou ao dispositivo",
    );
}

/// Pendente: "{string}" retransmite o mesmo pacote assinado com o mesmo identificador de envio.
#[when(
    regex = r#"^"([^"]*)" retransmite o mesmo pacote assinado com o mesmo identificador de envio$"#
)]
fn when_valor_retransmite_o_mesmo_pacote_assinado_com_o_mesmo_identificador_de_envio(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "\"{string}\" retransmite o mesmo pacote assinado com o mesmo identificador de envio",
    );
}

/// Pendente: seleciona a opção "{string}" e confirma o voto.
#[when(regex = r#"^seleciona a opção "([^"]*)" e confirma o voto$"#)]
fn when_seleciona_a_opcao_valor_e_confirma_o_voto(_world: &mut AppWorld, _value_1: String) {
    pending_step("seleciona a opção \"{string}\" e confirma o voto");
}

/// Pendente: a opção "{string}" não é oferecida.
#[then(regex = r#"^a opção "([^"]*)" não é oferecida$"#)]
fn then_a_opcao_valor_nao_e_oferecida(_world: &mut AppWorld, _value_1: String) {
    pending_step("a opção \"{string}\" não é oferecida");
}

/// Pendente: que a votação foi encerrada com dois participantes.
#[given(regex = r#"^que a votação foi encerrada com dois participantes$"#)]
fn given_que_a_votacao_foi_encerrada_com_dois_participantes(_world: &mut AppWorld) {
    pending_step("que a votação foi encerrada com dois participantes");
}

/// Pendente: todas as autoridades forneceram contribuições válidas.
#[given(regex = r#"^todas as autoridades forneceram contribuições válidas$"#)]
fn given_todas_as_autoridades_forneceram_contribuicoes_validas(_world: &mut AppWorld) {
    pending_step("todas as autoridades forneceram contribuições válidas");
}

/// Pendente: a apuração produz um voto para "{string}" e um voto nulo.
#[when(regex = r#"^a apuração produz um voto para "([^"]*)" e um voto nulo$"#)]
fn when_a_apuracao_produz_um_voto_para_valor_e_um_voto_nulo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a apuração produz um voto para \"{string}\" e um voto nulo");
}

/// Pendente: a contagem é consistente com os dois participantes.
#[then(regex = r#"^a contagem é consistente com os dois participantes$"#)]
fn then_a_contagem_e_consistente_com_os_dois_participantes(_world: &mut AppWorld) {
    pending_step("a contagem é consistente com os dois participantes");
}

/// Pendente: a soma das opções incluindo Nulo resulta em três votos.
#[when(regex = r#"^a soma das opções incluindo Nulo resulta em três votos$"#)]
fn when_a_soma_das_opcoes_incluindo_nulo_resulta_em_tres_votos(_world: &mut AppWorld) {
    pending_step("a soma das opções incluindo Nulo resulta em três votos");
}

/// Pendente: a eleição é anulada por inconsistência da contagem.
#[then(regex = r#"^a eleição é anulada por inconsistência da contagem$"#)]
fn then_a_eleicao_e_anulada_por_inconsistencia_da_contagem(_world: &mut AppWorld) {
    pending_step("a eleição é anulada por inconsistência da contagem");
}

/// Pendente: os registros permanecem disponíveis para auditoria.
#[then(regex = r#"^os registros permanecem disponíveis para auditoria$"#)]
fn then_os_registros_permanecem_disponiveis_para_auditoria(_world: &mut AppWorld) {
    pending_step("os registros permanecem disponíveis para auditoria");
}

/// Pendente: a soma das opções incluindo Nulo resulta em um voto.
#[when(regex = r#"^a soma das opções incluindo Nulo resulta em um voto$"#)]
fn when_a_soma_das_opcoes_incluindo_nulo_resulta_em_um_voto(_world: &mut AppWorld) {
    pending_step("a soma das opções incluindo Nulo resulta em um voto");
}

/// Pendente: que o período de votação foi encerrado.
#[given(regex = r#"^que o período de votação foi encerrado$"#)]
fn given_que_o_periodo_de_votacao_foi_encerrado(_world: &mut AppWorld) {
    pending_step("que o período de votação foi encerrado");
}

/// Pendente: falta uma contribuição válida de autoridade.
#[given(regex = r#"^falta uma contribuição válida de autoridade$"#)]
fn given_falta_uma_contribuicao_valida_de_autoridade(_world: &mut AppWorld) {
    pending_step("falta uma contribuição válida de autoridade");
}

/// Pendente: a apuração é solicitada.
#[when(regex = r#"^a apuração é solicitada$"#)]
fn when_a_apuracao_e_solicitada(_world: &mut AppWorld) {
    pending_step("a apuração é solicitada");
}

/// Pendente: a apuração permanece incompleta e sem totais definitivos.
#[then(regex = r#"^a apuração permanece incompleta e sem totais definitivos$"#)]
fn then_a_apuracao_permanece_incompleta_e_sem_totais_definitivos(_world: &mut AppWorld) {
    pending_step("a apuração permanece incompleta e sem totais definitivos");
}
