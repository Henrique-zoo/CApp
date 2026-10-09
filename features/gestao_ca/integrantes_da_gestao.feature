# language: pt
# Rastreabilidade: Relatório de Features — Aplicativo para Centros Acadêmicos,
# § 2.5 e § 2.6 (administração dos integrantes da gestão). Entrega do MVP do módulo.
# A administração dos integrantes ocorre sempre no contexto de um único CA;
# cargos e permissões atribuídos em um CA não produzem efeito em outro.
# Somente integrantes cujo cargo possui a permissão "Administrar integrantes da gestão"
# podem vincular, alterar ou remover integrantes da gestão do próprio CA.
# No MVP, cada usuário ocupa no máximo um cargo na gestão de um mesmo CA.
# O histórico administrativo é preservado: vinculações, alterações e remoções
# não apagam registros anteriores e não podem ser editadas ou excluídas.
# Os e-mails abaixo são exemplos fictícios de contas institucionais da UnB.
# @backend e @mobile indicam onde o cenário deve ser verificado.
@pending_backend @pending_mobile
Funcionalidade: Administração dos integrantes da gestão de um Centro Acadêmico
  Como gestor de um Centro Acadêmico com permissão para administrar a gestão
  Quero vincular, alterar e remover integrantes e seus cargos
  Para manter a composição da gestão atualizada sem perder o histórico administrativo

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente", "Diretor de Comunicação" e "Diretor de Eventos"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
    E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"

  Regra: Um gestor autorizado pode vincular usuários da plataforma a cargos da gestão do CA

    @happy @backend @mobile
    Cenário: Gestor vincula um estudante a um cargo da gestão
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E não integra a gestão de "CAER"
      Quando o gestor "231012345@aluno.unb.br" vincula "241098765@aluno.unb.br" ao cargo "Diretor de Comunicação" em "CAER"
      Então "241098765@aluno.unb.br" passa a integrar a gestão de "CAER" no cargo "Diretor de Comunicação"
      E "241098765@aluno.unb.br" passa a possuir as permissões do cargo "Diretor de Comunicação" em "CAER"

    @sad @backend @mobile
    Cenário: Integrante sem permissão de administrar a gestão não pode vincular usuários
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
      E o cargo "Diretor de Eventos" não possui a permissão de "Administrar integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" possui perfil no CApp
      Quando "222033333@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" ao cargo "Diretor de Comunicação" em "CAER"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" não passa a integrar a gestão de "CAER"

    @sad @backend
    Cenário: Gestor de um CA não pode vincular usuários à gestão de outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E o estudante "231012345@aluno.unb.br" não possui cargo de gestão em "CACC"
      E o estudante "241098765@aluno.unb.br" possui perfil no CApp
      Quando o gestor "231012345@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" a um cargo em "CACC"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" não passa a integrar a gestão de "CACC"

    @sad @backend @mobile
    Cenário: Não é possível vincular um usuário sem perfil no CApp
      Dado que não existe perfil no CApp para o e-mail "209999999@aluno.unb.br"
      Quando o gestor "231012345@aluno.unb.br" tenta vincular "209999999@aluno.unb.br" ao cargo "Diretor de Comunicação" em "CAER"
      Então a operação é recusada informando que o usuário não foi encontrado
      E a composição da gestão de "CAER" permanece inalterada

    @sad @backend @mobile
    Cenário: Não é possível vincular um usuário a um cargo inexistente no CA
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E o CA "CAER" não possui o cargo "Tesoureiro"
      Quando o gestor "231012345@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" ao cargo "Tesoureiro" em "CAER"
      Então a operação é recusada informando que o cargo não existe no CA
      E "241098765@aluno.unb.br" não passa a integrar a gestão de "CAER"

    @sad @backend @mobile
    Cenário: Usuário que já integra a gestão não recebe um segundo vínculo no mesmo CA
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" ao cargo "Diretor de Eventos" em "CAER"
      Então a operação é recusada informando que o usuário já integra a gestão do CA
      E "241098765@aluno.unb.br" permanece somente no cargo "Diretor de Comunicação" em "CAER"

    @happy @backend
    Cenário: Integrar a gestão de um CA não altera vínculos em outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CACC"
      Quando o gestor "231012345@aluno.unb.br" vincula "241098765@aluno.unb.br" ao cargo "Diretor de Comunicação" em "CAER"
      Então "241098765@aluno.unb.br" passa a integrar a gestão de "CAER" no cargo "Diretor de Comunicação"
      E "241098765@aluno.unb.br" permanece no cargo "Diretor de Eventos" em "CACC"

  Regra: Um gestor autorizado pode alterar o cargo de um integrante da gestão

    @happy @backend @mobile
    Cenário: Gestor altera o cargo de um integrante
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" altera o cargo de "241098765@aluno.unb.br" para "Diretor de Eventos" em "CAER"
      Então "241098765@aluno.unb.br" passa a exercer o cargo "Diretor de Eventos" em "CAER"
      E "241098765@aluno.unb.br" passa a possuir as permissões do cargo "Diretor de Eventos" em "CAER"
      E "241098765@aluno.unb.br" deixa de possuir as permissões exclusivas do cargo "Diretor de Comunicação" em "CAER"

    @sad @backend @mobile
    Cenário: Integrante sem permissão de administrar a gestão não pode alterar cargos
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
      E o cargo "Diretor de Eventos" não possui a permissão de "Administrar integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      Quando "222033333@aluno.unb.br" tenta alterar o cargo de "241098765@aluno.unb.br" para "Presidente" em "CAER"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" permanece no cargo "Diretor de Comunicação" em "CAER"

    @sad @backend @mobile
    Cenário: Não é possível alterar o cargo de quem não integra a gestão
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E não integra a gestão de "CAER"
      Quando o gestor "231012345@aluno.unb.br" tenta alterar o cargo de "241098765@aluno.unb.br" para "Diretor de Eventos" em "CAER"
      Então a operação é recusada informando que o usuário não integra a gestão do CA
      E "241098765@aluno.unb.br" não passa a integrar a gestão de "CAER"

    @sad @backend @mobile
    Cenário: Alterar para o cargo atualmente exercido não modifica a gestão
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" tenta alterar o cargo de "241098765@aluno.unb.br" para "Diretor de Comunicação" em "CAER"
      Então a operação é recusada informando que o integrante já exerce esse cargo
      E nenhum novo registro é adicionado ao histórico administrativo de "CAER"

  Regra: Um gestor autorizado pode remover integrantes, desde que a gestão continue administrável

    @happy @backend @mobile
    Cenário: Gestor remove um integrante da gestão
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" remove "241098765@aluno.unb.br" da gestão de "CAER"
      Então "241098765@aluno.unb.br" deixa de integrar a gestão de "CAER"
      E "241098765@aluno.unb.br" deixa de possuir as permissões do cargo "Diretor de Comunicação" em "CAER"
      E "241098765@aluno.unb.br" continua podendo acessar "CAER" como estudante

    @sad @backend @mobile
    Cenário: Integrante sem permissão de administrar a gestão não pode remover integrantes
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
      E o cargo "Diretor de Eventos" não possui a permissão de "Administrar integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      Quando "222033333@aluno.unb.br" tenta remover "241098765@aluno.unb.br" da gestão de "CAER"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" permanece no cargo "Diretor de Comunicação" em "CAER"

    @sad @backend @mobile
    Cenário: O último integrante que administra a gestão não pode ser removido
      Dado que "231012345@aluno.unb.br" é o único integrante de "CAER" com a permissão de "Administrar integrantes da gestão"
      Quando o gestor "231012345@aluno.unb.br" tenta remover "231012345@aluno.unb.br" da gestão de "CAER"
      Então a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão
      E "231012345@aluno.unb.br" permanece no cargo "Presidente" em "CAER"

    @sad @backend @mobile
    Cenário: O último integrante que administra a gestão não pode ser movido para cargo sem essa permissão
      Dado que "231012345@aluno.unb.br" é o único integrante de "CAER" com a permissão de "Administrar integrantes da gestão"
      E o cargo "Diretor de Eventos" não possui a permissão de "Administrar integrantes da gestão"
      Quando o gestor "231012345@aluno.unb.br" tenta alterar o cargo de "231012345@aluno.unb.br" para "Diretor de Eventos" em "CAER"
      Então a operação é recusada informando que a gestão precisa manter ao menos um integrante com essa permissão
      E "231012345@aluno.unb.br" permanece no cargo "Presidente" em "CAER"

    @happy @backend @mobile
    Cenário: Gestor pode deixar a gestão quando outro integrante também a administra
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" remove "231012345@aluno.unb.br" da gestão de "CAER"
      Então "231012345@aluno.unb.br" deixa de integrar a gestão de "CAER"
      E "241098765@aluno.unb.br" permanece com a permissão de "Administrar integrantes da gestão" em "CAER"

  Regra: Toda mudança na composição da gestão é registrada e o histórico é preservado

    @happy @backend
    Esquema do Cenário: Mudanças na gestão geram registro no histórico administrativo
      Dado que o estudante "241098765@aluno.unb.br" se encontra na situação "<situacao_inicial>" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" realiza a operação "<operacao>" sobre "241098765@aluno.unb.br" em "CAER"
      Então o histórico administrativo de "CAER" registra a operação "<operacao>"
      E o registro identifica o integrante afetado, o cargo anterior "<cargo_anterior>" e o cargo resultante "<cargo_resultante>"
      E o registro identifica o gestor "231012345@aluno.unb.br" como responsável e a data da operação

      Exemplos:
        | situacao_inicial                   | operacao   | cargo_anterior         | cargo_resultante       |
        | sem vínculo com a gestão           | vinculação | nenhum                 | Diretor de Comunicação |
        | no cargo de Diretor de Comunicação | alteração  | Diretor de Comunicação | Diretor de Eventos     |
        | no cargo de Diretor de Comunicação | remoção    | Diretor de Comunicação | nenhum                 |

    @happy @backend @mobile
    Cenário: Remoção de integrante preserva sua passagem pela gestão no histórico
      Dado que o estudante "241098765@aluno.unb.br" foi vinculado ao cargo "Diretor de Comunicação" em "CAER"
      E posteriormente teve o cargo alterado para "Diretor de Eventos" em "CAER"
      Quando o gestor "231012345@aluno.unb.br" remove "241098765@aluno.unb.br" da gestão de "CAER"
      Então o histórico administrativo de "CAER" apresenta, em ordem cronológica, a vinculação, a alteração e a remoção de "241098765@aluno.unb.br"
      E os registros anteriores à remoção permanecem inalterados

    @happy @backend @mobile
    Cenário: Integrante removido e vinculado novamente mantém os registros anteriores
      Dado que o histórico de "CAER" registra a remoção de "241098765@aluno.unb.br" do cargo "Diretor de Comunicação"
      Quando o gestor "231012345@aluno.unb.br" vincula "241098765@aluno.unb.br" ao cargo "Diretor de Eventos" em "CAER"
      Então o histórico administrativo de "CAER" registra a nova vinculação
      E os registros da passagem anterior de "241098765@aluno.unb.br" pela gestão permanecem disponíveis

    @sad @backend @mobile
    Cenário: Registros do histórico administrativo não podem ser alterados nem excluídos
      Dado que o histórico de "CAER" registra a vinculação de "241098765@aluno.unb.br" ao cargo "Diretor de Comunicação"
      Quando o gestor "231012345@aluno.unb.br" tenta alterar ou excluir esse registro do histórico
      Então a operação é recusada informando que o histórico administrativo não pode ser modificado
      E o registro permanece inalterado no histórico de "CAER"

    @sad @backend @mobile
    Cenário: Operação recusada não gera registro no histórico administrativo
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
      E o cargo "Diretor de Eventos" não possui a permissão de "Administrar integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" possui perfil no CApp
      Quando "222033333@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" ao cargo "Diretor de Comunicação" em "CAER"
      Então a operação é recusada por falta de permissão
      E nenhum novo registro é adicionado ao histórico administrativo de "CAER"
