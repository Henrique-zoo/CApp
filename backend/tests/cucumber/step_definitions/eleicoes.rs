//! Configuração, calendário e cancelamento do processo eleitoral.
//!
//! A gestão do CA designa os integrantes da comissão para uma eleição. Nessa
//! eleição, a comissão define a modalidade de votação e o calendário. Na
//! modalidade secreta, o contrato #377 permite chapa única com autoridades
//! independentes.
//!
//! Estes registros mantêm a especificação ligada ao runner até as funcionalidades
//! existirem. Cada corpo chama pending_step, sem simular autorização, persistência
//! ou apuração. Expressões reutilizadas pela votação são registradas em shared_steps
//! para conservar resolução única.
//!
//! A autorização da solicitação de apuração não representa sua conclusão. As
//! contribuições criptográficas são uma condição exclusiva da votação secreta;
//! o encerramento no prazo vigente também é especificado para a identificada.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que a gestão do CA designou a comissão para esta eleição.
#[given(regex = r#"^que a gestão do CA designou a comissão para esta eleição$"#)]
fn given_gestao_designou_comissao_para_eleicao(_world: &mut AppWorld) {
    pending_step("que a gestão do CA designou a comissão para esta eleição");
}

/// Pendente: estou autenticado como integrante autorizado dessa comissão.
#[given(regex = r#"^estou autenticado como integrante autorizado dessa comissão$"#)]
fn given_estou_autenticado_como_integrante_autorizado_dessa_comissao(_world: &mut AppWorld) {
    pending_step("estou autenticado como integrante autorizado dessa comissão");
}

/// Pendente: que possuo a permissão {string} nesta eleição.
#[given(regex = r#"^que possuo a permissão "([^"]*)" nesta eleição$"#)]
fn given_possuo_permissao_nesta_eleicao(_world: &mut AppWorld, _permissao: String) {
    pending_step("que possuo a permissão {string} nesta eleição");
}

/// Pendente: que estou preenchendo os dados iniciais da eleição.
#[given(regex = r#"^que estou preenchendo os dados iniciais da eleição$"#)]
fn given_preencho_dados_iniciais_da_eleicao(_world: &mut AppWorld) {
    pending_step("que estou preenchendo os dados iniciais da eleição");
}

/// Pendente: seleciono votação {string} e concluo a configuração inicial.
#[when(regex = r#"^seleciono votação "(secreta|identificada)" e concluo a configuração inicial$"#)]
fn when_seleciono_modalidade_e_concluo_configuracao(_world: &mut AppWorld, _modalidade: String) {
    pending_step("seleciono votação {string} e concluo a configuração inicial");
}

/// Pendente: a configuração da eleição registra a votação {string}.
#[then(regex = r#"^a configuração da eleição registra a votação "(secreta|identificada)"$"#)]
fn then_configuracao_registra_modalidade(_world: &mut AppWorld, _modalidade: String) {
    pending_step("a configuração da eleição registra a votação {string}");
}

/// Pendente: que existe uma única chapa aprovada pela comissão.
#[given(regex = r#"^que existe uma única chapa aprovada pela comissão$"#)]
fn given_que_existe_uma_unica_chapa_aprovada_pela_comissao(_world: &mut AppWorld) {
    pending_step("que existe uma única chapa aprovada pela comissão");
}

/// Pendente: a configuração eleitoral é concluída.
#[when(regex = r#"^a configuração eleitoral é concluída$"#)]
fn when_a_configuracao_eleitoral_e_concluida(_world: &mut AppWorld) {
    pending_step("a configuração eleitoral é concluída");
}

/// Pendente: o manifesto contém a chapa e a opção Nulo.
#[then(regex = r#"^o manifesto contém a chapa e a opção Nulo$"#)]
fn then_o_manifesto_contem_a_chapa_e_a_opcao_nulo(_world: &mut AppWorld) {
    pending_step("o manifesto contém a chapa e a opção Nulo");
}

/// Pendente: as autoridades são o servidor, a comissão e a chapa.
#[then(regex = r#"^as autoridades são o servidor, a comissão e a chapa$"#)]
fn then_as_autoridades_sao_o_servidor_a_comissao_e_a_chapa(_world: &mut AppWorld) {
    pending_step("as autoridades são o servidor, a comissão e a chapa");
}

