import '../../support/step_registry.dart';

final accessControlSteps = <StepDefinition>[
  // -----------------------------------------
  // WHEN / QUANDO
  // Ações executadas pelo usuário
  // -----------------------------------------
  pendingStep(
    r'^ele tenta acessar um serviço restrito da comunidade acadêmica$',
  ),

  pendingStep(r'^ele tenta realizar uma ação que exige autenticação$'),

  // -----------------------------------------
  // THEN / ENTÃO
  // Verificações dos resultados esperados
  // -----------------------------------------
  pendingStep(r'^o acesso à plataforma permanece bloqueado$'),

  pendingStep(
    r'^o acesso ao recurso é bloqueado por ausência de autenticação$',
  ),

  pendingStep(
    r'^o aplicativo solicita que o usuário realize a autenticação institucional$',
  ),

  pendingStep(r'^a operação é impedida por ausência de autenticação$'),

  pendingStep(r'^a ação solicitada não é realizada$'),

  pendingStep(
    r'^o acesso ao recurso é bloqueado por falta de uma sessão válida$',
  ),

  pendingStep(
    r'^o acesso a recursos restritos passa a exigir nova autenticação$',
  ),
];
