# language: pt
#
# Referência: Relatório de Features do Projeto, seção 4.5.
# O controle básico de capacidade faz parte do MVP simplificado.
# A fila de espera é uma evolução pós-MVP prevista na issue #279.
# Os cenários da fila não presumem uma ordenação ou prazo de confirmação
# ainda não definido pelo projeto.

@pending_backend @pending_mobile
Funcionalidade: Controle opcional de vagas em eventos

  Como proponente de um evento
  Quero definir opcionalmente um limite de vagas
  Para controlar a quantidade de participantes do evento

  Regra: O limite é opcional e, quando definido, restringe as inscrições à capacidade

    @mvp
    Cenário: Definir limite de vagas ao cadastrar um evento
      Dado que sou proponente do CA "CA de Computação"
      E estou cadastrando o evento "Palestra de IA"
      Quando defino o limite de 3 vagas e salvo o evento
      Então o evento é salvo com limite de 3 vagas
      E o limite passa a ser considerado nas novas inscrições

    @mvp
    Cenário: Inscrever-se em evento sem limite de vagas
      Dado que existe o evento publicado "Semana Acadêmica" no CA "CA de Computação"
      E o evento não possui limite de vagas
      E já existem participantes inscritos nesse evento
      E não tenho inscrição ativa nesse evento
      Quando solicito minha inscrição no evento "Semana Acadêmica"
      Então minha inscrição ativa é registrada
      E o aplicativo não recusa a inscrição por limite de capacidade

    @mvp
    Cenário: Inscrever-se quando ainda há vaga disponível
      Dado que existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E o evento possui limite de 3 vagas
      E 2 participantes estão inscritos no evento "Palestra de IA"
      E não tenho inscrição ativa nesse evento
      Quando solicito minha inscrição no evento "Palestra de IA"
      Então minha inscrição ativa é registrada
      E o evento passa a ter 3 participantes inscritos

    @mvp
    Cenário: Recusar inscrição direta quando todas as vagas estão ocupadas
      Dado que existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E o evento possui limite de 2 vagas
      E 2 participantes estão inscritos no evento "Palestra de IA"
      E não tenho inscrição ativa nesse evento
      Quando solicito minha inscrição direta no evento "Palestra de IA"
      Então minha inscrição ativa não é registrada
      E o evento continua com 2 participantes inscritos
      E o aplicativo informa que o evento está lotado

    @mvp
    Cenário: Liberar uma vaga após cancelamento
      Dado que existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E o evento possui limite de 2 vagas
      E 2 participantes estão inscritos no evento "Palestra de IA"
      Quando um participante cancela sua inscrição no evento "Palestra de IA"
      Então o evento passa a ter uma vaga disponível
      E o número de participantes inscritos não supera o limite

  Regra: A fila de espera atende interessados em eventos lotados

    @post_mvp
    Cenário: Entrar na fila de espera de um evento lotado
      Dado que existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E o evento possui limite de 2 vagas
      E 2 participantes estão inscritos no evento "Palestra de IA"
      E não tenho inscrição ativa nem participação na fila desse evento
      Quando solicito entrar na fila de espera do evento "Palestra de IA"
      Então minha participação na fila de espera é registrada
      E o aplicativo informa que estou aguardando uma vaga
      E o número de inscrições ativas permanece igual ao limite

    @post_mvp
    Cenário: Desistir da fila de espera
      Dado que estou aguardando na fila de espera do evento "Palestra de IA"
      Quando solicito sair da fila de espera desse evento
      Então minha participação na fila deixa de estar ativa
      E não sou considerado para futuras vagas liberadas nessa fila

    @post_mvp
    Cenário: Não colocar na fila quem já está inscrito no evento
      Dado que estou inscrito no evento publicado "Palestra de IA"
      Quando solicito entrar na fila de espera desse evento
      Então minha inscrição ativa é preservada
      E não sou registrado na fila de espera desse evento

    @post_mvp
    Cenário: Direcionar uma vaga liberada conforme a política da fila
      Dado que existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E o evento possui limite de 2 vagas
      E 2 participantes estão inscritos no evento "Palestra de IA"
      E existe um participante elegível aguardando na fila
      E a política de convocação da fila está definida para o evento
      Quando um participante cancela sua inscrição no evento "Palestra de IA"
      Então a vaga liberada é direcionada a um único participante elegível conforme a política definida
      E a mesma vaga não é atribuída simultaneamente a outro participante
      E o número de inscrições ativas não supera o limite

    @post_mvp
    Cenário: Impedir que duas pessoas ocupem simultaneamente a mesma vaga liberada
      Dado que existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E o evento possui limite de 2 vagas
      E 1 participante permanece inscrito após o cancelamento que liberou uma vaga
      E 2 participantes elegíveis aguardam na fila de espera
      Quando as tentativas de ocupar a única vaga disponível são processadas simultaneamente
      Então no máximo um dos participantes recebe a vaga
      E o número de inscrições ativas não supera o limite
      E a mesma vaga não é confirmada para os dois participantes
