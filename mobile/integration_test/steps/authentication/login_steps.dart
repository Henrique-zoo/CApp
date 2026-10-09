import '../../support/step_registry.dart';

final loginSteps = <StepDefinition>[
  // -----------------------------------------
  // GIVEN / DADO
  // Preparação das condições iniciais
  // -----------------------------------------
  pendingStep(r'^o estudante iniciou o login institucional$'),

  // -----------------------------------------
  // WHEN / QUANDO
  // Ações executadas pelo usuário
  // -----------------------------------------
  pendingStep(
    r'^ele realiza a autenticação com suas credenciais institucionais$',
  ),

  pendingStep(r'^ele conclui a validação de sua identidade institucional$'),

  pendingStep(r'^ele tenta realizar o login institucional$'),

  pendingStep(r'^ele tenta autenticar no CApp$'),

  pendingStep(r'^ele tenta realizar o login institucional com essa conta$'),

  pendingStep(r'^ele cancela a autenticação antes de concluí-la$'),

  // -----------------------------------------
  // THEN / ENTÃO
  // Verificações dos resultados esperados
  // -----------------------------------------
  pendingStep(r'^o acesso à plataforma é concedido com sucesso$'),

  pendingStep(r'^não é solicitado cadastro manual para entrar no CApp$'),

  pendingStep(r'^a autenticação é recusada indicando credenciais inválidas$'),

  pendingStep(
    r'^o aplicativo informa que não foi possível autenticar a conta institucional$',
  ),

  pendingStep(
    r'^o aplicativo informa que é necessário utilizar uma conta institucional da UnB$',
  ),

  pendingStep(r'^o estudante pode iniciar uma nova tentativa de login$'),

  pendingStep(
    r'^o aplicativo informa a indisponibilidade temporária da autenticação$',
  ),
];
