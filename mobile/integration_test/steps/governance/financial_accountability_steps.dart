/// A prestação de contas permite consultar e registrar informações financeiras com as permissões do CA.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final financialAccountabilitySteps = <StepDefinition>[
  pendingStep(
    r'^a alteração é bloqueada por falta de permissão administrativa$',
  ),
  pendingStep(r'^a consulta informa que não existem lançamentos publicados$'),
  pendingStep(r'^a lista de lançamentos financeiros publicados está vazia$'),
  pendingStep(
    r'^a operação de retificação é recusada informando que o lançamento não foi encontrado$',
  ),
  pendingStep(r'^a prestação de contas do CA ativo permanece inalterada$'),
  pendingStep(r'^a publicação é recusada por dados inválidos do lançamento$'),
  pendingStep(r'^a publicação é recusada por falta de permissão$'),
  pendingStep(
    r'^a retificação é recusada por ausência de justificativa obrigatória$',
  ),
  pendingStep(r'^altero o CA ativo e consulto sua prestação de contas$'),
  pendingStep(r'^as despesas publicadas ficam disponíveis para consulta$'),
  pendingStep(r'^consulto o documento desse lançamento$'),
  pendingStep(r'^consulto sua prestação de contas$'),
  pendingStep(
    r'^ele cadastra um lançamento financeiro manual com os seguintes dados:$',
  ),
  pendingStep(
    r'^ele cancela o lançamento "([^"]*)" com a justificativa "([^"]*)"$',
  ),
  pendingStep(r'^ele consulta o demonstrativo financeiro do "([^"]*)"$'),
  pendingStep(
    r'^ele retifica o lançamento "([^"]*)" alterando o valor para "([^"]*)" com a justificativa "([^"]*)"$',
  ),
  pendingStep(
    r'^ele retifica o lançamento "([^"]*)" com os seguintes metadados e a justificativa "([^"]*)":$',
  ),
  pendingStep(
    r'^ele salva um lançamento financeiro em rascunho com os seguintes dados:$',
  ),
  pendingStep(
    r'^ele tenta retificar o lançamento com identificador "([^"]*)" que não existe$',
  ),
  pendingStep(
    r'^ele tenta retificar o lançamento "([^"]*)" alterando o valor para "([^"]*)" sem justificativa$',
  ),
  pendingStep(r'^ele tenta retificar o valor do lançamento "([^"]*)"$'),
  pendingStep(r'^ele visualiza o total de despesas acumuladas de "([^"]*)"$'),
  pendingStep(r'^ele visualiza o total de receitas acumuladas de "([^"]*)"$'),
  pendingStep(
    r'^existe um lançamento publicado com um documento financeiro anexado$',
  ),
  pendingStep(r'^existem lançamentos publicados em dois CAs diferentes$'),
  pendingStep(r'^nenhum lançamento financeiro é criado$'),
  pendingStep(
    r'^nenhum lançamento é incluído nos registros contábeis do Centro Acadêmico$',
  ),
  pendingStep(r'^não possuo a permissão "([^"]*)" nesse CA$'),
  pendingStep(r'^o CA ativo não possui lançamentos financeiros publicados$'),
  pendingStep(r'^o CA ativo publicou receitas e despesas$'),
  pendingStep(
    r'^o Centro Acadêmico não possui lançamentos financeiros publicados$',
  ),
  pendingStep(
    r'^o comprovante anexado é disponibilizado para visualização pública e download$',
  ),
  pendingStep(r'^o documento fica disponível para consulta$'),
  pendingStep(
    r'^o histórico de cancelamento registra a justificativa, data e autor$',
  ),
  pendingStep(
    r'^o histórico de retificações registra a alteração com justificativa, data e autor$',
  ),
  pendingStep(
    r'^o lançamento fica disponível na prestação de contas do CA ativo$',
  ),
  pendingStep(r'^o lançamento financeiro fica salvo como rascunho$'),
  pendingStep(r'^o lançamento "([^"]*)" deixa de compor o saldo consolidado$'),
  pendingStep(r'^o lançamento "([^"]*)" exibe os metadados retificados$'),
  pendingStep(r'^o lançamento "([^"]*)" fica cancelado$'),
  pendingStep(
    r'^o lançamento "([^"]*)" é atualizado para o valor de "([^"]*)"$',
  ),
  pendingStep(
    r'^o saldo consolidado do Centro Acadêmico reflete a despesa retificada de "([^"]*)"$',
  ),
  pendingStep(r'^o valor do lançamento "([^"]*)" permanece "([^"]*)"$'),
  pendingStep(
    r'^qualquer estudante consulta o demonstrativo financeiro do Centro Acadêmico$',
  ),
  pendingStep(
    r'^que existe um lançamento de despesa publicado com o identificador "([^"]*)" no valor de "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe um lançamento de receita publicado com o identificador "([^"]*)" no valor de "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe um lançamento publicado com comprovante "([^"]*)" anexado$',
  ),
  pendingStep(r'^que integro a gestão do CA ativo$'),
  pendingStep(
    r'^que o Centro Acadêmico não possui lançamentos financeiros publicados$',
  ),
  pendingStep(
    r'^que o Centro Acadêmico possui lançamentos publicados contendo receitas de "([^"]*)" e despesas de "([^"]*)"$',
  ),
  pendingStep(
    r'^que o usuário é um estudante autenticado vinculado a outro Centro Acadêmico$',
  ),
  pendingStep(r'^que o usuário é um estudante sem permissão financeira$'),
  pendingStep(
    r'^que o usuário é um membro da gestão com permissão explícita de prestação de contas$',
  ),
  pendingStep(r'^que o "([^"]*)" é o CA ativo$'),
  pendingStep(r'^que possuo a permissão "([^"]*)" somente em outro CA$'),
  pendingStep(
    r'^registro uma "([^"]*)" com descrição "([^"]*)", valor "([^"]*)" e documento "([^"]*)"$',
  ),
  pendingStep(
    r'^seu tipo, descrição, valor e documento ficam associados ao lançamento$',
  ),
  pendingStep(r'^tento registrar um lançamento financeiro nesse CA$'),
  pendingStep(r'^tento registrar um lançamento financeiro no CA ativo$'),
  pendingStep(
    r'^todos os metadados como categoria, data e justificativa são exibidos com transparência$',
  ),
  pendingStep(r'^um estudante consulta os detalhes desse lançamento$'),
  pendingStep(r'^visualiza o saldo atual consolidado de "([^"]*)"$'),
  pendingStep(r'^visualiza o total de despesas acumuladas de "([^"]*)"$'),
  pendingStep(
    r'^visualizo os lançamentos publicados com seus tipos, descrições e valores$',
  ),
  pendingStep(
    r'^visualizo somente os lançamentos publicados pelo novo CA ativo$',
  ),
];
