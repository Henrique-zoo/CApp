/// Curso e situação acadêmica confirmados orientam o contexto e as regras de acesso do estudante.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final academicDataSteps = <StepDefinition>[
  pendingStep(r'^a informação "([^"]*)" não é apresentada como confirmada$'),
  pendingStep(
    r'^a instituição disponibiliza o curso e a situação acadêmica do estudante$',
  ),
  pendingStep(
    r'^a instituição não disponibiliza a informação "([^"]*)" do estudante$',
  ),
  pendingStep(
    r'^a instituição não disponibiliza os dados acadêmicos necessários para identificar seu contexto$',
  ),
  pendingStep(
    r'^a situação acadêmica do estudante fica associada ao seu perfil$',
  ),
  pendingStep(
    r'^as informações acadêmicas não obtidas permanecem indisponíveis para as regras de acesso$',
  ),
  pendingStep(
    r'^as regras de acesso não tratam os dados ausentes como informações acadêmicas confirmadas$',
  ),
  pendingStep(
    r'^essas informações ficam disponíveis para as regras de acesso do aplicativo$',
  ),
  pendingStep(r'^não foi possível obter os dados acadêmicos da instituição$'),
  pendingStep(r'^o curso do estudante fica associado ao seu perfil$'),
  pendingStep(
    r'^o perfil não apresenta esses dados como se estivessem confirmados$',
  ),
  pendingStep(
    r'^o sistema não associa dados acadêmicos que não foram recuperados$',
  ),
  pendingStep(r'^o sistema não confirma uma associação que não foi concluída$'),
  pendingStep(
    r'^o sistema não considera completo o contexto acadêmico do estudante$',
  ),
  pendingStep(r'^o sistema obtém os dados acadêmicos do estudante$'),
  pendingStep(r'^o sistema tenta associar os dados acadêmicos ao perfil$'),
  pendingStep(r'^o sistema tenta concluir a associação dos dados acadêmicos$'),
  pendingStep(r'^o sistema tenta obter os dados acadêmicos do estudante$'),
  pendingStep(
    r'^que o estudante possui uma identidade institucional autenticada$',
  ),
];
