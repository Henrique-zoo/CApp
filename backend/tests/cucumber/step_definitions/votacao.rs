//! A votação precisa assegurar elegibilidade, participação única e as garantias da modalidade eleitoral.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: a chave pública de {string} foi cadastrada antes do início da votação.
#[given(regex = r#"^a chave pública de "([^"]*)" foi cadastrada antes do início da votação$"#)]
fn given_a_chave_publica_de_valor_foi_cadastrada_antes_do_inicio_da_votacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a chave pública de {string} foi cadastrada antes do início da votação");
}

/// Pendente: a chave pública eleitoral de {string} foi cadastrada antes da abertura.
#[given(regex = r#"^a chave pública eleitoral de "([^"]*)" foi cadastrada antes da abertura$"#)]
fn given_a_chave_publica_eleitoral_de_valor_foi_cadastrada_antes_da_abertura(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a chave pública eleitoral de {string} foi cadastrada antes da abertura");
}

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

/// Pendente: o estudante {string} está cadastrado e atende aos critérios de elegibilidade.
#[given(
    regex = r#"^o estudante "([^"]*)" está cadastrado e atende aos critérios de elegibilidade$"#
)]
fn given_o_estudante_valor_esta_cadastrado_e_atende_aos_criterios_de_elegibilidade(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o estudante {string} está cadastrado e atende aos critérios de elegibilidade");
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

/// Pendente: o estudante {string} não atende a um critério de elegibilidade definido para a eleição.
#[given(
    regex = r#"^o estudante "([^"]*)" não atende a um critério de elegibilidade definido para a eleição$"#
)]
fn given_o_estudante_valor_nao_atende_a_um_criterio_de_elegibilidade_definido_para_a_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "o estudante {string} não atende a um critério de elegibilidade definido para a eleição",
    );
}

/// Pendente: o estudante {string} não consta no cadastro de eleitores definido antes do início da eleição.
#[given(
    regex = r#"^o estudante "([^"]*)" não consta no cadastro de eleitores definido antes do início da eleição$"#
)]
fn given_o_estudante_valor_nao_consta_no_cadastro_de_eleitores_definido_antes_do_inicio_da_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "o estudante {string} não consta no cadastro de eleitores definido antes do início da eleição",
    );
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

/// Pendente: {string} consta no cadastro de eleitores definido antes da abertura.
#[given(regex = r#"^"([^"]*)" consta no cadastro de eleitores definido antes da abertura$"#)]
fn given_valor_consta_no_cadastro_de_eleitores_definido_antes_da_abertura(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} consta no cadastro de eleitores definido antes da abertura");
}

/// Pendente: {string} está cadastrada e atende aos critérios de elegibilidade.
#[given(regex = r#"^"([^"]*)" está cadastrada e atende aos critérios de elegibilidade$"#)]
fn given_valor_esta_cadastrada_e_atende_aos_criterios_de_elegibilidade(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} está cadastrada e atende aos critérios de elegibilidade");
}

/// Pendente: {string} já registrou sua participação nessa eleição por votação secreta.
#[given(regex = r#"^"([^"]*)" já registrou sua participação nessa eleição por votação secreta$"#)]
fn given_valor_ja_registrou_sua_participacao_nessa_eleicao_por_votacao_secreta(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} já registrou sua participação nessa eleição por votação secreta");
}

/// Pendente: {string} possui uma chave pública cadastrada para a eleição.
#[given(regex = r#"^"([^"]*)" possui uma chave pública cadastrada para a eleição$"#)]
fn given_valor_possui_uma_chave_publica_cadastrada_para_a_eleicao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} possui uma chave pública cadastrada para a eleição");
}

/// Pendente: {string} só foi cadastrado como eleitor depois da abertura.
#[given(regex = r#"^"([^"]*)" só foi cadastrado como eleitor depois da abertura$"#)]
fn given_valor_so_foi_cadastrado_como_eleitor_depois_da_abertura(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("{string} só foi cadastrado como eleitor depois da abertura");
}