/// Pendente: o servidor não recebe os segredos das outras autoridades.
#[then(regex = r#"^o servidor não recebe os segredos das outras autoridades$"#)]
fn then_o_servidor_nao_recebe_os_segredos_das_outras_autoridades(_world: &mut AppWorld) {
    pending_step("o servidor não recebe os segredos das outras autoridades");
}

/// Pendente: que a eleição ainda está em preparação.
#[given(regex = r#"^que a eleição ainda está em preparação$"#)]
fn given_que_a_eleicao_ainda_esta_em_preparacao(_world: &mut AppWorld) {
    pending_step("que a eleição ainda está em preparação");
}

/// Pendente: a comissão adia a abertura e escolhe um fim posterior ao anterior.
#[when(regex = r#"^a comissão adia a abertura e escolhe um fim posterior ao anterior$"#)]
fn when_a_comissao_adia_a_abertura_e_escolhe_um_fim_posterior_ao_anterior(_world: &mut AppWorld) {
    pending_step("a comissão adia a abertura e escolhe um fim posterior ao anterior");
}

/// Pendente: o novo calendário é registrado com motivo e responsável.
#[then(regex = r#"^o novo calendário é registrado com motivo e responsável$"#)]
fn then_o_novo_calendario_e_registrado_com_motivo_e_responsavel(_world: &mut AppWorld) {
    pending_step("o novo calendário é registrado com motivo e responsável");
}

/// Pendente: a comissão não é obrigada a preservar a duração original.
#[then(regex = r#"^a comissão não é obrigada a preservar a duração original$"#)]
fn then_a_comissao_nao_e_obrigada_a_preservar_a_duracao_original(_world: &mut AppWorld) {
    pending_step("a comissão não é obrigada a preservar a duração original");
}

/// Pendente: a comissão adia a abertura mas mantém o encerramento anterior.
#[when(regex = r#"^a comissão adia a abertura mas mantém o encerramento anterior$"#)]
fn when_a_comissao_adia_a_abertura_mas_mantem_o_encerramento_anterior(_world: &mut AppWorld) {
    pending_step("a comissão adia a abertura mas mantém o encerramento anterior");
}

/// Pendente: a alteração do calendário é recusada.
#[then(regex = r#"^a alteração do calendário é recusada$"#)]
fn then_a_alteracao_do_calendario_e_recusada(_world: &mut AppWorld) {
    pending_step("a alteração do calendário é recusada");
}

/// Pendente: que a eleição está aberta e o prazo vigente ainda não venceu.
#[given(regex = r#"^que a eleição está aberta e o prazo vigente ainda não venceu$"#)]
fn given_que_a_eleicao_esta_aberta_e_o_prazo_vigente_ainda_nao_venceu(_world: &mut AppWorld) {
    pending_step("que a eleição está aberta e o prazo vigente ainda não venceu");
}

/// Pendente: a comissão escolhe um encerramento posterior ao vigente.
#[when(regex = r#"^a comissão escolhe um encerramento posterior ao vigente$"#)]
fn when_a_comissao_escolhe_um_encerramento_posterior_ao_vigente(_world: &mut AppWorld) {
    pending_step("a comissão escolhe um encerramento posterior ao vigente");
}

/// Pendente: o manifesto e a abertura já ocorrida permanecem inalterados.
#[then(regex = r#"^o manifesto e a abertura já ocorrida permanecem inalterados$"#)]
fn then_o_manifesto_e_a_abertura_ja_ocorrida_permanecem_inalterados(_world: &mut AppWorld) {
    pending_step("o manifesto e a abertura já ocorrida permanecem inalterados");
}

/// Pendente: as cédulas já aceitas continuam válidas.
#[then(regex = r#"^as cédulas já aceitas continuam válidas$"#)]
fn then_as_cedulas_ja_aceitas_continuam_validas(_world: &mut AppWorld) {
    pending_step("as cédulas já aceitas continuam válidas");
}

