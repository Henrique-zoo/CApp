import '../../support/step_registry.dart';

final sessionSteps = <StepDefinition>[
  // -----------------------------------------
  // GIVEN / DADO
  // Preparação das condições iniciais
  // -----------------------------------------
  pendingStep(r'^o usuário não está autenticado no aplicativo$'),

  pendingStep(r'^o estudante possui uma sessão "([^"]+)"$'),

  pendingStep(r'^o estudante possui uma sessão autenticada válida$'),

  // -----------------------------------------
  // WHEN / QUANDO
  // Ações executadas pelo usuário
  // -----------------------------------------
  pendingStep(r'^ele solicita sair do CApp$'),

  // -----------------------------------------
  // THEN / ENTÃO
  // Verificações dos resultados esperados
  // -----------------------------------------
  pendingStep(r'^uma sessão autenticada é estabelecida para o estudante$'),

  pendingStep(
    r'^a identidade autenticada do usuário fica disponível com o e-mail "([^"]+)"$',
  ),

  pendingStep(r'^o sistema estabelece a sessão autenticada do usuário$'),

  pendingStep(r'^nenhuma sessão autenticada é estabelecida$'),

  pendingStep(r'^a sessão do estudante é encerrada no aplicativo$'),
];
