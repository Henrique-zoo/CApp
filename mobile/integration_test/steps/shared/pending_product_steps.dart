/// O vocabulário comum dos cenários precisa ter um único registro entre as funcionalidades.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final pendingProductSteps = <StepDefinition>[
  pendingStep(
    r'^a operação é recusada informando que o cargo de Presidente não pode ser excluído$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o cargo não existe no CA$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o cargo possui integrante ativo$',
  ),
  pendingStep(r'^a operação é recusada por falta de permissão$'),
  pendingStep(r'^altero o CA ativo para "([^"]*)"$'),
  pendingStep(r'^estou autenticado como "([^"]*)"$'),
  pendingStep(r'^existe o material publicado "([^"]*)" nesse CA$'),
  pendingStep(r'^não recebeu uma concessão separada de "([^"]*)"$'),
  pendingStep(r'^não vejo "([^"]*)"$'),
  pendingStep(r'^o CA "([^"]*)" deixa de possuir o cargo "([^"]*)"$'),
  pendingStep(r'^o CA "([^"]*)" mantém o cargo "([^"]*)"$'),
  pendingStep(
    r'^o CA "([^"]*)" possui os cargos "([^"]*)", "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(r'^o Centro Acadêmico "([^"]*)" está cadastrado na UnB$'),
  pendingStep(
    r'^o cargo "([^"]*)" de "([^"]*)" não possui integrante vinculado$',
  ),
  pendingStep(r'^o cargo "([^"]*)" não possui a permissão de "([^"]*)"$'),
  pendingStep(r'^o cargo "([^"]*)" possui a permissão de "([^"]*)"$'),
  pendingStep(
    r'^o estudante "([^"]*)" exerce o cargo de "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^o estudante "([^"]*)" possui perfil no CApp$'),
  pendingStep(
    r'^o gestor "([^"]*)" altera o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^o gestor "([^"]*)" remove "([^"]*)" da gestão de "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" tenta vincular "([^"]*)" ao cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^o integrante possui a permissão "([^"]*)" nesse CA$'),
  pendingStep(r'^possuo a permissão "([^"]*)" nesse CA$'),
  pendingStep(r'^que estou autenticado no CApp$'),
  pendingStep(r'^que o CA ativo é "([^"]*)"$'),
  pendingStep(r'^que o Centro Acadêmico "([^"]*)" está cadastrado na UnB$'),
  pendingStep(
    r'^que o cargo "([^"]*)" de "([^"]*)" não possui integrante vinculado$',
  ),
  pendingStep(
    r'^que o estudante "([^"]*)" exerce o cargo de "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^que o estudante "([^"]*)" possui perfil no CApp$'),
  pendingStep(
    r'^que o histórico de "([^"]*)" registra a remoção de "([^"]*)" do cargo "([^"]*)"$',
  ),
  pendingStep(r'^que o "([^"]*)" está ativo no CApp$'),
  pendingStep(r'^solicito minha inscrição no evento "([^"]*)"$'),
  pendingStep(r'^vejo o título "([^"]*)"$'),
  pendingStep(r'^vejo "([^"]*)"$'),
  pendingStep(r'^vejo "([^"]*)" e "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" continua podendo acessar "([^"]*)" como estudante$'),
  pendingStep(r'^"([^"]*)" deixa de integrar a gestão de "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" não passa a integrar a gestão de "([^"]*)"$'),
  pendingStep(
    r'^"([^"]*)" permanece com a permissão de "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" permanece no cargo "([^"]*)" em "([^"]*)"$'),
];