/// Pendente: que o prazo vigente da eleição já terminou.
#[given(regex = r#"^que o prazo vigente da eleição já terminou$"#)]
fn given_que_o_prazo_vigente_da_eleicao_ja_terminou(_world: &mut AppWorld) {
    pending_step("que o prazo vigente da eleição já terminou");
}

/// Pendente: a comissão tenta prorrogar o encerramento.
#[when(regex = r#"^a comissão tenta prorrogar o encerramento$"#)]
fn when_a_comissao_tenta_prorrogar_o_encerramento(_world: &mut AppWorld) {
    pending_step("a comissão tenta prorrogar o encerramento");
}

/// Pendente: a eleição não é reaberta.
#[then(regex = r#"^a eleição não é reaberta$"#)]
fn then_a_eleicao_nao_e_reaberta(_world: &mut AppWorld) {
    pending_step("a eleição não é reaberta");
}

/// Pendente: a comissão solicita encerrar a votação para apurar o resultado.
#[when(regex = r#"^a comissão solicita encerrar a votação para apurar o resultado$"#)]
fn when_a_comissao_solicita_encerrar_a_votacao_para_apurar_o_resultado(_world: &mut AppWorld) {
    pending_step("a comissão solicita encerrar a votação para apurar o resultado");
}

/// Pendente: o encerramento antecipado é recusado.
#[then(regex = r#"^o encerramento antecipado é recusado$"#)]
fn then_o_encerramento_antecipado_e_recusado(_world: &mut AppWorld) {
    pending_step("o encerramento antecipado é recusado");
}

/// Pendente: nenhuma contagem antecipada por opção é publicada.
#[then(regex = r#"^nenhuma contagem antecipada por opção é publicada$"#)]
fn then_nenhuma_contagem_antecipada_por_opcao_e_publicada(_world: &mut AppWorld) {
    pending_step("nenhuma contagem antecipada por opção é publicada");
}

/// Pendente: que a eleição foi criada por engano e ainda não recebeu votos.
#[given(regex = r#"^que a eleição foi criada por engano e ainda não recebeu votos$"#)]
fn given_que_a_eleicao_foi_criada_por_engano_e_ainda_nao_recebeu_votos(_world: &mut AppWorld) {
    pending_step("que a eleição foi criada por engano e ainda não recebeu votos");
}

/// Pendente: a comissão solicita cancelamento com motivo.
#[when(regex = r#"^a comissão solicita cancelamento com motivo$"#)]
fn when_a_comissao_solicita_cancelamento_com_motivo(_world: &mut AppWorld) {
    pending_step("a comissão solicita cancelamento com motivo");
}

/// Pendente: a eleição fica cancelada com responsável e instante registrados.
#[then(regex = r#"^a eleição fica cancelada com responsável e instante registrados$"#)]
fn then_a_eleicao_fica_cancelada_com_responsavel_e_instante_registrados(_world: &mut AppWorld) {
    pending_step("a eleição fica cancelada com responsável e instante registrados");
}

/// Pendente: que a eleição está aberta e já recebeu cédulas.
#[given(regex = r#"^que a eleição está aberta e já recebeu cédulas$"#)]
fn given_que_a_eleicao_esta_aberta_e_ja_recebeu_cedulas(_world: &mut AppWorld) {
    pending_step("que a eleição está aberta e já recebeu cédulas");
}

/// Pendente: as cédulas e o histórico são preservados.
#[then(regex = r#"^as cédulas e o histórico são preservados$"#)]
fn then_as_cedulas_e_o_historico_sao_preservados(_world: &mut AppWorld) {
    pending_step("as cédulas e o histórico são preservados");
}

/// Pendente: nenhum novo voto é aceito.
#[then(regex = r#"^nenhum novo voto é aceito$"#)]
fn then_nenhum_novo_voto_e_aceito(_world: &mut AppWorld) {
    pending_step("nenhum novo voto é aceito");
}

// -----------------------------------------------------------------------------
// Designação e permissões específicas da comissão eleitoral — Issue #124.
// Estes passos permanecem pendentes até a implementação das operações eleitorais.
// -----------------------------------------------------------------------------

