/// O acervo institucional mantém os documentos e contatos oficiais acessíveis à comunidade.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final institutionalPublicationSteps = <StepDefinition>[
  pendingStep(
    r'^a operação é recusada indicando que o documento base não foi encontrado$',
  ),
  pendingStep(
    r'^a publicação é recusada pois o usuário não integra a gestão ativa$',
  ),
  pendingStep(
    r'^a publicação é recusada por falta de permissão administrativa$',
  ),
  pendingStep(r'^a versão "([^"]*)" torna-se a versão vigente para consulta$'),
  pendingStep(r'^as informações institucionais são publicadas com sucesso$'),
  pendingStep(r'^ele atualiza o conteúdo do "([^"]*)" com uma nova redação$'),
  pendingStep(
    r'^ele publica as informações de contato e a composição da gestão atual com os seguintes dados:$',
  ),
  pendingStep(
    r'^ele publica o documento institucional com os seguintes dados:$',
  ),
  pendingStep(
    r'^ele tenta atualizar o conteúdo de um documento "([^"]*)" inexistente$',
  ),
  pendingStep(r'^ele tenta publicar um novo documento institucional$'),
  pendingStep(
    r'^ele tenta publicar um regimento interno para o Centro Acadêmico$',
  ),
  pendingStep(
    r'^ele visualiza o estatuto social e o regimento interno vigentes$',
  ),
  pendingStep(
    r'^ele visualiza os canais de contato oficiais e a lista de membros da gestão ativa$',
  ),
  pendingStep(
    r'^fica disponível publicamente para consulta da comunidade acadêmica$',
  ),
  pendingStep(r'^mantém o registro de data e autor da alteração$'),
  pendingStep(r'^nenhum documento institucional é registrado$'),
  pendingStep(r'^o acervo documental permanece inalterado$'),
  pendingStep(
    r'^o documento institucional é publicado com a versão "([^"]*)"$',
  ),
  pendingStep(
    r'^o sistema disponibiliza o acesso à versão anterior "([^"]*)" para auditoria$',
  ),
  pendingStep(
    r'^passam a ser exibidas como os dados oficiais vigentes do Centro Acadêmico$',
  ),
  pendingStep(r'^qualquer usuário consulta o histórico do "([^"]*)"$'),
  pendingStep(
    r'^que o Centro Acadêmico possui contatos e composição da gestão publicados$',
  ),
  pendingStep(
    r'^que o Centro Acadêmico possui documentos institucionais vigentes publicados$',
  ),
  pendingStep(
    r'^que o Centro Acadêmico possui um "([^"]*)" publicado na versão "([^"]*)"$',
  ),
  pendingStep(
    r'^que o usuário fez parte de uma gestão anterior já encerrada do Centro Acadêmico$',
  ),
  pendingStep(
    r'^que o usuário é um estudante regularmente matriculado sem cargo na gestão do Centro Acadêmico$',
  ),
  pendingStep(r'^que o usuário é um membro autorizado da gestão ativa$'),
  pendingStep(
    r'^que o usuário é um membro autorizado da gestão ativa do Centro Acadêmico$',
  ),
  pendingStep(
    r'^que o "([^"]*)" possui as versões "([^"]*)" e "([^"]*)" registradas$',
  ),
  pendingStep(
    r'^um estudante acessa a seção institucional do Centro Acadêmico$',
  ),
  pendingStep(
    r'^um estudante consulta as informações institucionais do Centro Acadêmico$',
  ),
  pendingStep(r'^uma nova versão "([^"]*)" do documento é publicada$'),
  pendingStep(r'^visualiza a data da última atualização de cada documento$'),
];
