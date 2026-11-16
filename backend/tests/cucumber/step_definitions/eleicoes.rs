//! Configuração, calendário e cancelamento do processo eleitoral.
//!
//! A comissão escolhe votação secreta ou identificada ao criar a eleição. Na
//! modalidade secreta, o contrato #377 permite chapa única com autoridades
//! independentes. O calendário distingue encerramento válido de cancelamento.
//!
//! Estes registros mantêm a especificação ligada ao runner até as funcionalidades
//! existirem. Cada corpo chama pending_step, sem simular autorização, persistência
//! ou apuração. Expressões reutilizadas pela votação são registradas em shared_steps
//! para conservar resolução única.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que a comissão eleitoral foi indicada pela gestão do CA.
#[given(regex = r#"^que a comissão eleitoral foi indicada pela gestão do CA$"#)]
fn given_que_a_comissao_eleitoral_foi_indicada_pela_gestao_do_ca(_world: &mut AppWorld) {
    pending_step("que a comissão eleitoral foi indicada pela gestão do CA");
}

/// Pendente: estou autenticado como integrante autorizado dessa comissão.
#[given(regex = r#"^estou autenticado como integrante autorizado dessa comissão$"#)]
fn given_estou_autenticado_como_integrante_autorizado_dessa_comissao(_world: &mut AppWorld) {
    pending_step("estou autenticado como integrante autorizado dessa comissão");
}

/// Pendente: que estou preenchendo os dados de uma nova eleição do CA.
#[given(regex = r#"^que estou preenchendo os dados de uma nova eleição do CA$"#)]
fn given_estou_preenchendo_nova_eleicao(_world: &mut AppWorld) {
    pending_step("que estou preenchendo os dados de uma nova eleição do CA");
}

/// Pendente: seleciono votação {string} e concluo a criação da eleição.
#[when(regex = r#"^seleciono votação "(secreta|identificada)" e concluo a criação da eleição$"#)]
fn when_seleciono_modalidade_e_crio_eleicao(_world: &mut AppWorld, _modalidade: String) {
    pending_step("seleciono votação {string} e concluo a criação da eleição");
}

/// Pendente: a eleição é criada com votação {string}.
#[then(regex = r#"^a eleição é criada com votação "(secreta|identificada)"$"#)]
fn then_eleicao_criada_com_modalidade(_world: &mut AppWorld, _modalidade: String) {
    pending_step("a eleição é criada com votação {string}");
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

/// Pendente: ainda não integra a comissão eleitoral de {string}.
#[given(regex = r#"^ainda não integra a comissão eleitoral de "([^"]*)"$"#)]
fn given_ainda_nao_integra_comissao(_world: &mut AppWorld, _ca: String) {
    pending_step("ainda não integra a comissão eleitoral de {string}");
}

/// Pendente: o gestor {string} designa {string} para a comissão eleitoral de {string}.
#[when(regex = r#"^o gestor "([^"]*)" designa "([^"]*)" para a comissão eleitoral de "([^"]*)"$"#)]
fn when_gestor_designa_comissao(
    _world: &mut AppWorld,
    _gestor: String,
    _integrante: String,
    _ca: String,
) {
    pending_step("o gestor {string} designa {string} para a comissão eleitoral de {string}");
}

/// Pendente: {string} passa a integrar a comissão eleitoral de {string}.
#[then(regex = r#"^"([^"]*)" passa a integrar a comissão eleitoral de "([^"]*)"$"#)]
fn then_passou_a_integrar_comissao(_world: &mut AppWorld, _integrante: String, _ca: String) {
    pending_step("{string} passa a integrar a comissão eleitoral de {string}");
}

/// Pendente: que {string} integra a comissão eleitoral de {string}.
#[given(regex = r#"^que "([^"]*)" integra a comissão eleitoral de "([^"]*)"$"#)]
fn given_que_integra_comissao(_world: &mut AppWorld, _integrante: String, _ca: String) {
    pending_step("que {string} integra a comissão eleitoral de {string}");
}

/// Pendente: {string} integra a comissão eleitoral de {string}.
#[given(regex = r#"^"([^"]*)" integra a comissão eleitoral de "([^"]*)"$"#)]
fn given_integra_comissao(_world: &mut AppWorld, _integrante: String, _ca: String) {
    pending_step("{string} integra a comissão eleitoral de {string}");
}

/// Pendente: não integra a comissão eleitoral de {string}.
#[given(regex = r#"^não integra a comissão eleitoral de "([^"]*)"$"#)]
fn given_nao_integra_comissao(_world: &mut AppWorld, _ca: String) {
    pending_step("não integra a comissão eleitoral de {string}");
}

/// Pendente: possui a permissão {string} em {string}.
#[given(regex = r#"^possui a permissão "([^"]*)" em "([^"]*)"$"#)]
fn given_possui_permissao_eleitoral(_world: &mut AppWorld, _permissao: String, _ca: String) {
    pending_step("possui a permissão {string} em {string}");
}

/// Pendente: não possui a permissão {string} em {string}.
#[given(regex = r#"^não possui a permissão "([^"]*)" em "([^"]*)"$"#)]
fn given_nao_possui_permissao_eleitoral(_world: &mut AppWorld, _permissao: String, _ca: String) {
    pending_step("não possui a permissão {string} em {string}");
}

/// Pendente: tenta apurar o resultado de uma eleição de {string}.
#[when(regex = r#"^tenta apurar o resultado de uma eleição de "([^"]*)"$"#)]
fn when_tenta_apurar_resultado(_world: &mut AppWorld, _ca: String) {
    pending_step("tenta apurar o resultado de uma eleição de {string}");
}

/// Pendente: tenta alterar a configuração de uma eleição de {string}.
#[when(regex = r#"^tenta alterar a configuração de uma eleição de "([^"]*)"$"#)]
fn when_tenta_alterar_configuracao(_world: &mut AppWorld, _ca: String) {
    pending_step("tenta alterar a configuração de uma eleição de {string}");
}

/// Pendente: tenta configurar uma eleição de {string}.
#[when(regex = r#"^tenta configurar uma eleição de "([^"]*)"$"#)]
fn when_tenta_configurar_eleicao(_world: &mut AppWorld, _ca: String) {
    pending_step("tenta configurar uma eleição de {string}");
}

/// Pendente: realizou ações administrativas em uma eleição de {string}.
#[given(regex = r#"^realizou ações administrativas em uma eleição de "([^"]*)"$"#)]
fn given_realizou_acoes_administrativas(_world: &mut AppWorld, _ca: String) {
    pending_step("realizou ações administrativas em uma eleição de {string}");
}

/// Pendente: o gestor {string} revoga a designação de {string} da comissão eleitoral de {string}.
#[when(
    regex = r#"^o gestor "([^"]*)" revoga a designação de "([^"]*)" da comissão eleitoral de "([^"]*)"$"#
)]
fn when_gestor_revoga_comissao(
    _world: &mut AppWorld,
    _gestor: String,
    _integrante: String,
    _ca: String,
) {
    pending_step(
        "o gestor {string} revoga a designação de {string} da comissão eleitoral de {string}",
    );
}

