/// A votação precisa assegurar elegibilidade, participação única e as garantias da modalidade eleitoral.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final votingSteps = <StepDefinition>[
  pendingStep(r'^a chapa inativa "([^"]*)" não é apresentada como opção$'),
  pendingStep(r'^a confirmação não revela a chapa escolhida$'),
  pendingStep(
    r'^a cédula cifrada não revela a escolha em texto claro nos dados públicos$',
  ),
  pendingStep(r'^a cédula é aceita e o voto é confirmado$'),
  pendingStep(r'^a cédula é recusada$'),
  pendingStep(r'^a eleição está no período de votação$'),
  pendingStep(r'^a eleição possui as chapas ativas "([^"]*)" e "([^"]*)"$'),
  pendingStep(r'^a outra tentativa é recusada por participação já realizada$'),
  pendingStep(
    r'^a participação de "([^"]*)" continua limitada a um único voto nessa eleição$',
  ),
  pendingStep(
    r'^a participação de "([^"]*)" é marcada como realizada nessa eleição$',
  ),
  pendingStep(r'^a tentativa de voto é recusada$'),
  pendingStep(r'^a tentativa de voto é recusada por falta de elegibilidade$'),
  pendingStep(
    r'^a tentativa de voto é recusada porque a votação ainda não começou$',
  ),
  pendingStep(
    r'^a tentativa de voto é recusada porque a votação foi encerrada$',
  ),
  pendingStep(r'^apenas um voto fica registrado para "([^"]*)" nessa eleição$'),
  pendingStep(r'^apenas um voto é contabilizado para essa participação$'),
  pendingStep(r'^apenas uma das tentativas é aceita$'),
  pendingStep(
    r'^as solicitações válidas de voto de "([^"]*)" são assinadas com sua própria chave$',
  ),
  pendingStep(r'^consulta as opções de voto$'),
  pendingStep(r'^diferentes eleitores já registraram suas cédulas$'),
  pendingStep(r'^existe uma chapa inativa chamada "([^"]*)"$'),
  pendingStep(r'^nenhum voto é registrado para o estudante "([^"]*)"$'),
  pendingStep(r'^nenhum voto é registrado para "([^"]*)"$'),
  pendingStep(r'^o estudante "([^"]*)" ainda não votou nessa eleição$'),
  pendingStep(r'^o estudante "([^"]*)" já participou dessa eleição$'),
  pendingStep(
    r'^o estudante "([^"]*)" já registrou um voto na chapa "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" não possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(r'^o período de votação ainda não começou$'),
  pendingStep(r'^o período de votação foi encerrado$'),
  pendingStep(
    r'^o quadro público contém a cédula de "([^"]*)" em formato cifrado$',
  ),
  pendingStep(
    r'^o registro identifica "([^"]*)" como eleitora e "([^"]*)" como sua escolha$',
  ),
  pendingStep(
    r'^o registro público contém a identificação do eleitor, a cédula cifrada e a assinatura do eleitor$',
  ),
  pendingStep(r'^o servidor fornece um recibo assinado da cédula recebida$'),
  pendingStep(
    r'^o voto original na chapa "([^"]*)" permanece como o único voto de "([^"]*)" nessa eleição$',
  ),
  pendingStep(r'^o voto é confirmado$'),
  pendingStep(r'^que a eleição está configurada para votação identificada$'),
  pendingStep(r'^que a eleição está configurada para votação secreta$'),
  pendingStep(r'^que a eleição está no período de votação$'),
  pendingStep(r'^que existe uma eleição para o Centro Acadêmico "([^"]*)"$'),
  pendingStep(
    r'^que o estudante "([^"]*)" possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(r'^registra um voto na chapa "([^"]*)"$'),
  pendingStep(r'^registra um voto secreto$'),
  pendingStep(r'^submete uma cédula sem assinatura digital válida$'),
  pendingStep(r'^são apresentadas as chapas ativas "([^"]*)" e "([^"]*)"$'),
  pendingStep(
    r'^são realizadas simultaneamente duas tentativas de voto, uma na chapa "([^"]*)" e outra na chapa "([^"]*)"$',
  ),
  pendingStep(r'^tenta registrar outro voto na chapa "([^"]*)"$'),
  pendingStep(r'^tenta registrar um voto na chapa "([^"]*)"$'),
  pendingStep(r'^uma pessoa consulta os dados públicos da eleição$'),
  pendingStep(
    r'^uma pessoa observa a ordem e o momento de submissão das cédulas$',
  ),
  pendingStep(
    r'^"([^"]*)" consegue verificar que sua cédula cifrada consta no quadro público$',
  ),
  pendingStep(
    r'^"([^"]*)" já registrou sua participação nessa eleição por votação secreta$',
  ),
  pendingStep(r'^"([^"]*)" está autenticada pela sua conta Firebase$'),
  pendingStep(
    r'^"([^"]*)" cadastrou sua chave pública antes da abertura$',
  ),
  pendingStep(r'^registra seu primeiro voto na chapa "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" não possui chave cadastrada nessa eleição$'),
  pendingStep(
    r'^a assinatura da cédula confere com a chave cadastrada por "([^"]*)"$',
  ),
  pendingStep(r'^a chave privada permanece no dispositivo de "([^"]*)"$'),
  pendingStep(
    r'^uma nova verificação institucional de "([^"]*)" informa outro curso$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" está autenticado e atende aos critérios de elegibilidade$',
  ),
  pendingStep(
    r'^"([^"]*)" está autenticada e atende aos critérios de elegibilidade$',
  ),
  pendingStep(
    r'^"([^"]*)" envia uma cédula com o identificador eleitoral de "([^"]*)"$',
  ),
  pendingStep(
    r'^a cédula é recusada porque seu eleitor diverge da conta autenticada$',
  ),
  pendingStep(
    r'^submete uma cédula assinada com sua chave local, indicando a chapa "([^"]*)"$',
  ),
  pendingStep(
    r'^a publicação não expõe escolha em claro nem aleatoriedade da cifragem$',
  ),
  pendingStep(r'^os dados não publicam a escolha em claro de "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" já participou dessa eleição$'),
  pendingStep(
    r'^"([^"]*)" tenta votar em outro dispositivo com um novo par de chaves$',
  ),
  pendingStep(r'^que "([^"]*)" já participou dessa eleição$'),
  pendingStep(
    r'^"([^"]*)" está autenticada em outro dispositivo sem a chave privada anterior$',
  ),
  pendingStep(r'^solicita seu recibo de participação$'),
  pendingStep(r'^recebe o recibo do voto já aceito$'),
  pendingStep(r'^nenhum novo voto é registrado$'),
  pendingStep(
    r'^que o voto de "([^"]*)" foi aceito mas a confirmação não chegou ao dispositivo$',
  ),
  pendingStep(
    r'^"([^"]*)" retransmite o mesmo pacote assinado com o mesmo identificador de envio$',
  ),
  pendingStep(r'^seleciona a opção "([^"]*)" e confirma o voto$'),
  pendingStep(r'^a opção "([^"]*)" não é oferecida$'),
  pendingStep(r'^que a votação foi encerrada com dois participantes$'),
  pendingStep(r'^todas as autoridades forneceram contribuições válidas$'),
  pendingStep(r'^a apuração produz um voto para "([^"]*)" e um voto nulo$'),
  pendingStep(r'^a contagem é consistente com os dois participantes$'),
  pendingStep(r'^a soma das opções incluindo Nulo resulta em três votos$'),
  pendingStep(r'^a eleição é anulada por inconsistência da contagem$'),
  pendingStep(r'^os registros permanecem disponíveis para auditoria$'),
  pendingStep(r'^a soma das opções incluindo Nulo resulta em um voto$'),
  pendingStep(r'^que o período de votação foi encerrado$'),
  pendingStep(r'^falta uma contribuição válida de autoridade$'),
  pendingStep(r'^a apuração é solicitada$'),
  pendingStep(r'^a apuração permanece incompleta e sem totais definitivos$'),
];
