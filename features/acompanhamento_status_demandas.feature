# language: pt

# Referência: Relatório de Features do Projeto, seção 5.4.
# Issues relacionadas: #147 e #153.
# Recorte I07: acompanhamento do fórum de demandas sobre o espaço físico.
@pending_backend @pending_mobile
Funcionalidade: Acompanhamento de status das demandas sobre o espaço físico do CA

  Como usuário do aplicativo
  Quero acompanhar o status das demandas sobre o espaço físico do CA
  Para conhecer o andamento das solicitações

  Regra: O usuário pode consultar o status atual das demandas visíveis no fórum

    Cenário: Exibir o status das demandas na listagem
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existem as seguintes demandas sobre o espaço físico:
        | demanda          | status     |
        | Reparo da porta  | aberta     |
        | Troca da lâmpada | em análise |
        | Limpeza da sala  | concluída  |
      Quando consulto as demandas sobre o espaço físico
      Então cada demanda apresenta seu respectivo status atual

    Esquema do Cenário: Consultar o status atual de uma demanda
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E a demanda "Reparo da porta" possui o status "<status>"
      Quando consulto os detalhes da demanda "Reparo da porta"
      Então vejo que o status atual é "<status>"

      Exemplos:
        | status     |
        | aberta     |
        | em análise |
        | concluída  |

    Cenário: Consultar o status de uma demanda registrada por outro usuário
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E a demanda "Troca da lâmpada" foi registrada por "Usuário B"
      E seu status atual é "em análise"
      Quando consulto os detalhes da demanda "Troca da lâmpada"
      Então vejo que o status atual é "em análise"

  Regra: O fórum do espaço físico utiliza somente os estados previstos para esse tipo de demanda

    Cenário: Não apresentar o estado exclusivo do fórum institucional
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Reparo da porta" sobre o espaço físico
      Quando consulto os detalhes da demanda "Reparo da porta"
      Então o status apresentado pertence aos seguintes estados:
        | status     |
        | aberta     |
        | em análise |
        | concluída  |
      E a demanda não apresenta o status "enviada à coordenação"

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

  Regra: As informações consultadas refletem as alterações realizadas na demanda

    Cenário: Exibir o status mais recente após uma atualização
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E consultei anteriormente a demanda "Reparo da porta" com o status "aberta"
      E a gestão do CA atualizou o status da demanda para "em análise"
      Quando consulto novamente os detalhes da demanda "Reparo da porta"
      Então vejo que o status atual é "em análise"
      E o histórico apresenta a nova alteração
      E as alterações anteriores continuam disponíveis
