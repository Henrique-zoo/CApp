# language: pt
# Rastreabilidade: Relatório de Features — Aplicativo para Centros Acadêmicos, § 2.3
# (estrutura inicial de cargos de cada CA). Entrega do MVP do módulo.
# Todo Centro Acadêmico cadastrado nasce com os cargos padrão "Presidente",
# "Vice-Presidente" e "Tesoureiro", sem necessidade de configuração manual.
# O cargo "Presidente" é irrevogável: nenhuma permissão administrativa, inclusive a
# do administrador da plataforma, permite excluí-lo.
# Os cargos "Vice-Presidente" e "Tesoureiro" podem ser removidos depois, desde que
# nenhum integrante da gestão os exerça no momento.
# Os cargos padrão pertencem exclusivamente ao CA em que foram criados: criar, manter
# ou remover cargos em um CA não produz efeito em outro.
# Remover um cargo não apaga o histórico administrativo já registrado.
# Os e-mails abaixo são exemplos fictícios de contas institucionais da UnB.
# @backend e @mobile indicam onde o cenário deve ser verificado.
@gestao_ca @pending_backend @pending_mobile
Funcionalidade: Cargos padrão da gestão de um Centro Acadêmico
  Como administrador da plataforma ou gestor de um Centro Acadêmico
  Quero que cada CA tenha uma estrutura inicial de cargos com o cargo de Presidente preservado
  Para que a gestão possa ser organizada desde o cadastro sem risco de ficar sem presidência

  Contexto:
    Dado que o administrador "200000001@unb.br" está autorizado a cadastrar Centros Acadêmicos na plataforma
    E o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB

  Regra: Todo Centro Acadêmico cadastrado recebe automaticamente os três cargos padrão

    @happy @backend @mobile
    Cenário: Cadastro de um novo CA cria os cargos padrão
      Quando o administrador "200000001@unb.br" cadastra o Centro Acadêmico "Centro Acadêmico de Engenharia Elétrica (CAEE)"
      Então o CA "CAEE" passa a possuir exatamente os cargos "Presidente", "Vice-Presidente" e "Tesoureiro"
      E nenhum desses cargos possui integrante vinculado

    @happy @backend
    Cenário: Os cargos padrão do novo CA ficam disponíveis para vinculação de integrantes
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E o Centro Acadêmico "Centro Acadêmico de Engenharia Elétrica (CAEE)" foi recém-cadastrado
      Quando o administrador "200000001@unb.br" vincula "241098765@aluno.unb.br" ao cargo "Presidente" em "CAEE"
      Então "241098765@aluno.unb.br" passa a exercer o cargo "Presidente" em "CAEE"

    @sad @backend
    Cenário: Cadastro recusado não cria cargos
      Quando o administrador "200000001@unb.br" tenta cadastrar novamente o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)"
      Então o cadastro é recusado informando que o CA já existe
      E o CA "CAER" permanece com um único conjunto de cargos

    @sad @backend @mobile
    Cenário: Usuário sem autorização não cadastra Centro Acadêmico nem cria cargos
      Dado que o estudante "241098765@aluno.unb.br" não está autorizado a cadastrar Centros Acadêmicos
      Quando "241098765@aluno.unb.br" tenta cadastrar o Centro Acadêmico "Centro Acadêmico de Engenharia Elétrica (CAEE)"
      Então a operação é recusada por falta de permissão
      E o Centro Acadêmico "CAEE" não passa a existir

  Regra: O cargo de Presidente não pode ser excluído por nenhuma permissão administrativa

    @sad @backend @mobile
    Cenário: Gestor com permissão máxima na gestão não exclui o cargo de Presidente
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"
      Quando "231012345@aluno.unb.br" tenta excluir o cargo "Presidente" em "CAER"
      Então a operação é recusada informando que o cargo de Presidente não pode ser excluído
      E o CA "CAER" mantém o cargo "Presidente"

    @sad @backend
    Cenário: Administrador da plataforma não exclui o cargo de Presidente de um CA
      Quando o administrador "200000001@unb.br" tenta excluir o cargo "Presidente" em "CAER"
      Então a operação é recusada informando que o cargo de Presidente não pode ser excluído
      E o CA "CAER" mantém o cargo "Presidente"

    @sad @backend @mobile
    Cenário: Cargo de Presidente continua existindo sem integrante vinculado
      Dado que o cargo "Presidente" de "CAER" não possui integrante vinculado
      Quando o administrador "200000001@unb.br" tenta excluir o cargo "Presidente" em "CAER"
      Então a operação é recusada informando que o cargo de Presidente não pode ser excluído
      E o CA "CAER" mantém o cargo "Presidente"

    @happy @backend @mobile
    Cenário: Administrador da plataforma remove o Presidente e o cargo permanece no CA
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o cargo "Vice-Presidente" possui a permissão de "Administrar integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" exerce o cargo de "Vice-Presidente" em "CAER"
      Quando o administrador "200000001@unb.br" remove "231012345@aluno.unb.br" da gestão de "CAER"
      Então "231012345@aluno.unb.br" deixa de integrar a gestão de "CAER"
      E o CA "CAER" mantém o cargo "Presidente"
      E o cargo "Presidente" de "CAER" não possui integrante vinculado
      E "241098765@aluno.unb.br" permanece com a permissão de "Administrar integrantes da gestão" em "CAER"

  Regra: Os cargos de Vice-Presidente e Tesoureiro podem ser removidos quando não possuem integrantes

    @happy @backend @mobile
    Esquema do Cenário: Gestor remove cargo padrão sem integrantes vinculados
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"
      E o cargo "<cargo>" de "CAER" não possui integrante vinculado
      Quando "231012345@aluno.unb.br" remove o cargo "<cargo>" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "<cargo>"
      E o CA "CAER" mantém o cargo "Presidente"

      Exemplos:
        | cargo           |
        | Vice-Presidente |
        | Tesoureiro      |

    @sad @backend @mobile
    Esquema do Cenário: Cargo padrão com integrante ativo não pode ser removido
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" exerce o cargo de "<cargo>" em "CAER"
      Quando "231012345@aluno.unb.br" tenta remover o cargo "<cargo>" de "CAER"
      Então a operação é recusada informando que o cargo possui integrante ativo
      E o CA "CAER" mantém o cargo "<cargo>"
      E "241098765@aluno.unb.br" permanece no cargo "<cargo>" em "CAER"

      Exemplos:
        | cargo           |
        | Vice-Presidente |
        | Tesoureiro      |

    @happy @backend
    Cenário: Cargo volta a poder ser removido depois que o integrante deixa o cargo
      Dado que o estudante "241098765@aluno.unb.br" exerce o cargo de "Tesoureiro" em "CAER"
      E o gestor "231012345@aluno.unb.br" remove "241098765@aluno.unb.br" da gestão de "CAER"
      Quando "231012345@aluno.unb.br" remove o cargo "Tesoureiro" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Tesoureiro"

    @sad @backend @mobile
    Cenário: Integrante sem permissão de administrar a gestão não remove cargos
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Vice-Presidente" em "CAER"
      E o cargo "Vice-Presidente" não possui a permissão de "Administrar integrantes da gestão"
      E o cargo "Tesoureiro" de "CAER" não possui integrante vinculado
      Quando "222033333@aluno.unb.br" tenta remover o cargo "Tesoureiro" de "CAER"
      Então a operação é recusada por falta de permissão
      E o CA "CAER" mantém o cargo "Tesoureiro"

    @happy @backend
    Cenário: Remoção de cargo preserva o histórico administrativo do CA
      Dado que o histórico de "CAER" registra a passagem de "241098765@aluno.unb.br" pelo cargo "Tesoureiro"
      E o cargo "Tesoureiro" de "CAER" não possui integrante vinculado
      Quando "231012345@aluno.unb.br" remove o cargo "Tesoureiro" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Tesoureiro"
      E os registros anteriores do histórico administrativo de "CAER" permanecem disponíveis

  Regra: Os cargos padrão pertencem exclusivamente ao CA em que foram criados

    @happy @backend
    Cenário: Remover um cargo padrão em um CA não altera os cargos de outro CA
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"
      E o cargo "Tesoureiro" de "CAER" não possui integrante vinculado
      Quando "231012345@aluno.unb.br" remove o cargo "Tesoureiro" de "CAER"
      Então o CA "CAER" deixa de possuir o cargo "Tesoureiro"
      E o CA "CACC" continua possuindo os cargos "Presidente", "Vice-Presidente" e "Tesoureiro"

    @sad @backend
    Cenário: Gestor de um CA não remove cargos de outro CA
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o estudante "231012345@aluno.unb.br" não possui cargo de gestão em "CACC"
      E o cargo "Tesoureiro" de "CACC" não possui integrante vinculado
      Quando "231012345@aluno.unb.br" tenta remover o cargo "Tesoureiro" de "CACC"
      Então a operação é recusada por falta de permissão
      E o CA "CACC" mantém o cargo "Tesoureiro"

    @sad @backend
    Cenário: Não é possível vincular um integrante a cargo de outro CA
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E o cargo "Presidente" possui a permissão de "Administrar integrantes da gestão"
      Quando o gestor "231012345@aluno.unb.br" tenta vincular "241098765@aluno.unb.br" ao cargo "Tesoureiro" de "CACC"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" não passa a integrar a gestão de "CACC"

    @happy @backend
    Cenário: Presidentes de CAs distintos coexistem de forma independente
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      Quando o estudante "241098765@aluno.unb.br" é vinculado ao cargo "Presidente" em "CACC"
      Então "231012345@aluno.unb.br" permanece no cargo "Presidente" em "CAER"
      E "241098765@aluno.unb.br" exerce o cargo "Presidente" em "CACC"
      E nenhum deles possui permissões administrativas no outro CA
