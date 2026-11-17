/// A composição da gestão determina as permissões cumulativas de seus integrantes e precisa preservar seu histórico.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final managementMembersSteps = <StepDefinition>[
  pendingStep(r'^a composição da gestão de "([^"]*)" permanece inalterada$'),
  pendingStep(
    r'^a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o histórico administrativo não pode ser modificado$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o integrante já exerce esse cargo$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o usuário já exerce esse cargo$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o usuário não foi encontrado$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o usuário não integra a gestão do CA$',
  ),
  pendingStep(r'^esse cargo concede a permissão "([^"]*)"$'),
  pendingStep(r'^esse cargo possui a permissão de "([^"]*)"$'),
  pendingStep(
    r'^nenhum novo registro é adicionado ao histórico administrativo de "([^"]*)"$',
  ),
  pendingStep(r'^não integra a gestão de "([^"]*)"$'),
  pendingStep(r'^o CA "([^"]*)" não possui o cargo "([^"]*)"$'),
  pendingStep(r'^o cargo "([^"]*)" concede a permissão "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" remove somente seu vínculo com o cargo "([^"]*)"$',
  ),
  pendingStep(r'^o gestor "([^"]*)" tenta alterar esse registro do histórico$'),
  pendingStep(
    r'^o gestor "([^"]*)" tenta alterar o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^o gestor "([^"]*)" tenta excluir esse registro do histórico$'),
  pendingStep(
    r'^o gestor "([^"]*)" tenta remover "([^"]*)" da gestão de "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" tenta vincular "([^"]*)" novamente ao cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" vincula "([^"]*)" ao cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" vincula "([^"]*)" também ao cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^o histórico administrativo de "([^"]*)" apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de "([^"]*)"$',
  ),
  pendingStep(
    r'^o histórico administrativo de "([^"]*)" registra a nova vinculação$',
  ),
  pendingStep(r'^o histórico preserva a participação nos dois cargos$'),
  pendingStep(r'^o integrante permanece no cargo "([^"]*)" em "([^"]*)"$'),
  pendingStep(r'^o integrante permanece nos dois cargos em "([^"]*)"$'),
  pendingStep(r'^o registro permanece inalterado no histórico de "([^"]*)"$'),
  pendingStep(r'^os registros anteriores à remoção permanecem inalterados$'),
  pendingStep(
    r'^os registros da passagem anterior de "([^"]*)" pela gestão permanecem disponíveis$',
  ),
  pendingStep(
    r'^perde apenas as permissões que não sejam concedidas por seus cargos restantes$',
  ),
  pendingStep(
    r'^possui as permissões de publicação de notícias e gerenciamento de eventos nesse CA$',
  ),
  pendingStep(
    r'^posteriormente teve o cargo alterado para "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^que não existe perfil no CApp para o e-mail "([^"]*)"$'),
  pendingStep(r'^que o cargo "([^"]*)" possui a permissão de "([^"]*)"$'),
  pendingStep(
    r'^que o estudante "([^"]*)" foi vinculado ao cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^que o histórico de "([^"]*)" registra a vinculação de "([^"]*)" ao cargo "([^"]*)"$',
  ),
  pendingStep(r'^que "([^"]*)" exerce o cargo "([^"]*)" em "([^"]*)"$'),
  pendingStep(
    r'^que "([^"]*)" exerce os cargos "([^"]*)" e "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^que "([^"]*)" é o único integrante autorizado a administrar a gestão de "([^"]*)"$',
  ),
  pendingStep(
    r'^que "([^"]*)" é o único integrante de "([^"]*)" com a permissão de "([^"]*)"$',
  ),
  pendingStep(r'^somente um de seus cargos concede a permissão "([^"]*)"$'),
  pendingStep(r'^tenta remover seu vínculo com esse cargo$'),
  pendingStep(r'^todos os vínculos do integrante permanecem inalterados$'),
  pendingStep(
    r'^"([^"]*)" deixa de possuir a permissão de "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" deixa de possuir as permissões do cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" passa a exercer o cargo "([^"]*)" em "([^"]*)"$'),
  pendingStep(
    r'^"([^"]*)" passa a integrar a gestão de "([^"]*)" no cargo "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" passa a possuir a permissão de "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" passa a possuir as permissões do cargo "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" permanece sem integrar a gestão de "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" permanece somente no cargo "([^"]*)" em "([^"]*)"$'),
  pendingStep(
    r'^"([^"]*)" tenta alterar o cargo de "([^"]*)" para "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" tenta remover "([^"]*)" da gestão de "([^"]*)"$'),
  pendingStep(
    r'^"([^"]*)" tenta vincular "([^"]*)" ao cargo "([^"]*)" em "([^"]*)"$',
  ),
];
