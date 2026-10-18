import '../../support/step_registry.dart';

final academicDataSteps = <StepDefinition>[
  // -----------------------------------------
  // GIVEN / DADO
  // Preparação das condições iniciais
  // -----------------------------------------
  pendingStep(
    r'^que o estudante possui uma identidade institucional autenticada$',
  ),
  pendingStep(
    r'^a instituição disponibiliza o curso e a situação acadêmica do estudante$',
  ),
  pendingStep(
    r'^a instituição não disponibiliza os dados acadêmicos necessários para identificar seu contexto$',
  ),
  pendingStep(
    r'^a instituição não disponibiliza a informação "[^"]+" do estudante$',
  ),
  pendingStep(r'^não foi possível obter os dados acadêmicos da instituição$'),

  // -----------------------------------------
  // WHEN / QUANDO
  // Ações executadas pelo sistema
  // -----------------------------------------
  pendingStep(r'^o sistema obtém os dados acadêmicos do estudante$'),
  pendingStep(r'^o sistema tenta obter os dados acadêmicos do estudante$'),
  pendingStep(r'^o sistema tenta associar os dados acadêmicos ao perfil$'),
  pendingStep(r'^o sistema tenta concluir a associação dos dados acadêmicos$'),

  // -----------------------------------------
  // THEN / ENTÃO
  // Verificações dos resultados esperados
  // -----------------------------------------
  pendingStep(r'^o curso do estudante fica associado ao seu perfil$'),
  pendingStep(
    r'^a situação acadêmica do estudante fica associada ao seu perfil$',
  ),
  pendingStep(
    r'^essas informações ficam disponíveis para as regras de acesso do aplicativo$',
  ),
  pendingStep(
    r'^o sistema não associa dados acadêmicos que não foram recuperados$',
  ),
  pendingStep(
    r'^o perfil não apresenta esses dados como se estivessem confirmados$',
  ),
  pendingStep(
    r'^as regras de acesso não tratam os dados ausentes como informações acadêmicas confirmadas$',
  ),
  pendingStep(r'^a informação "[^"]+" não é apresentada como confirmada$'),
  pendingStep(
    r'^o sistema não considera completo o contexto acadêmico do estudante$',
  ),
  pendingStep(r'^o sistema não confirma uma associação que não foi concluída$'),
  pendingStep(
    r'^as informações acadêmicas não obtidas permanecem indisponíveis para as regras de acesso$',
  ),
];
