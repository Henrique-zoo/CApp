/// Configuração e calendário eleitoral ligados ao contrato compartilhado.
///
/// Os cenários especificam a escolha entre votação secreta e identificada,
/// chapa única na modalidade secreta, adiamento e cancelamento. As expressões
/// ficam registradas para futura automação real do aplicativo; pendingStep falha
/// quando executado e não prepara resultados fictícios. Vocabulário comum à
/// votação permanece no registro compartilhado.
///
/// A autorização da solicitação de apuração não representa sua conclusão; apenas
/// a modalidade secreta exige contribuições criptográficas após o encerramento.
library;

import '../../support/step_registry.dart';

/// Passos ainda sem implementação funcional, preservando as tags de pendência.
final electionManagementSteps = <StepDefinition>[
  pendingStep(r'^que a comissão eleitoral foi indicada pela gestão do CA$'),
  pendingStep(r'^estou autenticado como integrante autorizado dessa comissão$'),
  pendingStep(r'^que estou preenchendo os dados de uma nova eleição do CA$'),
  pendingStep(
    r'^seleciono votação "(secreta|identificada)" e concluo a criação da eleição$',
  ),
  pendingStep(r'^a eleição é criada com votação "(secreta|identificada)"$'),
  pendingStep(r'^que existe uma única chapa aprovada pela comissão$'),
  pendingStep(r'^a configuração eleitoral é concluída$'),
  pendingStep(r'^o manifesto contém a chapa e a opção Nulo$'),
  pendingStep(r'^as autoridades são o servidor, a comissão e a chapa$'),
  pendingStep(r'^o servidor não recebe os segredos das outras autoridades$'),
  pendingStep(r'^que a eleição ainda está em preparação$'),
  pendingStep(
    r'^a comissão adia a abertura e escolhe um fim posterior ao anterior$',
  ),
  pendingStep(r'^o novo calendário é registrado com motivo e responsável$'),
  pendingStep(r'^a comissão não é obrigada a preservar a duração original$'),
  pendingStep(
    r'^a comissão adia a abertura mas mantém o encerramento anterior$',
  ),
  pendingStep(r'^a alteração do calendário é recusada$'),
  pendingStep(
    r'^que a eleição está aberta e o prazo vigente ainda não venceu$',
  ),
  pendingStep(r'^a comissão escolhe um encerramento posterior ao vigente$'),
  pendingStep(r'^o manifesto e a abertura já ocorrida permanecem inalterados$'),
  pendingStep(r'^as cédulas já aceitas continuam válidas$'),
  pendingStep(r'^que o prazo vigente da eleição já terminou$'),
  pendingStep(r'^a comissão tenta prorrogar o encerramento$'),
  pendingStep(r'^a eleição não é reaberta$'),
  pendingStep(
    r'^a comissão solicita encerrar a votação para apurar o resultado$',
  ),
  pendingStep(r'^o encerramento antecipado é recusado$'),
  pendingStep(r'^nenhuma contagem antecipada por opção é publicada$'),
  pendingStep(
    r'^que a eleição foi criada por engano e ainda não recebeu votos$',
  ),
  pendingStep(r'^a comissão solicita cancelamento com motivo$'),
  pendingStep(
    r'^a eleição fica cancelada com responsável e instante registrados$',
  ),
  pendingStep(r'^que a eleição está aberta e já recebeu cédulas$'),
  pendingStep(r'^as cédulas e o histórico são preservados$'),
  pendingStep(r'^nenhum novo voto é aceito$'),
  // Designação e permissões específicas da comissão eleitoral — Issue #124.
  pendingStep(r'^ainda não integra a comissão eleitoral de "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" designa "([^"]*)" para a comissão eleitoral de "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" passa a integrar a comissão eleitoral de "([^"]*)"$',
  ),
  pendingStep(r'^que "([^"]*)" integra a comissão eleitoral de "([^"]*)"$'),
  pendingStep(r'^"([^"]*)" integra a comissão eleitoral de "([^"]*)"$'),
  pendingStep(r'^não integra a comissão eleitoral de "([^"]*)"$'),
  pendingStep(r'^possui a permissão "([^"]*)" em "([^"]*)"$'),
  pendingStep(r'^não possui a permissão "([^"]*)" em "([^"]*)"$'),
  pendingStep(r'^tenta apurar o resultado de uma eleição de "([^"]*)"$'),
  pendingStep(r'^tenta alterar a configuração de uma eleição de "([^"]*)"$'),
  pendingStep(r'^tenta configurar uma eleição de "([^"]*)"$'),
  pendingStep(r'^realizou ações administrativas em uma eleição de "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" revoga a designação de "([^"]*)" da comissão eleitoral de "([^"]*)"$',
  ),
  pendingStep(
    r'^"([^"]*)" deixa de integrar a comissão eleitoral de "([^"]*)"$',
  ),
  pendingStep(
    r'^deixa de possuir as autorizações eleitorais vinculadas à designação revogada$',
  ),
  pendingStep(
    r'^as ações anteriores permanecem registradas no histórico da eleição$',
  ),
  pendingStep(
    r'^"([^"]*)" tenta designar "([^"]*)" para a comissão eleitoral de "([^"]*)"$',
  ),

  // Validação do período, das autoridades e da publicação do manifesto — Issue #124.
  pendingStep(
    r'^informo um período de votação válido e concluo a criação da eleição$',
  ),
  pendingStep(
    r'^o calendário inicial registra o início e o encerramento informados$',
  ),
  pendingStep(
    r'^informo um período cujo encerramento é anterior ao início e tento criar a eleição$',
  ),
  pendingStep(
    r'^informo um período cujo encerramento é igual ao início e tento criar a eleição$',
  ),
  pendingStep(r'^a criação da eleição é recusada por período inválido$'),
  pendingStep(
    r'^que a eleição está em preparação sem todas as autoridades definidas$',
  ),
  pendingStep(r'^a comissão solicita a abertura da votação$'),
  pendingStep(r'^a abertura é recusada por falta de autoridades$'),
  pendingStep(r'^que a eleição está em preparação com duas chapas aprovadas$'),
  pendingStep(r'^cada chapa possui uma autoridade independente$'),
  pendingStep(r'^o servidor e a comissão possuem autoridades independentes$'),
  pendingStep(
    r'^a abertura é aceita após a validação de todas as autoridades$',
  ),
  pendingStep(
    r'^o manifesto publicado contém as duas chapas na ordem definida$',
  ),
  pendingStep(
    r'^o manifesto publicado contém a opção Nulo uma única vez na última posição$',
  ),
  pendingStep(
    r'^o manifesto contém uma autoridade do servidor, uma da comissão e uma para cada chapa$',
  ),
  pendingStep(
    r'^o manifesto é assinado pelo servidor e permanece imutável após a abertura$',
  ),
  pendingStep(r'^que a eleição foi aberta e possui manifesto publicado$'),
  pendingStep(r'^a comissão tenta alterar as chapas ou a ordem das opções$'),
  pendingStep(r'^a alteração da configuração eleitoral é recusada$'),
  pendingStep(r'^o manifesto publicado permanece inalterado$'),
  pendingStep(
    r'^"([^"]*)" não passa a integrar a comissão eleitoral de "([^"]*)"$',
  ),
  // Permissões específicas da comissão eleitoral — Issue #124.
  pendingStep(r'^possui a permissão administrativa "([^"]*)" em "([^"]*)"$'),
  pendingStep(r'^configura uma eleição de "([^"]*)"$'),
  pendingStep(r'^a configuração da eleição é autorizada$'),
  pendingStep(r'^administra as chapas de uma eleição de "([^"]*)"$'),
  pendingStep(r'^a administração das chapas é concluída$'),
  pendingStep(
    r'^solicita a apuração do resultado de uma eleição de "([^"]*)"$',
  ),
  pendingStep(r'^a solicitação de apuração é autorizada$'),

  // Calendário e encerramento normal da votação — Issue #124.
  pendingStep(
    r'^a comissão adia a abertura e escolhe um fim anterior ao anterior$',
  ),
  pendingStep(r'^o processo eleitoral é encerrado$'),
  pendingStep(r'^a eleição passa ao estado encerrado$'),
  pendingStep(r'^o registro final das cédulas é preservado$'),
  pendingStep(
    r'^a apuração aguarda as contribuições válidas de todas as autoridades$',
  ),
  pendingStep(
    r'^a apuração não exige contribuições de autoridades criptográficas$',
  ),
  pendingStep(r'^que a eleição está aberta e o prazo vigente foi atingido$'),
];
