/// Passos pendentes de provisionamento usuario.
///
/// Registra o fluxo esperado sem simular autenticação, backend ou persistência.
/// A automação futura deverá exercitar o aplicativo e seus resultados antes de
/// remover `@pending_mobile` da especificação compartilhada.
library;

import '../../support/step_registry.dart';

final userProvisioningSteps = <StepDefinition>[
  pendingStep(
    r'^que a identidade institucional do usuário foi autenticada e verificada$',
  ),
  pendingStep(
    r'^ainda não existe um perfil do CApp associado a essa identidade$',
  ),
  pendingStep(r'^o usuário conclui seu primeiro acesso institucional$'),
  pendingStep(r'^o CApp cria um perfil associado à identidade autenticada$'),
  pendingStep(r'^o perfil é criado com o status "([^"]*)"$'),
  pendingStep(
    r'^a criação do perfil não exige o preenchimento prévio dos dados complementares$',
  ),
  pendingStep(
    r'^o aplicativo solicita a conclusão dos dados obrigatórios do perfil$',
  ),
  pendingStep(r'^que o usuário possui um perfil com status "([^"]*)"$'),
  pendingStep(r'^já informou parte dos dados complementares do perfil$'),
  pendingStep(
    r'^retorna ao CApp com a mesma identidade institucional autenticada$',
  ),
  pendingStep(r'^o aplicativo recupera o mesmo perfil incompleto$'),
  pendingStep(r'^os dados complementares já salvos permanecem disponíveis$'),
  pendingStep(r'^o aplicativo permite continuar o preenchimento$'),
  pendingStep(
    r'^duas solicitações de primeiro acesso dessa identidade são processadas simultaneamente$',
  ),
  pendingStep(r'^existe somente um perfil do CApp para essa identidade$'),
  pendingStep(
    r'^as duas solicitações recuperam o mesmo identificador interno$',
  ),
  pendingStep(
    r'^informa todos os dados complementares obrigatórios válidos e confirma a conclusão$',
  ),
  pendingStep(r'^o perfil existente passa ao status "([^"]*)"$'),
  pendingStep(r'^os dados informados ficam associados ao mesmo perfil$'),
  pendingStep(r'^seu identificador interno permanece inalterado$'),
  pendingStep(
    r'^tenta concluir o perfil sem informar todos os dados obrigatórios$',
  ),
  pendingStep(r'^o perfil permanece com o status "([^"]*)"$'),
  pendingStep(
    r'^o aplicativo informa quais dados obrigatórios ainda precisam ser preenchidos$',
  ),
  pendingStep(r'^o aplicativo recupera o mesmo perfil ativo$'),
  pendingStep(r'^nenhum novo perfil é criado para essa identidade$'),
  pendingStep(r'^seus dados próprios do CApp permanecem preservados$'),
  pendingStep(
    r'^que o usuário já possui um perfil associado à sua identidade institucional$',
  ),
  pendingStep(r'^o e\-mail dessa identidade foi alterado pela instituição$'),
  pendingStep(
    r'^o aplicativo recupera o perfil associado à identidade autenticada$',
  ),
  pendingStep(r'^o identificador interno original permanece inalterado$'),
  pendingStep(
    r'^nenhum novo perfil é criado apenas pela alteração do e\-mail$',
  ),
];
