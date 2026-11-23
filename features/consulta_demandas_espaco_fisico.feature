# language: pt

# Referência: Relatório de Features do Projeto, seção 5.2.
# Issue: #147.
# Recorte I07: consulta do fórum de demandas sobre o espaço físico do CA.
@v1 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Consulta de demandas sobre o espaço físico do CA

  Como usuário do aplicativo
  Quero consultar as demandas sobre o espaço físico do CA
  Para acompanhar as necessidades comunicadas pela comunidade

  Regra: O fórum do espaço físico é acessível aos usuários do aplicativo

    Cenário: Permitir a consulta por usuário de outro curso
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário de outro curso"
      E existe a demanda "Reparo da porta" sobre o espaço físico desse CA
      Quando consulto as demandas sobre o espaço físico
      Então vejo a demanda "Reparo da porta"

    Cenário: Consultar demandas registradas por diferentes usuários
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Reparo da porta" registrada por mim
      E existe a demanda "Troca da lâmpada" registrada por "Usuário B"
      Quando consulto as demandas sobre o espaço físico
      Então vejo a demanda "Reparo da porta"
      E vejo a demanda "Troca da lâmpada"

    Cenário: Consultar os detalhes de uma demanda registrada por outro usuário
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Troca da lâmpada" registrada por "Usuário B"
      Quando consulto os detalhes da demanda "Troca da lâmpada"
      Então os detalhes da demanda são apresentados
      E consigo identificar a demanda consultada

  Regra: A consulta apresenta somente demandas pertencentes ao CA ativo

    Cenário: Não misturar demandas de Centros Acadêmicos diferentes
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Reparo da porta" no "CA de Computação"
      E existe a demanda "Pintura da sala" no "CA de Matemática"
      Quando consulto as demandas sobre o espaço físico
      Então vejo a demanda "Reparo da porta"
      E não vejo a demanda "Pintura da sala"

    Cenário: Atualizar a consulta após a troca do CA ativo
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Reparo da porta" no "CA de Computação"
      E existe a demanda "Pintura da sala" no "CA de Matemática"
      Quando altero o CA ativo para "CA de Matemática"
      E consulto as demandas sobre o espaço físico
      Então vejo a demanda "Pintura da sala"
      E não vejo a demanda "Reparo da porta"

    Cenário: Não disponibilizar detalhes de demanda pertencente a outro CA
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E existe a demanda "Pintura da sala" no "CA de Matemática"
      Quando tento consultar os detalhes da demanda "Pintura da sala"
      Então os detalhes da demanda não são disponibilizados no contexto ativo

  Regra: A consulta representa corretamente a existência de demandas no fórum

    Cenário: Informar quando o fórum não possui demandas
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      E não existem demandas sobre o espaço físico nesse CA
      Quando consulto as demandas sobre o espaço físico
      Então a listagem informa que não existem demandas
