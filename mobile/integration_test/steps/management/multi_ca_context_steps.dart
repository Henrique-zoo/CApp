/// A navegação entre CAs precisa preservar a preferência inicial e as permissões atribuídas em cada contexto.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final multiCaContextSteps = <StepDefinition>[
  pendingStep(
    r'^a preferência de CA favorito do estudante permanece "([^"]*)"$',
  ),
  pendingStep(r'^está autenticado no aplicativo$'),
  pendingStep(r'^está com o contexto ativo em "([^"]*)"$'),
  pendingStep(
    r'^não recebe permissões administrativas apenas pela mudança de contexto$',
  ),
  pendingStep(r'^o cargo possui a permissão de "([^"]*)"$'),
  pendingStep(r'^o contexto ativo de navegação deve ser "([^"]*)"$'),
  pendingStep(r'^o estudante abre o aplicativo CApp$'),
  pendingStep(r'^o estudante altera o contexto ativo para "([^"]*)"$'),
  pendingStep(r'^o estudante está com o contexto ativo em "([^"]*)"$'),
  pendingStep(r'^o estudante não possui cargo de gestão em "([^"]*)"$'),
  pendingStep(r'^o estudante não possui permissão para "([^"]*)" no CA ativo$'),
  pendingStep(
    r'^o estudante não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito$',
  ),
  pendingStep(r'^o estudante possui permissão para "([^"]*)" no CA ativo$'),
  pendingStep(
    r'^o estudante visualiza as informações correspondentes a "([^"]*)"$',
  ),
  pendingStep(
    r'^que existem os seguintes Centros Acadêmicos cadastrados na UnB:$',
  ),
  pendingStep(
    r'^que o estudante "([^"]*)" possui "([^"]*)" definido como seu CA favorito$',
  ),
];
