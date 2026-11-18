/// A consulta da gestão vigente e de seus registros históricos permite conhecer a organização e a trajetória do CA.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final governanceConsultationSteps = <StepDefinition>[
  pendingStep(
    r'^a ausência de permissão administrativa não impede a consulta$',
  ),
  pendingStep(
    r'^a composição da gestão anterior não é substituída pela nova composição$',
  ),
  pendingStep(r'^a consulta não exige uma permissão administrativa$'),
  pendingStep(
    r'^a consulta utiliza o contexto do Centro Acadêmico selecionado$',
  ),
  pendingStep(r'^a consulta é disponibilizada normalmente$'),
  pendingStep(
    r'^a exclusão desse cargo da estrutura atual do CA foi permitida$',
  ),
  pendingStep(
    r'^a exclusão do cargo da estrutura atual não apaga seu registro histórico$',
  ),
  pendingStep(r'^a gestão de "([^"]*)" continua disponível no histórico$'),
  pendingStep(r'^a gestão de "([^"]*)" é apresentada como a gestão vigente$'),
  pendingStep(r'^a gestão possui "([^"]*)" no cargo de "([^"]*)"$'),
  pendingStep(r'^acesso a consulta de governança do Centro Acadêmico ativo$'),
  pendingStep(
    r'^acesso as informações de governança sem alterar o contexto ativo$',
  ),
  pendingStep(r'^altero o Centro Acadêmico ativo para "([^"]*)"$'),
  pendingStep(
    r'^consigo identificar a participação histórica de cada integrante$',
  ),
  pendingStep(r'^consigo identificar o período de cada gestão registrada$'),
  pendingStep(r'^consigo identificar os respectivos períodos de mandato$'),
  pendingStep(r'^consigo identificar "([^"]*)" como "([^"]*)"$'),
  pendingStep(r'^consigo visualizar a gestão vigente$'),
  pendingStep(r'^consigo visualizar as datas de início e término do mandato$'),
  pendingStep(r'^consigo visualizar as gestões registradas$'),
  pendingStep(r'^consigo visualizar os cargos ocupados por cada integrante$'),
  pendingStep(r'^consigo visualizar os integrantes da gestão vigente$'),
  pendingStep(r'^consulto a composição histórica da gestão de "([^"]*)"$'),
  pendingStep(
    r'^consulto a composição vigente e o histórico de participação dessa gestão$',
  ),
  pendingStep(r'^consulto as informações de governança$'),
  pendingStep(
    r'^consulto as informações de governança do Centro Acadêmico ativo$',
  ),
  pendingStep(r'^consulto novamente as informações de governança$'),
  pendingStep(r'^consulto o histórico de gestões$'),
  pendingStep(r'^consulto o histórico de gestões do Centro Acadêmico ativo$'),
  pendingStep(
    r'^consulto os detalhes da gestão com mandato de "([^"]*)" a "([^"]*)"$',
  ),
  pendingStep(r'^encontro a gestão com mandato de "([^"]*)" a "([^"]*)"$'),
  pendingStep(
    r'^esse cargo fez parte da gestão com mandato de "([^"]*)" a "([^"]*)"$',
  ),
  pendingStep(r'^estou autenticado no CApp com uma conta de estudante$'),
  pendingStep(
    r'^existe uma gestão registrada com mandato de "([^"]*)" a "([^"]*)"$',
  ),
  pendingStep(r'^existe uma gestão vigente nesse Centro Acadêmico$'),
  pendingStep(r'^existe uma gestão vigente no "([^"]*)"$'),
  pendingStep(r'^existe uma gestão vigente registrada$'),
  pendingStep(r'^existe uma gestão vigente registrada nesse Centro Acadêmico$'),
  pendingStep(
    r'^existe uma nova gestão vigente com mandato de "([^"]*)" a "([^"]*)"$',
  ),
  pendingStep(r'^existem gestões anteriores registradas no CApp$'),
  pendingStep(
    r'^nenhuma gestão foi registrada nesse Centro Acadêmico desde sua adesão$',
  ),
  pendingStep(r'^não apresenta as gestões do "([^"]*)"$'),
  pendingStep(r'^não apresenta cargos atuais sem registro correspondente$'),
  pendingStep(
    r'^não apresenta cargos sem associação com uma gestão registrada$',
  ),
  pendingStep(
    r'^não apresenta integrantes como membros de uma gestão vigente$',
  ),
  pendingStep(
    r'^não apresenta integrantes sem participação histórica registrada$',
  ),
  pendingStep(
    r'^não possuo permissões administrativas nesse Centro Acadêmico$',
  ),
  pendingStep(r'^não são apresentadas informações exclusivas do "([^"]*)"$'),
  pendingStep(
    r'^não utiliza registros de outro Centro Acadêmico para preencher o histórico do CA ativo$',
  ),
  pendingStep(
    r'^não visualizo "([^"]*)" como integrante da gestão do CA ativo$',
  ),
  pendingStep(
    r'^o aplicativo informa que não existe gestão vigente registrada$',
  ),
  pendingStep(
    r'^o aplicativo informa que não existem gestões anteriores registradas$',
  ),
  pendingStep(
    r'^o aplicativo informa que não existem gestões registradas para o "([^"]*)"$',
  ),
  pendingStep(
    r'^o cargo de "([^"]*)" continua associado à participação histórica de "([^"]*)"$',
  ),
  pendingStep(
    r'^o cargo "([^"]*)" continua associado à participação histórica de "([^"]*)"$',
  ),
  pendingStep(
    r'^o mandato dessa gestão começa em "([^"]*)" e termina em "([^"]*)"$',
  ),
  pendingStep(r'^o vínculo ativo de "([^"]*)" com esse cargo foi encerrado$'),
  pendingStep(r'^o "([^"]*)" está selecionado como Centro Acadêmico ativo$'),
  pendingStep(r'^o "([^"]*)" possui gestões registradas no CApp$'),
  pendingStep(r'^o "([^"]*)" é o Centro Acadêmico ativo$'),
  pendingStep(r'^o "([^"]*)" é selecionado como contexto inicial$'),
  pendingStep(
    r'^os dados apresentados pertencem ao CA favorito selecionado como contexto inicial$',
  ),
  pendingStep(
    r'^que existe uma gestão encerrada com mandato de "([^"]*)" a "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe uma gestão vigente com mandato de "([^"]*)" a "([^"]*)"$',
  ),
  pendingStep(r'^que existe uma gestão vigente no "([^"]*)"$'),
  pendingStep(r'^que não faço parte da gestão do "([^"]*)"$'),
  pendingStep(
    r'^que não possuo cargo nem permissão administrativa no "([^"]*)"$',
  ),
  pendingStep(r'^que o "([^"]*)" aderiu ao CApp$'),
  pendingStep(r'^que o "([^"]*)" aderiu ao CApp em "([^"]*)"$'),
  pendingStep(r'^que o "([^"]*)" está cadastrado no CApp$'),
  pendingStep(r'^que o "([^"]*)" não possui gestões registradas no CApp$'),
  pendingStep(
    r'^que o "([^"]*)" não possui uma gestão vigente registrada no CApp$',
  ),
  pendingStep(r'^que o "([^"]*)" é o CA favorito da minha conta$'),
  pendingStep(r'^que "([^"]*)" ocupou o cargo personalizado de "([^"]*)"$'),
  pendingStep(r'^visualizo a gestão do "([^"]*)"$'),
  pendingStep(r'^visualizo as informações do "([^"]*)"$'),
  pendingStep(r'^visualizo o período de mandato dessa gestão$'),
  pendingStep(r'^visualizo "([^"]*)" como antiga "([^"]*)"$'),
  pendingStep(r'^visualizo "([^"]*)" como antigo "([^"]*)"$'),
  pendingStep(r'^visualizo "([^"]*)" como integrante da gestão do CA ativo$'),
  pendingStep(
    r'^"([^"]*)" continua apresentado como integrante daquela gestão$',
  ),
  pendingStep(
    r'^"([^"]*)" continua associada ao cargo de "([^"]*)" na gestão de "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" continua associado ao cargo de "([^"]*)" na gestão de "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" continua registrada no histórico de participação dessa gestão$',
  ),
  pendingStep(r'^"([^"]*)" integra a gestão de "([^"]*)" como "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" integra a gestão do "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" integrou a gestão de "([^"]*)" como "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" integrou essa gestão no cargo de "([^"]*)"$'),
  pendingStep(
    r'^"([^"]*)" não é apresentada entre os integrantes atualmente vinculados à gestão$',
  ),
  pendingStep(r'^"([^"]*)" ocupou o cargo de "([^"]*)" nessa gestão$'),
];
