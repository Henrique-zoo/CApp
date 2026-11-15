/// Configuração e calendário eleitoral ligados ao contrato compartilhado.
///
/// Os cenários especificam chapa única, adiamento e cancelamento. As expressões
/// ficam registradas para futura automação real do aplicativo; pendingStep falha
/// quando executado e não prepara resultados fictícios. Vocabulário comum à
/// votação permanece no registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Passos ainda sem implementação funcional, preservando as tags de pendência.
final electionManagementSteps = <StepDefinition>[
  pendingStep(r'^que a comissão eleitoral foi indicada pela gestão do CA$'),
  pendingStep(r'^estou autenticado como integrante autorizado dessa comissão$'),
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
];
