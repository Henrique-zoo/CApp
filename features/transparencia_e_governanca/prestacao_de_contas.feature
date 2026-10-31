# language: pt

Funcionalidade: Prestação de contas e transparência financeira do Centro Acadêmico
  Como estudante ou membro da gestão de um Centro Acadêmico
  Quero registrar, retificar e consultar receitas, despesas e demonstrativos financeiros
  Para garantir a integridade, transparência e controle social dos recursos da entidade

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está ativo no CApp

  # ---------------------------------------------------------------------------
  # 1. Cadastro e Publicação de Informações Financeiras Manuais
  # ---------------------------------------------------------------------------
  Regra: Apenas membros da gestão com permissão explícita podem publicar lançamentos financeiros

    Cenário: Membro da gestão com permissão financeira publica uma receita manual com comprovante
      Dado que o usuário é um membro da gestão com permissão explícita de prestação de contas
      Quando ele cadastra um lançamento financeiro manual com os seguintes dados:
        | tipo de transação | Receita                                 |
        | título            | Venda de Ingressos da Semana Acadêmica  |
        | valor             | 1500.00                                 |
        | data              | 2026-10-01                              |
        | categoria         | Eventos Acadêmicos                      |
        | comprovante       | relatorio_vendas_lote1.pdf              |
      Então o lançamento financeiro é publicado com sucesso
      E o saldo consolidado do Centro Acadêmico reflete a nova receita de "1500.00"

    Cenário: Membro da gestão com permissão financeira publica uma despesa manual com comprovante
      Dado que o usuário é um membro da gestão com permissão explícita de prestação de contas
      Quando ele cadastra um lançamento financeiro manual com os seguintes dados:
        | tipo de transação | Despesa                                 |
        | título            | Compra de Suprimentos para a Sala do CA |
        | valor             | 350.50                                  |
        | data              | 2026-10-02                              |
        | categoria         | Manutenção e Infraestrutura             |
        | comprovante       | nota_fiscal_suprimentos_089.pdf         |
      Então o lançamento financeiro é publicado com sucesso
      E o saldo consolidado do Centro Acadêmico deduz a nova despesa de "350.50"

    Cenário: Estudante comum tenta publicar registro de prestação de contas
      Dado que o usuário é um estudante regularmente matriculado sem cargo financeiro na gestão
      Quando ele tenta cadastrar um novo lançamento de despesa no valor de "200.00"
      Então a publicação é recusada por ausência de permissão financeira
      E nenhum lançamento é incluído nos registros contábeis do Centro Acadêmico

    Cenário: Membro da gestão sem permissão explícita tenta publicar registro financeiro
      Dado que o usuário é um membro da gestão ativa, mas não possui permissão de prestação de contas
      Quando ele tenta publicar um lançamento financeiro
      Então a publicação é recusada indicando falta de autorização para movimentação contábil
      E o balanço financeiro permanece inalterado

  # ---------------------------------------------------------------------------
  # 2. Retificação e Atualização de Lançamentos Financeiros
  # ---------------------------------------------------------------------------
  Regra: Retificações em lançamentos contábeis exigem autorização e preservam histórico de justificativa

    Cenário: Gestor autorizado retifica valor de lançamento publicado
      Dado que existe um lançamento de despesa publicado com o identificador "DESP-001" no valor de "120.00"
      E que o usuário é um membro da gestão com permissão explícita de prestação de contas
      Quando ele retifica o lançamento "DESP-001" alterando o valor para "150.00" com a justificativa "Ajuste conforme nota fiscal corrigida"
      Então o lançamento "DESP-001" é atualizado para o valor de "150.00"
      E o histórico de retificações registra a alteração com justificativa, data e autor

    Cenário: Tentativa de retificar lançamento inexistente
      Dado que o usuário é um membro da gestão com permissão explícita de prestação de contas
      Quando ele tenta retificar o lançamento com identificador "DESP-999" que não existe
      Então a operação de retificação é recusada informando que o lançamento não foi encontrado

    Cenário: Usuário não autorizado tenta retificar lançamento publicado
      Dado que existe um lançamento de receita publicado com o identificador "REC-001" no valor de "800.00"
      E que o usuário é um estudante sem permissão financeira
      Quando ele tenta retificar o valor do lançamento "REC-001"
      Então a alteração é bloqueada por falta de permissão administrativa
      E o valor do lançamento "REC-001" permanece "800.00"

  # ---------------------------------------------------------------------------
  # 3. Consulta Pública, Demonstrativos e Comprovantes
  # ---------------------------------------------------------------------------
  Regra: Balanços consolidados, demonstrativos e comprovantes são públicos para consulta da comunidade acadêmica

    Cenário: Estudante consulta balanço consolidado e demonstrativo financeiro
      Dado que o Centro Acadêmico possui lançamentos publicados contendo receitas de "3000.00" e despesas de "1200.00"
      Quando qualquer estudante consulta o demonstrativo financeiro do Centro Acadêmico
      Então ele visualiza o total de receitas acumuladas de "3000.00"
      E visualiza o total de despesas acumuladas de "1200.00"
      E visualiza o saldo atual consolidado de "1800.00"

    Cenário: Estudante visualiza detalhes e comprovante de um lançamento financeiro
      Dado que existe um lançamento publicado com comprovante "comprovante_reforma_lab.pdf" anexado
      Quando um estudante consulta os detalhes desse lançamento
      Então o comprovante anexado é disponibilizado para visualização pública e download
      E todos os metadados como categoria, data e justificativa são exibidos com transparência
