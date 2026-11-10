/// O acompanhamento de demandas precisa apresentar seu andamento e as consultas previstas em cada entrega.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final demandTrackingSteps = <StepDefinition>[
  pendingStep(r'^a alteração anterior não foi substituída pela mais recente$'),
  pendingStep(r'^a consulta coletiva do fórum continua disponível$'),
  pendingStep(r'^a demanda não apresenta o status "([^"]*)"$'),
  pendingStep(r'^a demanda "([^"]*)" foi registrada por "([^"]*)"$'),
  pendingStep(
    r'^a demanda "([^"]*)" possui as seguintes alterações de status:$',
  ),
  pendingStep(r'^a demanda "([^"]*)" possui o status "([^"]*)"$'),
  pendingStep(r'^a demanda "([^"]*)" teve uma alteração para "([^"]*)"$'),
  pendingStep(r'^a gestão do CA atualizou o status da demanda para "([^"]*)"$'),
  pendingStep(r'^a listagem informa que não existem demandas$'),
  pendingStep(
    r'^a listagem informa que não existem demandas registradas por mim$',
  ),
  pendingStep(r'^cada alteração apresenta o status correspondente$'),
  pendingStep(r'^cada alteração apresenta sua data$'),
  pendingStep(r'^cada alteração identifica quem realizou a alteração$'),
  pendingStep(r'^cada demanda apresenta seu respectivo status atual$'),
  pendingStep(r'^consigo identificar a demanda consultada$'),
  pendingStep(
    r'^consultei anteriormente a demanda "([^"]*)" com o status "([^"]*)"$',
  ),
  pendingStep(r'^consulto as demandas sobre o espaço físico$'),
  pendingStep(r'^consulto novamente os detalhes da demanda "([^"]*)"$'),
  pendingStep(r'^consulto o histórico da demanda "([^"]*)"$'),
  pendingStep(r'^consulto os detalhes da demanda "([^"]*)"$'),
  pendingStep(r'^estou consultando somente as minhas demandas$'),
  pendingStep(r'^existe a demanda "([^"]*)" no "([^"]*)"$'),
  pendingStep(r'^existe a demanda "([^"]*)" registrada por mim$'),
  pendingStep(r'^existe a demanda "([^"]*)" registrada por "([^"]*)"$'),
  pendingStep(r'^existe a demanda "([^"]*)" sobre o espaço físico$'),
  pendingStep(r'^existe a demanda "([^"]*)" sobre o espaço físico desse CA$'),
  pendingStep(r'^existem as seguintes demandas sobre o espaço físico:$'),
  pendingStep(
    r'^existem demandas registradas por mim e por outros usuários nesse CA$',
  ),
  pendingStep(r'^existem demandas registradas por outros usuários nesse CA$'),
  pendingStep(r'^filtro as demandas para exibir somente as minhas$'),
  pendingStep(r'^não existem demandas sobre o espaço físico nesse CA$'),
  pendingStep(r'^não registrei demandas nesse CA$'),
  pendingStep(r'^não vejo a demanda "([^"]*)"$'),
  pendingStep(r'^o status apresentado pertence aos seguintes estados:$'),
  pendingStep(
    r'^os detalhes da demanda não são disponibilizados no contexto ativo$',
  ),
  pendingStep(r'^os detalhes da demanda são apresentados$'),
  pendingStep(r'^posteriormente teve uma alteração para "([^"]*)"$'),
  pendingStep(r'^removo o filtro das próprias demandas$'),
  pendingStep(r'^seu status atual é "([^"]*)"$'),
  pendingStep(r'^tento consultar os detalhes da demanda "([^"]*)"$'),
  pendingStep(r'^vejo a demanda "([^"]*)"$'),
  pendingStep(r'^vejo demandas registradas por mim e por outros usuários$'),
  pendingStep(r'^vejo o histórico de alterações de status$'),
  pendingStep(r'^vejo o registro da alteração para "([^"]*)"$'),
  pendingStep(r'^vejo o registro da alteração posterior para "([^"]*)"$'),
  pendingStep(r'^vejo que o status atual é "([^"]*)"$'),
  pendingStep(r'^volto a consultar as demandas visíveis do fórum$'),
];
