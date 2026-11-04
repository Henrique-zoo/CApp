/// Steps compartilhados entre funcionalidades dos cenários de produto.
///
/// O vocabulário reutilizado precisa de um registro único para evitar definições
/// ambíguas. Esta library fornece [sharedSteps] ao registro global do runner;
/// as libraries de cada funcionalidade mantêm seus passos específicos.
///
/// Os passos ainda sem automação usam [pendingStep] e falham explicitamente se
/// executados. Ao implementar as ações e verificações reais sobre o aplicativo,
/// seus registros permanecem aqui e a tag `@pending_mobile` é removida dos
/// cenários já automatizados.
library;

import '../../support/step_registry.dart';

/// Steps reutilizados entre funcionalidades, consumidos pelo registro global.
final sharedSteps = <StepDefinition>[
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