/// Pendente: existe a eleição {string} de {string}.
#[given(regex = r#"^existe a eleição "([^"]*)" de "([^"]*)"$"#)]
fn given_existe_eleicao_do_ca(_world: &mut AppWorld, _eleicao: String, _ca: String) {
    pending_step("existe a eleição {string} de {string}");
}

/// Pendente: ainda não integra a comissão da eleição {string}.
#[given(regex = r#"^ainda não integra a comissão da eleição "([^"]*)"$"#)]
fn given_ainda_nao_integra_comissao(_world: &mut AppWorld, _eleicao: String) {
    pending_step("ainda não integra a comissão da eleição {string}");
}

/// Pendente: o gestor {string} designa {string} para a comissão da eleição {string}.
#[when(regex = r#"^o gestor "([^"]*)" designa "([^"]*)" para a comissão da eleição "([^"]*)"$"#)]
fn when_gestor_designa_comissao(
    _world: &mut AppWorld,
    _gestor: String,
    _integrante: String,
    _eleicao: String,
) {
    pending_step("o gestor {string} designa {string} para a comissão da eleição {string}");
}

/// Pendente: {string} passa a integrar a comissão da eleição {string}.
#[then(regex = r#"^"([^"]*)" passa a integrar a comissão da eleição "([^"]*)"$"#)]
fn then_passou_a_integrar_comissao(_world: &mut AppWorld, _integrante: String, _eleicao: String) {
    pending_step("{string} passa a integrar a comissão da eleição {string}");
}

/// Pendente: que {string} integra a comissão da eleição {string}.
#[given(regex = r#"^que "([^"]*)" integra a comissão da eleição "([^"]*)"$"#)]
fn given_que_integra_comissao(_world: &mut AppWorld, _integrante: String, _eleicao: String) {
    pending_step("que {string} integra a comissão da eleição {string}");
}

/// Pendente: não integra a comissão da eleição {string}.
#[given(regex = r#"^não integra a comissão da eleição "([^"]*)"$"#)]
fn given_nao_integra_comissao(_world: &mut AppWorld, _eleicao: String) {
    pending_step("não integra a comissão da eleição {string}");
}

/// Pendente: possui a permissão {string} na eleição {string}.
#[given(regex = r#"^possui a permissão "([^"]*)" na eleição "([^"]*)"$"#)]
fn given_possui_permissao_eleitoral(_world: &mut AppWorld, _permissao: String, _eleicao: String) {
    pending_step("possui a permissão {string} na eleição {string}");
}

/// Pendente: não possui a permissão {string} na eleição {string}.
#[given(regex = r#"^não possui a permissão "([^"]*)" na eleição "([^"]*)"$"#)]
fn given_nao_possui_permissao_eleitoral(
    _world: &mut AppWorld,
    _permissao: String,
    _eleicao: String,
) {
    pending_step("não possui a permissão {string} na eleição {string}");
}

/// Pendente: tenta apurar o resultado da eleição {string}.
#[when(regex = r#"^tenta apurar o resultado da eleição "([^"]*)"$"#)]
fn when_tenta_apurar_resultado(_world: &mut AppWorld, _eleicao: String) {
    pending_step("tenta apurar o resultado da eleição {string}");
}

/// Pendente: tenta configurar a eleição {string}.
#[when(regex = r#"^tenta configurar a eleição "([^"]*)"$"#)]
fn when_tenta_configurar_eleicao(_world: &mut AppWorld, _eleicao: String) {
    pending_step("tenta configurar a eleição {string}");
}

/// Pendente: realizou ações administrativas na eleição {string}.
#[given(regex = r#"^realizou ações administrativas na eleição "([^"]*)"$"#)]
fn given_realizou_acoes_administrativas(_world: &mut AppWorld, _eleicao: String) {
    pending_step("realizou ações administrativas na eleição {string}");
}

