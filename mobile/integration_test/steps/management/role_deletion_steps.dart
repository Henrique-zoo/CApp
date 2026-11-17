/// A exclusão de cargos precisa respeitar os vínculos ativos e preservar as gestões anteriores.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final roleDeletionSteps = <StepDefinition>[
  pendingStep(
    r'^a estrutura atual de cargos de "([^"]*)" não lista o cargo "([^"]*)"$',
  ),
  pendingStep(r'^a estrutura de cargos de "([^"]*)" permanece inalterada$'),
  pendingStep(
    r'^a gestão anterior de 2024 continua exibindo "([^"]*)" no cargo "([^"]*)"$',
  ),
  pendingStep(
    r'^o CA "([^"]*)" mantém os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(
    r'^o CA "([^"]*)" possui os cargos "([^"]*)", "([^"]*)", "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(
    r'^o cargo "([^"]*)" de "([^"]*)" mantém a permissão de "([^"]*)"$',
  ),
  pendingStep(
    r'^o cargo "([^"]*)" de "([^"]*)" possui a permissão de "([^"]*)"$',
  ),
  pendingStep(r'^o gestor "([^"]*)" exclui o cargo "([^"]*)" de "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" tenta excluir o cargo "([^"]*)" de "([^"]*)"$',
  ),
  pendingStep(r'^que o CA "([^"]*)" não possui o cargo "([^"]*)"$'),
  pendingStep(
    r'^que o CA "([^"]*)" possui a gestão anterior de 2024 com "([^"]*)" no cargo "([^"]*)"$',
  ),
  pendingStep(
    r'^que o estudante "([^"]*)" não possui cargo de gestão em "([^"]*)"$',
  ),
  pendingStep(
    r'^que o gestor "([^"]*)" excluiu o cargo "([^"]*)" de "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" tenta excluir o cargo "([^"]*)" de "([^"]*)"$'),
];
