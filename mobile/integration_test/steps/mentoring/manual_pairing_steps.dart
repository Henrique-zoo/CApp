/// O pareamento manual precisa relacionar participantes válidos sob autorização do CA do programa.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final manualPairingSteps = <StepDefinition>[
  pendingStep(r'^a associação entre os dois participantes é registrada$'),
  pendingStep(r'^a seleção é recusada$'),
  pendingStep(r'^associa outro participante como afilhado$'),
  pendingStep(r'^ele seleciona um participante como padrinho$'),
  pendingStep(
    r'^ele tenta confirmar o pareamento sem selecionar um padrinho ou um afilhado$',
  ),
  pendingStep(r'^ele tenta selecionar essa pessoa como padrinho ou afilhado$'),
  pendingStep(r'^existe uma pessoa que não está inscrita no programa$'),
  pendingStep(r'^existem participantes com inscrição válida no programa$'),
  pendingStep(r'^nenhum pareamento é registrado$'),
  pendingStep(r'^nenhum pareamento é registrado com essa pessoa$'),
  pendingStep(r'^o gestor possui a permissão "([^"]*)" no CA do programa$'),
  pendingStep(
    r'^o gestor é informado de que precisa selecionar os dois participantes$',
  ),
  pendingStep(
    r'^o integrante da gestão não possui a permissão "([^"]*)" nesse CA$',
  ),
  pendingStep(
    r'^o integrante possui a permissão "([^"]*)" somente em outro CA$',
  ),
  pendingStep(r'^o pareamento fica disponível na organização do programa$'),
  pendingStep(r'^o pareamento não é registrado$'),
  pendingStep(r'^os pareamentos do programa permanecem inalterados$'),
  pendingStep(r'^que o programa de apadrinhamento está configurado$'),
  pendingStep(r'^tenta confirmar um pareamento manual$'),
  pendingStep(r'^tenta confirmar um pareamento manual neste programa$'),
];