/// Pendente: o gestor {string} solicita a revogação da designação de {string} da comissão da eleição {string}.
#[when(
    regex = r#"^o gestor "([^"]*)" solicita a revogação da designação de "([^"]*)" da comissão da eleição "([^"]*)"$"#
)]
fn when_gestor_solicita_revogacao_comissao(
    _world: &mut AppWorld,
    _gestor: String,
    _integrante: String,
    _eleicao: String,
) {
    pending_step(
        "o gestor {string} solicita a revogação da designação de {string} da comissão da eleição {string}",
    );
}

/// Pendente: que {string} representa a comissão como autoridade da eleição {string}.
#[given(regex = r#"^que "([^"]*)" representa a comissão como autoridade da eleição "([^"]*)"$"#)]
fn given_representante_da_comissao(_world: &mut AppWorld, _integrante: String, _eleicao: String) {
    pending_step("que {string} representa a comissão como autoridade da eleição {string}");
}

/// Pendente: que o manifesto da eleição foi publicado e a eleição está {string}.
#[given(regex = r#"^que o manifesto da eleição foi publicado e a eleição está "([^"]*)"$"#)]
fn given_manifesto_publicado_na_fase(_world: &mut AppWorld, _fase: String) {
    pending_step("que o manifesto da eleição foi publicado e a eleição está {string}");
}

/// Pendente: a revogação é recusada enquanto a eleição está em andamento.
#[then(regex = r#"^a revogação é recusada enquanto a eleição está em andamento$"#)]
fn then_recusa_revogacao_durante_a_eleicao(_world: &mut AppWorld) {
    pending_step("a revogação é recusada enquanto a eleição está em andamento");
}

/// Pendente: {string} permanece como representante da comissão da eleição {string}.
#[then(regex = r#"^"([^"]*)" permanece como representante da comissão da eleição "([^"]*)"$"#)]
fn then_mantem_representante_da_comissao(
    _world: &mut AppWorld,
    _integrante: String,
    _eleicao: String,
) {
    pending_step("{string} permanece como representante da comissão da eleição {string}");
}

/// Pendente: {string} deixa de integrar a comissão da eleição {string}.
#[then(regex = r#"^"([^"]*)" deixa de integrar a comissão da eleição "([^"]*)"$"#)]
fn then_deixa_de_integrar_comissao(_world: &mut AppWorld, _integrante: String, _eleicao: String) {
    pending_step("{string} deixa de integrar a comissão da eleição {string}");
}

/// Pendente: deixa de possuir as autorizações eleitorais vinculadas à designação revogada.
#[then(regex = r#"^deixa de possuir as autorizações eleitorais vinculadas à designação revogada$"#)]
fn then_perde_autorizacoes_da_designacao(_world: &mut AppWorld) {
    pending_step("deixa de possuir as autorizações eleitorais vinculadas à designação revogada");
}

/// Pendente: as ações anteriores permanecem registradas no histórico da eleição.
#[then(regex = r#"^as ações anteriores permanecem registradas no histórico da eleição$"#)]
fn then_acoes_anteriores_no_historico(_world: &mut AppWorld) {
    pending_step("as ações anteriores permanecem registradas no histórico da eleição");
}

/// Pendente: {string} tenta designar {string} para a comissão da eleição {string}.
#[when(regex = r#"^"([^"]*)" tenta designar "([^"]*)" para a comissão da eleição "([^"]*)"$"#)]
fn when_integrante_tenta_designar_comissao(
    _world: &mut AppWorld,
    _ator: String,
    _integrante: String,
    _eleicao: String,
) {
    pending_step("{string} tenta designar {string} para a comissão da eleição {string}");
}

// -----------------------------------------------------------------------------
// Validação do período, das autoridades e da publicação do manifesto — Issue #124.
// -----------------------------------------------------------------------------

/// Pendente: informo um período de votação válido e concluo a configuração inicial.
#[when(regex = r#"^informo um período de votação válido e concluo a configuração inicial$"#)]
fn when_informa_periodo_valido(_world: &mut AppWorld) {
    pending_step("informo um período de votação válido e concluo a configuração inicial");
}

/// Pendente: o calendário inicial registra o início e o encerramento informados.
#[then(regex = r#"^o calendário inicial registra o início e o encerramento informados$"#)]
fn then_calendario_inicial_registra_periodo(_world: &mut AppWorld) {
    pending_step("o calendário inicial registra o início e o encerramento informados");
}

