# language: pt
# Rastreabilidade: Relatório de Features — Aplicativo para Centros Acadêmicos
# (relatorio_features_app_ca.pdf), § 4.1 (criação e gerenciamento de eventos pela gestão).
# Entrega prevista no MVP do módulo, incluindo as simplificações explicitadas no relatório.
# A criação, a edição e a publicação direta de eventos ocorrem sempre no contexto
# de um único Centro Acadêmico; permissões de um CA não produzem efeito em outro.
# Somente integrantes da gestão cujo cargo possui a permissão
# "Publicar e gerenciar eventos diretamente" podem criar eventos já publicados,
# publicar rascunhos sem homologação e editar eventos do próprio CA.
# Os e-mails abaixo são exemplos fictícios de contas institucionais da UnB.
# @backend e @mobile indicam onde o cenário deve ser verificado.
@isolated_database @pending_mobile
Funcionalidade: Criação e gerenciamento de eventos pela gestão do Centro Acadêmico
  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero criar, editar e publicar diretamente eventos acadêmicos, sociais e institucionais do CA
  Para divulgar atividades oficiais à comunidade estudantil sem depender de fila de homologação

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente", "Diretor de Eventos" e "Diretor de Comunicação"
    E o cargo "Diretor de Eventos" possui a permissão de "Publicar e gerenciar eventos diretamente"
    E o cargo "Diretor de Comunicação" não possui a permissão de "Publicar e gerenciar eventos diretamente"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
    E o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
    E o estudante "241098765@aluno.unb.br" possui perfil no CApp e não integra a gestão de "CAER"

  Regra: Integrante autorizado da gestão pode criar e publicar eventos diretamente no CA

    @isolated_database @happy @backend @mobile
    Cenário: Gestor autorizado cria e publica diretamente um evento no CA
      Quando o gestor "231012345@aluno.unb.br" cria e publica diretamente em "CAER" um evento com os seguintes dados:
        | título      | Semana Acadêmica de Engenharia de Redes                   |
        | descrição   | Ciclo de palestras e oficinas técnicas abertas ao curso   |
        | início      | 2026-11-10 14:00                                          |
        | término     | 2026-11-10 18:00                                          |
        | local       | Auditório do Departamento de Engenharia Elétrica          |
      Então o evento "Semana Acadêmica de Engenharia de Redes" passa a estar publicado em "CAER"
      E o evento registra "231012345@aluno.unb.br" como autor responsável em "CAER"
      E o evento fica visível na agenda de eventos publicados de "CAER" sem exigir homologação

    @isolated_database @happy @backend @mobile
    Cenário: Gestor autorizado salva um evento como rascunho antes de publicar
      Quando o gestor "231012345@aluno.unb.br" cria como rascunho em "CAER" um evento com os seguintes dados:
        | título      | Oficina de Redes Definidas por Software                   |
        | descrição   | Prática em laboratório sobre controladores e comutadores  |
        | início      | 2026-11-18 10:00                                          |
        | término     | 2026-11-18 12:00                                          |
        | local       | Laboratório de Redes - Bloco B                            |
      Então o evento "Oficina de Redes Definidas por Software" fica registrado como rascunho em "CAER"
      E o evento não aparece na agenda de eventos publicados de "CAER"

    @isolated_database @happy @backend @mobile
    Cenário: Gestor autorizado publica diretamente um evento salvo em rascunho
      Dado que existe em "CAER" o evento em rascunho "Oficina de Redes Definidas por Software" criado por "231012345@aluno.unb.br"
      Quando o gestor "231012345@aluno.unb.br" publica diretamente o evento "Oficina de Redes Definidas por Software" em "CAER"
      Então o evento "Oficina de Redes Definidas por Software" passa a estar publicado em "CAER"
      E o evento passa a figurar na agenda de eventos publicados de "CAER"
      E a data de publicação do evento fica registrada

    @isolated_database @happy @backend
    Cenário: Publicar evento em um CA não altera a agenda de outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      Quando o gestor "231012345@aluno.unb.br" cria e publica diretamente em "CAER" um evento com os seguintes dados:
        | título      | Recepção de Calouros de Redes                             |
        | descrição   | Apresentação do curso e integração com veteranos          |
        | início      | 2026-11-05 09:00                                          |
        | término     | 2026-11-05 11:30                                          |
        | local       | Anfiteatro 9                                              |
      Então o evento "Recepção de Calouros de Redes" fica visível na agenda de eventos publicados de "CAER"
      E a agenda de eventos publicados de "CACC" permanece sem o evento "Recepção de Calouros de Redes"

  Regra: Todo evento criado exige informações obrigatórias válidas e período consistente

    @isolated_database @sad @backend @mobile
    Esquema do Cenário: Criação de evento sem informação obrigatória é recusada
      Quando o gestor "231012345@aluno.unb.br" tenta criar em "CAER" um evento com os seguintes dados:
        | título      | <titulo>    |
        | descrição   | <descricao> |
        | início      | <inicio>    |
        | término     | <termino>   |
        | local       | <local>     |
      Então a operação é recusada informando que "<campo_obrigatorio>" do evento é obrigatório
      E nenhum novo evento é registrado em "CAER"

      Exemplos:
        | titulo                    | descricao                              | inicio           | termino          | local            | campo_obrigatorio |
        |                           | Palestra sobre segurança em redes      | 2026-11-20 14:00 | 2026-11-20 16:00 | Auditório FT     | título            |
        | Palestra de Cibersegurança|                                        | 2026-11-20 14:00 | 2026-11-20 16:00 | Auditório FT     | descrição         |
        | Palestra de Cibersegurança| Palestra sobre segurança em redes      |                  | 2026-11-20 16:00 | Auditório FT     | início            |
        | Palestra de Cibersegurança| Palestra sobre segurança em redes      | 2026-11-20 14:00 | 2026-11-20 16:00 |                  | local             |

    @isolated_database @sad @backend @mobile
    Cenário: Criação de evento com término anterior ao início é recusada
      Quando o gestor "231012345@aluno.unb.br" tenta criar em "CAER" um evento com os seguintes dados:
        | título      | Minicurso de Roteamento                                   |
        | descrição   | Configuração prática de protocolos de roteamento          |
        | início      | 2026-11-22 16:00                                          |
        | término     | 2026-11-22 14:00                                          |
        | local       | Laboratório de Redes - Bloco B                            |
      Então a operação é recusada informando que o horário de término deve ser posterior ao início
      E nenhum novo evento é registrado em "CAER"

  Regra: Integrante autorizado da gestão pode editar eventos do próprio CA preservando sua validade

    @isolated_database @happy @backend @mobile
    Cenário: Gestor autorizado edita informações de um evento já publicado
      Dado que existe em "CAER" o evento publicado "Semana Acadêmica de Engenharia de Redes" com os seguintes dados:
        | descrição   | Ciclo de palestras técnicas                               |
        | início      | 2026-11-10 14:00                                          |
        | término     | 2026-11-10 18:00                                          |
        | local       | Sala de Reuniões do CAER                                  |
      Quando o gestor "231012345@aluno.unb.br" edita o evento "Semana Acadêmica de Engenharia de Redes" em "CAER" para:
        | título      | Semana Acadêmica de Engenharia de Redes 2026              |
        | descrição   | Ciclo ampliado de palestras e oficinas práticas           |
        | início      | 2026-11-10 14:30                                          |
        | término     | 2026-11-10 18:30                                          |
        | local       | Auditório do Departamento de Engenharia Elétrica          |
      Então o evento passa a apresentar em "CAER" os dados atualizados:
        | título      | Semana Acadêmica de Engenharia de Redes 2026              |
        | descrição   | Ciclo ampliado de palestras e oficinas práticas           |
        | início      | 2026-11-10 14:30                                          |
        | término     | 2026-11-10 18:30                                          |
        | local       | Auditório do Departamento de Engenharia Elétrica          |
      E o evento permanece publicado na agenda de "CAER"

    @isolated_database @happy @backend @mobile
    Cenário: Gestor autorizado edita informações de um evento em rascunho sem publicá-lo
      Dado que existe em "CAER" o evento em rascunho "Oficina de Redes" com os seguintes dados:
        | descrição   | Ementa preliminar da oficina                              |
        | início      | 2026-11-18 10:00                                          |
        | término     | 2026-11-18 12:00                                          |
        | local       | Sala B1                                                   |
      Quando o gestor "231012345@aluno.unb.br" edita o evento "Oficina de Redes" em "CAER" para:
        | título      | Oficina Prática de Redes                                  |
        | descrição   | Ementa definitiva com roteiro de laboratório              |
        | início      | 2026-11-18 10:00                                          |
        | término     | 2026-11-18 12:00                                          |
        | local       | Laboratório de Redes - Bloco B                            |
      Então o evento "Oficina Prática de Redes" mantém a situação de rascunho em "CAER"
      E permanece fora da agenda de eventos publicados de "CAER"

    @isolated_database @sad @backend @mobile
    Esquema do Cenário: Edição que remove informação obrigatória do evento é recusada
      Dado que existe em "CAER" o evento publicado "Palestra de Cibersegurança" com os seguintes dados:
        | descrição   | Palestra sobre segurança em redes                         |
        | início      | 2026-11-20 14:00                                          |
        | término     | 2026-11-20 16:00                                          |
        | local       | Auditório FT                                              |
      Quando o gestor "231012345@aluno.unb.br" tenta editar o evento "Palestra de Cibersegurança" em "CAER" para:
        | título      | <titulo>    |
        | descrição   | <descricao> |
        | início      | <inicio>    |
        | término     | <termino>   |
        | local       | <local>     |
      Então a operação é recusada informando que "<campo_obrigatorio>" do evento é obrigatório
      E o evento "Palestra de Cibersegurança" permanece inalterado em "CAER"

      Exemplos:
        | titulo                    | descricao                         | inicio           | termino          | local        | campo_obrigatorio |
        |                           | Palestra sobre segurança em redes | 2026-11-20 14:00 | 2026-11-20 16:00 | Auditório FT | título            |
        | Palestra de Cibersegurança|                                   | 2026-11-20 14:00 | 2026-11-20 16:00 | Auditório FT | descrição         |
        | Palestra de Cibersegurança| Palestra sobre segurança em redes |                  | 2026-11-20 16:00 | Auditório FT | início            |
        | Palestra de Cibersegurança| Palestra sobre segurança em redes | 2026-11-20 14:00 | 2026-11-20 16:00 |              | local             |

    @isolated_database @sad @backend @mobile
    Cenário: Edição de evento com término anterior ao início é recusada
      Dado que existe em "CAER" o evento publicado "Palestra de Cibersegurança" com os seguintes dados:
        | descrição   | Palestra sobre segurança em redes                         |
        | início      | 2026-11-20 14:00                                          |
        | término     | 2026-11-20 16:00                                          |
        | local       | Auditório FT                                              |
      Quando o gestor "231012345@aluno.unb.br" tenta editar o evento "Palestra de Cibersegurança" em "CAER" para:
        | título      | Palestra de Cibersegurança                                |
        | descrição   | Palestra sobre segurança em redes                         |
        | início      | 2026-11-20 17:00                                          |
        | término     | 2026-11-20 15:00                                          |
        | local       | Auditório FT                                              |
      Então a operação é recusada informando que o horário de término deve ser posterior ao início
      E o evento "Palestra de Cibersegurança" permanece inalterado em "CAER"

  Regra: Usuários sem permissão no CA não podem publicar diretamente nem editar eventos da gestão

    @sad @backend @mobile @isolated_database
    Cenário: Integrante da gestão sem permissão de eventos não pode publicar evento diretamente
      Quando "222033333@aluno.unb.br" tenta criar e publicar diretamente em "CAER" um evento com os seguintes dados:
        | título      | Roda de Conversa sobre Estágios                           |
        | descrição   | Relatos de experiências no mercado de redes               |
        | início      | 2026-11-25 16:00                                          |
        | término     | 2026-11-25 18:00                                          |
        | local       | Sala de Multiuso FT                                       |
      Então a operação é recusada por falta de permissão
      E o evento "Roda de Conversa sobre Estágios" não é publicado na agenda de "CAER"

    @sad @backend @mobile @isolated_database
    Cenário: Estudante sem cargo na gestão não pode criar ou publicar evento diretamente
      Quando o estudante "241098765@aluno.unb.br" tenta criar e publicar diretamente em "CAER" um evento com os seguintes dados:
        | título      | Encontro de Estudos de Redes                              |
        | descrição   | Revisão coletiva para provas do semestre                  |
        | início      | 2026-11-26 14:00                                          |
        | término     | 2026-11-26 16:00                                          |
        | local       | Biblioteca Central - Sala de Estudos                      |
      Então a operação é recusada por falta de permissão
      E o evento "Encontro de Estudos de Redes" não é publicado na agenda de "CAER"

    @sad @backend @mobile @isolated_database
    Cenário: Integrante da gestão sem permissão de eventos não pode publicar rascunho diretamente
      Dado que existe em "CAER" o evento em rascunho "Oficina de Redes Definidas por Software" criado por "231012345@aluno.unb.br"
      Quando "222033333@aluno.unb.br" tenta publicar diretamente o evento "Oficina de Redes Definidas por Software" em "CAER"
      Então a operação é recusada por falta de permissão
      E o evento "Oficina de Redes Definidas por Software" permanece como rascunho em "CAER"
      E não aparece na agenda de eventos publicados de "CAER"

    @sad @backend @mobile @isolated_database
    Cenário: Usuário sem permissão de gerenciar eventos não pode editar evento existente
      Dado que existe em "CAER" o evento publicado "Semana Acadêmica de Engenharia de Redes" com os seguintes dados:
        | descrição   | Ciclo de palestras técnicas                               |
        | início      | 2026-11-10 14:00                                          |
        | término     | 2026-11-10 18:00                                          |
        | local       | Auditório do Departamento de Engenharia Elétrica          |
      Quando "222033333@aluno.unb.br" tenta editar o evento "Semana Acadêmica de Engenharia de Redes" em "CAER" para:
        | título      | Evento Alterado Indevidamente                             |
        | descrição   | Descrição modificada sem autorização                      |
        | início      | 2026-11-10 14:00                                          |
        | término     | 2026-11-10 18:00                                          |
        | local       | Outro Local                                               |
      Então a operação é recusada por falta de permissão
      E o evento "Semana Acadêmica de Engenharia de Redes" permanece inalterado em "CAER"

    @sad @backend @mobile @isolated_database
    Cenário: Gestor autorizado de um CA não pode publicar ou editar eventos na gestão de outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E o estudante "231012345@aluno.unb.br" não possui cargo de gestão em "CACC"
      E existe em "CACC" o evento publicado "Maratona de Programação da UnB"
      Quando o gestor "231012345@aluno.unb.br" tenta editar o evento "Maratona de Programação da UnB" em "CACC"
      Então a operação é recusada por falta de permissão
      E o evento "Maratona de Programação da UnB" permanece inalterado em "CACC"
