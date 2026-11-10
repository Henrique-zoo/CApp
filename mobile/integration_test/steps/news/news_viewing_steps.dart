/// A consulta de notícias precisa refletir o contexto ativo, a publicação e a ordenação selecionada.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final newsViewingSteps = <StepDefinition>[
  pendingStep(r'^a listagem apresenta "([^"]*)" antes de "([^"]*)"$'),
  pendingStep(r'^a listagem informa que não há notícias publicadas$'),
  pendingStep(r'^a ordenação não altera quais notícias estão publicadas$'),
  pendingStep(r'^as notícias aparecem nesta ordem:$'),
  pendingStep(r'^consulto a listagem de notícias$'),
  pendingStep(r'^consulto novamente a listagem de notícias$'),
  pendingStep(r'^consulto os detalhes da notícia "([^"]*)"$'),
  pendingStep(r'^escolhi a ordenação "([^"]*)"$'),
  pendingStep(r'^escolho a ordenação "([^"]*)"$'),
  pendingStep(r'^existe a notícia publicada "([^"]*)" nesse CA$'),
  pendingStep(r'^existe a notícia publicada "([^"]*)" no CA "([^"]*)"$'),
  pendingStep(r'^existe a notícia "([^"]*)" arquivada nesse CA$'),
  pendingStep(r'^existe a notícia "([^"]*)" em rascunho nesse CA$'),
  pendingStep(r'^existe a notícia "([^"]*)" pendente de homologação nesse CA$'),
  pendingStep(r'^existe a notícia "([^"]*)" rejeitada nesse CA$'),
  pendingStep(
    r'^existem as notícias publicadas "([^"]*)" e "([^"]*)" nesse CA$',
  ),
  pendingStep(r'^existem as seguintes notícias publicadas nesse CA:$'),
  pendingStep(r'^não existem notícias publicadas nesse CA$'),
  pendingStep(
    r'^não vejo notícias em rascunho, pendentes de homologação, rejeitadas ou arquivadas$',
  ),
  pendingStep(r'^o conteúdo da notícia não é disponibilizado$'),
  pendingStep(r'^o conteúdo da notícia é "([^"]*)"$'),
  pendingStep(
    r'^que o CA ativo possui as notícias publicadas "([^"]*)" em "([^"]*)" e "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^somente notícias disponíveis no contexto ativo são apresentadas$',
  ),
  pendingStep(r'^tento consultar os detalhes da notícia "([^"]*)"$'),
  pendingStep(r'^vejo o conteúdo "([^"]*)"$'),
];