/// Pendente: informo um período cujo encerramento é anterior ao início e tento concluir a configuração inicial.
#[when(
    regex = r#"^informo um período cujo encerramento é anterior ao início e tento concluir a configuração inicial$"#
)]
fn when_informa_periodo_com_fim_anterior_inicio(_world: &mut AppWorld) {
    pending_step(
        "informo um período cujo encerramento é anterior ao início e tento concluir a configuração inicial",
    );
}

/// Pendente: informo um período cujo encerramento é igual ao início e tento concluir a configuração inicial.
#[when(
    regex = r#"^informo um período cujo encerramento é igual ao início e tento concluir a configuração inicial$"#
)]
fn when_informa_periodo_com_fim_igual_inicio(_world: &mut AppWorld) {
    pending_step(
        "informo um período cujo encerramento é igual ao início e tento concluir a configuração inicial",
    );
}

/// Pendente: a configuração inicial é recusada por período inválido.
#[then(regex = r#"^a configuração inicial é recusada por período inválido$"#)]
fn then_configuracao_inicial_recusada_periodo_invalido(_world: &mut AppWorld) {
    pending_step("a configuração inicial é recusada por período inválido");
}

/// Pendente: que a eleição está em preparação sem todas as autoridades definidas.
#[given(regex = r#"^que a eleição está em preparação sem todas as autoridades definidas$"#)]
fn given_eleicao_preparacao_sem_todas_autoridades(_world: &mut AppWorld) {
    pending_step("que a eleição está em preparação sem todas as autoridades definidas");
}

/// Pendente: é atingido o horário previsto para a abertura.
#[when(regex = r#"^é atingido o horário previsto para a abertura$"#)]
fn when_chega_horario_da_abertura(_world: &mut AppWorld) {
    pending_step("é atingido o horário previsto para a abertura");
}

/// Pendente: a votação permanece fechada por falta de autoridades.
#[then(regex = r#"^a votação permanece fechada por falta de autoridades$"#)]
fn then_votacao_fechada_falta_autoridades(_world: &mut AppWorld) {
    pending_step("a votação permanece fechada por falta de autoridades");
}

/// Pendente: que a eleição está em preparação com duas chapas aprovadas.
#[given(regex = r#"^que a eleição está em preparação com duas chapas aprovadas$"#)]
fn given_eleicao_preparacao_duas_chapas(_world: &mut AppWorld) {
    pending_step("que a eleição está em preparação com duas chapas aprovadas");
}

/// Pendente: cada chapa possui uma autoridade independente.
#[given(regex = r#"^cada chapa possui uma autoridade independente$"#)]
fn given_cada_chapa_possui_autoridade_independente(_world: &mut AppWorld) {
    pending_step("cada chapa possui uma autoridade independente");
}

/// Pendente: o servidor e a comissão possuem autoridades independentes.
#[given(regex = r#"^o servidor e a comissão possuem autoridades independentes$"#)]
fn given_servidor_comissao_autoridades_independentes(_world: &mut AppWorld) {
    pending_step("o servidor e a comissão possuem autoridades independentes");
}

/// Pendente: a eleição aguarda o horário de abertura.
#[then(regex = r#"^a eleição aguarda o horário de abertura$"#)]
fn then_eleicao_aguarda_horario_abertura(_world: &mut AppWorld) {
    pending_step("a eleição aguarda o horário de abertura");
}

/// Pendente: que a eleição aguarda o horário de abertura com o manifesto publicado.
#[given(regex = r#"^que a eleição aguarda o horário de abertura com o manifesto publicado$"#)]
fn given_eleicao_aguarda_abertura_com_manifesto(_world: &mut AppWorld) {
    pending_step("que a eleição aguarda o horário de abertura com o manifesto publicado");
}

/// Pendente: a votação é aberta automaticamente.
#[then(regex = r#"^a votação é aberta automaticamente$"#)]
fn then_votacao_abre_automaticamente(_world: &mut AppWorld) {
    pending_step("a votação é aberta automaticamente");
}

