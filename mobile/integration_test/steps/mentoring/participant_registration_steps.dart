/// A inscrição no programa de apadrinhamento identifica o papel de calouro ou veterano.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final participantRegistrationSteps = <StepDefinition>[
  pendingStep(r'^a inscrição do calouro é registrada no programa$'),
  pendingStep(r'^a inscrição do veterano é registrada no programa$'),
  pendingStep(
    r'^a inscrição no programa "([^"]*)" é registrada com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^a inscrição no programa "([^"]*)" permanece com o papel "([^"]*)"$',
  ),
  pendingStep(r'^a inscrição não é registrada$'),
  pendingStep(r'^a nova inscrição é recusada$'),
  pendingStep(r'^nenhum participante é inscrito nesse programa$'),
  pendingStep(r'^o calouro ainda não está inscrito nesse programa$'),
  pendingStep(r'^o calouro está autenticado no CApp$'),
  pendingStep(
    r'^o calouro solicita a inscrição no programa com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^o calouro é informado de que o programa não está aberto para inscrições$',
  ),
  pendingStep(r'^o estudante ainda não está inscrito nesse programa$'),
  pendingStep(r'^o estudante ainda não está inscrito no programa "([^"]*)"$'),
  pendingStep(r'^o estudante está autenticado no CApp$'),
  pendingStep(
    r'^o estudante está inscrito no programa "([^"]*)" com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante já está inscrito nesse programa com o papel "([^"]*)"$',
  ),
  pendingStep(r'^o estudante não está autenticado no CApp$'),
  pendingStep(
    r'^o estudante permanece inscrito somente com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante solicita a inscrição no programa "([^"]*)" com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante solicita a inscrição no programa com um papel diferente de "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante solicita a inscrição no programa sem informar um papel$',
  ),
  pendingStep(
    r'^o estudante solicita novamente a inscrição no programa com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante tenta solicitar a inscrição no programa com o papel "([^"]*)"$',
  ),
  pendingStep(r'^o estudante é informado de que a autenticação é obrigatória$'),
  pendingStep(
    r'^o estudante é informado de que precisa identificar o papel de calouro ou veterano$',
  ),
  pendingStep(r'^o papel desse participante é identificado como "([^"]*)"$'),
  pendingStep(r'^o programa "([^"]*)" está aberto para inscrições$'),
  pendingStep(r'^o veterano ainda não está inscrito nesse programa$'),
  pendingStep(r'^o veterano está autenticado no CApp$'),
  pendingStep(
    r'^o veterano solicita a inscrição no programa com o papel "([^"]*)"$',
  ),
  pendingStep(
    r'^o veterano é informado de que o programa não está aberto para inscrições$',
  ),
  pendingStep(r'^que o estudante está autenticado no CApp$'),
  pendingStep(r'^que o programa de apadrinhamento ainda está em configuração$'),
  pendingStep(
    r'^que o programa de apadrinhamento está aberto para inscrições$',
  ),
  pendingStep(r'^que o programa de apadrinhamento está encerrado$'),
];