/// Pendente: {string} deixa de integrar a comissão eleitoral de {string}.
#[then(regex = r#"^"([^"]*)" deixa de integrar a comissão eleitoral de "([^"]*)"$"#)]
fn then_deixa_de_integrar_comissao(_world: &mut AppWorld, _integrante: String, _ca: String) {
    pending_step("{string} deixa de integrar a comissão eleitoral de {string}");
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

/// Pendente: {string} tenta designar {string} para a comissão eleitoral de {string}.
#[when(regex = r#"^"([^"]*)" tenta designar "([^"]*)" para a comissão eleitoral de "([^"]*)"$"#)]
fn when_integrante_tenta_designar_comissao(
    _world: &mut AppWorld,
    _ator: String,
    _integrante: String,
    _ca: String,
) {
    pending_step("{string} tenta designar {string} para a comissão eleitoral de {string}");
}

// -----------------------------------------------------------------------------
// Validação do período, das autoridades e da publicação do manifesto — Issue #124.
// -----------------------------------------------------------------------------

/// Pendente: informo um período de votação válido e concluo a criação da eleição.
#[when(regex = r#"^informo um período de votação válido e concluo a criação da eleição$"#)]
fn when_informa_periodo_valido(_world: &mut AppWorld) {
    pending_step("informo um período de votação válido e concluo a criação da eleição");
}

/// Pendente: o calendário inicial registra o início e o encerramento informados.
#[then(regex = r#"^o calendário inicial registra o início e o encerramento informados$"#)]
fn then_calendario_inicial_registra_periodo(_world: &mut AppWorld) {
    pending_step("o calendário inicial registra o início e o encerramento informados");
}

/// Pendente: informo um período cujo encerramento é anterior ao início e tento criar a eleição.
#[when(
    regex = r#"^informo um período cujo encerramento é anterior ao início e tento criar a eleição$"#
)]
fn when_informa_periodo_com_fim_anterior_inicio(_world: &mut AppWorld) {
    pending_step(
        "informo um período cujo encerramento é anterior ao início e tento criar a eleição",
    );
}

/// Pendente: informo um período cujo encerramento é igual ao início e tento criar a eleição.
#[when(
    regex = r#"^informo um período cujo encerramento é igual ao início e tento criar a eleição$"#
)]
fn when_informa_periodo_com_fim_igual_inicio(_world: &mut AppWorld) {
    pending_step("informo um período cujo encerramento é igual ao início e tento criar a eleição");
}

/// Pendente: a criação da eleição é recusada por período inválido.
#[then(regex = r#"^a criação da eleição é recusada por período inválido$"#)]
fn then_criacao_eleicao_recusada_periodo_invalido(_world: &mut AppWorld) {
    pending_step("a criação da eleição é recusada por período inválido");
}

