/// A consulta ao FAQ precisa encontrar respostas cadastradas no CA ativo.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final faqConsultationSteps = <StepDefinition>[
  pendingStep(r'^busco no FAQ por "([^"]*)"$'),
  pendingStep(r'^consulto o FAQ$'),
  pendingStep(
    r'^existe a pergunta frequente "([^"]*)" com a resposta "([^"]*)"$',
  ),
  pendingStep(r'^existe a pergunta frequente "([^"]*)" nesse CA$'),
  pendingStep(r'^existe a pergunta frequente "([^"]*)" somente no "([^"]*)"$'),
  pendingStep(
    r'^existe uma pergunta frequente que corresponde a "([^"]*)" somente no "([^"]*)"$',
  ),
  pendingStep(
    r'^existem perguntas frequentes que correspondem a "([^"]*)" nesse CA$',
  ),
  pendingStep(
    r'^existem perguntas frequentes que não correspondem a "([^"]*)" nesse CA$',
  ),
  pendingStep(r'^nenhuma pergunta frequente desse CA corresponde a "([^"]*)"$'),
  pendingStep(r'^nenhuma pergunta frequente é apresentada$'),
  pendingStep(r'^não existem perguntas frequentes disponíveis nesse CA$'),
  pendingStep(r'^não vejo a pergunta do "([^"]*)"$'),
  pendingStep(r'^não vejo a pergunta "([^"]*)"$'),
  pendingStep(r'^não vejo perguntas correspondentes à busca$'),
  pendingStep(r'^não vejo perguntas correspondentes à busca no CA ativo$'),
  pendingStep(r'^não vejo perguntas que não correspondem à busca$'),
  pendingStep(r'^seleciono a pergunta "([^"]*)"$'),
  pendingStep(r'^seleciono o "([^"]*)" como CA ativo$'),
  pendingStep(r'^vejo a pergunta "([^"]*)"$'),
  pendingStep(r'^vejo a resposta "([^"]*)"$'),
  pendingStep(r'^vejo as perguntas correspondentes a "([^"]*)"$'),
];