/// Pendente: a assinatura digital é validada com a chave pública cadastrada de {string}.
#[then(regex = r#"^a assinatura digital é validada com a chave pública cadastrada de "([^"]*)"$"#)]
fn then_a_assinatura_digital_e_validada_com_a_chave_publica_cadastrada_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a assinatura digital é validada com a chave pública cadastrada de {string}");
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

/// Pendente: a cédula é recusada porque a assinatura não corresponde à identidade eleitoral de {string}.
#[then(
    regex = r#"^a cédula é recusada porque a assinatura não corresponde à identidade eleitoral de "([^"]*)"$"#
)]
fn then_a_cedula_e_recusada_porque_a_assinatura_nao_corresponde_a_identidade_eleitoral_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "a cédula é recusada porque a assinatura não corresponde à identidade eleitoral de {string}",
    );
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

/// Pendente: a tentativa de voto é recusada por falta de cadastro eleitoral.
#[then(regex = r#"^a tentativa de voto é recusada por falta de cadastro eleitoral$"#)]
fn then_a_tentativa_de_voto_e_recusada_por_falta_de_cadastro_eleitoral(_world: &mut AppWorld) {
    pending_step("a tentativa de voto é recusada por falta de cadastro eleitoral");
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

/// Pendente: não consegue determinar a escolha em texto claro de {string} a partir desses dados.
#[then(
    regex = r#"^não consegue determinar a escolha em texto claro de "([^"]*)" a partir desses dados$"#
)]
fn then_nao_consegue_determinar_a_escolha_em_texto_claro_de_valor_a_partir_desses_dados(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "não consegue determinar a escolha em texto claro de {string} a partir desses dados",
    );
}

/// Pendente: não consegue determinar a escolha individual de cada eleitor com base nessas informações.
#[then(
    regex = r#"^não consegue determinar a escolha individual de cada eleitor com base nessas informações$"#
)]
fn then_nao_consegue_determinar_a_escolha_individual_de_cada_eleitor_com_base_nessas_informacoes(
    _world: &mut AppWorld,
) {
    pending_step(
        "não consegue determinar a escolha individual de cada eleitor com base nessas informações",
    );
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

/// Pendente: o voto é recusado por ausência no cadastro eleitoral anterior à abertura.
#[then(regex = r#"^o voto é recusado por ausência no cadastro eleitoral anterior à abertura$"#)]
fn then_o_voto_e_recusado_por_ausencia_no_cadastro_eleitoral_anterior_a_abertura(
    _world: &mut AppWorld,
) {
    pending_step("o voto é recusado por ausência no cadastro eleitoral anterior à abertura");
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

/// Pendente: submete uma cédula assinada com a chave privada correspondente à sua chave pública cadastrada, indicando a chapa {string}.
#[when(
    regex = r#"^submete uma cédula assinada com a chave privada correspondente à sua chave pública cadastrada, indicando a chapa "([^"]*)"$"#
)]
fn when_submete_uma_cedula_assinada_com_a_chave_privada_correspondente_a_sua_chave_publica_cadastrada_indica(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "submete uma cédula assinada com a chave privada correspondente à sua chave pública cadastrada, indicando a chapa {string}",
    );
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

/// Pendente: tenta registrar um voto com assinatura válida.
#[when(regex = r#"^tenta registrar um voto com assinatura válida$"#)]
fn when_tenta_registrar_um_voto_com_assinatura_valida(_world: &mut AppWorld) {
    pending_step("tenta registrar um voto com assinatura válida");
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

/// Pendente: {string} submete uma cédula assinada com a chave privada correspondente à chave pública de {string}.
#[when(
    regex = r#"^"([^"]*)" submete uma cédula assinada com a chave privada correspondente à chave pública de "([^"]*)"$"#
)]
fn when_valor_submete_uma_cedula_assinada_com_a_chave_privada_correspondente_a_chave_publica_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "{string} submete uma cédula assinada com a chave privada correspondente à chave pública de {string}",
    );
}