/// Pendente: que a eleição está em preparação sem todas as autoridades definidas.
#[given(regex = r#"^que a eleição está em preparação sem todas as autoridades definidas$"#)]
fn given_eleicao_preparacao_sem_todas_autoridades(_world: &mut AppWorld) {
    pending_step("que a eleição está em preparação sem todas as autoridades definidas");
}

/// Pendente: a comissão solicita a abertura da votação.
#[when(regex = r#"^a comissão solicita a abertura da votação$"#)]
fn when_comissao_solicita_abertura_votacao(_world: &mut AppWorld) {
    pending_step("a comissão solicita a abertura da votação");
}

/// Pendente: a abertura é recusada por falta de autoridades.
#[then(regex = r#"^a abertura é recusada por falta de autoridades$"#)]
fn then_abertura_recusada_falta_autoridades(_world: &mut AppWorld) {
    pending_step("a abertura é recusada por falta de autoridades");
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

/// Pendente: a abertura é aceita após a validação de todas as autoridades.
#[then(regex = r#"^a abertura é aceita após a validação de todas as autoridades$"#)]
fn then_abertura_aceita_autoridades_validadas(_world: &mut AppWorld) {
    pending_step("a abertura é aceita após a validação de todas as autoridades");
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

/// Pendente: o manifesto é assinado pelo servidor e permanece imutável após a abertura.
#[then(regex = r#"^o manifesto é assinado pelo servidor e permanece imutável após a abertura$"#)]
fn then_manifesto_assinado_e_immutavel(_world: &mut AppWorld) {
    pending_step("o manifesto é assinado pelo servidor e permanece imutável após a abertura");
}

/// Pendente: que a eleição foi aberta e possui manifesto publicado.
#[given(regex = r#"^que a eleição foi aberta e possui manifesto publicado$"#)]
fn given_eleicao_aberta_manifesto_publicado(_world: &mut AppWorld) {
    pending_step("que a eleição foi aberta e possui manifesto publicado");
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

/// Pendente: {string} não passa a integrar a comissão eleitoral de {string}.
#[then(regex = r#"^"([^"]*)" não passa a integrar a comissão eleitoral de "([^"]*)"$"#)]
fn then_nao_passou_a_integrar_comissao(_world: &mut AppWorld, _integrante: String, _ca: String) {
    pending_step("{string} não passa a integrar a comissão eleitoral de {string}");
}

// -----------------------------------------------------------------------------
// Permissões específicas da comissão eleitoral — Issue #124.
// -----------------------------------------------------------------------------

/// Pendente: possui a permissão administrativa {string} em {string}.
#[given(regex = r#"^possui a permissão administrativa "([^"]*)" em "([^"]*)"$"#)]
fn given_possui_permissao_administrativa(_world: &mut AppWorld, _permissao: String, _ca: String) {
    pending_step("possui a permissão administrativa {string} em {string}");
}

/// Pendente: configura uma eleição de {string}.
#[when(regex = r#"^configura uma eleição de "([^"]*)"$"#)]
fn when_configura_eleicao(_world: &mut AppWorld, _ca: String) {
    pending_step("configura uma eleição de {string}");
}

/// Pendente: a configuração da eleição é autorizada.
#[then(regex = r#"^a configuração da eleição é autorizada$"#)]
fn then_configuracao_eleicao_autorizada(_world: &mut AppWorld) {
    pending_step("a configuração da eleição é autorizada");
}

/// Pendente: administra as chapas de uma eleição de {string}.
#[when(regex = r#"^administra as chapas de uma eleição de "([^"]*)"$"#)]
fn when_administra_chapas_eleicao(_world: &mut AppWorld, _ca: String) {
    pending_step("administra as chapas de uma eleição de {string}");
}

/// Pendente: a administração das chapas é concluída.
#[then(regex = r#"^a administração das chapas é concluída$"#)]
fn then_administracao_chapas_concluida(_world: &mut AppWorld) {
    pending_step("a administração das chapas é concluída");
}

/// Pendente: apura o resultado de uma eleição de {string}.
#[when(regex = r#"^apura o resultado de uma eleição de "([^"]*)"$"#)]
fn when_apura_resultado_eleicao(_world: &mut AppWorld, _ca: String) {
    pending_step("apura o resultado de uma eleição de {string}");
}

/// Pendente: a apuração é concluída.
#[then(regex = r#"^a apuração é concluída$"#)]
fn then_apuracao_concluida(_world: &mut AppWorld) {
    pending_step("a apuração é concluída");
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

/// Pendente: que a eleição está aberta e o prazo vigente foi atingido.
#[given(regex = r#"^que a eleição está aberta e o prazo vigente foi atingido$"#)]
fn given_eleicao_aberta_prazo_atingido(_world: &mut AppWorld) {
    pending_step("que a eleição está aberta e o prazo vigente foi atingido");
}
