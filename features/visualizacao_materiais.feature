# language: pt

# Referência: Relatório de Features do Projeto, seção 6.1.
# A referência não define uma ordenação específica para os materiais.
@pending_backend @pending_mobile
Funcionalidade: Visualização de materiais acadêmicos

  Como estudante
  Quero consultar materiais publicados do CA ativo
  Para encontrar conteúdos que apoiem meus estudos

  Regra: A listagem apresenta somente materiais publicados do CA ativo

    Cenário: Exibir somente materiais publicados do CA ativo
      Dado que o CA ativo é "CA de Computação"
      E existem os materiais publicados "Lista de Cálculo I" e "Resumo de Estruturas de Dados" nesse CA
      E existe o material "Lista de Cálculo em elaboração" não publicado nesse CA
      Quando consulto a lista de materiais
      Então vejo "Lista de Cálculo I" e "Resumo de Estruturas de Dados"
      E não vejo "Lista de Cálculo em elaboração"

    Cenário: Informar quando não existem materiais publicados
      Dado que o CA ativo é "CA de Computação"
      E não existem materiais publicados nesse CA
      Quando consulto a lista de materiais
      Então a listagem informa que não há materiais publicados

    Cenário: Exibir materiais somente do CA ativo
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Computação" nesse CA
      E existe o material publicado "Lista de Matemática" no CA "CA de Matemática"
      Quando consulto a lista de materiais
      Então vejo "Resumo de Computação"
      E não vejo "Lista de Matemática"
      Quando altero o CA ativo para "CA de Matemática"
      E consulto novamente a lista de materiais
      Então vejo "Lista de Matemática"
      E não vejo "Resumo de Computação"

    Cenário: Informar falha ao carregar os materiais
      Dado que não foi possível obter os materiais publicados do CA ativo
      Quando consulto a lista de materiais
      Então o aplicativo informa que não foi possível carregar os materiais

  Regra: O estudante acessa o conteúdo disponibilizado de materiais publicados

    Cenário: Visualizar os detalhes de um material publicado
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Estruturas de Dados" nesse CA
      E a descrição do material é "Resumo para a primeira avaliação."
      Quando consulto os detalhes do material "Resumo de Estruturas de Dados"
      Então vejo o título "Resumo de Estruturas de Dados"
      E vejo a descrição "Resumo para a primeira avaliação."

    Cenário: Acessar o conteúdo de um material publicado
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Referências de Cálculo I" nesse CA
      E o material possui conteúdo disponível
      Quando acesso o conteúdo do material "Referências de Cálculo I"
      Então o conteúdo publicado é disponibilizado

    Cenário: Não disponibilizar conteúdo de material não publicado
      Dado que o CA ativo é "CA de Computação"
      E existe o material "Lista de Cálculo em elaboração" não publicado nesse CA
      Quando tento acessar o conteúdo do material "Lista de Cálculo em elaboração"
      Então o conteúdo do material não é disponibilizado
