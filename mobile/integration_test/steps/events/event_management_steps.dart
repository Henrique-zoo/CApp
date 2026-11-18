/// A gestão de eventos precisa respeitar as permissões e a homologação, inclusive no ambiente físico do CA.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final eventManagementSteps = <StepDefinition>[
  pendingStep(r'^a criação não publica automaticamente o evento$'),
  pendingStep(r'^a data de publicação do evento fica registrada$'),
  pendingStep(
    r'^a operação é recusada informando que o horário de término deve ser posterior ao início$',
  ),
  pendingStep(
    r'^a operação é recusada informando que o término deve ser posterior ao início$',
  ),
  pendingStep(
    r'^a operação é recusada informando que "([^"]*)" do evento é obrigatório$',
  ),
  pendingStep(r'^a opção "([^"]*)" permanece identificada como seu local$'),
  pendingStep(r'^aprova a alteração do evento$'),
  pendingStep(r'^aprova a publicação do evento$'),
  pendingStep(
    r'^cria e publica diretamente um evento válido em um local diferente do ambiente físico do próprio CA$',
  ),
  pendingStep(
    r'^cria um evento válido com o local "([^"]*)" e solicita a publicação$',
  ),
  pendingStep(
    r'^cria um evento válido com o local "([^"]*)" e solicita a publicação direta$',
  ),
  pendingStep(r'^existe em "([^"]*)" o evento publicado "([^"]*)"$'),
  pendingStep(r'^nenhum horário de término é inventado pelo sistema$'),
  pendingStep(r'^nenhum novo evento é registrado em "([^"]*)"$'),
  pendingStep(r'^não aparece na agenda de eventos publicados de "([^"]*)"$'),
  pendingStep(r'^o aplicativo identifica que a publicação exige homologação$'),
  pendingStep(
    r'^o estudante "([^"]*)" não possui cargo de gestão em "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" possui perfil no CApp e não integra a gestão de "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" tenta criar e publicar diretamente em "([^"]*)" um evento com os seguintes dados:$',
  ),
  pendingStep(r'^o evento aguarda uma ação explícita de homologação$'),
  pendingStep(r'^o evento aparece na agenda pública de "([^"]*)"$'),
  pendingStep(r'^o evento fica pendente de homologação em "([^"]*)"$'),
  pendingStep(r'^o evento fica pendente de nova homologação$'),
  pendingStep(
    r'^o evento fica visível na agenda de eventos publicados de "([^"]*)" sem exigir homologação$',
  ),
  pendingStep(
    r'^o evento não aparece na agenda de eventos publicados de "([^"]*)"$',
  ),
  pendingStep(r'^o evento não aparece na agenda pública antes da aprovação$'),
  pendingStep(r'^o evento não é publicado na agenda de "([^"]*)"$'),
  pendingStep(
    r'^o evento passa a apresentar em "([^"]*)" os dados atualizados:$',
  ),
  pendingStep(r'^o evento passa a ficar pendente de homologação$'),
  pendingStep(
    r'^o evento passa a ficar pendente de nova homologação em "([^"]*)"$',
  ),
  pendingStep(
    r'^o evento passa a figurar na agenda de eventos publicados de "([^"]*)"$',
  ),
  pendingStep(
    r'^o evento registra "([^"]*)" como autor responsável em "([^"]*)"$',
  ),
  pendingStep(r'^o evento retorna à agenda pública com os dados homologados$'),
  pendingStep(
    r'^o evento "([^"]*)" fica registrado como rascunho em "([^"]*)"$',
  ),
  pendingStep(
    r'^o evento "([^"]*)" mantém a situação de rascunho em "([^"]*)"$',
  ),
  pendingStep(r'^o evento "([^"]*)" não é publicado na agenda de "([^"]*)"$'),
  pendingStep(r'^o evento "([^"]*)" passa a estar publicado em "([^"]*)"$'),
  pendingStep(r'^o evento "([^"]*)" permanece como rascunho em "([^"]*)"$'),
  pendingStep(r'^o evento "([^"]*)" permanece inalterado em "([^"]*)"$'),
  pendingStep(r'^o evento é publicado na agenda de "([^"]*)"$'),
  pendingStep(r'^o evento é registrado sem horário de término$'),
  pendingStep(
    r'^o evento é retirado da agenda pública enquanto aguarda a homologação$',
  ),
  pendingStep(
    r'^o gestor com publicação direta solicita a publicação desse rascunho em "([^"]*)"$',
  ),
  pendingStep(r'^o gestor "([^"]*)" altera o local para "([^"]*)"$'),
  pendingStep(
    r'^o gestor "([^"]*)" cria como rascunho em "([^"]*)" um evento com os seguintes dados:$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" cria e publica diretamente em "([^"]*)" um evento com os seguintes dados:$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" cria em "([^"]*)" um evento com título, descrição, início e local válidos sem informar o término$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" edita o evento "([^"]*)" em "([^"]*)" para:$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" publica diretamente o evento "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" seleciona "([^"]*)" como local de um evento válido$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" tenta criar em "([^"]*)" um evento com início e término em "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" tenta criar em "([^"]*)" um evento com os seguintes dados:$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" tenta editar o evento "([^"]*)" em "([^"]*)"$',
  ),
  pendingStep(
    r'^o gestor "([^"]*)" tenta editar o evento "([^"]*)" em "([^"]*)" para:$',
  ),
  pendingStep(r'^o local desse evento não é o ambiente físico do próprio CA$'),
  pendingStep(r'^o local do evento fica associado ao espaço do CA "([^"]*)"$'),
  pendingStep(
    r'^o proponente pode identificar que a publicação foi rejeitada$',
  ),
  pendingStep(r'^permanece fora da agenda de eventos publicados de "([^"]*)"$'),
  pendingStep(r'^permanece fora da agenda pública$'),
  pendingStep(
    r'^que a edição de um evento publicado em "([^"]*)" o retirou da agenda pública$',
  ),
  pendingStep(
    r'^que existe em "([^"]*)" o evento em rascunho "([^"]*)" com os seguintes dados:$',
  ),
  pendingStep(
    r'^que existe em "([^"]*)" o evento em rascunho "([^"]*)" criado por "([^"]*)"$',
  ),
  pendingStep(
    r'^que existe em "([^"]*)" o evento publicado "([^"]*)" com os seguintes dados:$',
  ),
  pendingStep(
    r'^que existe em "([^"]*)" um evento no local "([^"]*)" aguardando homologação$',
  ),
  pendingStep(r'^que existe em "([^"]*)" um evento publicado em outro local$'),
  pendingStep(r'^que existe um evento em rascunho no local "([^"]*)"$'),
  pendingStep(
    r'^que o gestor "([^"]*)" possui publicação direta de eventos em "([^"]*)"$',
  ),
  pendingStep(r'^que o integrante possui a permissão "([^"]*)" em "([^"]*)"$'),
  pendingStep(r'^rejeita a publicação do evento$'),
  pendingStep(r'^sai da agenda pública até que a alteração seja aprovada$'),
  pendingStep(
    r'^"([^"]*)" tenta criar e publicar diretamente em "([^"]*)" um evento com os seguintes dados:$',
  ),
  pendingStep(
    r'^"([^"]*)" tenta editar o evento "([^"]*)" em "([^"]*)" para:$',
  ),
  pendingStep(
    r'^"([^"]*)" tenta publicar diretamente o evento "([^"]*)" em "([^"]*)"$',
  ),
];
