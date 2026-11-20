/// A comunicação em demandas permite interações entre estudantes e a gestão do CA.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final demandCommunicationSteps = <StepDefinition>[
  pendingStep(r'^a data e hora da resposta são registradas$'),
  pendingStep(
    r'^a demanda institucional "([^"]*)" possui o seguinte histórico de interações:$',
  ),
  pendingStep(
    r'^a notificação informa que a demanda agora está com o status "([^"]*)"$',
  ),
  pendingStep(
    r'^a notificação informa que a gestão de "([^"]*)" respondeu à solicitação$',
  ),
  pendingStep(
    r'^a operação é recusada informando que a demanda pertence a outro Centro Acadêmico$',
  ),
  pendingStep(r'^a publicação do comentário é recusada$'),
  pendingStep(
    r'^cada evento identifica seu tipo, autor, conteúdo e instante de registro$',
  ),
  pendingStep(r'^consulto a linha do tempo da demanda "([^"]*)"$'),
  pendingStep(r'^existe a demanda "([^"]*)" associada ao CA "([^"]*)"$'),
  pendingStep(
    r'^existe a demanda "([^"]*)" sobre o espaço físico registrada por "([^"]*)"$',
  ),
  pendingStep(r'^existe a demanda institucional "([^"]*)" no "([^"]*)"$'),
  pendingStep(
    r'^existe a demanda institucional "([^"]*)" registrada por "([^"]*)"$',
  ),
  pendingStep(r'^existe a demanda institucional "([^"]*)" registrada por mim$'),
  pendingStep(r'^meu CA ativo é "([^"]*)"$'),
  pendingStep(r'^nenhum comentário é adicionado à demanda$'),
  pendingStep(r'^nenhum comentário é registrado na demanda$'),
  pendingStep(
    r'^nenhum evento anterior foi sobrescrito pelas mensagens posteriores$',
  ),
  pendingStep(
    r'^o aplicativo informa que o comentário ultrapassou o limite máximo de 2000 caracteres$',
  ),
  pendingStep(
    r'^o aplicativo informa que o conteúdo do comentário é obrigatório$',
  ),
  pendingStep(
    r'^o autor "([^"]*)" recebe uma notificação de alteração de status$',
  ),
  pendingStep(
    r'^o autor "([^"]*)" recebe uma notificação de nova resposta na demanda$',
  ),
  pendingStep(r'^o autor do comentário é "([^"]*)"$'),
  pendingStep(
    r'^o autor do comentário é identificado como gestão de "([^"]*)"$',
  ),
  pendingStep(
    r'^o comentário fica visível para os usuários do fórum do espaço físico$',
  ),
  pendingStep(r'^o comentário é registrado na demanda "([^"]*)"$'),
  pendingStep(r'^o conteúdo do comentário é "([^"]*)"$'),
  pendingStep(
    r'^o envio do comentário é recusado por falta de acesso ao fórum institucional$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" é aluno vinculado ao curso de Engenharia de Redes$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" não possui vínculo com o curso de Engenharia de Redes$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" atualiza o status da demanda "([^"]*)" para "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" envia uma resposta na demanda "([^"]*)"$',
  ),
  pendingStep(r'^o gestor "([^"]*)" está autenticado no CApp$'),
  pendingStep(
    r'^publico um comentário na demanda com o texto "([^"]*)"$',
  ),
  pendingStep(
    r'^publico uma resposta na demanda com o texto "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe a demanda institucional "([^"]*)" registrada por "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe a demanda institucional "([^"]*)" registrada por "([^"]*)" com o status "([^"]*)"$',
  ),
  pendingStep(
    r'^tento publicar um comentário na demanda com o texto "([^"]*)"$',
  ),
  pendingStep(
    r'^tento publicar um comentário na demanda com texto excedendo 2000 caracteres$',
  ),
  pendingStep(
    r'^tento publicar um comentário na demanda "([^"]*)" de "([^"]*)"$',
  ),
  pendingStep(
    r'^tento publicar um comentário na demanda institucional "([^"]*)"$',
  ),
  pendingStep(r'^vejo os eventos apresentados em ordem cronológica$'),
];
