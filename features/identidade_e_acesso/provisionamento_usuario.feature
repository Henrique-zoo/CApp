# language: pt

# Convenção de Domínio - Universidade de Brasília (UnB):
# A identificação discente na UnB segue a estrutura de matrícula com 9 dígitos no formato AASXXXXXX:
#   AA     -> Ano de ingresso (ex.: 23 para veterano de 2023, 26 para calouro de 2026)
#   S      -> Semestre de ingresso (1 para o 1º semestre e 2 para o 2º semestre)
#   XXXXXX -> Sequencial do registro acadêmico na Secretaria de Administração Acadêmica (SAA)
# O endereço de e-mail institucional oficial de qualquer estudante é sempre:
#   <matricula>@aluno.unb.br
# Exemplos didáticos:
#   Calouro no primeiro acesso: "262001234@aluno.unb.br"
#   Veterano com perfil existente: "232012345@aluno.unb.br"
# Os cenários utilizam dados exclusivamente didáticos e fictícios, sem identificadores reais.

Funcionalidade: Provisionamento do usuário
  Como estudante da instituição de ensino
  Quero que meu perfil seja criado automaticamente após minha autenticação
  Para que eu não precise preencher manualmente meus dados de cadastro

  Contexto:
    Dado que a "Universidade de Brasília" é uma instituição de ensino ativa no CApp

  # ---------------------------------------------------------------------------
  # 1. Primeiro acesso e criação automática de perfil
  # ---------------------------------------------------------------------------
  Regra: O primeiro acesso autenticado cria automaticamente o perfil da aplicação

    Cenário: Provisionamento automático de perfil no primeiro login com credencial institucional válida
      Dado que o estudante calouro possui a credencial institucional ativa "262001234@aluno.unb.br"
      E o estudante ainda não possui cadastro no aplicativo
      Quando ele realiza o primeiro login com sucesso utilizando sua credencial institucional
      Então o sistema cria automaticamente o perfil interno do usuário
      E a conta interna do estudante é ativada sem exigir preenchimento de formulário de cadastro
      E o perfil da aplicação fica associado ao e-mail institucional "262001234@aluno.unb.br"

  # ---------------------------------------------------------------------------
  # 2. Vínculo unívoco entre identidade externa e interna
  # ---------------------------------------------------------------------------
  Regra: A identidade institucional externa deve ser associada de forma unívoca ao registro interno

    Cenário: Associação unívoca entre a identidade institucional externa e o registro interno do usuário
      Dado que um estudante com e-mail institucional "262001234@aluno.unb.br" conclui a validação externa
      Quando o sistema processa o provisionamento do perfil do usuário
      Então uma identidade institucional externa unívoca é associada ao registro interno do estudante
      E nenhum outro usuário interno pode ser vinculado à mesma identidade institucional externa

  # ---------------------------------------------------------------------------
  # 3. Recuperação de perfil para usuários já cadastrados
  # ---------------------------------------------------------------------------
  Regra: Usuários com cadastro existente não sofrem duplicação de perfil em novos acessos

    Cenário: Login de estudante já cadastrado apenas recupera o perfil e atualiza o acesso sem recriar dados
      Dado que o estudante veterano com e-mail "232012345@aluno.unb.br" já possui perfil cadastrado no aplicativo
      Quando ele realiza login no aplicativo utilizando sua credencial institucional
      Então o sistema apenas recupera o perfil existente do estudante
      E nenhum novo registro de usuário é criado no aplicativo
      E a data do último acesso da identidade institucional é atualizada

  # ---------------------------------------------------------------------------
  # 4. Prevenção de duplicidade e preservação de identificador
  # ---------------------------------------------------------------------------
  Regra: Reautenticações consecutivas preservam a integridade e unicidade do identificador interno

    Cenário: Prevenção de duplicidade: reautenticações preservam a unicidade do identificador interno
      Dado que o estudante veterano com e-mail "232012345@aluno.unb.br" possui um identificador interno único
      Quando ele se reautentica no aplicativo consecutivas vezes com a mesma credencial institucional
      Então o identificador interno original do estudante permanece inalterado
      E o sistema preserva um único registro de perfil para o estudante
