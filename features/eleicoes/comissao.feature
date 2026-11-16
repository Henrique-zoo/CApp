# language: pt
# Rastreabilidade: Issue #124 — especificação BDD da configuração eleitoral.
# Complementa as regras gerais de gestão e permissões do Centro Acadêmico.
# A designação da comissão eleitoral é realizada por integrante autorizado
# da gestão do CA. As permissões eleitorais são específicas e independentes
# das permissões administrativas gerais.
@pending_backend @pending_mobile
Funcionalidade: Designação e permissões da comissão eleitoral
  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero designar os integrantes da comissão eleitoral e definir suas permissões
  Para manter a administração do processo eleitoral restrita às pessoas autorizadas

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente", "Diretor de Comunicação" e "Diretor de Eventos"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
    E o cargo "Presidente" possui a permissão de "Administrar perfis e integrantes da gestão"

  Regra: A designação da comissão eleitoral exige autorização da gestão do CA

    @happy @backend @mobile
    Cenário: Integrante autorizado designa uma pessoa para a comissão eleitoral
      Dado que o estudante "241098765@aluno.unb.br" possui perfil no CApp
      E ainda não integra a comissão eleitoral de "CAER"
      Quando o gestor "231012345@aluno.unb.br" designa "241098765@aluno.unb.br" para a comissão eleitoral de "CAER"
      Então "241098765@aluno.unb.br" passa a integrar a comissão eleitoral de "CAER"

    @sad @backend @mobile
    Cenário: Integrante sem permissão para administrar a gestão não designa a comissão eleitoral
      Dado que o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
      E o cargo "Diretor de Comunicação" não possui a permissão de "Administrar perfis e integrantes da gestão"
      E o estudante "241098765@aluno.unb.br" possui perfil no CApp
      Quando "222033333@aluno.unb.br" tenta designar "241098765@aluno.unb.br" para a comissão eleitoral de "CAER"
      Então a operação é recusada por falta de permissão
      E "241098765@aluno.unb.br" não passa a integrar a comissão eleitoral de "CAER"

  Regra: As permissões eleitorais são específicas e independentes

    @sad @backend @mobile
    Cenário: Permissão administrativa da gestão não concede poderes eleitorais
      Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
      E possui a permissão administrativa "Administrar perfis e integrantes da gestão" em "CAER"
      E não integra a comissão eleitoral de "CAER"
      E não possui a permissão "Configurar eleições" em "CAER"
      Quando tenta configurar uma eleição de "CAER"
      Então a operação é recusada por falta de permissão

    @happy @backend @mobile
    Cenário: Permissão para configurar eleições autoriza configurar uma eleição
      Dado que "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E possui a permissão "Configurar eleições" em "CAER"
      Quando configura uma eleição de "CAER"
      Então a configuração da eleição é autorizada

    @happy @backend @mobile
    Cenário: Permissão para administrar chapas autoriza administrar as chapas
      Dado que "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E possui a permissão "Administrar chapas" em "CAER"
      Quando administra as chapas de uma eleição de "CAER"
      Então a administração das chapas é concluída

    @happy @backend @mobile
    Cenário: Permissão para apurar resultados autoriza realizar a apuração
      Dado que "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E possui a permissão "Apurar resultados eleitorais" em "CAER"
      Quando apura o resultado de uma eleição de "CAER"
      Então a apuração é concluída

    @sad @backend @mobile
    Cenário: Permissão para configurar eleições não concede permissão para apurar resultados
      Dado que "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E possui a permissão "Configurar eleições" em "CAER"
      E não possui a permissão "Apurar resultados eleitorais" em "CAER"
      Quando tenta apurar o resultado de uma eleição de "CAER"
      Então a operação é recusada por falta de permissão

    @sad @backend @mobile
    Cenário: Permissão para administrar chapas não concede permissão para configurar eleições
      Dado que "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E possui a permissão "Administrar chapas" em "CAER"
      E não possui a permissão "Configurar eleições" em "CAER"
      Quando tenta alterar a configuração de uma eleição de "CAER"
      Então a operação é recusada por falta de permissão

  Regra: A designação e as permissões eleitorais respeitam o contexto do Centro Acadêmico

    @sad @backend @mobile
    Cenário: Designação em um CA não concede autorização eleitoral em outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACOMP)" está cadastrado na UnB
      E "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E não integra a comissão eleitoral de "CACOMP"
      Quando tenta configurar uma eleição de "CACOMP"
      Então a operação é recusada por falta de permissão

  Regra: A revogação da designação remove as autorizações futuras e preserva o histórico

    @happy @backend @mobile
    Cenário: Revogar integrante da comissão eleitoral preserva os registros anteriores
      Dado que "241098765@aluno.unb.br" integra a comissão eleitoral de "CAER"
      E realizou ações administrativas em uma eleição de "CAER"
      Quando o gestor "231012345@aluno.unb.br" revoga a designação de "241098765@aluno.unb.br" da comissão eleitoral de "CAER"
      Então "241098765@aluno.unb.br" deixa de integrar a comissão eleitoral de "CAER"
      E deixa de possuir as autorizações eleitorais vinculadas à designação revogada
      E as ações anteriores permanecem registradas no histórico da eleição
