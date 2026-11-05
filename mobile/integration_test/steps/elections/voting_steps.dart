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
  pendingStep(
    r'^a assinatura digital é validada com a chave pública cadastrada de "([^"]*)"$',
  ),
  pendingStep(r'^a chapa inativa "([^"]*)" não é apresentada como opção$'),
  pendingStep(
    r'^a chave pública de "([^"]*)" foi cadastrada antes do início da votação$',
  ),
  pendingStep(
    r'^a chave pública eleitoral de "([^"]*)" foi cadastrada antes da abertura$',
  ),
  pendingStep(r'^a confirmação não revela a chapa escolhida$'),
  pendingStep(
    r'^a cédula cifrada não revela a escolha em texto claro nos dados públicos$',
  ),
  pendingStep(r'^a cédula é aceita e o voto é confirmado$'),
  pendingStep(r'^a cédula é recusada$'),
  pendingStep(
    r'^a cédula é recusada porque a assinatura não corresponde à identidade eleitoral de "([^"]*)"$',
  ),
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
  pendingStep(
    r'^a tentativa de voto é recusada por falta de cadastro eleitoral$',
  ),
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
  pendingStep(
    r'^não consegue determinar a escolha em texto claro de "([^"]*)" a partir desses dados$',
  ),
  pendingStep(
    r'^não consegue determinar a escolha individual de cada eleitor com base nessas informações$',
  ),
  pendingStep(r'^o estudante "([^"]*)" ainda não votou nessa eleição$'),
  pendingStep(
    r'^o estudante "([^"]*)" está cadastrado e atende aos critérios de elegibilidade$',
  ),
  pendingStep(r'^o estudante "([^"]*)" já participou dessa eleição$'),
  pendingStep(
    r'^o estudante "([^"]*)" já registrou um voto na chapa "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" não atende a um critério de elegibilidade definido para a eleição$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" não consta no cadastro de eleitores definido antes do início da eleição$',
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
  pendingStep(
    r'^o voto é recusado por ausência no cadastro eleitoral anterior à abertura$',
  ),
  pendingStep(r'^que a eleição está configurada para votação identificada$'),
  pendingStep(r'^que a eleição está configurada para votação secreta$'),
  pendingStep(r'^que a eleição está no período de votação$'),
  pendingStep(r'^que existe uma eleição para o Centro Acadêmico "([^"]*)"$'),
  pendingStep(
    r'^que o estudante "([^"]*)" possui vínculo acadêmico válido com o Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(r'^registra um voto na chapa "([^"]*)"$'),
  pendingStep(r'^registra um voto secreto$'),
  pendingStep(
    r'^submete uma cédula assinada com a chave privada correspondente à sua chave pública cadastrada, indicando a chapa "([^"]*)"$',
  ),
  pendingStep(r'^submete uma cédula sem assinatura digital válida$'),
  pendingStep(r'^são apresentadas as chapas ativas "([^"]*)" e "([^"]*)"$'),
  pendingStep(
    r'^são realizadas simultaneamente duas tentativas de voto, uma na chapa "([^"]*)" e outra na chapa "([^"]*)"$',
  ),
  pendingStep(r'^tenta registrar outro voto na chapa "([^"]*)"$'),
  pendingStep(r'^tenta registrar um voto com assinatura válida$'),
  pendingStep(r'^tenta registrar um voto na chapa "([^"]*)"$'),
  pendingStep(r'^uma pessoa consulta os dados públicos da eleição$'),
  pendingStep(
    r'^uma pessoa observa a ordem e o momento de submissão das cédulas$',
  ),
  pendingStep(
    r'^"([^"]*)" consegue verificar que sua cédula cifrada consta no quadro público$',
  ),
  pendingStep(
    r'^"([^"]*)" consta no cadastro de eleitores definido antes da abertura$',
  ),
  pendingStep(
    r'^"([^"]*)" está cadastrada e atende aos critérios de elegibilidade$',
  ),
  pendingStep(
    r'^"([^"]*)" já registrou sua participação nessa eleição por votação secreta$',
  ),
  pendingStep(
    r'^"([^"]*)" possui uma chave pública cadastrada para a eleição$',
  ),
  pendingStep(
    r'^"([^"]*)" submete uma cédula assinada com a chave privada correspondente à chave pública de "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" só foi cadastrado como eleitor depois da abertura$'),
];
