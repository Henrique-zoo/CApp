/// O fórum do espaço físico permite comunicar necessidades do CA sem exigir vínculo administrativo.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final demandCreationSteps = <StepDefinition>[
  pendingStep(r'^a demanda fica associada somente ao "([^"]*)"$'),
  pendingStep(r'^a demanda não é criada$'),
  pendingStep(r'^a demanda não é encaminhada à coordenação do curso$'),
  pendingStep(r'^a demanda é criada no fórum do espaço físico do "([^"]*)"$'),
  pendingStep(r'^não tenho vínculo de gestão com "([^"]*)"$'),
  pendingStep(r'^o aplicativo informa que o conteúdo é obrigatório$'),
  pendingStep(r'^o autor da demanda é "([^"]*)"$'),
  pendingStep(r'^o conteúdo registrado é "([^"]*)"$'),
  pendingStep(r'^o status inicial da demanda é "([^"]*)"$'),
  pendingStep(
    r'^registro uma demanda sobre o espaço físico com o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^tento registrar uma demanda sobre o espaço físico sem informar o conteúdo$',
  ),
];
