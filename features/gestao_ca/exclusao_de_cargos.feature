# language: pt
# Rastreabilidade: Relatório de Features — Aplicativo para Centros Acadêmicos, § 2.8
# (remoção de cargos administrativos). Entrega do MVP do módulo.
# Complementa "cargos_padrao_ca.feature": as regras de integridade dos cargos padrão
# (Presidente irrevogável; Vice-Presidente e Tesoureiro removíveis sem integrantes
# ativos) valem aqui e não são repetidas. Este arquivo trata o ato da exclusão, os
# cargos personalizados (como "Diretor de Eventos") e as gestões anteriores.
# Um cargo só pode ser excluído quando nenhum integrante ativo o exerce; o cargo de
# "Presidente" nunca pode ser excluído.
# Excluir um cargo não apaga o histórico: as gestões anteriores continuam exibindo
# o cargo e quem o ocupou à época.
# Cargos pertencem exclusivamente ao CA em que foram criados: a exclusão em um CA
# não altera cargos ou permissões de outro.
# Os e-mails abaixo são exemplos fictícios de contas institucionais da UnB.
# @backend e @mobile indicam onde o cenário deve ser verificado.
@gestao_ca @pending_backend @pending_mobile
Funcionalidade: Exclusão de cargos administrativos de um Centro Acadêmico
  Como gestor de um Centro Acadêmico com permissão para administrar a gestão
  Quero excluir cargos que deixaram de ser necessários
  Para manter a estrutura da gestão enxuta sem perder o cargo de Presidente nem o histórico das gestões

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente", "Vice-Presidente", "Tesoureiro" e "Diretor de Eventos"
    E o CA "CACC" possui os cargos "Presidente", "Vice-Presidente", "Tesoureiro" e "Diretor de Eventos"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
    E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"

  Regra: Cargos personalizados podem ser excluídos quando não possuem integrantes ativos

    @happy @backend @mobile
    Cenário: Gestor exclui cargo personalizado sem integrante ativo
      Dado que o cargo "Diretor de Eventos" de "CAER" não possui integrante vinculado
      Quando o gestor "231012345@aluno.unb.br" exclui o cargo "Diretor de Eventos" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Diretor de Eventos"
      E o CA "CAER" mantém os cargos "Presidente", "Vice-Presidente" e "Tesoureiro"

    @sad @backend @mobile
    Cenário: Cargo personalizado com integrante ativo não pode ser excluído
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" tenta excluir o cargo "Diretor de Eventos" de "CAER"
      Então a operação é recusada informando que o cargo possui integrante ativo
      E o CA "CAER" mantém o cargo "Diretor de Eventos"
      E "241098765@aluno.unb.br" permanece no cargo "Diretor de Eventos" em "CAER"

    @happy @backend @mobile
    Cenário: Cargo personalizado ocupado pode ser excluído depois que o integrante o deixa
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
      E o gestor "231012345@aluno.unb.br" altera o cargo de "241098765@aluno.unb.br" para "Tesoureiro" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" exclui o cargo "Diretor de Eventos" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Diretor de Eventos"
      E "241098765@aluno.unb.br" permanece no cargo "Tesoureiro" em "CAER"

    @happy @backend @mobile
    Cenário: Cargo cujos únicos ocupantes são ex-integrantes pode ser excluído
      Dado que o histórico de "CAER" registra a remoção de "241098765@aluno.unb.br" do cargo "Diretor de Eventos"
      E o cargo "Diretor de Eventos" de "CAER" não possui integrante vinculado
      Quando o gestor "231012345@aluno.unb.br" exclui o cargo "Diretor de Eventos" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Diretor de Eventos"
      E "241098765@aluno.unb.br" continua podendo acessar "CAER" como estudante

    @sad @backend @mobile
    Cenário: Integrante sem permissão de administrar a gestão não exclui cargos personalizados
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Tesoureiro" em "CAER"
      E o cargo "Tesoureiro" não possui a permissão de "Administrar integrantes da gestão"
      E o cargo "Diretor de Eventos" de "CAER" não possui integrante vinculado
      Quando "222033333@aluno.unb.br" tenta excluir o cargo "Diretor de Eventos" de "CAER"
      Então a operação é recusada por falta de permissão
      E o CA "CAER" mantém o cargo "Diretor de Eventos"

    @sad @backend @mobile
    Cenário: Cargo excluído deixa de estar disponível para novas vinculações
      Dado que o gestor "231012345@aluno.unb.br" excluiu o cargo "Diretor de Eventos" de "CAER"
      E o estudante "241098765@aluno.unb.br" possui perfil no CApp
      Quando o gestor "231012345@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" ao cargo "Diretor de Eventos" em "CAER"
      Então a operação é recusada informando que o cargo não existe no CA
      E "241098765@aluno.unb.br" não passa a integrar a gestão de "CAER"

    @sad @backend @mobile
    Cenário: Não é possível excluir um cargo que não existe no CA
      Dado que o CA "CAER" não possui o cargo "Diretor de Marketing"
      Quando o gestor "231012345@aluno.unb.br" tenta excluir o cargo "Diretor de Marketing" de "CAER"
      Então a operação é recusada informando que o cargo não existe no CA
      E a estrutura de cargos de "CAER" permanece inalterada

  Regra: Nenhuma exclusão altera o cargo de Presidente nem o histórico das gestões

    @sad @backend @mobile
    Cenário: Tentativa de excluir o Presidente não altera o histórico de gestões anteriores
      Dado que o CA "CAER" possui a gestão anterior de 2024 com "222033333@aluno.unb.br" no cargo "Presidente"
      Quando o gestor "231012345@aluno.unb.br" tenta excluir o cargo "Presidente" de "CAER"
      Então a operação é recusada informando que o cargo de Presidente não pode ser excluído
      E o CA "CAER" mantém o cargo "Presidente"
      E a gestão anterior de 2024 continua exibindo "222033333@aluno.unb.br" no cargo "Presidente"

  Regra: As gestões anteriores continuam exibindo os cargos excluídos

    @happy @backend @mobile
    Cenário: Cargo excluído sai da estrutura atual, mas permanece nas gestões anteriores
      Dado que o CA "CAER" possui a gestão anterior de 2024 com "222033333@aluno.unb.br" no cargo "Diretor de Eventos"
      E o cargo "Diretor de Eventos" de "CAER" não possui integrante vinculado
      Quando o gestor "231012345@aluno.unb.br" exclui o cargo "Diretor de Eventos" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Diretor de Eventos"
      E a estrutura atual de cargos de "CAER" não lista o cargo "Diretor de Eventos"
      E a gestão anterior de 2024 continua exibindo "222033333@aluno.unb.br" no cargo "Diretor de Eventos"

  Regra: A exclusão de um cargo não afeta cargos nem permissões de outros CAs

    @happy @backend @mobile
    Cenário: Excluir um cargo personalizado em um CA não altera o cargo de mesmo nome em outro CA
      Dado que o cargo "Diretor de Eventos" de "CAER" não possui integrante vinculado
      E o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CACC"
      E o cargo "Diretor de Eventos" de "CACC" possui a permissão de "Publicar e gerenciar eventos diretamente"
      Quando o gestor "231012345@aluno.unb.br" exclui o cargo "Diretor de Eventos" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Diretor de Eventos"
      E o CA "CACC" mantém o cargo "Diretor de Eventos"
      E "241098765@aluno.unb.br" permanece no cargo "Diretor de Eventos" em "CACC"
      E o cargo "Diretor de Eventos" de "CACC" mantém a permissão de "Publicar e gerenciar eventos diretamente"

    @sad @backend @mobile
    Cenário: Gestor de um CA não exclui cargo personalizado de outro CA
      Dado que o estudante "231012345@aluno.unb.br" não possui cargo de gestão em "CACC"
      E o cargo "Diretor de Eventos" de "CACC" não possui integrante vinculado
      Quando o gestor "231012345@aluno.unb.br" tenta excluir o cargo "Diretor de Eventos" de "CACC"
      Então a operação é recusada por falta de permissão
      E o CA "CACC" mantém o cargo "Diretor de Eventos"
