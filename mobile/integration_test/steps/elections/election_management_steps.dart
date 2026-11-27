/// Configuração e calendário eleitoral ligados ao contrato compartilhado.
///
/// Os cenários especificam a designação da comissão e as permissões por eleição,
/// além da escolha da modalidade, do adiamento e do cancelamento. As expressões
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
  pendingStep(r'^que a gestão do CA designou a comissão para esta eleição$'),
  pendingStep(r'^estou autenticado como integrante autorizado dessa comissão$'),
  pendingStep(r'^que possuo a permissão "([^"]*)" nesta eleição$'),
  pendingStep(r'^que estou preenchendo os dados iniciais da eleição$'),
  pendingStep(
    r'^seleciono votação "(secreta|identificada)" e concluo a configuração inicial$',
  ),
  pendingStep(
    r'^a configuração da eleição registra a votação "(secreta|identificada)"$',
  ),
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
  pendingStep(r'^existe a eleição "([^"]*)" de "([^"]*)"$'),
  pendingStep(r'^ainda não integra a comissão da eleição "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" designa "([^"]*)" para a comissão da eleição "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" passa a integrar a comissão da eleição "([^"]*)"$'),
  pendingStep(r'^que "([^"]*)" integra a comissão da eleição "([^"]*)"$'),
  pendingStep(r'^não integra a comissão da eleição "([^"]*)"$'),
  pendingStep(r'^possui a permissão "([^"]*)" na eleição "([^"]*)"$'),
  pendingStep(r'^não possui a permissão "([^"]*)" na eleição "([^"]*)"$'),
  pendingStep(r'^tenta apurar o resultado da eleição "([^"]*)"$'),
  pendingStep(r'^tenta configurar a eleição "([^"]*)"$'),
  pendingStep(r'^realizou ações administrativas na eleição "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" solicita a revogação da designação de "([^"]*)" da comissão da eleição "([^"]*)"$',
  ),
  pendingStep(
    r'^que "([^"]*)" representa a comissão como autoridade da eleição "([^"]*)"$',
  ),
  pendingStep(
    r'^que o manifesto da eleição foi publicado e a eleição está "([^"]*)"$',
  ),
  pendingStep(r'^a revogação é recusada enquanto a eleição está em andamento$'),
  pendingStep(
    r'^"([^"]*)" permanece como representante da comissão da eleição "([^"]*)"$',
  ),
  pendingStep(r'^"([^"]*)" deixa de integrar a comissão da eleição "([^"]*)"$'),
  pendingStep(
    r'^deixa de possuir as autorizações eleitorais vinculadas à designação revogada$',
  ),
  pendingStep(
    r'^as ações anteriores permanecem registradas no histórico da eleição$',
  ),
  pendingStep(
    r'^"([^"]*)" tenta designar "([^"]*)" para a comissão da eleição "([^"]*)"$',
  ),

  // Validação do período, das autoridades e da publicação do manifesto — Issue #124.
  pendingStep(
    r'^informo um período de votação válido e concluo a configuração inicial$',
  ),
  pendingStep(
    r'^o calendário inicial registra o início e o encerramento informados$',
  ),
  pendingStep(
    r'^informo um período cujo encerramento é anterior ao início e tento concluir a configuração inicial$',
  ),
  pendingStep(
    r'^informo um período cujo encerramento é igual ao início e tento concluir a configuração inicial$',
  ),
  pendingStep(r'^a configuração inicial é recusada por período inválido$'),
  pendingStep(
    r'^que a eleição está em preparação sem todas as autoridades definidas$',
  ),
  pendingStep(r'^é atingido o horário previsto para a abertura$'),
  pendingStep(r'^a votação permanece fechada por falta de autoridades$'),
  pendingStep(r'^que a eleição está em preparação com duas chapas aprovadas$'),
  pendingStep(r'^cada chapa possui uma autoridade independente$'),
  pendingStep(r'^o servidor e a comissão possuem autoridades independentes$'),
  pendingStep(r'^a eleição aguarda o horário de abertura$'),
  pendingStep(
    r'^que a eleição aguarda o horário de abertura com o manifesto publicado$',
  ),
  pendingStep(r'^a votação é aberta automaticamente$'),
  pendingStep(
    r'^o manifesto publicado contém as duas chapas na ordem definida$',
  ),
  pendingStep(
    r'^o manifesto publicado contém a opção Nulo uma única vez na última posição$',
  ),
  pendingStep(
    r'^o manifesto contém uma autoridade do servidor, uma da comissão e uma para cada chapa$',
  ),
  pendingStep(r'^o manifesto é publicado com a assinatura do servidor$'),
  pendingStep(
    r'^que o manifesto oficial da eleição foi publicado e ainda não houve votos$',
  ),
  pendingStep(r'^a comissão tenta alterar as chapas ou a ordem das opções$'),
  pendingStep(r'^a alteração da configuração eleitoral é recusada$'),
  pendingStep(r'^o manifesto publicado permanece inalterado$'),
  pendingStep(
    r'^"([^"]*)" não passa a integrar a comissão da eleição "([^"]*)"$',
  ),
  // Permissões específicas da comissão eleitoral — Issue #124.
  pendingStep(r'^possui a permissão administrativa "([^"]*)" em "([^"]*)"$'),
  pendingStep(
    r'^que a eleição "([^"]*)" está em preparação, sem manifesto publicado$',
  ),
  pendingStep(r'^configura a eleição "([^"]*)"$'),
  pendingStep(r'^a configuração da eleição é autorizada$'),
  pendingStep(r'^administra as chapas da eleição "([^"]*)"$'),
  pendingStep(r'^a administração das chapas é concluída$'),
  pendingStep(r'^solicita a apuração do resultado da eleição "([^"]*)"$'),
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
