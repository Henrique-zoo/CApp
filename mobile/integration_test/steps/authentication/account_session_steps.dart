/// Vocabulário dos fluxos de conta interna, seleção e retomada da sessão.
///
/// As especificações de identidade compartilham estas expressões enquanto sua
/// automação Gherkin é integrada. Os registros usam pendingStep e falham ao ser
/// executados. A cobertura técnica do controlador e dos adaptadores permanece
/// nas suítes próprias, sem substituir a futura execução destes cenários.
library;

import '../../support/step_registry.dart';

/// Expressões do fluxo de contas consumidas pelo registro global do mobile.
final accountSessionSteps = <StepDefinition>[
  pendingStep(
    r'^que seu perfil interno está completo e possui uma credencial de e-mail e senha vinculada$',
  ),
  pendingStep(
    r'^o sistema provisiona uma única conta interna com status Incomplete$',
  ),
  pendingStep(
    r'^a primeira etapa da conclusão pede apelido, período atual do curso e semestre de ingresso na instituição$',
  ),
  pendingStep(
    r'^a segunda etapa pede e-mail de entrada, senha e confirmação da senha$',
  ),
  pendingStep(r'^o acesso à área principal aguarda a conclusão do perfil$'),
  pendingStep(r'^que nenhuma conta está selecionada no aplicativo$'),
  pendingStep(r'^o usuário abre a tela de entrada$'),
  pendingStep(r'^as contas salvas no dispositivo aparecem primeiro$'),
  pendingStep(
    r'^abaixo da lista aparecem as opções de entrar com e-mail e senha e entrar com Microsoft$',
  ),
  pendingStep(
    r'^a opção Microsoft apresenta seu logo e indica que cria a conta no primeiro acesso$',
  ),
  pendingStep(
    r'^os campos de e-mail e senha não são apresentados automaticamente$',
  ),
  pendingStep(
    r'^as duas opções continuam disponíveis quando não há contas salvas$',
  ),
  pendingStep(r'^que o usuário está no seletor de contas salvas$'),
  pendingStep(r'^ele escolhe entrar com e-mail e senha$'),
  pendingStep(r'^o formulário de acesso interno é apresentado$'),
  pendingStep(r'^ele preenche as credenciais e volta ao seletor$'),
  pendingStep(r'^as contas salvas voltam a aparecer$'),
  pendingStep(r'^as credenciais digitadas são descartadas$'),
  pendingStep(r'^nenhuma conta salva é esquecida$'),
  pendingStep(
    r'^que a conta foi provisionada como Incomplete após a confirmação institucional$',
  ),
  pendingStep(r'^o usuário abre o formulário de conclusão do cadastro$'),
  pendingStep(
    r'^a primeira etapa pede apelido, período atual do curso e semestre de ingresso na instituição$',
  ),
  pendingStep(r'^apresenta os dados do Graph em campos de texto$'),
  pendingStep(r'^só permite completar os campos cujo valor no Graph é nulo$'),
  pendingStep(r'^não pede as credenciais internas$'),
  pendingStep(r'^ele informa dados acadêmicos válidos e avança$'),
  pendingStep(
    r'^o mesmo formulário muda para a segunda etapa sem abrir outra tela$',
  ),
  pendingStep(
    r'^passa a pedir e-mail de entrada, senha e confirmação da senha$',
  ),
  pendingStep(r'^o e-mail institucional aparece como sugestão editável$'),
  pendingStep(r'^a conta permanece Incomplete até concluir a segunda etapa$'),
  pendingStep(r'^ele escolhe outro e-mail e define sua senha interna$'),
  pendingStep(r'^confirma a conclusão do cadastro$'),
  pendingStep(r'^a credencial interna é vinculada ao mesmo usuário Firebase$'),
  pendingStep(r'^o backend ativa a conta após validar os dados obrigatórios$'),
  pendingStep(r'^o usuário pode entrar na área principal$'),
  pendingStep(
    r'^que o usuário está na primeira etapa da conclusão do cadastro$',
  ),
  pendingStep(
    r'^ele tenta avançar sem um apelido válido, com período zero ou ingresso fora do formato AAAA\.1 ou AAAA\.2$',
  ),
  pendingStep(r'^o aplicativo indica os campos inválidos$'),
  pendingStep(r'^não abre a etapa de e-mail e senha$'),
  pendingStep(r'^nenhuma credencial interna é vinculada$'),
  pendingStep(
    r'^que o usuário avançou à segunda etapa com dados acadêmicos válidos$',
  ),
  pendingStep(r'^editou o e-mail de entrada e começou a preencher a senha$'),
  pendingStep(r'^ele volta à primeira etapa$'),
  pendingStep(
    r'^os dados acadêmicos preenchidos continuam disponíveis para edição$',
  ),
  pendingStep(r'^ele revisa esses dados e avança novamente$'),
  pendingStep(r'^o e-mail escolhido é preservado$'),
  pendingStep(r'^os campos de senha e confirmação estão vazios$'),
  pendingStep(r'^a conclusão usará os dados acadêmicos revisados$'),
  pendingStep(
    r'^que o Graph retornou nome, e-mail e curso nulos para um estudante confirmado$',
  ),
  pendingStep(r'^ele completa esses dados e conclui seu cadastro$'),
  pendingStep(r'^os complementos são guardados em seu perfil CApp$'),
  pendingStep(
    r'^a evidência institucional preserva os valores nulos do Graph$',
  ),
  pendingStep(
    r'^os complementos são apresentados como informados pelo usuário$',
  ),
  pendingStep(r'^que o Graph retornou nome, e-mail e curso não nulos$'),
  pendingStep(
    r'^uma requisição tenta substituir esses dados por complementos$',
  ),
  pendingStep(r'^a atualização é recusada$'),
  pendingStep(r'^os dados institucionais são preservados$'),
  pendingStep(r'^que o semestre civil corrente é 2026\.2$'),
  pendingStep(r'^o usuário informa que está no quarto período do curso$'),
  pendingStep(r'^o ingresso sugerido é 2025\.1$'),
  pendingStep(r'^ele ajusta manualmente o ingresso e muda o período atual$'),
  pendingStep(r'^o ingresso informado manualmente é preservado$'),
  pendingStep(r'^que o usuário adiou a conclusão do cadastro$'),
  pendingStep(
    r'^sua conta incompleta e sua confirmação institucional vigente foram preservadas$',
  ),
  pendingStep(r'^ele seleciona sua conta salva com sessão utilizável$'),
  pendingStep(r'^a conclusão do cadastro é apresentada novamente$'),
  pendingStep(r'^não é necessária uma nova autenticação Microsoft$'),
  pendingStep(r'^que a conta está Incomplete$'),
  pendingStep(
    r'^faltam apelido, período atual ou semestre de ingresso no perfil enviado$',
  ),
  pendingStep(r'^a conclusão é recusada$'),
  pendingStep(r'^a conta permanece Incomplete$'),
  pendingStep(
    r'^que o usuário concluiu a entrada com perfil ativo e confirmação institucional vigente$',
  ),
  pendingStep(r'^a aba Início é apresentada sem os dados pessoais da conta$'),
  pendingStep(r'^o rodapé oferece ícones de Início e Conta$'),
  pendingStep(r'^ele seleciona Conta$'),
  pendingStep(r'^os dados de sua conta são apresentados$'),
  pendingStep(r'^ao fim da página aparece o botão vermelho Sair da conta$'),
  pendingStep(r'^ele seleciona Início novamente$'),
  pendingStep(r'^os dados da conta deixam de ser apresentados$'),
  pendingStep(r'^que o usuário abriu a página Conta$'),
  pendingStep(r'^ele seleciona Sair da conta$'),
  pendingStep(r'^a janela pergunta Tem certeza que deseja sair\?$'),
  pendingStep(r'^Esquecer conta está desmarcado$'),
  pendingStep(r'^Exigir login está marcado e pode ser editado$'),
  pendingStep(r'^a sessão permanece ativa até a confirmação$'),
  pendingStep(r'^que o usuário abriu a janela de confirmação da saída$'),
  pendingStep(r'^desmarcou Exigir login$'),
  pendingStep(r'^ele marca Esquecer conta$'),
  pendingStep(r'^Exigir login fica marcado e não pode ser editado$'),
  pendingStep(r'^ele desmarca Esquecer conta$'),
  pendingStep(r'^Exigir login continua marcado e pode ser editado$'),
  pendingStep(r'^ele cancela a janela$'),
  pendingStep(r'^a sessão permanece ativa$'),
  pendingStep(
    r'^as contas salvas e suas políticas de entrada são preservadas$',
  ),
  pendingStep(
    r'^que o usuário está em uma conta com perfil ativo e confirmação institucional vigente$',
  ),
  pendingStep(
    r'^ele desmarca Esquecer conta e Exigir login e confirma a saída$',
  ),
  pendingStep(r'^nenhuma conta fica selecionada no aplicativo$'),
  pendingStep(r'^sua conta continua na lista de contas salvas$'),
  pendingStep(r'^ele seleciona essa conta com sessão utilizável$'),
  pendingStep(
    r'^pode entrar sem informar senha ou autenticar novamente pela Microsoft$',
  ),
  pendingStep(
    r'^que o usuário possui uma credencial interna vinculada à conta selecionada$',
  ),
  pendingStep(
    r'^ele confirma a saída com Exigir login marcado e Esquecer conta desmarcado$',
  ),
  pendingStep(r'^a conta continua na lista de contas salvas$'),
  pendingStep(r'^as credenciais locais dessa conta são encerradas$'),
  pendingStep(
    r'^ele seleciona essa conta novamente, inclusive após reiniciar o aplicativo$',
  ),
  pendingStep(r'^deve informar sua senha interna antes de entrar$'),
  pendingStep(r'^que há mais de uma conta salva no dispositivo$'),
  pendingStep(
    r'^o usuário marca Esquecer conta e confirma a saída da conta selecionada$',
  ),
  pendingStep(
    r'^as credenciais e as fotografias locais dessa conta são removidas$',
  ),
  pendingStep(r'^ela deixa de aparecer no seletor$'),
  pendingStep(r'^as outras contas salvas são preservadas$'),
  pendingStep(r'^nenhuma conta remota é excluída$'),
  pendingStep(
    r'^que uma conta interna possui confirmação institucional ainda vigente$',
  ),
  pendingStep(
    r'^o usuário entra com sua credencial interna ou sessão salva utilizável$',
  ),
  pendingStep(r'^o backend restaura o perfil sem consultar o Microsoft Graph$'),
  pendingStep(r'^não é solicitado login Microsoft$'),
  pendingStep(r'^que os 120 dias da confirmação institucional venceram$'),
  pendingStep(
    r'^o MSAL consegue autorizar silenciosamente a mesma identidade Microsoft$',
  ),
  pendingStep(r'^a conta é retomada$'),
  pendingStep(r'^o aplicativo sincroniza seu vínculo institucional$'),
  pendingStep(r'^o backend atribui uma nova validade de 120 dias$'),
  pendingStep(r'^nenhuma interface Microsoft é aberta$'),
  pendingStep(
    r'^que a sincronização institucional precisa de uma nova autorização Microsoft$',
  ),
  pendingStep(
    r'^a tentativa silenciosa informa que é necessária interação do usuário$',
  ),
  pendingStep(r'^a entrada é apresentada$'),
  pendingStep(
    r'^o usuário pode continuar com Microsoft para concluir a sincronização$',
  ),
  pendingStep(
    r'^a sessão Firebase não é confundida com uma confirmação institucional vigente$',
  ),
  pendingStep(
    r'^que uma tentativa silenciosa falhou por indisponibilidade temporária$',
  ),
  pendingStep(r'^o aplicativo permanece aberto ou é retomado$'),
  pendingStep(r'^novas tentativas respeitam a espera progressiva da sessão$'),
  pendingStep(r'^não são executadas tentativas simultâneas$'),
  pendingStep(r'^uma falha de rede não abre o login Microsoft$'),
  pendingStep(r'^o usuário autentica novamente pela opção Microsoft$'),
  pendingStep(r'^o aplicativo não solicita uma credencial Graph ao MSAL$'),
  pendingStep(r'^o prazo original da confirmação institucional é preservado$'),
  pendingStep(
    r'^um perfil incompleto retoma o cadastro e um perfil ativo permite a entrada$',
  ),
  pendingStep(r'^que o login Microsoft estabeleceu a identidade Firebase$'),
  pendingStep(
    r'^a consulta da conta no backend falhou por indisponibilidade temporária$',
  ),
  pendingStep(
    r'^repetem a consulta da conta antes de decidir se é necessário sincronizar$',
  ),
  pendingStep(
    r'^uma falha de rede não é interpretada como conta ainda não provisionada$',
  ),
];
