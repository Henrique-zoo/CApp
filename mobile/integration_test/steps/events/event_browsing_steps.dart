/// A agenda apresenta eventos publicados e permite ao estudante administrar sua própria inscrição.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final eventBrowsingSteps = <StepDefinition>[
  pendingStep(r'^a listagem informa que não há eventos publicados$'),
  pendingStep(r'^ainda não estou inscrito nesse evento$'),
  pendingStep(r'^consulto a lista de eventos$'),
  pendingStep(r'^consulto novamente a lista de eventos$'),
  pendingStep(r'^consulto os detalhes do evento "([^"]*)"$'),
  pendingStep(r'^estou consultando a agenda pública de eventos$'),
  pendingStep(r'^estou inscrito no evento publicado "([^"]*)"$'),
  pendingStep(r'^existe o evento publicado "([^"]*)" nesse CA$'),
  pendingStep(r'^existe o evento publicado "([^"]*)" no CA "([^"]*)"$'),
  pendingStep(r'^existe o evento "([^"]*)" não publicado nesse CA$'),
  pendingStep(
    r'^existe o evento "([^"]*)" proposto e ainda não homologado nesse CA$',
  ),
  pendingStep(
    r'^existe o evento "([^"]*)" proposto e ainda não homologado no CA "([^"]*)"$',
  ),
  pendingStep(
    r'^existem os eventos publicados "([^"]*)" e "([^"]*)" nesse CA$',
  ),
  pendingStep(r'^já estou inscrito nesse evento$'),
  pendingStep(r'^minha inscrição continua única$'),
  pendingStep(r'^minha inscrição no evento deixa de estar ativa$'),
  pendingStep(r'^minha inscrição no evento é registrada$'),
  pendingStep(r'^minha inscrição não é registrada$'),
  pendingStep(r'^nenhuma inscrição é alterada$'),
  pendingStep(r'^não estou inscrito no evento publicado "([^"]*)"$'),
  pendingStep(r'^não existem eventos publicados nesse CA$'),
  pendingStep(r'^o aplicativo informa que a inscrição foi realizada$'),
  pendingStep(r'^o aplicativo informa que já estou inscrito nesse evento$'),
  pendingStep(
    r'^o aplicativo informa que não foi possível carregar os eventos$',
  ),
  pendingStep(
    r'^o aplicativo informa que não há inscrição ativa para cancelar$',
  ),
  pendingStep(r'^o aplicativo informa que o cancelamento foi realizado$'),
  pendingStep(
    r'^o aplicativo informa que o evento não está disponível para inscrição$',
  ),
  pendingStep(r'^o evento contém a informação "([^"]*)"$'),
  pendingStep(r'^os detalhes do evento não são disponibilizados$'),
  pendingStep(
    r'^que não foi possível obter os eventos publicados do CA ativo$',
  ),
  pendingStep(r'^solicito cancelar minha inscrição no evento "([^"]*)"$'),
  pendingStep(r'^solicito novamente minha inscrição no evento "([^"]*)"$'),
  pendingStep(r'^tento consultar os detalhes do evento "([^"]*)"$'),
  pendingStep(r'^vejo a informação "([^"]*)"$'),
];
