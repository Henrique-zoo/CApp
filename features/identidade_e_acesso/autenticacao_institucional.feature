# language: pt
# Instituições fazem parte do domínio e devem estar cadastradas no CApp.
# Inicialmente, somente a UnB tem suporte; outras instituições nos exemplos são
# fictícias e especificam o comportamento quando forem cadastradas e integradas.
# Cada instituição possui seu próprio diretório e grupo de estudantes configurados.
# A origem da conta e a participação nesse grupo devem ser confirmadas pelo serviço;
# o domínio do e-mail e o curso informado não comprovam o vínculo estudantil.
# O status do catálogo institucional não é uma regra de bloqueio de autenticação.
# Os e-mails abaixo são fictícios. Tokens, tenant IDs e group IDs pertencem à
# preparação dos testes, não à linguagem funcional dos cenários compartilhados.
# Provisionamento do perfil e associação de dados acadêmicos têm regras próprias.
# @backend e @mobile indicam onde o cenário deve ser verificado.
# Mensagens funcionais são verificadas na resposta da API e na interface, respectivamente.
# Fluxos do provedor de login e navegação entre telas são exclusivos do mobile.
@pending_backend @pending_mobile
Funcionalidade: Autenticação de estudantes de instituições cadastradas
  Como estudante de uma instituição cadastrada no CApp
  Quero autenticar minha conta institucional Microsoft e confirmar meu vínculo estudantil
  Para acessar os serviços e recursos acadêmicos restritos da comunidade

  Contexto:
    Dado que o usuário não está autenticado no aplicativo

  Regra: Uma conta validada de instituição cadastrada com vínculo estudantil confirmado permite acesso

    @happy @backend @mobile
    Esquema do Cenário: Login bem-sucedido de estudante de instituição cadastrada
      Dado que a instituição "<instituicao>" está cadastrada no CApp
      E que o estudante possui uma conta institucional válida da instituição "<instituicao>" com o e-mail "<email>"
      E a instituição "<instituicao>" reconhece essa conta como estudante
      Quando ele realiza a autenticação com suas credenciais institucionais
      Então o acesso à plataforma é concedido com sucesso
      E uma sessão autenticada é estabelecida para o estudante
      E a identidade autenticada do usuário fica disponível com o e-mail "<email>"

      Exemplos:
        | instituicao              | email                      |
        | Universidade de Brasília | 232012345@aluno.unb.br      |
        | Universidade Exemplo     | estudante@aluno.example.edu |

    @happy @backend @mobile
    Cenário: Primeiro acesso institucional dispensa cadastro manual do usuário
      Dado que a instituição "Universidade de Brasília" está cadastrada no CApp
      E que um estudante com o e-mail "262001234@aluno.unb.br" realiza o primeiro acesso com conta institucional válida da instituição "Universidade de Brasília"
      E a instituição "Universidade de Brasília" reconhece essa conta como estudante
      Quando ele conclui a validação de sua identidade institucional
      Então o sistema estabelece a sessão autenticada do usuário
      E não é solicitado cadastro manual para entrar no CApp

  Regra: O login exige credenciais aceitas e uma instituição cadastrada

    @sad @mobile
    Cenário: Tentativa de autenticação com credenciais institucionais inválidas
      Dado que um usuário possui credenciais institucionais inválidas para o e-mail "242099999@aluno.unb.br"
      Quando ele tenta realizar o login institucional
      Então a autenticação é recusada indicando credenciais inválidas
      E nenhuma sessão autenticada é estabelecida

    @sad @mobile
    Cenário: Tentativa de login com conta revogada pela instituição
      Dado que a instituição "Universidade de Brasília" está cadastrada no CApp
      E que o estudante possui uma conta institucional com o e-mail "191099999@aluno.unb.br" revogada pela instituição "Universidade de Brasília"
      E o serviço de autenticação recusa o acesso dessa conta
      Quando ele tenta autenticar no CApp
      Então o aplicativo informa que não foi possível autenticar a conta institucional
      E o acesso à plataforma permanece bloqueado

    @sad @backend @mobile
    Cenário: Conta pessoal Microsoft não permite login institucional
      Dado que o usuário possui uma conta pessoal Microsoft válida
      Quando ele tenta realizar o login institucional com essa conta
      Então o aplicativo informa que é necessário utilizar uma conta institucional
      E o acesso à plataforma permanece bloqueado

    @sad @backend @mobile
    Cenário: Conta válida de instituição ainda não cadastrada não permite acesso
      Dado que a instituição "Universidade Exemplo" não está cadastrada no CApp
      E que o estudante possui uma conta institucional válida da instituição "Universidade Exemplo" com o e-mail "estudante@aluno.example.edu"
      E o serviço de autenticação confirma que a conta pertence à instituição "Universidade Exemplo"
      Quando ele tenta realizar o login institucional com essa conta
      Então o aplicativo informa que a instituição do usuário ainda não está cadastrada no CApp
      E o acesso à plataforma permanece bloqueado

  Regra: A instituição da conta deve confirmar o vínculo estudantil

    @sad @backend @mobile
    Cenário: Curso informado não substitui a confirmação de vínculo estudantil
      Dado que a instituição "Universidade de Brasília" está cadastrada no CApp
      E que o estudante possui uma conta institucional válida da instituição "Universidade de Brasília" com o e-mail "232012345@aluno.unb.br"
      E a conta informa o curso "Ciência da Computação"
      E a instituição "Universidade de Brasília" não reconhece essa conta como estudante
      Quando ele realiza a autenticação com suas credenciais institucionais
      Então o aplicativo informa que o usuário não foi reconhecido como aluno da instituição "Universidade de Brasília"
      E o acesso à plataforma permanece bloqueado

    @sad @backend @mobile
    Cenário: Reconhecimento por outra instituição não comprova vínculo com a instituição da conta
      Dado que a instituição "Universidade de Brasília" está cadastrada no CApp
      E que a instituição "Universidade Exemplo" está cadastrada no CApp
      E que o estudante possui uma conta institucional válida da instituição "Universidade Exemplo" com o e-mail "estudante@aluno.example.edu"
      E a instituição "Universidade de Brasília" reconhece essa conta como estudante
      E a instituição "Universidade Exemplo" não reconhece essa conta como estudante
      Quando ele realiza a autenticação com suas credenciais institucionais
      Então o aplicativo informa que o usuário não foi reconhecido como aluno da instituição "Universidade Exemplo"
      E o acesso à plataforma permanece bloqueado

  Regra: Um login não concluído mantém o usuário sem sessão e permite nova tentativa

    @mobile
    Cenário: Estudante cancela o login institucional
      Dado que o estudante iniciou o login institucional
      Quando ele cancela a autenticação antes de concluí-la
      Então o aplicativo retorna à tela de entrada
      E nenhuma sessão autenticada é estabelecida
      E o estudante pode iniciar uma nova tentativa de login

    @sad @mobile
    Cenário: Serviço de autenticação indisponível durante o login
      Dado que a instituição "Universidade de Brasília" está cadastrada no CApp
      E que o estudante possui uma conta institucional válida da instituição "Universidade de Brasília" com o e-mail "232012345@aluno.unb.br"
      E o serviço de autenticação está temporariamente indisponível
      Quando ele tenta realizar o login institucional
      Então o aplicativo informa a indisponibilidade temporária da autenticação
      E nenhuma sessão autenticada é estabelecida
      E o estudante pode iniciar uma nova tentativa de login

  Regra: Recursos protegidos exigem uma sessão autenticada válida

    @sad @backend @mobile
    Cenário: Usuário não autenticado tenta acessar recurso restrito aos estudantes
      Quando ele tenta acessar um serviço restrito da comunidade acadêmica
      Então o acesso ao recurso é bloqueado por ausência de autenticação
      E o aplicativo solicita que o usuário realize a autenticação institucional

    @sad @backend @mobile
    Cenário: Usuário não autenticado tenta realizar uma ação protegida
      Quando ele tenta realizar uma ação que exige autenticação
      Então a operação é impedida por ausência de autenticação
      E a ação solicitada não é realizada

    @sad @backend @mobile
    Esquema do Cenário: Sessão sem validade não permite acesso a recurso restrito
      Dado que o estudante possui uma sessão "<situacao>"
      Quando ele tenta acessar um serviço restrito da comunidade acadêmica
      Então o acesso ao recurso é bloqueado por falta de uma sessão válida
      E o aplicativo solicita que o usuário realize a autenticação institucional

      Exemplos:
        | situacao                                |
        | expirada sem possibilidade de renovação |
        | inválida                                |

  Regra: Encerrar a sessão interrompe o acesso autenticado no aplicativo

    @happy @mobile
    Cenário: Estudante encerra a própria sessão
      Dado que o estudante possui uma sessão autenticada válida
      Quando ele solicita sair do CApp
      Então a sessão do estudante é encerrada no aplicativo
      E o aplicativo retorna à tela de entrada
      E o acesso a recursos restritos passa a exigir nova autenticação
