# language: pt

# Referência: Relatório de Features do Projeto, seção 4.4.
# Controle opcional de vagas e fila de espera não fazem parte desta especificação básica.
# Regra proposta para revisão: impedir inscrições duplicadas e informar cancelamentos sem inscrição ativa.
@pending_backend @pending_mobile
Funcionalidade: Inscrição e cancelamento em eventos

  Como estudante
  Quero me inscrever e cancelar minha inscrição em eventos publicados
  Para controlar minha participação nas atividades

  Regra: Somente eventos publicados aceitam inscrição

    Cenário: Inscrever-se em um evento publicado
      Dado que estou autenticado no CApp
      E existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E ainda não estou inscrito nesse evento
      Quando solicito minha inscrição no evento "Palestra de IA"
      Então minha inscrição no evento é registrada
      E o aplicativo informa que a inscrição foi realizada

    Cenário: Não permitir inscrição em evento não publicado
      Dado que estou autenticado no CApp
      E existe o evento "Hackathon em análise" proposto e ainda não homologado no CA "CA de Computação"
      Quando solicito minha inscrição no evento "Hackathon em análise"
      Então minha inscrição não é registrada
      E o aplicativo informa que o evento não está disponível para inscrição

    Cenário: Não duplicar inscrição no mesmo evento
      Dado que estou autenticado no CApp
      E existe o evento publicado "Palestra de IA" no CA "CA de Computação"
      E já estou inscrito nesse evento
      Quando solicito novamente minha inscrição no evento "Palestra de IA"
      Então minha inscrição continua única
      E o aplicativo informa que já estou inscrito nesse evento

  Regra: O estudante pode cancelar sua própria inscrição

    Cenário: Cancelar inscrição em evento publicado
      Dado que estou autenticado no CApp
      E estou inscrito no evento publicado "Palestra de IA"
      Quando solicito cancelar minha inscrição no evento "Palestra de IA"
      Então minha inscrição no evento deixa de estar ativa
      E o aplicativo informa que o cancelamento foi realizado

    Cenário: Informar tentativa de cancelar sem inscrição ativa
      Dado que estou autenticado no CApp
      E não estou inscrito no evento publicado "Palestra de IA"
      Quando solicito cancelar minha inscrição no evento "Palestra de IA"
      Então nenhuma inscrição é alterada
      E o aplicativo informa que não há inscrição ativa para cancelar
