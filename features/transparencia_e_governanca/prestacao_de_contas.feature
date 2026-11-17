# language: pt

# Referência: Relatório de Features, seções 10.4 e 2.4.
# Primeira entrega: receitas, despesas, documentos e consulta, sem integração bancária.
# @v1 identifica a entrega; @pending_backend e @pending_mobile suspendem a automação.
@v1 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Prestação de contas do Centro Acadêmico
  Como estudante ou membro autorizado da gestão
  Quero consultar ou publicar receitas, despesas e documentos financeiros
  Para acompanhar o uso dos recursos do CA

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" é o CA ativo

  Regra: Publicar a prestação de contas exige a permissão correspondente nesse CA

    Esquema do Cenário: Publicar um lançamento manual com documento financeiro
      Dado que integro a gestão do CA ativo
      E possuo a permissão "Publicar a prestação de contas" nesse CA
      Quando registro uma "<tipo>" com descrição "<descricao>", valor "<valor>" e documento "<documento>"
      Então o lançamento fica disponível na prestação de contas do CA ativo
      E seu tipo, descrição, valor e documento ficam associados ao lançamento

      Exemplos:
        | tipo    | descricao              | valor   | documento        |
        | receita | Venda de ingressos     | 1500.00 | vendas.pdf       |
        | despesa | Compra de suprimentos  | 350.50  | nota_fiscal.pdf  |

    Cenário: Integrante sem a permissão não publica lançamentos
      Dado que integro a gestão do CA ativo
      E não possuo a permissão "Publicar a prestação de contas" nesse CA
      Quando tento registrar um lançamento financeiro nesse CA
      Então a publicação é recusada por falta de permissão
      E nenhum lançamento financeiro é criado

    Cenário: Permissão financeira em outro CA não autoriza publicação no CA ativo
      Dado que possuo a permissão "Publicar a prestação de contas" somente em outro CA
      Quando tento registrar um lançamento financeiro no CA ativo
      Então a publicação é recusada por falta de permissão
      E a prestação de contas do CA ativo permanece inalterada

  Regra: Usuários podem consultar a prestação de contas do CA ativo

    Cenário: Consultar receitas e despesas publicadas
      Dado que estou autenticado no CApp
      E o CA ativo publicou receitas e despesas
      Quando consulto sua prestação de contas
      Então visualizo os lançamentos publicados com seus tipos, descrições e valores

    Cenário: Acessar o documento de um lançamento
      Dado que estou autenticado no CApp
      E existe um lançamento publicado com um documento financeiro anexado
      Quando consulto o documento desse lançamento
      Então o documento fica disponível para consulta

    Cenário: Trocar o contexto altera a prestação de contas consultada
      Dado que estou autenticado no CApp
      E existem lançamentos publicados em dois CAs diferentes
      Quando altero o CA ativo e consulto sua prestação de contas
      Então visualizo somente os lançamentos publicados pelo novo CA ativo

    Cenário: Informar ausência de lançamentos publicados
      Dado que estou autenticado no CApp
      E o CA ativo não possui lançamentos financeiros publicados
      Quando consulto sua prestação de contas
      Então a consulta informa que não existem lançamentos publicados
