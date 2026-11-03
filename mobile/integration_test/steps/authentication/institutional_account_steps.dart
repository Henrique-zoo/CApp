import '../../support/step_registry.dart';

final institutionalAccountSteps = <StepDefinition>[
  // -----------------------------------------
  // GIVEN / DADO
  // Preparação das condições iniciais
  // -----------------------------------------
  pendingStep(
    r'^que a instituição "([^"]+)" está cadastrada no CApp$',
  ),

  pendingStep(
    r'^que a instituição "([^"]+)" não está cadastrada no CApp$',
  ),

  pendingStep(
    r'^que o estudante possui uma conta institucional válida da instituição "([^"]+)" com o e-mail "([^"]+)"$',
  ),

  pendingStep(
    r'^que um estudante com o e-mail "([^"]+)" realiza o primeiro acesso com conta institucional válida da instituição "([^"]+)"$',
  ),

  pendingStep(
    r'^a instituição "([^"]+)" reconhece essa conta como estudante$',
  ),

  pendingStep(
    r'^a instituição "([^"]+)" não reconhece essa conta como estudante$',
  ),

  pendingStep(r'^a conta informa o curso "([^"]+)"$'),

  pendingStep(
    r'^que um usuário possui credenciais institucionais inválidas para o e-mail "([^"]+)"$',
  ),

  pendingStep(
    r'^que o estudante possui uma conta institucional com o e-mail "([^"]+)" revogada pela instituição "([^"]+)"$',
  ),

  pendingStep(r'^o serviço de autenticação recusa o acesso dessa conta$'),

  pendingStep(r'^que o usuário possui uma conta pessoal Microsoft válida$'),

  pendingStep(
    r'^o serviço de autenticação confirma que a conta pertence à instituição "([^"]+)"$',
  ),

  pendingStep(r'^o serviço de autenticação está temporariamente indisponível$'),
];
