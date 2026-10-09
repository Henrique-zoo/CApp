import '../../support/step_registry.dart';

final institutionalAccountSteps = <StepDefinition>[
  // -----------------------------------------
  // GIVEN / DADO
  // Preparação das condições iniciais
  // -----------------------------------------
  pendingStep(
    r'^o estudante possui uma conta institucional ativa da UnB com o e-mail "([^"]+)"$',
  ),

  pendingStep(
    r'^um estudante com o e-mail "([^"]+)" realiza o primeiro acesso com conta institucional ativa da UnB$',
  ),

  pendingStep(
    r'^um usuário possui credenciais institucionais inválidas para o e-mail "([^"]+)"$',
  ),

  pendingStep(
    r'^o estudante possui uma conta institucional com o e-mail "([^"]+)" revogada pela UnB$',
  ),

  pendingStep(r'^o serviço de autenticação recusa o acesso dessa conta$'),

  pendingStep(r'^o usuário possui uma conta pessoal Microsoft válida$'),

  pendingStep(
    r'^o usuário possui uma conta institucional Microsoft válida de outra universidade$',
  ),

  pendingStep(
    r'^o serviço de autenticação confirma que a conta pertence a essa outra universidade$',
  ),

  pendingStep(r'^o serviço de autenticação está temporariamente indisponível$'),
];
