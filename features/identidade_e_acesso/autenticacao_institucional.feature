# language: pt

# Convenção de Domínio - Universidade de Brasília (UnB):
# A identificação discente na UnB segue a estrutura de matrícula com 9 dígitos no formato AASXXXXXX:
#   AA     -> Ano de ingresso (ex.: 19, 23, 24, 26)
#   S      -> Semestre de ingresso (1 para o 1º semestre e 2 para o 2º semestre)
#   XXXXXX -> Sequencial do registro acadêmico na Secretaria de Administração Acadêmica (SAA)
# O endereço de e-mail institucional oficial de qualquer estudante é sempre:
#   <matricula>@aluno.unb.br
# Os cenários abaixo utilizam valores didáticos fictícios representando veteranos, calouros e egressos.

Funcionalidade: Autenticação institucional de usuários
  Como estudante da instituição de ensino
  Quero autenticar minha conta institucional no CApp
  Para acessar os serviços e recursos acadêmicos restritos da comunidade

  Contexto:
    Dado que a "Universidade de Brasília" é uma instituição de ensino ativa no CApp

  # ---------------------------------------------------------------------------
  # 1. Autenticação bem-sucedida e contexto de usuário
  # ---------------------------------------------------------------------------
  Regra: Estudantes com credenciais institucionais ativas autenticam com sucesso e obtêm sessão de acesso

    Cenário: Login bem-sucedido com conta institucional ativa
      Dado que o estudante possui uma conta institucional ativa com o e-mail "232012345@aluno.unb.br"
      Quando ele realiza a autenticação com suas credenciais institucionais
      Então o acesso à plataforma é concedido com sucesso
      E uma sessão autenticada é estabelecida para o estudante
      E o perfil acadêmico do usuário fica disponível com o e-mail "232012345@aluno.unb.br"

    Cenário: Primeiro acesso de estudante autenticado provisiona o perfil institucional
      Dado que um estudante com o e-mail "262001234@aluno.unb.br" realiza o primeiro acesso com conta institucional ativa
      Quando ele conclui a validação de sua identidade institucional
      Então o sistema estabelece a sessão autenticada do usuário
      E associa a identidade institucional ao novo perfil do estudante

  # ---------------------------------------------------------------------------
  # 2. Falhas e recusas de autenticação institucional
  # ---------------------------------------------------------------------------
  Regra: Credenciais inválidas, contas inativas ou instituições não conveniadas impedem o acesso

    Cenário: Tentativa de autenticação com credenciais institucionais inválidas
      Dado que um usuário possui credenciais institucionais inválidas para o e-mail "242099999@aluno.unb.br"
      Quando ele tenta realizar o login institucional
      Então a autenticação é recusada indicando credenciais inválidas
      E nenhuma sessão autenticada é estabelecida

    Cenário: Tentativa de login com conta institucional desativada ou revogada
      Dado que o estudante possui uma conta institucional com o e-mail "191099999@aluno.unb.br" revogada pela instituição
      Quando ele tenta autenticar no CApp
      Então a autenticação é recusada informando que o vínculo institucional está inativo
      E o acesso à plataforma permanece bloqueado

    Cenário: Tentativa de autenticação com instituição não cadastrada no CApp
      Dado que um estudante tenta autenticar com o e-mail "estudante@externa.edu.br" da "Universidade Desconhecida"
      Quando ele solicita a validação da sua identidade institucional
      Então a autenticação é recusada indicando instituição não reconhecida
      E nenhuma credencial de acesso é gerada

  # ---------------------------------------------------------------------------
  # 3. Bloqueio de recursos restritos para usuários não autenticados
  # ---------------------------------------------------------------------------
  Regra: Recursos acadêmicos restritos exigem sessão autenticada e barram acessos não autorizados

    Cenário: Usuário não autenticado tenta acessar recurso restrito aos estudantes
      Dado que o usuário não está autenticado no aplicativo
      Quando ele tenta acessar um serviço restrito da comunidade acadêmica
      Então o acesso ao recurso é bloqueado por ausência de autenticação
      E o aplicativo solicita que o usuário realize a autenticação institucional

    Cenário: Visitante não autenticado tenta realizar ação acadêmica protegida
      Dado que um visitante anônimo não possui sessão de usuário estabelecida
      Quando ele tenta submeter uma proposta em consulta estudantil restrita
      Então a operação é impedida exigindo credenciais institucionais ativas
      E nenhum registro acadêmico é persistido no sistema
