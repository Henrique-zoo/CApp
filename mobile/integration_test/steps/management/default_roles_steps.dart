/// O cadastro de um CA precisa entregar sua estrutura inicial de cargos e preservar a presidência.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final defaultRolesSteps = <StepDefinition>[
  pendingStep(r'^esse cargo possui a permissão "([^"]*)"$'),
  pendingStep(
    r'^o CA "([^"]*)" passa a possuir exatamente os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(r'^o administrador "([^"]*)" administra a plataforma$'),
  pendingStep(
    r'^o administrador "([^"]*)" tenta excluir o cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^o cargo "([^"]*)" fica vinculado ao responsável informado na gestão inicial$',
  ),
  pendingStep(r'^o estudante "([^"]*)" cadastra o Centro Acadêmico "([^"]*)"$'),
  pendingStep(r'^o estudante "([^"]*)" exerce o cargo "([^"]*)" em "([^"]*)"$'),
  pendingStep(
    r'^os cargos "([^"]*)" e "([^"]*)" ficam disponíveis para novas vinculações$',
  ),
  pendingStep(
    r'^que o estudante informa a si mesmo como Presidente da gestão inicial de "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" remove o cargo "([^"]*)" de "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" tenta excluir o cargo "([^"]*)" em "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" tenta remover o cargo "([^"]*)" de "([^"]*)"$'),
];
