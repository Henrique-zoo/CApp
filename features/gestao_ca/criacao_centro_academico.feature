# language: pt

# Rastreabilidade:
# Relatório de Features — Aplicativo para Centros Acadêmicos, § 2.2.
#
# Escopo do MVP:
# Cadastro de um Centro Acadêmico com informações básicas e definição
# da gestão inicial, sem exigir fluxos avançados de importação ou migração.
#
# A criação dos cargos padrão e as regras específicas do cargo de Presidente
# são tratadas na funcionalidade de cargos padrão da gestão.
#
# A administração posterior dos integrantes é tratada na funcionalidade
# de gerenciamento de integrantes da gestão.

@gestao_ca @pending_backend @pending_mobile
Funcionalidade: Criação de Centro Acadêmico
  Como aluno interessado em criar um Centro Acadêmico
  Quero cadastrar um novo CA no aplicativo
  Para iniciar sua utilização pela comunidade acadêmica

  Contexto:
    Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
    E está autenticado no aplicativo

  Regra: Somente estudantes autorizados podem cadastrar Centros Acadêmicos

    @happy @backend @mobile
    Cenário: Estudante autorizado cadastra um CA com informações básicas e gestão inicial
      Dado que o estudante está autorizado a cadastrar Centros Acadêmicos
      Quando informa os seguintes dados básicos para o novo CA:
        | informação              | valor                                             |
        | nome oficial            | Centro Acadêmico de Engenharia de Redes          |
        | sigla                   | CAER                                              |
        | descrição institucional | Representação estudantil de Engenharia de Redes  |
      E indica a si mesmo como responsável inicial pela administração
      E confirma o cadastro
      Então o Centro Acadêmico "CAER" deve ser criado no CApp
      E o CA deve possuir o nome oficial "Centro Acadêmico de Engenharia de Redes"
      E o CA deve possuir a sigla "CAER"
      E o CA deve possuir a descrição institucional informada
      E o estudante "241098765@aluno.unb.br" deve constar como responsável inicial pela administração de "CAER"
      E o cadastro deve ser concluído sem exigir importação ou migração de dados anteriores

    @sad @backend @mobile
    Cenário: Estudante sem autorização não consegue cadastrar um Centro Acadêmico
      Dado que o estudante não está autorizado a cadastrar Centros Acadêmicos
      Quando tenta cadastrar um Centro Acadêmico com a sigla "CAER"
      Então a operação deve ser recusada por falta de autorização
      E o Centro Acadêmico "CAER" não deve ser criado
      E nenhuma associação à gestão inicial de "CAER" deve ser registrada

  Regra: O cadastro exige as informações básicas obrigatórias

    @sad @backend @mobile
    Cenário: Cadastro recusado quando o nome oficial não é informado
      Dado que o estudante está autorizado a cadastrar Centros Acadêmicos
      Quando tenta cadastrar um CA com a sigla "CAER" sem informar o nome oficial
      Então o cadastro deve ser recusado informando que o nome oficial é obrigatório
      E o Centro Acadêmico "CAER" não deve ser criado
      E nenhuma associação à gestão inicial deve ser registrada

    @sad @backend @mobile
    Cenário: Cadastro recusado quando a sigla não é informada
      Dado que o estudante está autorizado a cadastrar Centros Acadêmicos
      Quando tenta cadastrar um CA com o nome "Centro Acadêmico de Engenharia de Redes" sem informar a sigla
      Então o cadastro deve ser recusado informando que a sigla é obrigatória
      E nenhum Centro Acadêmico deve ser criado a partir da solicitação
      E nenhuma associação à gestão inicial deve ser registrada

  Regra: Os responsáveis iniciais devem ser usuários identificáveis no CApp

    @sad @backend @mobile
    Cenário: Cadastro recusado quando o responsável inicial não possui perfil no CApp
      Dado que o estudante está autorizado a cadastrar Centros Acadêmicos
      E não existe perfil no CApp para o e-mail "209999999@aluno.unb.br"
      Quando tenta cadastrar o CA "CAER"
      E indica "209999999@aluno.unb.br" como responsável inicial pela administração
      Então o cadastro deve ser recusado informando que o usuário não foi encontrado
      E o Centro Acadêmico "CAER" não deve ser criado
      E nenhuma associação à gestão inicial deve ser registrada
