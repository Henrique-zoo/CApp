# language: pt
# Rastreabilidade: Relatório de Features — Aplicativo para Centros Acadêmicos
# (relatorio_features_app_ca.pdf), § 4.1 (criação e gerenciamento de eventos pela gestão).
# Entrega prevista no MVP do módulo, incluindo as simplificações explicitadas no relatório.
# A criação, a edição e a publicação direta de eventos ocorrem sempre no contexto
# de um único Centro Acadêmico; permissões de um CA não produzem efeito em outro.
# Somente integrantes da gestão cujo cargo possui a permissão
# "Publicar e gerenciar eventos diretamente" podem criar eventos já publicados fora do ambiente físico do próprio CA,
# publicar esses rascunhos sem homologação e editar eventos vinculados ao CA.
# O local especial "o ambiente físico do próprio CA" sempre exige homologação.
# Editar um evento publicado o retira da agenda até nova homologação.
# Título, descrição, início e local são obrigatórios; término é opcional e, quando
# informado, deve ser estritamente posterior ao início.
# Os e-mails abaixo são exemplos fictícios de contas institucionais da UnB.
# @backend e @mobile indicam onde o cenário deve ser verificado.
@pending_backend @pending_mobile
Funcionalidade: Criação e gerenciamento de eventos pela gestão do Centro Acadêmico
  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero criar, editar e publicar diretamente eventos acadêmicos, sociais e institucionais do CA
  Para divulgar atividades oficiais respeitando as permissões e a homologação exigida

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente", "Diretor de Eventos" e "Diretor de Comunicação"
    E o cargo "Diretor de Eventos" possui a permissão de "Publicar e gerenciar eventos diretamente"
    E o cargo "Diretor de Comunicação" não possui a permissão de "Publicar e gerenciar eventos diretamente"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Diretor de Eventos" em "CAER"
    E o estudante "222033333@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "CAER"
    E o estudante "241098765@aluno.unb.br" possui perfil no CApp e não integra a gestão de "CAER"

  Regra: Integrante autorizado pode publicar diretamente eventos cujo local não seja o ambiente físico do próprio CA

    @happy @backend @mobile
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

    @happy @backend @mobile
    Cenário: Gestor autorizado salva um evento como rascunho antes de publicar
      Quando o gestor "231012345@aluno.unb.br" cria como rascunho em "CAER" um evento com os seguintes dados:
        | título      | Oficina de Redes Definidas por Software                   |
        | descrição   | Prática em laboratório sobre controladores e comutadores  |
        | início      | 2026-11-18 10:00                                          |
        | término     | 2026-11-18 12:00                                          |
        | local       | Laboratório de Redes - Bloco B                            |
      Então o evento "Oficina de Redes Definidas por Software" fica registrado como rascunho em "CAER"
      E o evento não aparece na agenda de eventos publicados de "CAER"

    @happy @backend @mobile
    Cenário: Gestor autorizado publica diretamente um evento salvo em rascunho
      Dado que existe em "CAER" o evento em rascunho "Oficina de Redes Definidas por Software" criado por "231012345@aluno.unb.br"
      E o local desse evento não é o ambiente físico do próprio CA
      Quando o gestor "231012345@aluno.unb.br" publica diretamente o evento "Oficina de Redes Definidas por Software" em "CAER"
      Então o evento "Oficina de Redes Definidas por Software" passa a estar publicado em "CAER"
      E o evento passa a figurar na agenda de eventos publicados de "CAER"
      E a data de publicação do evento fica registrada

    @happy @backend
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

    @sad @backend @mobile
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

    @sad @backend @mobile
    Cenário: Criação de evento com término anterior ao início é recusada
      Quando o gestor "231012345@aluno.unb.br" tenta criar em "CAER" um evento com os seguintes dados:
        | título      | Minicurso de Roteamento                                   |
        | descrição   | Configuração prática de protocolos de roteamento          |
        | início      | 2026-11-22 16:00                                          |
        | término     | 2026-11-22 14:00                                          |
        | local       | Laboratório de Redes - Bloco B                            |
      Então a operação é recusada informando que o horário de término deve ser posterior ao início
      E nenhum novo evento é registrado em "CAER"

  Regra: Integrante autorizado da gestão pode editar eventos vinculados ao CA preservando sua validade

    @happy @backend @mobile
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
      E o evento passa a ficar pendente de nova homologação em "CAER"
      E o evento é retirado da agenda pública enquanto aguarda a homologação

    @happy @backend @mobile
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

    @sad @backend @mobile
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

    @sad @backend @mobile
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

    @sad @backend @mobile
    Cenário: Integrante da gestão sem permissão de eventos não pode publicar evento diretamente
      Quando "222033333@aluno.unb.br" tenta criar e publicar diretamente em "CAER" um evento com os seguintes dados:
        | título      | Roda de Conversa sobre Estágios                           |
        | descrição   | Relatos de experiências no mercado de redes               |
        | início      | 2026-11-25 16:00                                          |
        | término     | 2026-11-25 18:00                                          |
        | local       | Sala de Multiuso FT                                       |
      Então a operação é recusada por falta de permissão
      E o evento "Roda de Conversa sobre Estágios" não é publicado na agenda de "CAER"

    @sad @backend @mobile
    Cenário: Estudante sem cargo na gestão não pode criar ou publicar evento diretamente
      Quando o estudante "241098765@aluno.unb.br" tenta criar e publicar diretamente em "CAER" um evento com os seguintes dados:
        | título      | Encontro de Estudos de Redes                              |
        | descrição   | Revisão coletiva para provas do semestre                  |
        | início      | 2026-11-26 14:00                                          |
        | término     | 2026-11-26 16:00                                          |
        | local       | Biblioteca Central - Sala de Estudos                      |
      Então a operação é recusada por falta de permissão
      E o evento "Encontro de Estudos de Redes" não é publicado na agenda de "CAER"

    @sad @backend @mobile
    Cenário: Integrante da gestão sem permissão de eventos não pode publicar rascunho diretamente
      Dado que existe em "CAER" o evento em rascunho "Oficina de Redes Definidas por Software" criado por "231012345@aluno.unb.br"
      Quando "222033333@aluno.unb.br" tenta publicar diretamente o evento "Oficina de Redes Definidas por Software" em "CAER"
      Então a operação é recusada por falta de permissão
      E o evento "Oficina de Redes Definidas por Software" permanece como rascunho em "CAER"
      E não aparece na agenda de eventos publicados de "CAER"

    @sad @backend @mobile
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

    @sad @backend @mobile
    Cenário: Gestor autorizado de um CA não pode publicar ou editar eventos na gestão de outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E o estudante "231012345@aluno.unb.br" não possui cargo de gestão em "CACC"
      E existe em "CACC" o evento publicado "Maratona de Programação da UnB"
      Quando o gestor "231012345@aluno.unb.br" tenta editar o evento "Maratona de Programação da UnB" em "CACC"
      Então a operação é recusada por falta de permissão
      E o evento "Maratona de Programação da UnB" permanece inalterado em "CACC"

  Regra: O término é opcional e o período informado deve ser válido

    @happy @backend @mobile
    Cenário: Criar evento sem horário de término
      Quando o gestor "231012345@aluno.unb.br" cria em "CAER" um evento com título, descrição, início e local válidos sem informar o término
      Então o evento é registrado sem horário de término
      E nenhum horário de término é inventado pelo sistema

    @sad @backend @mobile
    Cenário: Término igual ao início não constitui um período válido
      Quando o gestor "231012345@aluno.unb.br" tenta criar em "CAER" um evento com início e término em "2026-11-10 14:00"
      Então a operação é recusada informando que o término deve ser posterior ao início
      E nenhum novo evento é registrado em "CAER"

  Regra: A permissão de homologação também permite publicação direta quando o local não é o ambiente físico do próprio CA

    @happy @backend @mobile
    Cenário: Homologador cria evento em outro local sem concessão separada de publicação direta
      Dado que o integrante possui a permissão "Homologar eventos" em "CAER"
      E não recebeu uma concessão separada de "Publicar e gerenciar eventos diretamente"
      Quando cria e publica diretamente um evento válido em um local diferente do ambiente físico do próprio CA
      Então o evento é publicado na agenda de "CAER"

  Regra: Usar o ambiente físico do próprio CA como local exige homologação em qualquer tentativa de publicação

    @happy @backend @mobile
    Cenário: Selecionar o ambiente físico do próprio CA como opção especial de local
      Quando o gestor "231012345@aluno.unb.br" seleciona "o ambiente físico do próprio CA" como local de um evento válido
      Então o local do evento fica associado ao espaço do CA "CAER"
      E o aplicativo identifica que a publicação exige homologação

    @happy @backend @mobile
    Cenário: Publicação direta de evento no ambiente físico do próprio CA segue para homologação
      Dado que o gestor "231012345@aluno.unb.br" possui publicação direta de eventos em "CAER"
      Quando cria um evento válido com o local "o ambiente físico do próprio CA" e solicita a publicação
      Então o evento fica pendente de homologação em "CAER"
      E o evento não aparece na agenda pública antes da aprovação

    @sad @backend @mobile
    Cenário: Permissão de homologação não elimina a etapa obrigatória no ambiente físico do próprio CA
      Dado que o integrante possui a permissão "Homologar eventos" em "CAER"
      Quando cria um evento válido com o local "o ambiente físico do próprio CA" e solicita a publicação direta
      Então o evento aguarda uma ação explícita de homologação
      E a criação não publica automaticamente o evento

    @happy @backend @mobile
    Cenário: Publicar rascunho no ambiente físico do próprio CA exige homologação
      Dado que existe um evento em rascunho no local "o ambiente físico do próprio CA"
      Quando o gestor com publicação direta solicita a publicação desse rascunho em "CAER"
      Então o evento passa a ficar pendente de homologação
      E permanece fora da agenda pública

    @happy @backend @mobile
    Cenário: Aprovar o evento no ambiente físico do próprio CA libera sua publicação
      Dado que existe em "CAER" um evento no local "o ambiente físico do próprio CA" aguardando homologação
      E o integrante possui a permissão "Homologar eventos" nesse CA
      Quando aprova a publicação do evento
      Então o evento aparece na agenda pública de "CAER"
      E a opção "o ambiente físico do próprio CA" permanece identificada como seu local

    @sad @backend @mobile
    Cenário: Rejeição da homologação mantém o evento no ambiente físico do próprio CA fora da agenda
      Dado que existe em "CAER" um evento no local "o ambiente físico do próprio CA" aguardando homologação
      E o integrante possui a permissão "Homologar eventos" nesse CA
      Quando rejeita a publicação do evento
      Então o evento não é publicado na agenda de "CAER"
      E o proponente pode identificar que a publicação foi rejeitada

    @happy @backend @mobile
    Cenário: Mudar o local de evento publicado para o ambiente físico do próprio CA exige nova homologação
      Dado que existe em "CAER" um evento publicado em outro local
      Quando o gestor "231012345@aluno.unb.br" altera o local para "o ambiente físico do próprio CA"
      Então o evento fica pendente de nova homologação
      E sai da agenda pública até que a alteração seja aprovada

  Regra: Alterações de eventos publicados só retornam à agenda após homologação

    @happy @backend @mobile
    Cenário: Homologação de alteração devolve o evento à agenda
      Dado que a edição de um evento publicado em "CAER" o retirou da agenda pública
      E o integrante possui a permissão "Homologar eventos" nesse CA
      Quando aprova a alteração do evento
      Então o evento retorna à agenda pública com os dados homologados
