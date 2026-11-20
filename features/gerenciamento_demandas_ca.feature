# language: pt

# Referência: Relatório de Features do Projeto, seções 2.4, 5.1, 5.2 e 5.4.
# Rastreabilidade: Issue #148 — Especificar cenários BDD de gerenciamento pelo CA.
@pending_backend @pending_mobile
Funcionalidade: Gerenciamento de demandas pelo Centro Acadêmico

  Como integrante da gestão do Centro Acadêmico
  Quero gerenciar as demandas recebidas nos fóruns institucional e de espaço físico
  Para analisar solicitações da comunidade, atualizar seus status com parecer e prestar os devidos encaminhamentos

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente" e "Diretor de Assuntos Estudantis"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
    E o cargo "Presidente" possui a permissão de "Gerenciar demandas"
    E estou autenticado como "231012345@aluno.unb.br"
    E o CA ativo é "CAER"

  Regra: A gestão do CA realiza a consulta administrativa nos fóruns institucional e de espaço físico

    @happy @backend @mobile
    Cenário: Gestor consulta demandas recebidas no fórum institucional do curso
      Dado que existem as seguintes demandas no fórum institucional do curso:
        | demanda                           | autor                   | status     |
        | Revisão da ementa de Redes 1      | 241098765@aluno.unb.br  | aberta     |
        | Oferta de turmas em período extra | 262001234@aluno.unb.br  | em análise |
      Quando consulto administrativamente as demandas do fórum institucional
      Então vejo a demanda "Revisão da ementa de Redes 1" com o status "aberta" e autor "241098765@aluno.unb.br"
      E vejo a demanda "Oferta de turmas em período extra" com o status "em análise" e autor "262001234@aluno.unb.br"

    @happy @backend @mobile
    Cenário: Gestor consulta demandas recebidas no fórum do espaço físico do CA
      Dado que existem as seguintes demandas no fórum do espaço físico:
        | demanda                   | autor                   | status     |
        | Reparo no ar-condicionado | 241098765@aluno.unb.br  | aberta     |
        | Troca de lâmpadas da sala | 251022222@aluno.unb.br  | em análise |
      Quando consulto administrativamente as demandas do espaço físico
      Então vejo a demanda "Reparo no ar-condicionado" com o status "aberta"
      E vejo a demanda "Troca de lâmpadas da sala" com o status "em análise"

    @happy @backend @mobile
    Cenário: Gestor organiza demandas filtrando por status de atendimento
      Dado que existem as seguintes demandas no fórum institucional do curso:
        | demanda                          | status     |
        | Ajuste de pré-requisitos         | aberta     |
        | Instalação de software nos labs  | em análise |
        | Calendário de provas conflitante | concluída  |
      Quando filtro as demandas institucionais pelo status "aberta"
      Então vejo a demanda "Ajuste de pré-requisitos" na listagem
      E não vejo a demanda "Instalação de software nos labs" na listagem
      E não vejo a demanda "Calendário de provas conflitante" na listagem

    @sad @backend @mobile
    Cenário: Gestão de um CA não visualiza demandas pertencentes a outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E existe a demanda institucional "Reforma do lab de informática" associada somente ao "CACC"
      Quando consulto administrativamente as demandas do fórum institucional
      Então a demanda "Reforma do lab de informática" não é apresentada para a gestão do "CAER"

  Regra: A gestão atualiza o status das demandas com justificativa e parecer

    @happy @backend @mobile
    Cenário: Gestor inicia a análise de uma demanda aberta registrando parecer inicial
      Dado que existe a demanda institucional "Ajuste de pré-requisitos" com o status "aberta"
      Quando atualizo o status da demanda "Ajuste de pré-requisitos" para "em análise" com a justificativa "Demanda recebida pela diretoria acadêmica para triagem"
      Então o status atual da demanda "Ajuste de pré-requisitos" passa a ser "em análise"
      E o parecer registrado na demanda é "Demanda recebida pela diretoria acadêmica para triagem"
      E o histórico registra a alteração realizada por "231012345@aluno.unb.br"

    @happy @backend @mobile
    Cenário: Gestor encaminha demanda institucional à coordenação com justificativa formal
      Dado que a demanda institucional "Revisão da ementa de Redes 1" possui o status "em análise"
      Quando atualizo o status da demanda "Revisão da ementa de Redes 1" para "encaminhada à coordenação" com a justificativa "Pauta formalizada no processo SEI e remetida à coordenação do curso"
      Então o status atual da demanda "Revisão da ementa de Redes 1" passa a ser "encaminhada à coordenação"
      E o parecer registrado na demanda é "Pauta formalizada no processo SEI e remetida à coordenação do curso"
      E a demanda institucional registra o encaminhamento à coordenação do curso

    @happy @backend @mobile
    Cenário: Gestor conclui demanda institucional com parecer de resolução
      Dado que a demanda institucional "Revisão da ementa de Redes 1" possui o status "encaminhada à coordenação"
      Quando atualizo o status da demanda "Revisão da ementa de Redes 1" para "concluída" com a justificativa "Coordenação aprovou a atualização da ementa para o próximo semestre"
      Então o status atual da demanda "Revisão da ementa de Redes 1" passa a ser "concluída"
      E o parecer registrado na demanda é "Coordenação aprovou a atualização da ementa para o próximo semestre"

    @happy @backend @mobile
    Cenário: Gestor conclui demanda de espaço físico com parecer de resolução interna
      Dado que existe a demanda "Reparo no ar-condicionado" sobre o espaço físico com o status "em análise"
      Quando atualizo o status da demanda "Reparo no ar-condicionado" para "concluída" com a justificativa "Manutenção realizada e equipamento em pleno funcionamento"
      Então o status atual da demanda "Reparo no ar-condicionado" passa a ser "concluída"
      E o parecer registrado na demanda é "Manutenção realizada e equipamento em pleno funcionamento"

  Regra: Demandas sobre o espaço físico são tratadas internamente pelo CA e não são encaminhadas à coordenação

    @sad @backend @mobile
    Cenário: Recusar encaminhamento de demanda do espaço físico à coordenação do curso
      Dado que existe a demanda "Reparo no ar-condicionado" sobre o espaço físico com o status "em análise"
      Quando tento atualizar o status da demanda "Reparo no ar-condicionado" para "encaminhada à coordenação" com a justificativa "Solicitando auxílio da coordenação para manutenção"
      Então a alteração de status é recusada informando que demandas de espaço físico não podem ser encaminhadas à coordenação
      E o status da demanda "Reparo no ar-condicionado" permanece "em análise"

  Regra: Tentativas de gerenciamento sem permissão administrativa ativa no CA são recusadas

    @sad @backend @mobile
    Cenário: Estudante sem cargo de gestão não pode gerenciar demandas do CA
      Dado que estou autenticado como "262001234@aluno.unb.br"
      E o estudante "262001234@aluno.unb.br" não possui cargo de gestão em "CAER"
      E existe a demanda institucional "Ajuste de pré-requisitos" com o status "aberta"
      Quando tento atualizar o status da demanda "Ajuste de pré-requisitos" para "em análise" com a justificativa "Tentativa não autorizada"
      Então a operação é recusada por falta de permissão
      E o status da demanda "Ajuste de pré-requisitos" permanece "aberta"

    @sad @backend @mobile
    Cenário: Gestor de outro CA não pode alterar demandas do CA ativo
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E o estudante "221033333@aluno.unb.br" exerce o cargo de "Presidente" em "CACC"
      E "221033333@aluno.unb.br" não possui cargo de gestão em "CAER"
      E estou autenticado como "221033333@aluno.unb.br"
      E existe a demanda institucional "Ajuste de pré-requisitos" no "CAER" com o status "aberta"
      Quando tento atualizar o status da demanda "Ajuste de pré-requisitos" para "em análise" com a justificativa "Intervenção indevida"
      Então a operação é recusada por falta de permissão
      E o status da demanda "Ajuste de pré-requisitos" permanece "aberta"

  Regra: Transições de status devem respeitar o ciclo de vida e a justificativa obrigatória

    @sad @backend @mobile
    Cenário: Recusar reabertura direta de demanda concluída
      Dado que existe a demanda institucional "Calendário de provas conflitante" com o status "concluída"
      Quando tento atualizar o status da demanda "Calendário de provas conflitante" para "aberta" com a justificativa "Reabrindo solicitação"
      Então a alteração de status é recusada informando transição inválida
      E o status da demanda "Calendário de provas conflitante" permanece "concluída"

    @sad @backend @mobile
    Cenário: Recusar alteração de status sem justificativa ou parecer
      Dado que existe a demanda institucional "Ajuste de pré-requisitos" com o status "aberta"
      Quando tento atualizar o status da demanda "Ajuste de pré-requisitos" para "em análise" sem informar justificativa
      Então a alteração de status é recusada informando que a justificativa é obrigatória
      E o status da demanda "Ajuste de pré-requisitos" permanece "aberta"
