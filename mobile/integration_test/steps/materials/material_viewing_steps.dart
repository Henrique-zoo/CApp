/// Os materiais publicados precisam permanecer acessíveis no contexto acadêmico e de CA selecionado.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final materialViewingSteps = <StepDefinition>[
  pendingStep(r'^a descrição do material é "([^"]*)"$'),
  pendingStep(r'^a listagem informa que não há materiais publicados$'),
  pendingStep(r'^acesso o conteúdo do material "([^"]*)"$'),
  pendingStep(r'^consulto a lista de materiais$'),
  pendingStep(r'^consulto novamente a lista de materiais$'),
  pendingStep(r'^consulto os detalhes do material "([^"]*)"$'),
  pendingStep(r'^existe o material publicado "([^"]*)" no CA "([^"]*)"$'),
  pendingStep(r'^existe o material "([^"]*)" não publicado nesse CA$'),
  pendingStep(
    r'^existem os materiais publicados "([^"]*)" e "([^"]*)" nesse CA$',
  ),
  pendingStep(r'^não existem materiais publicados nesse CA$'),
  pendingStep(
    r'^o aplicativo informa que não foi possível carregar os materiais$',
  ),
  pendingStep(r'^o conteúdo do material não é disponibilizado$'),
  pendingStep(r'^o conteúdo publicado é disponibilizado no formato "([^"]*)"$'),
  pendingStep(r'^o material possui conteúdo disponível no formato "([^"]*)"$'),
  pendingStep(
    r'^que não foi possível obter os materiais publicados do CA ativo$',
  ),
  pendingStep(r'^tento acessar o conteúdo do material "([^"]*)"$'),
  pendingStep(r'^vejo a descrição "([^"]*)"$'),
];
