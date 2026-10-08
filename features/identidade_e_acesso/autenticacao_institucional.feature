# language: pt

# O acesso inicial ao CApp é exclusivo para contas institucionais da UnB.
# Os e-mails abaixo são exemplos fictícios dessas contas.
# O formato do e-mail, por si só, não comprova a identidade nem que a conta é da UnB.
# A origem institucional da conta deve ser confirmada pelo serviço de autenticação,
# sem cadastro, convênio ou situação "ativa" de uma instituição no CApp.
# Provisionamento do perfil e associação de dados acadêmicos têm regras próprias.

@specification_fixture
Funcionalidade: Autenticação institucional de usuários da UnB
  Como estudante da Universidade de Brasília (UnB)
  Quero autenticar minha conta institucional Microsoft da UnB no CApp
  Para acessar os serviços e recursos acadêmicos restritos da comunidade

  Contexto:
    Dado que o usuário não está autenticado no aplicativo

  Regra: Uma conta institucional da UnB validada permite estabelecer a sessão de acesso

    @happy
    Cenário: Login bem-sucedido com conta institucional ativa da UnB
      Dado que o estudante possui uma conta institucional ativa da UnB com o e-mail "232012345@aluno.unb.br"
      Quando ele realiza a autenticação com suas credenciais institucionais
      Então o acesso à plataforma é concedido com sucesso
      E uma sessão autenticada é estabelecida para o estudante
      E a identidade autenticada do usuário fica disponível com o e-mail "232012345@aluno.unb.br"

    @happy
    Cenário: Primeiro acesso com conta institucional da UnB dispensa cadastro manual
      Dado que um estudante com o e-mail "262001234@aluno.unb.br" realiza o primeiro acesso com conta institucional ativa da UnB
      Quando ele conclui a validação de sua identidade institucional
      Então o sistema estabelece a sessão autenticada do usuário
      E não é solicitado cadastro manual para entrar no CApp

  Regra: O login exige uma conta institucional da UnB cuja autenticação seja aceita

    @sad
    Cenário: Tentativa de autenticação com credenciais institucionais inválidas
      Dado que um usuário possui credenciais institucionais inválidas para o e-mail "242099999@aluno.unb.br"
      Quando ele tenta realizar o login institucional
      Então a autenticação é recusada indicando credenciais inválidas
      E nenhuma sessão autenticada é estabelecida

    @sad
    Cenário: Tentativa de login com conta institucional revogada pela UnB
      Dado que o estudante possui uma conta institucional com o e-mail "191099999@aluno.unb.br" revogada pela UnB
      E o serviço de autenticação recusa o acesso dessa conta
      Quando ele tenta autenticar no CApp
      Então o aplicativo informa que não foi possível autenticar a conta institucional
      E o acesso à plataforma permanece bloqueado

    @sad
    Cenário: Conta pessoal Microsoft não permite login institucional
      Dado que o usuário possui uma conta pessoal Microsoft válida
      Quando ele tenta realizar o login institucional com essa conta
      Então o aplicativo informa que é necessário utilizar uma conta institucional da UnB
      E nenhuma sessão autenticada é estabelecida

    @sad
    Cenário: Conta institucional válida de outra universidade não permite acesso ao CApp
      Dado que o usuário possui uma conta institucional Microsoft válida de outra universidade
      E o serviço de autenticação confirma que a conta pertence a essa outra universidade
      Quando ele tenta realizar o login institucional com essa conta
      Então o aplicativo informa que é necessário utilizar uma conta institucional da UnB
      E nenhuma sessão autenticada é estabelecida

  Regra: Um login não concluído mantém o usuário sem sessão e permite nova tentativa

    Cenário: Estudante cancela o login institucional
      Dado que o estudante iniciou o login institucional
      Quando ele cancela a autenticação antes de concluí-la
      Então o aplicativo retorna à tela de entrada
      E nenhuma sessão autenticada é estabelecida
      E o estudante pode iniciar uma nova tentativa de login

    @sad
    Cenário: Serviço de autenticação indisponível durante o login
      Dado que o estudante possui uma conta institucional ativa da UnB com o e-mail "232012345@aluno.unb.br"
      E o serviço de autenticação está temporariamente indisponível
      Quando ele tenta realizar o login institucional
      Então o aplicativo informa a indisponibilidade temporária da autenticação
      E nenhuma sessão autenticada é estabelecida
      E o estudante pode iniciar uma nova tentativa de login

  Regra: Recursos protegidos exigem uma sessão autenticada válida

    @sad
    Cenário: Usuário não autenticado tenta acessar recurso restrito aos estudantes
      Quando ele tenta acessar um serviço restrito da comunidade acadêmica
      Então o acesso ao recurso é bloqueado por ausência de autenticação
      E o aplicativo solicita que o usuário realize a autenticação institucional

    @sad
    Cenário: Usuário não autenticado tenta realizar uma ação protegida
      Quando ele tenta realizar uma ação que exige autenticação
      Então a operação é impedida por ausência de autenticação
      E a ação solicitada não é realizada

    @sad
    Esquema do Cenário: Sessão sem validade não permite acesso a recurso restrito
      Dado que o estudante possui uma sessão "<situacao>"
      Quando ele tenta acessar um serviço restrito da comunidade acadêmica
      Então o acesso ao recurso é bloqueado por falta de uma sessão válida
      E o aplicativo solicita que o usuário realize a autenticação institucional

      Exemplos:
        | situacao                               |
        | expirada sem possibilidade de renovação |
        | inválida                               |

  Regra: Encerrar a sessão interrompe o acesso autenticado no aplicativo

    @happy
    Cenário: Estudante encerra a própria sessão
      Dado que o estudante possui uma sessão autenticada válida
      Quando ele solicita sair do CApp
      Então a sessão do estudante é encerrada no aplicativo
      E o aplicativo retorna à tela de entrada
      E o acesso a recursos restritos passa a exigir nova autenticação