/// Pendente: o manifesto publicado contém as duas chapas na ordem definida.
#[then(regex = r#"^o manifesto publicado contém as duas chapas na ordem definida$"#)]
fn then_manifesto_publicado_duas_chapas_ordem(_world: &mut AppWorld) {
    pending_step("o manifesto publicado contém as duas chapas na ordem definida");
}

/// Pendente: o manifesto publicado contém a opção Nulo uma única vez na última posição.
#[then(regex = r#"^o manifesto publicado contém a opção Nulo uma única vez na última posição$"#)]
fn then_manifesto_publicado_nulo_unico_ultima_posicao(_world: &mut AppWorld) {
    pending_step("o manifesto publicado contém a opção Nulo uma única vez na última posição");
}

/// Pendente: o manifesto contém uma autoridade do servidor, uma da comissão e uma para cada chapa.
#[then(
    regex = r#"^o manifesto contém uma autoridade do servidor, uma da comissão e uma para cada chapa$"#
)]
fn then_manifesto_contem_autoridades_independentes(_world: &mut AppWorld) {
    pending_step(
        "o manifesto contém uma autoridade do servidor, uma da comissão e uma para cada chapa",
    );
}

/// Pendente: o manifesto é publicado com a assinatura do servidor.
#[then(regex = r#"^o manifesto é publicado com a assinatura do servidor$"#)]
fn then_manifesto_publicado_com_assinatura_servidor(_world: &mut AppWorld) {
    pending_step("o manifesto é publicado com a assinatura do servidor");
}

/// Pendente: que o manifesto oficial da eleição foi publicado e ainda não houve votos.
#[given(regex = r#"^que o manifesto oficial da eleição foi publicado e ainda não houve votos$"#)]
fn given_manifesto_oficial_publicado_sem_votos(_world: &mut AppWorld) {
    pending_step("que o manifesto oficial da eleição foi publicado e ainda não houve votos");
}

/// Pendente: a comissão tenta alterar as chapas ou a ordem das opções.
#[when(regex = r#"^a comissão tenta alterar as chapas ou a ordem das opções$"#)]
fn when_comissao_tenta_alterar_chapas_opcoes(_world: &mut AppWorld) {
    pending_step("a comissão tenta alterar as chapas ou a ordem das opções");
}

/// Pendente: a alteração da configuração eleitoral é recusada.
#[then(regex = r#"^a alteração da configuração eleitoral é recusada$"#)]
fn then_alteracao_configuracao_eleitoral_recusada(_world: &mut AppWorld) {
    pending_step("a alteração da configuração eleitoral é recusada");
}

/// Pendente: o manifesto publicado permanece inalterado.
#[then(regex = r#"^o manifesto publicado permanece inalterado$"#)]
fn then_manifesto_publicado_inalterado(_world: &mut AppWorld) {
    pending_step("o manifesto publicado permanece inalterado");
}

/// Pendente: {string} não passa a integrar a comissão da eleição {string}.
#[then(regex = r#"^"([^"]*)" não passa a integrar a comissão da eleição "([^"]*)"$"#)]
fn then_nao_passou_a_integrar_comissao(
    _world: &mut AppWorld,
    _integrante: String,
    _eleicao: String,
) {
    pending_step("{string} não passa a integrar a comissão da eleição {string}");
}

// -----------------------------------------------------------------------------
// Permissões específicas da comissão eleitoral — Issue #124.
// -----------------------------------------------------------------------------

/// Pendente: possui a permissão administrativa {string} em {string}.
#[given(regex = r#"^possui a permissão administrativa "([^"]*)" em "([^"]*)"$"#)]
fn given_possui_permissao_administrativa(_world: &mut AppWorld, _permissao: String, _ca: String) {
    pending_step("possui a permissão administrativa {string} em {string}");
}

/// Pendente: que a eleição {string} está em preparação, sem manifesto publicado.
#[given(regex = r#"^que a eleição "([^"]*)" está em preparação, sem manifesto publicado$"#)]
fn given_eleicao_em_preparacao_sem_manifesto(_world: &mut AppWorld, _eleicao: String) {
    pending_step("que a eleição {string} está em preparação, sem manifesto publicado");
}

