# language: pt

# Rastreabilidade: Relatório de Features, seção 2.2, e decisões de produto.
# Qualquer estudante autenticado pode criar um CA; não há autorização prévia.
# O nome oficial é obrigatório; a sigla é opcional.
# A criação dos cargos padrão e a administração posterior dos integrantes são
# especificadas nas respectivas funcionalidades da gestão.

@gestao_ca @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Criação de Centro Acadêmico
  Como aluno interessado em criar um Centro Acadêmico
  Quero cadastrar um novo CA no aplicativo
  Para iniciar sua utilização pela comunidade acadêmica

  Contexto:
    Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
    E está autenticado no aplicativo

  Regra: Qualquer estudante autenticado pode cadastrar um Centro Acadêmico

    Cenário: Estudante cadastra um CA com informações básicas e gestão inicial
      Quando informa os seguintes dados básicos para o novo CA:
        | informação              | valor                                            |
        | nome oficial            | Centro Acadêmico de Engenharia de Redes           |
        | sigla                   | CAER                                             |
        | descrição institucional | Representação estudantil de Engenharia de Redes   |
      E indica a si mesmo como responsável inicial pela administração
      E confirma o cadastro
      Então o Centro Acadêmico "CAER" deve ser criado no CApp
      E o CA deve possuir o nome oficial "Centro Acadêmico de Engenharia de Redes"
      E o CA deve possuir a sigla "CAER"
      E o CA deve possuir a descrição institucional informada
      E o estudante "241098765@aluno.unb.br" deve constar como responsável inicial pela administração de "CAER"
      E o cadastro deve ser concluído sem exigir importação ou migração de dados anteriores

    Cenário: Usuário sem sessão autenticada não consegue cadastrar um CA
      Dado que encerrei minha sessão no aplicativo
      Quando tento cadastrar um Centro Acadêmico com a sigla "CAER"
      Então a operação deve ser recusada informando que a autenticação é obrigatória
      E o Centro Acadêmico "CAER" não deve ser criado
      E nenhuma associação à gestão inicial de "CAER" deve ser registrada

  Regra: O nome oficial é obrigatório e a sigla é opcional

    Cenário: Cadastro recusado quando o nome oficial não é informado
      Quando tenta cadastrar um CA com a sigla "CAER" sem informar o nome oficial
      Então o cadastro deve ser recusado informando que o nome oficial é obrigatório
      E o Centro Acadêmico "CAER" não deve ser criado
      E nenhuma associação à gestão inicial deve ser registrada

    Cenário: Estudante cadastra um CA sem informar sigla
      Quando informa o nome oficial "Centro Acadêmico de Engenharia de Redes"
      E não informa uma sigla para o novo CA
      E indica a si mesmo como responsável inicial pela administração
      E confirma o cadastro
      Então o CA com o nome oficial "Centro Acadêmico de Engenharia de Redes" deve ser criado
      E o CA deve permanecer sem sigla
      E o estudante deve constar como responsável inicial pela administração do CA criado

  Regra: Os responsáveis iniciais devem ser usuários identificáveis no CApp

    Cenário: Cadastro recusado quando o responsável inicial não possui perfil no CApp
      Dado que não existe perfil no CApp para o e-mail "209999999@aluno.unb.br"
      Quando tenta cadastrar o CA "CAER"
      E indica "209999999@aluno.unb.br" como responsável inicial pela administração
      Então o cadastro deve ser recusado informando que o usuário não foi encontrado
      E o Centro Acadêmico "CAER" não deve ser criado
      E nenhuma associação à gestão inicial deve ser registrada
