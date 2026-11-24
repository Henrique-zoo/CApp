/// Passos de apuração e auditoria que serão exercitados pelo app real.
library;

import '../../support/step_registry.dart';

final tallySteps = <StepDefinition>[
  pendingStep(r'^um pleito secreto cujo prazo de votação ainda está vigente$'),
  pendingStep(r'^a comissão tenta iniciar a apuração desse pleito$'),
  pendingStep(
    r'^a apuração é recusada e as contagens por chapa permanecem indisponíveis$',
  ),
  pendingStep(
    r'^um pleito secreto encerrado com dois votos para Aurora e um para Horizonte$',
  ),
  pendingStep(
    r'^todas as autoridades enviaram contribuições válidas para o agregado encerrado$',
  ),
  pendingStep(r'^uma estudante consulta o resultado publicado do pleito$'),
  pendingStep(
    r'^o resultado da apuração informa dois votos para Aurora e um para Horizonte$',
  ),
  pendingStep(
    r'^um pleito secreto encerrado em que falta a contribuição de uma autoridade$',
  ),
  pendingStep(r'^a apuração consta como incompleta, sem totais definitivos$'),
  pendingStep(r'^um pleito secreto encerrado que aguarda contribuições$'),
  pendingStep(
    r'^uma autoridade envia uma contribuição incompatível com o agregado encerrado$',
  ),
  pendingStep(r'^a contribuição é recusada e a apuração continua incompleta$'),
  pendingStep(
    r'^um pleito secreto encerrado cujos totais apurados divergem do número de eleitores$',
  ),
  pendingStep(
    r'^o resultado é divulgado como anulado por inconsistência da contagem$',
  ),
  pendingStep(r'^um pleito com resultado final publicado$'),
  pendingStep(r'^uma estudante consulta a prestação de contas eleitoral$'),
  pendingStep(r'^vê o resultado e os dados públicos do processo$'),
  pendingStep(
    r'^que uma eleitora guardou seu recibo e carregou o quadro público$',
  ),
  pendingStep(r'^verifica a inclusão da própria cédula sem conexão$'),
  pendingStep(
    r'^obtém a confirmação de que seu recibo corresponde à entrada do quadro$',
  ),
  pendingStep(r'^uma cópia do quadro em que uma cédula foi adulterada$'),
  pendingStep(r'^uma estudante verifica as evidências eleitorais no CApp$'),
  pendingStep(r'^a auditoria sinaliza a adulteração do quadro eleitoral$'),
  pendingStep(
    r'^uma cópia internamente coerente do quadro sem âncora externa confiável$',
  ),
  pendingStep(
    r'^a auditoria informa que as evidências são insuficientes para atestar a completude$',
  ),
  pendingStep(
    r'^um pleito com resultado final apto a declarar uma chapa eleita$',
  ),
  pendingStep(r'^a conclusão da eleição é processada duas vezes$'),
  pendingStep(r'^há uma única gestão sucessora correspondente à chapa eleita$'),
  pendingStep(
    r'^um pleito cancelado, anulado ou sem resultado apto a declarar vencedora$',
  ),
  pendingStep(r'^é consultada a sucessão da gestão do Centro Acadêmico$'),
  pendingStep(r'^o pleito não efetiva uma nova gestão$'),
];