/// Pendente: configura a eleição {string}.
#[when(regex = r#"^configura a eleição "([^"]*)"$"#)]
fn when_configura_eleicao(_world: &mut AppWorld, _eleicao: String) {
    pending_step("configura a eleição {string}");
}

/// Pendente: a configuração da eleição é autorizada.
#[then(regex = r#"^a configuração da eleição é autorizada$"#)]
fn then_configuracao_eleicao_autorizada(_world: &mut AppWorld) {
    pending_step("a configuração da eleição é autorizada");
}

/// Pendente: administra as chapas da eleição {string}.
#[when(regex = r#"^administra as chapas da eleição "([^"]*)"$"#)]
fn when_administra_chapas_eleicao(_world: &mut AppWorld, _eleicao: String) {
    pending_step("administra as chapas da eleição {string}");
}

/// Pendente: a administração das chapas é concluída.
#[then(regex = r#"^a administração das chapas é concluída$"#)]
fn then_administracao_chapas_concluida(_world: &mut AppWorld) {
    pending_step("a administração das chapas é concluída");
}

/// Pendente: solicita a apuração do resultado da eleição {string}.
#[when(regex = r#"^solicita a apuração do resultado da eleição "([^"]*)"$"#)]
fn when_solicita_apuracao_resultado_eleicao(_world: &mut AppWorld, _eleicao: String) {
    pending_step("solicita a apuração do resultado da eleição {string}");
}

/// Pendente: a solicitação de apuração é autorizada.
#[then(regex = r#"^a solicitação de apuração é autorizada$"#)]
fn then_solicitacao_apuracao_autorizada(_world: &mut AppWorld) {
    pending_step("a solicitação de apuração é autorizada");
}

// -----------------------------------------------------------------------------
// Calendário e encerramento normal da votação — Issue #124.
// -----------------------------------------------------------------------------

/// Pendente: a comissão adia a abertura e escolhe um fim anterior ao anterior.
#[when(regex = r#"^a comissão adia a abertura e escolhe um fim anterior ao anterior$"#)]
fn when_comissao_adia_abertura_fim_anterior(_world: &mut AppWorld) {
    pending_step("a comissão adia a abertura e escolhe um fim anterior ao anterior");
}

/// Pendente: o processo eleitoral é encerrado.
#[when(regex = r#"^o processo eleitoral é encerrado$"#)]
fn when_processo_eleitoral_encerrado(_world: &mut AppWorld) {
    pending_step("o processo eleitoral é encerrado");
}

/// Pendente: a eleição passa ao estado encerrado.
#[then(regex = r#"^a eleição passa ao estado encerrado$"#)]
fn then_eleicao_estado_encerrado(_world: &mut AppWorld) {
    pending_step("a eleição passa ao estado encerrado");
}

/// Pendente: o registro final das cédulas é preservado.
#[then(regex = r#"^o registro final das cédulas é preservado$"#)]
fn then_registro_final_cedulas_preservado(_world: &mut AppWorld) {
    pending_step("o registro final das cédulas é preservado");
}

/// Pendente: a apuração aguarda as contribuições válidas de todas as autoridades.
#[then(regex = r#"^a apuração aguarda as contribuições válidas de todas as autoridades$"#)]
fn then_apuracao_aguarda_contribuicoes_autoridades(_world: &mut AppWorld) {
    pending_step("a apuração aguarda as contribuições válidas de todas as autoridades");
}

/// Pendente: a apuração não exige contribuições de autoridades criptográficas.
#[then(regex = r#"^a apuração não exige contribuições de autoridades criptográficas$"#)]
fn then_apuracao_dispensa_contribuicoes_criptograficas(_world: &mut AppWorld) {
    pending_step("a apuração não exige contribuições de autoridades criptográficas");
}

/// Pendente: que a eleição está aberta e o prazo vigente foi atingido.
#[given(regex = r#"^que a eleição está aberta e o prazo vigente foi atingido$"#)]
fn given_eleicao_aberta_prazo_atingido(_world: &mut AppWorld) {
    pending_step("que a eleição está aberta e o prazo vigente foi atingido");
}
