# language: pt

# Referência: Relatório de Features do Projeto, seção 4.4.
# A referência não define uma ordenação específica para os eventos.
@pending_backend @pending_mobile
Funcionalidade: Visualização de eventos publicados

  Como estudante
  Quero consultar eventos publicados do CA ativo
  Para conhecer as atividades disponíveis para participação

  Regra: A agenda apresenta somente eventos publicados do CA ativo

    Cenário: Exibir somente eventos publicados do CA ativo
      Dado que o CA ativo é "CA de Computação"
      E existem os eventos publicados "Semana Acadêmica" e "Palestra de IA" nesse CA
      E existe o evento "Hackathon em análise" proposto e ainda não homologado nesse CA
      E existe o evento "Encontro em rascunho" não publicado nesse CA
      Quando consulto a lista de eventos
      Então vejo "Semana Acadêmica" e "Palestra de IA"
      E não vejo "Hackathon em análise"
      E não vejo "Encontro em rascunho"

    Cenário: Informar quando não existem eventos publicados
      Dado que o CA ativo é "CA de Computação"
      E não existem eventos publicados nesse CA
      Quando consulto a lista de eventos
      Então a listagem informa que não há eventos publicados

    Cenário: Visualizar os detalhes de um evento publicado
      Dado que o CA ativo é "CA de Computação"
      E existe o evento publicado "Mudança de sala" nesse CA
      E o evento contém a informação "A palestra acontecerá no auditório 2."
      Quando consulto os detalhes do evento "Mudança de sala"
      Então vejo o título "Mudança de sala"
      E vejo a informação "A palestra acontecerá no auditório 2."

    Cenário: Não disponibilizar detalhes de evento não publicado
      Dado que o CA ativo é "CA de Computação"
      E existe o evento "Hackathon em análise" proposto e ainda não homologado nesse CA
      Quando tento consultar os detalhes do evento "Hackathon em análise"
      Então os detalhes do evento não são disponibilizados

    Cenário: Exibir eventos somente do CA ativo
      Dado que o CA ativo é "CA de Computação"
      E existe o evento publicado "Hackathon de Computação" nesse CA
      E existe o evento publicado "Semana de Matemática" no CA "CA de Matemática"
      Quando consulto a lista de eventos
      Então vejo "Hackathon de Computação"
      E não vejo "Semana de Matemática"
      Quando altero o CA ativo para "CA de Matemática"
      E consulto novamente a lista de eventos
      Então vejo "Semana de Matemática"
      E não vejo "Hackathon de Computação"

    Cenário: Informar falha ao carregar os eventos
      Dado que não foi possível obter os eventos publicados do CA ativo
      Quando consulto a lista de eventos
      Então o aplicativo informa que não foi possível carregar os eventos
