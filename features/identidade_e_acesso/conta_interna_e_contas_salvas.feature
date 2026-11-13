# language: pt
# Credenciais são administradas pelo Firebase; nunca são gravadas no cache de perfil.
# A confirmação institucional é independente da sessão e dura 120 dias.
# Os cenários permanecem pendentes no runner BDD até suas integrações de steps.
@pending_backend @pending_mobile
Funcionalidade: Conta interna e seleção de contas salvas
  Como usuário do CApp
  Quero escolher como entrar novamente nas minhas contas
  Para retomar o aplicativo sem repetir a autenticação institucional desnecessariamente

  Regra: A entrada apresenta as contas salvas antes de pedir credenciais

    @happy @mobile
    Cenário: Escolher a forma de entrada
      Dado que nenhuma conta está selecionada no aplicativo
      Quando o usuário abre a tela de entrada
      Então as contas salvas no dispositivo aparecem primeiro
      E abaixo da lista aparecem as opções de entrar com e-mail e senha e entrar com Microsoft
      E a opção Microsoft apresenta seu logo e indica que cria a conta no primeiro acesso
      E os campos de e-mail e senha não são apresentados automaticamente
      E as duas opções continuam disponíveis quando não há contas salvas

    @happy @mobile
    Cenário: Abrir a entrada interna e voltar ao seletor
      Dado que o usuário está no seletor de contas salvas
      Quando ele escolhe entrar com e-mail e senha
      Então o formulário de acesso interno é apresentado
      Quando ele preenche as credenciais e volta ao seletor
      Então as contas salvas voltam a aparecer
      E as credenciais digitadas são descartadas
      E nenhuma conta salva é esquecida

  Regra: O perfil e a credencial interna pertencem à mesma conta autenticada institucionalmente

    @happy @backend @mobile
    Cenário: Concluir o perfil com um e-mail escolhido pelo usuário
      Dado que a conta foi provisionada como Incomplete após a confirmação institucional
      Quando o usuário abre o formulário de conclusão do cadastro
      Então a primeira etapa pede apelido, período atual do curso e semestre de ingresso na instituição
      E apresenta os dados do Graph em campos de texto
      E só permite completar os campos cujo valor no Graph é nulo
      E não pede as credenciais internas
      Quando ele informa dados acadêmicos válidos e avança
      Então o mesmo formulário muda para a segunda etapa sem abrir outra tela
      E passa a pedir e-mail de entrada, senha e confirmação da senha
      E o e-mail institucional aparece como sugestão editável
      E a conta permanece Incomplete até concluir a segunda etapa
      Quando ele escolhe outro e-mail e define sua senha interna
      E confirma a conclusão do cadastro
      Então a credencial interna é vinculada ao mesmo usuário Firebase
      E o backend ativa a conta após validar os dados obrigatórios
      E o usuário pode entrar na área principal

    @sad @mobile
    Cenário: Dados acadêmicos inválidos impedem avançar para as credenciais
      Dado que o usuário está na primeira etapa da conclusão do cadastro
      Quando ele tenta avançar sem um apelido válido, com período zero ou ingresso fora do formato AAAA.1 ou AAAA.2
      Então o aplicativo indica os campos inválidos
      E não abre a etapa de e-mail e senha
      E nenhuma credencial interna é vinculada

    @happy @mobile
    Cenário: Revisar os dados acadêmicos antes de concluir o cadastro
      Dado que o usuário avançou à segunda etapa com dados acadêmicos válidos
      E editou o e-mail de entrada e começou a preencher a senha
      Quando ele volta à primeira etapa
      Então os dados acadêmicos preenchidos continuam disponíveis para edição
      Quando ele revisa esses dados e avança novamente
      Então o e-mail escolhido é preservado
      E os campos de senha e confirmação estão vazios
      E a conclusão usará os dados acadêmicos revisados

    @happy @backend @mobile
    Cenário: Completar ausências do Graph sem alterar a evidência institucional
      Dado que o Graph retornou nome, e-mail e curso nulos para um estudante confirmado
      Quando ele completa esses dados e conclui seu cadastro
      Então os complementos são guardados em seu perfil CApp
      E a evidência institucional preserva os valores nulos do Graph
      E os complementos são apresentados como informados pelo usuário

    @sad @backend @mobile
    Cenário: Recusar a substituição de dados preenchidos pelo Graph
      Dado que o Graph retornou nome, e-mail e curso não nulos
      Quando uma requisição tenta substituir esses dados por complementos
      Então a atualização é recusada
      E os dados institucionais são preservados

    @happy @mobile
    Cenário: Sugerir ingresso contando o período em andamento
      Dado que o semestre civil corrente é 2026.2
      Quando o usuário informa que está no quarto período do curso
      Então o ingresso sugerido é 2025.1
      Quando ele ajusta manualmente o ingresso e muda o período atual
      Então o ingresso informado manualmente é preservado

    @happy @backend @mobile
    Cenário: Retomar uma conta incompleta
      Dado que o usuário adiou a conclusão do cadastro
      E sua conta incompleta e sua confirmação institucional vigente foram preservadas
      Quando ele seleciona sua conta salva com sessão utilizável
      Então a conclusão do cadastro é apresentada novamente
      E não é necessária uma nova autenticação Microsoft

    @sad @backend @mobile
    Cenário: Dados incompletos não ativam a conta
      Dado que a conta está Incomplete
      Quando faltam apelido, período atual ou semestre de ingresso no perfil enviado
      Então a conclusão é recusada
      E a conta permanece Incomplete

  Regra: Os dados da conta são consultados pela navegação da área autenticada

    @happy @mobile
    Cenário: Entrar no início e consultar a conta pelo rodapé
      Dado que o usuário concluiu a entrada com perfil ativo e confirmação institucional vigente
      Então a aba Início é apresentada sem os dados pessoais da conta
      E o rodapé oferece ícones de Início e Conta
      Quando ele seleciona Conta
      Então os dados de sua conta são apresentados
      E ao fim da página aparece o botão vermelho Sair da conta
      Quando ele seleciona Início novamente
      Então os dados da conta deixam de ser apresentados

  Regra: A saída confirmada define a forma de entrada futura somente para a conta escolhida

    @happy @mobile
    Cenário: Confirmar a saída com proteção de login por padrão
      Dado que o usuário abriu a página Conta
      Quando ele seleciona Sair da conta
      Então a janela pergunta Tem certeza que deseja sair?
      E Esquecer conta está desmarcado
      E Exigir login está marcado e pode ser editado
      E a sessão permanece ativa até a confirmação

    @happy @mobile
    Cenário: Esquecer conta exige autenticação na próxima entrada
      Dado que o usuário abriu a janela de confirmação da saída
      E desmarcou Exigir login
      Quando ele marca Esquecer conta
      Então Exigir login fica marcado e não pode ser editado
      Quando ele desmarca Esquecer conta
      Então Exigir login continua marcado e pode ser editado

    @happy @mobile
    Cenário: Cancelar a confirmação preserva a sessão
      Dado que o usuário abriu a janela de confirmação da saída
      Quando ele cancela a janela
      Então a sessão permanece ativa
      E as contas salvas e suas políticas de entrada são preservadas

    @happy @mobile
    Cenário: Manter entrada rápida ao sair
      Dado que o usuário está em uma conta com perfil ativo e confirmação institucional vigente
      Quando ele desmarca Esquecer conta e Exigir login e confirma a saída
      Então nenhuma conta fica selecionada no aplicativo
      E sua conta continua na lista de contas salvas
      Quando ele seleciona essa conta com sessão utilizável
      Então pode entrar sem informar senha ou autenticar novamente pela Microsoft

    @happy @mobile
    Cenário: Exigir senha na próxima entrada
      Dado que o usuário possui uma credencial interna vinculada à conta selecionada
      Quando ele confirma a saída com Exigir login marcado e Esquecer conta desmarcado
      Então a conta continua na lista de contas salvas
      E as credenciais locais dessa conta são encerradas
      Quando ele seleciona essa conta novamente, inclusive após reiniciar o aplicativo
      Então deve informar sua senha interna antes de entrar

    @happy @mobile
    Cenário: Esquecer uma conta
      Dado que há mais de uma conta salva no dispositivo
      Quando o usuário marca Esquecer conta e confirma a saída da conta selecionada
      Então as credenciais e as fotografias locais dessa conta são removidas
      E ela deixa de aparecer no seletor
      E as outras contas salvas são preservadas
      E nenhuma conta remota é excluída

  Regra: A sincronização institucional usa Microsoft somente quando necessária

    @happy @backend @mobile
    Cenário: Reutilizar a confirmação vigente após autenticar pela Microsoft
      Dado que uma conta interna possui confirmação institucional ainda vigente
      Quando o usuário autentica novamente pela opção Microsoft
      Então o backend restaura o perfil sem consultar o Microsoft Graph
      E o aplicativo não solicita uma credencial Graph ao MSAL
      E o prazo original da confirmação institucional é preservado
      E um perfil incompleto retoma o cadastro e um perfil ativo permite a entrada

    @sad @mobile
    Cenário: Repetir a consulta da conta após falha de rede no login Microsoft
      Dado que o login Microsoft estabeleceu a identidade Firebase
      E a consulta da conta no backend falhou por indisponibilidade temporária
      Quando o aplicativo permanece aberto ou é retomado
      Então novas tentativas respeitam a espera progressiva da sessão
      E repetem a consulta da conta antes de decidir se é necessário sincronizar
      E uma falha de rede não é interpretada como conta ainda não provisionada

    @happy @backend @mobile
    Cenário: Restaurar conta dentro dos 120 dias da confirmação
      Dado que uma conta interna possui confirmação institucional ainda vigente
      Quando o usuário entra com sua credencial interna ou sessão salva utilizável
      Então o backend restaura o perfil sem consultar o Microsoft Graph
      E não é solicitado login Microsoft

    @happy @backend @mobile
    Cenário: Renovar silenciosamente a confirmação vencida
      Dado que os 120 dias da confirmação institucional venceram
      E o MSAL consegue autorizar silenciosamente a mesma identidade Microsoft
      Quando a conta é retomada
      Então o aplicativo sincroniza seu vínculo institucional
      E o backend atribui uma nova validade de 120 dias
      E nenhuma interface Microsoft é aberta

    @sad @mobile
    Cenário: Solicitar interação somente quando necessária
      Dado que a sincronização institucional precisa de uma nova autorização Microsoft
      E a tentativa silenciosa informa que é necessária interação do usuário
      Quando a entrada é apresentada
      Então o usuário pode continuar com Microsoft para concluir a sincronização
      E a sessão Firebase não é confundida com uma confirmação institucional vigente

    @sad @mobile
    Cenário: Repetir silenciosamente após uma falha temporária
      Dado que uma tentativa silenciosa falhou por indisponibilidade temporária
      Quando o aplicativo permanece aberto ou é retomado
      Então novas tentativas respeitam a espera progressiva da sessão
      E não são executadas tentativas simultâneas
      E uma falha de rede não abre o login Microsoft
