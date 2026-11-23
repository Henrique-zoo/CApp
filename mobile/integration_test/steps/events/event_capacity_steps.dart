/// O limite opcional de vagas precisa ser respeitado inclusive em inscrições simultâneas.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final eventCapacitySteps = <StepDefinition>[
  pendingStep(
    r'^1 participante permanece inscrito após o cancelamento que liberou uma vaga$',
  ),
  pendingStep(r'^2 participantes elegíveis aguardam na fila de espera$'),
  pendingStep(r'^2 participantes estão inscritos no evento "([^"]*)"$'),
  pendingStep(r'^a capacidade não é ultrapassada$'),
  pendingStep(
    r'^a mesma vaga não é atribuída simultaneamente a outro participante$',
  ),
  pendingStep(r'^a mesma vaga não é confirmada para os dois participantes$'),
  pendingStep(
    r'^a política de convocação da fila está definida para o evento$',
  ),
  pendingStep(
    r'^a vaga liberada é direcionada a um único participante elegível conforme a política definida$',
  ),
  pendingStep(
    r'^as tentativas de ocupar a única vaga disponível são processadas simultaneamente$',
  ),
  pendingStep(r'^defino o limite de 3 vagas e salvo o evento$'),
  pendingStep(
    r'^duas pessoas sem inscrição solicitam simultaneamente a última vaga$',
  ),
  pendingStep(r'^estou cadastrando o evento "([^"]*)"$'),
  pendingStep(r'^existe um participante elegível aguardando na fila$'),
  pendingStep(r'^já existem participantes inscritos nesse evento$'),
  pendingStep(r'^já possui 2 participantes inscritos$'),
  pendingStep(r'^minha inscrição ativa não é registrada$'),
  pendingStep(r'^minha inscrição ativa é preservada$'),
  pendingStep(r'^minha inscrição ativa é registrada$'),
  pendingStep(r'^minha participação na fila de espera é registrada$'),
  pendingStep(r'^minha participação na fila deixa de estar ativa$'),
  pendingStep(r'^no máximo um dos participantes recebe a vaga$'),
  pendingStep(r'^não sou considerado para futuras vagas liberadas nessa fila$'),
  pendingStep(r'^não sou registrado na fila de espera desse evento$'),
  pendingStep(
    r'^não tenho inscrição ativa nem participação na fila desse evento$',
  ),
  pendingStep(r'^não tenho inscrição ativa nesse evento$'),
  pendingStep(r'^o aplicativo informa que estou aguardando uma vaga$'),
  pendingStep(r'^o aplicativo informa que o evento está lotado$'),
  pendingStep(
    r'^o aplicativo não recusa a inscrição por limite de capacidade$',
  ),
  pendingStep(r'^o evento continua com 2 participantes inscritos$'),
  pendingStep(r'^o evento fica com 3 participantes inscritos$'),
  pendingStep(r'^o evento não possui limite de vagas$'),
  pendingStep(r'^o evento passa a ter 3 participantes inscritos$'),
  pendingStep(r'^o evento passa a ter uma vaga disponível$'),
  pendingStep(r'^o evento possui limite de 2 vagas$'),
  pendingStep(r'^o evento possui limite de 3 vagas$'),
  pendingStep(r'^o evento é salvo com limite de 3 vagas$'),
  pendingStep(r'^o limite passa a ser considerado nas novas inscrições$'),
  pendingStep(r'^o número de inscrições ativas não supera o limite$'),
  pendingStep(r'^o número de inscrições ativas permanece igual ao limite$'),
  pendingStep(r'^o número de participantes inscritos não supera o limite$'),
  pendingStep(r'^que estou aguardando na fila de espera do evento "([^"]*)"$'),
  pendingStep(r'^que estou inscrito no evento publicado "([^"]*)"$'),
  pendingStep(r'^que estou propondo um evento para o CA "([^"]*)"$'),
  pendingStep(r'^que existe o evento publicado "([^"]*)" no CA "([^"]*)"$'),
  pendingStep(r'^que o evento publicado "([^"]*)" possui limite de 3 vagas$'),
  pendingStep(r'^solicito entrar na fila de espera desse evento$'),
  pendingStep(r'^solicito entrar na fila de espera do evento "([^"]*)"$'),
  pendingStep(r'^solicito minha inscrição direta no evento "([^"]*)"$'),
  pendingStep(r'^solicito sair da fila de espera desse evento$'),
  pendingStep(r'^somente uma nova inscrição ativa é registrada$'),
  pendingStep(r'^um participante cancela sua inscrição no evento "([^"]*)"$'),
];
