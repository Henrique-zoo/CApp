# language: pt

# Segunda entrega: filtro pessoal sem substituir a consulta coletiva do fórum.
@v2 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Filtro das próprias demandas sobre o espaço físico
  Como usuário do aplicativo
  Quero localizar as demandas que registrei
  Para acompanhar minhas solicitações sem perder acesso ao fórum coletivo

  Regra: O usuário pode localizar suas demandas sem perder a visão coletiva do fórum

    Cenário: Filtrar as demandas registradas pelo próprio usuário
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Reparo da porta" registrada por mim
      E existe a demanda "Troca da lâmpada" registrada por "Usuário B"
      Quando filtro as demandas para exibir somente as minhas
      Então vejo a demanda "Reparo da porta"
      E não vejo a demanda "Troca da lâmpada"

    Cenário: Informar quando não existem demandas registradas pelo usuário
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existem demandas registradas por outros usuários nesse CA
      E não registrei demandas nesse CA
      Quando filtro as demandas para exibir somente as minhas
      Então a listagem informa que não existem demandas registradas por mim
      E a consulta coletiva do fórum continua disponível

    Cenário: Retornar à consulta coletiva após remover o filtro pessoal
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existem demandas registradas por mim e por outros usuários nesse CA
      E estou consultando somente as minhas demandas
      Quando removo o filtro das próprias demandas
      Então volto a consultar as demandas visíveis do fórum
      E vejo demandas registradas por mim e por outros usuários
