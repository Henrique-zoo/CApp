//! Passos reservados para a apuração, a divulgação e a auditoria eleitorais.
//!
//! A especificação descreve resultados observáveis. Os corpos permanecem
//! pendentes até o runner usar o fluxo eleitoral e as evidências reais.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito secreto cujo prazo de votação ainda está vigente$"#)]
fn given_pleito_em_votacao(_world: &mut AppWorld) {
    pending_step("um pleito secreto cujo prazo de votação ainda está vigente");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^a comissão tenta iniciar a apuração desse pleito$"#)]
fn when_solicita_apuracao(_world: &mut AppWorld) {
    pending_step("a comissão tenta iniciar a apuração desse pleito");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^a apuração é recusada e as contagens por chapa permanecem indisponíveis$"#)]
fn then_apuracao_recusada(_world: &mut AppWorld) {
    pending_step("a apuração é recusada e as contagens por chapa permanecem indisponíveis");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito secreto encerrado com dois votos para Aurora e um para Horizonte$"#)]
fn given_pleito_com_tres_cedulas(_world: &mut AppWorld) {
    pending_step("um pleito secreto encerrado com dois votos para Aurora e um para Horizonte");
}

/// Passo pendente do recorte de apuração.
#[given(
    regex = r#"^todas as autoridades enviaram contribuições válidas para o agregado encerrado$"#
)]
fn given_todas_contribuicoes_validas(_world: &mut AppWorld) {
    pending_step("todas as autoridades enviaram contribuições válidas para o agregado encerrado");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^uma estudante consulta o resultado publicado do pleito$"#)]
fn when_consulta_resultado(_world: &mut AppWorld) {
    pending_step("uma estudante consulta o resultado publicado do pleito");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^o resultado da apuração informa dois votos para Aurora e um para Horizonte$"#)]
fn then_resultado_dois_aurora_um_horizonte(_world: &mut AppWorld) {
    pending_step("o resultado da apuração informa dois votos para Aurora e um para Horizonte");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito secreto encerrado em que falta a contribuição de uma autoridade$"#)]
fn given_pleito_com_autoridade_ausente(_world: &mut AppWorld) {
    pending_step("um pleito secreto encerrado em que falta a contribuição de uma autoridade");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^a apuração consta como incompleta, sem totais definitivos$"#)]
fn then_apuracao_incompleta(_world: &mut AppWorld) {
    pending_step("a apuração consta como incompleta, sem totais definitivos");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito secreto encerrado que aguarda contribuições$"#)]
fn given_pleito_aguarda_contribuicoes(_world: &mut AppWorld) {
    pending_step("um pleito secreto encerrado que aguarda contribuições");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^uma autoridade envia uma contribuição incompatível com o agregado encerrado$"#)]
fn when_envia_contribuicao_incompativel(_world: &mut AppWorld) {
    pending_step("uma autoridade envia uma contribuição incompatível com o agregado encerrado");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^a contribuição é recusada e a apuração continua incompleta$"#)]
fn then_recusa_contribuicao(_world: &mut AppWorld) {
    pending_step("a contribuição é recusada e a apuração continua incompleta");
}

/// Passo pendente do recorte de apuração.
#[given(
    regex = r#"^um pleito secreto encerrado cujos totais apurados divergem do número de eleitores$"#
)]
fn given_pleito_com_totais_divergentes(_world: &mut AppWorld) {
    pending_step(
        "um pleito secreto encerrado cujos totais apurados divergem do número de eleitores",
    );
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^o resultado é divulgado como anulado por inconsistência da contagem$"#)]
fn then_resultado_anulado(_world: &mut AppWorld) {
    pending_step("o resultado é divulgado como anulado por inconsistência da contagem");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito com resultado final publicado$"#)]
fn given_pleito_com_resultado_final(_world: &mut AppWorld) {
    pending_step("um pleito com resultado final publicado");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^uma estudante consulta a prestação de contas eleitoral$"#)]
fn when_consulta_prestacao_eleitoral(_world: &mut AppWorld) {
    pending_step("uma estudante consulta a prestação de contas eleitoral");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^vê o resultado e os dados públicos do processo$"#)]
fn then_ve_dados_publicos(_world: &mut AppWorld) {
    pending_step("vê o resultado e os dados públicos do processo");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^que uma eleitora guardou seu recibo e carregou o quadro público$"#)]
fn given_eleitora_guardou_recibo(_world: &mut AppWorld) {
    pending_step("que uma eleitora guardou seu recibo e carregou o quadro público");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^verifica a inclusão da própria cédula sem conexão$"#)]
fn when_verifica_inclusao_offline(_world: &mut AppWorld) {
    pending_step("verifica a inclusão da própria cédula sem conexão");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^obtém a confirmação de que seu recibo corresponde à entrada do quadro$"#)]
fn then_confirma_inclusao(_world: &mut AppWorld) {
    pending_step("obtém a confirmação de que seu recibo corresponde à entrada do quadro");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^uma cópia do quadro em que uma cédula foi adulterada$"#)]
fn given_copia_com_cedula_adulterada(_world: &mut AppWorld) {
    pending_step("uma cópia do quadro em que uma cédula foi adulterada");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^uma estudante verifica as evidências eleitorais no CApp$"#)]
fn when_verifica_evidencias_no_app(_world: &mut AppWorld) {
    pending_step("uma estudante verifica as evidências eleitorais no CApp");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^a auditoria sinaliza a adulteração do quadro eleitoral$"#)]
fn then_sinaliza_adulteracao(_world: &mut AppWorld) {
    pending_step("a auditoria sinaliza a adulteração do quadro eleitoral");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^uma cópia internamente coerente do quadro sem âncora externa confiável$"#)]
fn given_copia_sem_ancora_externa(_world: &mut AppWorld) {
    pending_step("uma cópia internamente coerente do quadro sem âncora externa confiável");
}

/// Passo pendente do recorte de apuração.
#[then(
    regex = r#"^a auditoria informa que as evidências são insuficientes para atestar a completude$"#
)]
fn then_completude_inconclusiva(_world: &mut AppWorld) {
    pending_step(
        "a auditoria informa que as evidências são insuficientes para atestar a completude",
    );
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito com resultado final apto a declarar uma chapa eleita$"#)]
fn given_pleito_apto_para_posse(_world: &mut AppWorld) {
    pending_step("um pleito com resultado final apto a declarar uma chapa eleita");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^a conclusão da eleição é processada duas vezes$"#)]
fn when_processa_conclusao_duas_vezes(_world: &mut AppWorld) {
    pending_step("a conclusão da eleição é processada duas vezes");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^há uma única gestão sucessora correspondente à chapa eleita$"#)]
fn then_gestao_sucessora_unica(_world: &mut AppWorld) {
    pending_step("há uma única gestão sucessora correspondente à chapa eleita");
}

/// Passo pendente do recorte de apuração.
#[given(regex = r#"^um pleito cancelado, anulado ou sem resultado apto a declarar vencedora$"#)]
fn given_pleito_sem_resultado_apto(_world: &mut AppWorld) {
    pending_step("um pleito cancelado, anulado ou sem resultado apto a declarar vencedora");
}

/// Passo pendente do recorte de apuração.
#[when(regex = r#"^é consultada a sucessão da gestão do Centro Acadêmico$"#)]
fn when_consulta_sucessao(_world: &mut AppWorld) {
    pending_step("é consultada a sucessão da gestão do Centro Acadêmico");
}

/// Passo pendente do recorte de apuração.
#[then(regex = r#"^o pleito não efetiva uma nova gestão$"#)]
fn then_sem_posse(_world: &mut AppWorld) {
    pending_step("o pleito não efetiva uma nova gestão");
}
