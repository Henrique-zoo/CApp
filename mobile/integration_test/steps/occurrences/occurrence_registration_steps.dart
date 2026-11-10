/// O registro de ocorrências precisa distinguir seus alvos e respeitar as permissões específicas de cada CA.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final occurrenceRegistrationSteps = <StepDefinition>[
  pendingStep(r'^a ocorrência fica associada ao estudante "([^"]*)"$'),
  pendingStep(r'^a ocorrência fica vinculada ao "([^"]*)"$'),
  pendingStep(r'^a ocorrência não exige associação a um estudante$'),
  pendingStep(r'^a ocorrência não é criada$'),
  pendingStep(r'^a ocorrência é encerrada$'),
  pendingStep(r'^a ocorrência é registrada no "([^"]*)"$'),
  pendingStep(r'^iniciei o registro de uma ocorrência para "([^"]*)"$'),
  pendingStep(r'^meu perfil não tem essa permissão no "([^"]*)"$'),
  pendingStep(
    r'^meu perfil não tem permissão para registrar ocorrências do próprio CA nesse CA$',
  ),
  pendingStep(
    r'^meu perfil não tem permissão para registrar ocorrências individuais nesse CA$',
  ),
  pendingStep(
    r'^meu perfil tem permissão para registrar ocorrências do próprio CA nesse CA$',
  ),
  pendingStep(
    r'^meu perfil tem permissão para registrar ocorrências individuais nesse CA$',
  ),
  pendingStep(
    r'^meu perfil tem permissão para registrar ocorrências individuais no "([^"]*)"$',
  ),
  pendingStep(r'^o aplicativo informa que a descrição é obrigatória$'),
  pendingStep(
    r'^o aplicativo informa que não tenho permissão para esse tipo de registro$',
  ),
  pendingStep(
    r'^o aplicativo informa que não tenho permissão para registrar ocorrências nesse CA$',
  ),
  pendingStep(r'^o aplicativo informa que o motivo é obrigatório$'),
  pendingStep(r'^o encerramento não remove o registro histórico$'),
  pendingStep(r'^o estudante "([^"]*)" participa de outros CAs$'),
  pendingStep(
    r'^o estudante "([^"]*)" pode ser associado a uma ocorrência nesse CA$',
  ),
  pendingStep(r'^o próprio CA é identificado como alvo da ocorrência$'),
  pendingStep(
    r'^o registro contém motivo, descrição, data e situação da ocorrência$',
  ),
  pendingStep(r'^o registro da ocorrência é preservado$'),
  pendingStep(r'^o registro não impõe banimento global ao estudante$'),
  pendingStep(r'^preenchi a descrição da ocorrência$'),
  pendingStep(r'^preenchi o motivo da ocorrência$'),
  pendingStep(r'^que existe uma ocorrência registrada no "([^"]*)"$'),
  pendingStep(
    r'^registro uma ocorrência contra o próprio CA com motivo "([^"]*)" e descrição "([^"]*)"$',
  ),
  pendingStep(
    r'^registro uma ocorrência individual para "([^"]*)" com motivo "([^"]*)" e descrição "([^"]*)"$',
  ),
  pendingStep(r'^registro uma ocorrência individual para "([^"]*)" nesse CA$'),
  pendingStep(r'^tento registrar uma ocorrência contra o próprio CA$'),
  pendingStep(r'^tento registrar uma ocorrência individual para "([^"]*)"$'),
  pendingStep(r'^tento salvar a ocorrência sem informar a descrição$'),
  pendingStep(r'^tento salvar a ocorrência sem informar o motivo$'),
  pendingStep(r'^"([^"]*)" é identificado como responsável pelo registro$'),
];
