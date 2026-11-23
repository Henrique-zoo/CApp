# language: pt
# Rastreabilidade: Issue #124 — especificação BDD da configuração eleitoral.
# Complementa as regras gerais de gestão e permissões do Centro Acadêmico.
# A gestão do CA designa uma comissão para cada eleição. As permissões
# eleitorais valem para a eleição em que foram concedidas.
@pending_backend @pending_mobile
Funcionalidade: Designação e permissões da comissão eleitoral
  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero designar os integrantes da comissão de cada eleição e definir suas permissões
  Para manter a administração do processo eleitoral restrita às pessoas autorizadas

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente", "Diretor de Comunicação" e "Diretor de Eventos"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
    E o cargo "Presidente" possui a permissão de "Administrar perfis e integrantes da gestão"
    E existe a eleição "Eleição 2027" de "CAER"

  Regra: A designação da comissão eleitoral exige autorização da gestão do CA

    @happy @backend @mobile
    Cenário: Integrante autorizado designa uma pessoa para a comissão eleitoral
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E ainda não integra a comissão da eleição "Eleição 2027"
      Quando o gestor "231012345@aluno.unb.br" designa "241098765@aluno.unb.br" para a comissão da eleição "Eleição 2027"
      Então "241098765@aluno.unb.br" passa a integrar a comissão da eleição "Eleição 2027"

    @sad @backend @mobile
    Cenário: Integrante sem permissão para administrar a gestão não designa a comissão eleitoral
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      E o cargo "Diretor de Comunicação" não possui a permissão de "Administrar perfis e integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" possui perfil no CApp
      Quando "222033333@aluno.unb.br" tenta designar "241098765@aluno.unb.br" para a comissão da eleição "Eleição 2027"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" não passa a integrar a comissão da eleição "Eleição 2027"

  Regra: As permissões eleitorais são específicas e independentes

    @sad @backend @mobile
    Cenário: Permissão administrativa da gestão não concede poderes eleitorais
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E possui a permissão administrativa "Administrar perfis e integrantes da gestão" em "CAER"
      E não integra a comissão da eleição "Eleição 2027"
      E não possui a permissão "Configurar eleições" na eleição "Eleição 2027"
      Quando tenta configurar a eleição "Eleição 2027"
      Então a operação é recusada por falta de permissão

    @happy @backend @mobile
    Cenário: Permissão para configurar eleições autoriza configurar uma eleição
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Configurar eleições" na eleição "Eleição 2027"
      E que a eleição "Eleição 2027" está em preparação, sem manifesto publicado
      Quando configura a eleição "Eleição 2027"
      Então a configuração da eleição é autorizada

    @happy @backend @mobile
    Cenário: Permissão para administrar chapas autoriza administrar as chapas
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Administrar chapas" na eleição "Eleição 2027"
      Quando administra as chapas da eleição "Eleição 2027"
      Então a administração das chapas é concluída

    @happy @backend @mobile
    Cenário: Permissão para apurar resultados autoriza a solicitação de apuração
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Apurar resultados eleitorais" na eleição "Eleição 2027"
      Quando solicita a apuração do resultado da eleição "Eleição 2027"
      Então a solicitação de apuração é autorizada

    @sad @backend @mobile
    Cenário: Permissão para configurar eleições não concede permissão para apurar resultados
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Configurar eleições" na eleição "Eleição 2027"
      E não possui a permissão "Apurar resultados eleitorais" na eleição "Eleição 2027"
      Quando tenta apurar o resultado da eleição "Eleição 2027"
      Então a operação é recusada por falta de permissão

    @sad @backend @mobile
    Cenário: Permissão para administrar chapas não concede permissão para configurar eleições
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Administrar chapas" na eleição "Eleição 2027"
      E não possui a permissão "Configurar eleições" na eleição "Eleição 2027"
      Quando tenta configurar a eleição "Eleição 2027"
      Então a operação é recusada por falta de permissão

  Regra: A designação e as permissões eleitorais valem para uma única eleição

    @sad @backend @mobile
    Cenário: Designação para uma eleição não autoriza configurar outra do mesmo CA
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Configurar eleições" na eleição "Eleição 2027"
      E existe a eleição "Eleição 2028" de "CAER"
      E não integra a comissão da eleição "Eleição 2028"
      Quando tenta configurar a eleição "Eleição 2028"
      Então a operação é recusada por falta de permissão

    @sad @backend @mobile
    Cenário: Designação para uma eleição não autoriza configurar uma eleição de outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACOMP)" está cadastrado na UnB
      E que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E possui a permissão "Configurar eleições" na eleição "Eleição 2027"
      E existe a eleição "Eleição CACOMP 2027" de "CACOMP"
      E não integra a comissão da eleição "Eleição CACOMP 2027"
      Quando tenta configurar a eleição "Eleição CACOMP 2027"
      Então a operação é recusada por falta de permissão

  Regra: A revogação autorizada remove as permissões e preserva o histórico

    @happy @backend @mobile
    Cenário: Revogar integrante da comissão eleitoral preserva os registros anteriores
      Dado que "241098765@aluno.unb.br" integra a comissão da eleição "Eleição 2027"
      E que a eleição "Eleição 2027" está em preparação, sem manifesto publicado
      E realizou ações administrativas na eleição "Eleição 2027"
      Quando o gestor "231012345@aluno.unb.br" solicita a revogação da designação de "241098765@aluno.unb.br" da comissão da eleição "Eleição 2027"
      Então "241098765@aluno.unb.br" deixa de integrar a comissão da eleição "Eleição 2027"
      E deixa de possuir as autorizações eleitorais vinculadas à designação revogada
      E as ações anteriores permanecem registradas no histórico da eleição

  Regra: O representante da comissão permanece no cargo até o fim da eleição

    @sad @backend @mobile
    Esquema do Cenário: Recusar revogação do representante após a publicação do manifesto
      Dado que "241098765@aluno.unb.br" representa a comissão como autoridade da eleição "Eleição 2027"
      E que o manifesto da eleição foi publicado e a eleição está "<fase>"
      Quando o gestor "231012345@aluno.unb.br" solicita a revogação da designação de "241098765@aluno.unb.br" da comissão da eleição "Eleição 2027"
      Então a revogação é recusada enquanto a eleição está em andamento
      E "241098765@aluno.unb.br" permanece como representante da comissão da eleição "Eleição 2027"

      Exemplos:
        | fase                 |
        | aguardando abertura  |
        | aberta               |
        | encerrada            |
        | em apuração          |
