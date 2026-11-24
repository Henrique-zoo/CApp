# language: pt

# Rastreabilidade: Relatório de Features — Aplicativo para Centros Acadêmicos, § 9.1.
# MVP: inscrição e identificação do papel de calouro ou veterano em um programa.
# Pareamento, configuração do programa e estratégias de matching ficam fora desta especificação.

@backend @mobile @pending_backend @pending_mobile
Funcionalidade: Cadastro de participantes no apadrinhamento

  Como calouro ou veterano
  Quero participar do apadrinhamento
  Para criar conexões acadêmicas

  Regra: A inscrição em um programa aberto registra o participante e identifica o seu papel

    Cenário: Calouro se inscreve no programa
      Dado que o programa de apadrinhamento está aberto para inscrições
      E o calouro está autenticado no CApp
      E o calouro ainda não está inscrito nesse programa
      Quando o calouro solicita a inscrição no programa com o papel "calouro"
      Então a inscrição do calouro é registrada no programa
      E o papel desse participante é identificado como "calouro"

    Cenário: Veterano se inscreve no programa
      Dado que o programa de apadrinhamento está aberto para inscrições
      E o veterano está autenticado no CApp
      E o veterano ainda não está inscrito nesse programa
      Quando o veterano solicita a inscrição no programa com o papel "veterano"
      Então a inscrição do veterano é registrada no programa
      E o papel desse participante é identificado como "veterano"

  Regra: A inscrição exige o papel de calouro ou de veterano

    Cenário: Inscrição sem papel é recusada
      Dado que o programa de apadrinhamento está aberto para inscrições
      E o estudante está autenticado no CApp
      E o estudante ainda não está inscrito nesse programa
      Quando o estudante solicita a inscrição no programa sem informar um papel
      Então a inscrição não é registrada
      E o estudante é informado de que precisa identificar o papel de calouro ou veterano

    Cenário: Inscrição com papel diferente de calouro ou veterano é recusada
      Dado que o programa de apadrinhamento está aberto para inscrições
      E o estudante está autenticado no CApp
      E o estudante ainda não está inscrito nesse programa
      Quando o estudante solicita a inscrição no programa com um papel diferente de "calouro" e "veterano"
      Então a inscrição não é registrada
      E nenhum participante é inscrito nesse programa

  Regra: Cada pessoa possui no máximo uma inscrição no mesmo programa

    Cenário: Estudante já inscrito não recebe uma segunda inscrição
      Dado que o programa de apadrinhamento está aberto para inscrições
      E o estudante está autenticado no CApp
      E o estudante já está inscrito nesse programa com o papel "calouro"
      Quando o estudante solicita novamente a inscrição no programa com o papel "veterano"
      Então a nova inscrição é recusada
      E o estudante permanece inscrito somente com o papel "calouro"

    Cenário: O mesmo estudante pode se inscrever em outro programa
      Dado que o estudante está autenticado no CApp
      E o estudante está inscrito no programa "Apadrinhamento 2026.1" com o papel "calouro"
      E o programa "Apadrinhamento 2026.2" está aberto para inscrições
      E o estudante ainda não está inscrito no programa "Apadrinhamento 2026.2"
      Quando o estudante solicita a inscrição no programa "Apadrinhamento 2026.2" com o papel "veterano"
      Então a inscrição no programa "Apadrinhamento 2026.2" é registrada com o papel "veterano"
      E a inscrição no programa "Apadrinhamento 2026.1" permanece com o papel "calouro"

  Regra: Somente um programa aberto aceita novas inscrições

    Cenário: Programa ainda em configuração não aceita inscrição
      Dado que o programa de apadrinhamento ainda está em configuração
      E o calouro está autenticado no CApp
      Quando o calouro solicita a inscrição no programa com o papel "calouro"
      Então a inscrição não é registrada
      E o calouro é informado de que o programa não está aberto para inscrições

    Cenário: Programa encerrado não aceita inscrição
      Dado que o programa de apadrinhamento está encerrado
      E o veterano está autenticado no CApp
      Quando o veterano solicita a inscrição no programa com o papel "veterano"
      Então a inscrição não é registrada
      E o veterano é informado de que o programa não está aberto para inscrições

    Cenário: Estudante sem sessão autenticada não se inscreve
      Dado que o programa de apadrinhamento está aberto para inscrições
      E o estudante não está autenticado no CApp
      Quando o estudante tenta solicitar a inscrição no programa com o papel "calouro"
      Então a inscrição não é registrada
      E o estudante é informado de que a autenticação é obrigatória
