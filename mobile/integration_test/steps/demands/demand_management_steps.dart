/// O gerenciamento de demandas permite à gestão do CA analisar solicitações e atualizar seus status.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final demandManagementSteps = <StepDefinition>[
  pendingStep(
    r'^a alteração de status é recusada informando que a justificativa é obrigatória$',
  ),
  pendingStep(
    r'^a alteração de status é recusada informando que demandas de espaço físico não podem ser encaminhadas à coordenação$',
  ),
  pendingStep(
    r'^a alteração de status é recusada informando transição inválida$',
  ),
  pendingStep(
    r'^a demanda "([^"]*)" não é apresentada para a gestão do "([^"]*)"$',
  ),
  pendingStep(
    r'^a demanda institucional registra o encaminhamento à coordenação do curso$',
  ),
  pendingStep(
    r'^atualizo o status da demanda "([^"]*)" para "([^"]*)" com a justificativa "([^"]*)"$',
  ),
  pendingStep(r'^consulto administrativamente as demandas do espaço físico$'),
  pendingStep(
    r'^consulto administrativamente as demandas do fórum institucional$',
  ),
  pendingStep(
    r'^existe a demanda institucional "([^"]*)" associada somente ao "([^"]*)"$',
  ),
  pendingStep(
    r'^existe a demanda institucional "([^"]*)" com o status "([^"]*)"$',
  ),
  pendingStep(
    r'^existe a demanda institucional "([^"]*)" no "([^"]*)" com o status "([^"]*)"$',
  ),
  pendingStep(
    r'^filtro as demandas institucionais pelo status "([^"]*)"$',
  ),
  pendingStep(r'^não vejo a demanda "([^"]*)" na listagem$'),
  pendingStep(
    r'^o histórico registra a alteração realizada por "([^"]*)"$',
  ),
  pendingStep(r'^o parecer registrado na demanda é "([^"]*)"$'),
  pendingStep(
    r'^o status atual da demanda "([^"]*)" passa a ser "([^"]*)"$',
  ),
  pendingStep(r'^o status da demanda "([^"]*)" permanece "([^"]*)"$'),
  pendingStep(
    r'^que a demanda institucional "([^"]*)" possui o status "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe a demanda "([^"]*)" sobre o espaço físico com o status "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe a demanda institucional "([^"]*)" com o status "([^"]*)"$',
  ),
  pendingStep(
    r'^que existem as seguintes demandas no fórum do espaço físico:$',
  ),
  pendingStep(
    r'^que existem as seguintes demandas no fórum institucional do curso:$',
  ),
  pendingStep(
    r'^tento atualizar o status da demanda "([^"]*)" para "([^"]*)" com a justificativa "([^"]*)"$',
  ),
  pendingStep(
    r'^tento atualizar o status da demanda "([^"]*)" para "([^"]*)" sem informar justificativa$',
  ),
  pendingStep(r'^vejo a demanda "([^"]*)" com o status "([^"]*)"$'),
  pendingStep(
    r'^vejo a demanda "([^"]*)" com o status "([^"]*)" e autor "([^"]*)"$',
  ),
  pendingStep(r'^vejo a demanda "([^"]*)" na listagem$'),
];
