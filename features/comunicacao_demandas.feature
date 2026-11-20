@pending
# language: pt

# Referência: Relatório de Features do Projeto, seções 5.1, 5.2 e 5.3.
# Rastreabilidade: Issue #149 — Especificar cenários BDD de comunicação entre estudante e CA.
@pending_backend @pending_mobile
Funcionalidade: Comunicação e comentários entre estudante e Centro Acadêmico

  Como estudante ou integrante da gestão do Centro Acadêmico
  Quero publicar comentários, esclarecimentos e acompanhar a linha do tempo de interações em uma demanda
  Para manter uma comunicação transparente, contextualizada e ágil sobre o andamento das solicitações

  Contexto:
    Dado que o Centro Acadêmico "Centro Acadêmico de Engenharia de Redes (CAER)" está cadastrado na UnB
    E o CA "CAER" possui os cargos "Presidente" e "Diretor de Assuntos Estudantis"
    E o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "CAER"
    E o cargo "Presidente" possui a permissão de "Gerenciar demandas"
    E o estudante "241098765@aluno.unb.br" é aluno vinculado ao curso de Engenharia de Redes
    E o CA ativo é "CAER"

  Regra: A gestão e os estudantes autorizados podem publicar comentários e respostas nas demandas

    @happy @backend @mobile
    Cenário: Gestor do CA publica resposta oficial em demanda institucional
      Dado que estou autenticado como "231012345@aluno.unb.br"
      E existe a demanda institucional "Revisão da ementa de Redes 1" registrada por "241098765@aluno.unb.br"
      Quando publico uma resposta na demanda com o texto "Entramos em contato com a coordenação para agendar reunião sobre a ementa"
      Então o comentário é registrado na demanda "Revisão da ementa de Redes 1"
      E o autor do comentário é identificado como gestão de "CAER"
      E o conteúdo do comentário é "Entramos em contato com a coordenação para agendar reunião sobre a ementa"
      E a data e hora da resposta são registradas

    @happy @backend @mobile
    Cenário: Estudante autor publica mensagem de esclarecimento em sua demanda institucional
      Dado que estou autenticado como "241098765@aluno.unb.br"
      E existe a demanda institucional "Revisão da ementa de Redes 1" registrada por mim
      Quando publico um comentário na demanda com o texto "Anexei também a comparação com o currículo de referência"
      Então o comentário é registrado na demanda "Revisão da ementa de Redes 1"
      E o autor do comentário é "241098765@aluno.unb.br"
      E o conteúdo registrado é "Anexei também a comparação com o currículo de referência"

    @happy @backend @mobile
    Cenário: Estudante participante comenta em demanda pública sobre o espaço físico
      Dado que estou autenticado como "262001234@aluno.unb.br"
      E existe a demanda "Reparo no ar-condicionado" sobre o espaço físico registrada por "241098765@aluno.unb.br"
      Quando publico um comentário na demanda com o texto "Confirmado, o aparelho da sala B também parou de refrigerar hoje cedo"
      Então o comentário é registrado na demanda "Reparo no ar-condicionado"
      E o autor do comentário é "262001234@aluno.unb.br"
      E o comentário fica visível para os usuários do fórum do espaço físico

  Regra: A demanda disponibiliza a linha do tempo cronológica com comentários e atualizações de status

    @happy @backend @mobile
    Cenário: Visualizar a linha do tempo em ordem cronológica de eventos
      Dado que estou autenticado como "241098765@aluno.unb.br"
      E a demanda institucional "Revisão da ementa de Redes 1" possui o seguinte histórico de interações:
        | tipo do evento       | autor                  | conteúdo ou detalhe                                    | data_hora        |
        | criação              | 241098765@aluno.unb.br | Necessidade de atualizar tópicos de nuvem na ementa    | 2026-10-01 10:00 |
        | comentário de gestão | 231012345@aluno.unb.br | Triagem inicial realizada pela gestão                  | 2026-10-01 14:30 |
        | alteração de status  | 231012345@aluno.unb.br | Status alterado de aberta para em análise              | 2026-10-01 14:35 |
        | comentário do autor  | 241098765@aluno.unb.br | Obrigado pelo retorno, fico à disposição               | 2026-10-02 09:15 |
      Quando consulto a linha do tempo da demanda "Revisão da ementa de Redes 1"
      Então vejo os eventos apresentados em ordem cronológica
      E cada evento identifica seu tipo, autor, conteúdo e instante de registro
      E nenhum evento anterior foi sobrescrito pelas mensagens posteriores

  Regra: O autor da demanda é notificado sobre novas respostas e atualizações

    @happy @backend @mobile
    Cenário: Autor da demanda recebe notificação quando a gestão publica uma resposta
      Dado que existe a demanda institucional "Revisão da ementa de Redes 1" registrada por "241098765@aluno.unb.br"
      E o gestor "231012345@aluno.unb.br" está autenticado no CApp
      Quando o gestor "231012345@aluno.unb.br" envia uma resposta na demanda "Revisão da ementa de Redes 1"
      Então o autor "241098765@aluno.unb.br" recebe uma notificação de nova resposta na demanda
      E a notificação informa que a gestão de "CAER" respondeu à solicitação

    @happy @backend @mobile
    Cenário: Autor da demanda recebe notificação quando o status é atualizado pela gestão
      Dado que existe a demanda institucional "Revisão da ementa de Redes 1" registrada por "241098765@aluno.unb.br" com o status "aberta"
      E o gestor "231012345@aluno.unb.br" está autenticado no CApp
      Quando o gestor "231012345@aluno.unb.br" atualiza o status da demanda "Revisão da ementa de Redes 1" para "em análise"
      Então o autor "241098765@aluno.unb.br" recebe uma notificação de alteração de status
      E a notificação informa que a demanda agora está com o status "em análise"

  Regra: Validação de conteúdo impede comentários vazios ou que excedam o limite máximo

    @sad @backend @mobile
    Cenário: Recusar publicação de comentário sem conteúdo textual
      Dado que estou autenticado como "241098765@aluno.unb.br"
      E existe a demanda institucional "Revisão da ementa de Redes 1" registrada por mim
      Quando tento publicar um comentário na demanda com o texto "   "
      Então a publicação do comentário é recusada
      E o aplicativo informa que o conteúdo do comentário é obrigatório
      E nenhum comentário é adicionado à demanda

    @sad @backend @mobile
    Cenário: Recusar publicação de comentário que excede o limite máximo de caracteres
      Dado que estou autenticado como "241098765@aluno.unb.br"
      E existe a demanda institucional "Revisão da ementa de Redes 1" registrada por mim
      Quando tento publicar um comentário na demanda com texto excedendo 2000 caracteres
      Então a publicação do comentário é recusada
      E o aplicativo informa que o comentário ultrapassou o limite máximo de 2000 caracteres
      E nenhum comentário é adicionado à demanda

  Regra: O acesso e a participação nas interações respeitam os fóruns e o isolamento por CA

    @sad @backend @mobile
    Cenário: Estudante de outro curso sem vínculo de gestão não pode interagir no fórum institucional
      Dado que estou autenticado como "251022222@aluno.unb.br"
      E o estudante "251022222@aluno.unb.br" não possui vínculo com o curso de Engenharia de Redes
      E "251022222@aluno.unb.br" não possui cargo de gestão em "CAER"
      E existe a demanda institucional "Revisão da ementa de Redes 1" no "CAER"
      Quando tento publicar um comentário na demanda institucional "Revisão da ementa de Redes 1"
      Então o envio do comentário é recusado por falta de acesso ao fórum institucional
      E nenhum comentário é adicionado à demanda

    @sad @backend @mobile
    Cenário: Usuário não pode publicar comentários em demanda pertencente a outro CA
      Dado que o Centro Acadêmico "Centro Acadêmico de Ciência da Computação (CACC)" está cadastrado na UnB
      E existe a demanda "Manutenção das bancadas" associada ao CA "CACC"
      Dado que estou autenticado como "241098765@aluno.unb.br"
      E meu CA ativo é "CAER"
      Quando tento publicar um comentário na demanda "Manutenção das bancadas" de "CACC"
      Então a operação é recusada informando que a demanda pertence a outro Centro Acadêmico
      E nenhum comentário é registrado na demanda
