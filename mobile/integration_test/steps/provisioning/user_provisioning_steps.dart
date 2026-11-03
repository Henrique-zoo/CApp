/// O perfil precisa ser criado uma única vez e permitir a retomada do preenchimento até sua ativação.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final userProvisioningSteps = <StepDefinition>[
  pendingStep(
    r'^a criação do perfil não exige o preenchimento prévio dos dados complementares$',
  ),
  pendingStep(
    r'^ainda não existe um perfil do CApp associado a essa identidade$',
  ),
  pendingStep(
    r'^as duas solicitações recuperam o mesmo identificador interno$',
  ),
  pendingStep(
    r'^duas solicitações de primeiro acesso dessa identidade são processadas simultaneamente$',
  ),
  pendingStep(r'^existe somente um perfil do CApp para essa identidade$'),
  pendingStep(
    r'^informa todos os dados complementares obrigatórios válidos e confirma a conclusão$',
  ),
  pendingStep(r'^já informou parte dos dados complementares do perfil$'),
  pendingStep(r'^nenhum novo perfil é criado apenas pela alteração do e-mail$'),
  pendingStep(r'^nenhum novo perfil é criado para essa identidade$'),
  pendingStep(r'^o CApp cria um perfil associado à identidade autenticada$'),
  pendingStep(
    r'^o aplicativo informa quais dados obrigatórios ainda precisam ser preenchidos$',
  ),
  pendingStep(r'^o aplicativo permite continuar o preenchimento$'),
  pendingStep(r'^o aplicativo recupera o mesmo perfil ativo$'),
  pendingStep(r'^o aplicativo recupera o mesmo perfil incompleto$'),
  pendingStep(
    r'^o aplicativo recupera o perfil associado à identidade autenticada$',
  ),
  pendingStep(
    r'^o aplicativo solicita a conclusão dos dados obrigatórios do perfil$',
  ),
  pendingStep(r'^o e-mail dessa identidade foi alterado pela instituição$'),
  pendingStep(r'^o identificador interno original permanece inalterado$'),
  pendingStep(r'^o perfil existente passa ao status "([^"]*)"$'),
  pendingStep(r'^o perfil permanece com o status "([^"]*)"$'),
  pendingStep(r'^o perfil é criado com o status "([^"]*)"$'),
  pendingStep(r'^o usuário conclui seu primeiro acesso institucional$'),
  pendingStep(r'^os dados complementares já salvos permanecem disponíveis$'),
  pendingStep(r'^os dados informados ficam associados ao mesmo perfil$'),
  pendingStep(
    r'^que a identidade institucional do usuário foi autenticada e verificada$',
  ),
  pendingStep(
    r'^que o usuário já possui um perfil associado à sua identidade institucional$',
  ),
  pendingStep(r'^que o usuário possui um perfil com status "([^"]*)"$'),
  pendingStep(
    r'^retorna ao CApp com a mesma identidade institucional autenticada$',
  ),
  pendingStep(r'^seu identificador interno permanece inalterado$'),
  pendingStep(r'^seus dados próprios do CApp permanecem preservados$'),
  pendingStep(
    r'^tenta concluir o perfil sem informar todos os dados obrigatórios$',
  ),
];
