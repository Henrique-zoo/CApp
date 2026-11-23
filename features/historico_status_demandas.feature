# language: pt

# Segunda entrega: rastreabilidade das alterações de status, com autor e data.
@v2 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Histórico de status das demandas sobre o espaço físico
  Como usuário do aplicativo
  Quero consultar o histórico de alterações de status
  Para acompanhar como a gestão tratou uma demanda

  Regra: O histórico preserva as alterações realizadas no andamento da demanda

    Cenário: Consultar o histórico de alterações de status
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E a demanda "Reparo da porta" possui as seguintes alterações de status:
        | status     | autor da alteração | data       |
        | em análise | Gestor A    | 2026-10-01 |
        | concluída  | Gestor B    | 2026-10-03 |
      Quando consulto os detalhes da demanda "Reparo da porta"
      Então vejo o histórico de alterações de status
      E cada alteração apresenta o status correspondente
      E cada alteração identifica quem realizou a alteração
      E cada alteração apresenta sua data

    Cenário: Preservar alterações anteriores no histórico
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E a demanda "Reparo da porta" teve uma alteração para "em análise"
      E posteriormente teve uma alteração para "concluída"
      Quando consulto o histórico da demanda "Reparo da porta"
      Então vejo o registro da alteração para "em análise"
      E vejo o registro da alteração posterior para "concluída"
      E a alteração anterior não foi substituída pela mais recente
