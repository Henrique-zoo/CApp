# language: pt
# Os cenários descrevem regras de negócio e resultados observáveis.
# Detalhes técnicos de persistência, concorrência e privacidade devem ser
# verificados nos componentes correspondentes.
# Contrato #377. Chave de assinatura por eleição, elegibilidade no aceite e opção Nulo.
# App Check/cliente são premissas; a soma do agregado é um guardrail.
# Os passos de produto continuam pendentes até existir implementação real.

@v1 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Votação em eleições de Centros Acadêmicos

  Como estudante elegível de um Centro Acadêmico
  Quero consultar as opções e registrar meu voto
  Para participar do processo eleitoral conforme as regras da eleição

  Contexto:
    Dado que existe uma eleição para o Centro Acadêmico "CA Engenharia"
    E a eleição possui as chapas ativas "Aurora" e "Horizonte"
    E existe uma chapa inativa chamada "Legado"
    E "Ana" está autenticada pela sua conta Firebase
    E as solicitações válidas de voto de "Ana" são assinadas com sua própria chave

  Regra: O primeiro voto não exige cadastro eleitoral prévio

    @happy
    Cenário: Votar pela primeira vez após a abertura
      Dado que a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E "Ana" não possui inscrição ou chave prévia nessa eleição
      Quando registra seu primeiro voto na chapa "Aurora"
      Então o voto é confirmado
      E a chave pública da cédula é gravada no registro de participação de "Ana" nessa eleição
      E a chave privada permanece no dispositivo de "Ana"

  Regra: Somente estudantes com vínculo verificado com o curso do CA podem votar

    @sad @backend @mobile
    Cenário: Recusar voto de estudante sem vínculo acadêmico válido
      Dado que a eleição está no período de votação
      E o estudante "Bruno" não possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada por falta de elegibilidade
      E nenhum voto é registrado para o estudante "Bruno"

    @sad @backend @mobile
    Cenário: Reavaliar o curso no momento do voto
      Dado que a eleição está no período de votação
      E uma nova verificação institucional de "Bruno" informa outro curso
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada por falta de elegibilidade
      E nenhum voto é registrado para o estudante "Bruno"

  Regra: As opções de voto devem respeitar a situação das chapas

    @happy @backend @mobile
    Cenário: Apresentar somente as chapas ativas como opções de voto
      Dado que a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      Quando consulta as opções de voto
      Então são apresentadas as chapas ativas "Aurora" e "Horizonte"
      E a chapa inativa "Legado" não é apresentada como opção

    @sad @backend @mobile
    Cenário: Recusar voto em chapa inativa
      Dado que a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      Quando tenta registrar um voto na chapa "Legado"
      Então a tentativa de voto é recusada
      E nenhum voto é registrado para o estudante "Ana"

  Regra: O voto só pode ser registrado durante o período de votação

    @sad @backend @mobile
    Cenário: Recusar voto antes do início da votação
      Dado que o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o período de votação ainda não começou
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada porque a votação ainda não começou
      E nenhum voto é registrado para o estudante "Ana"

    @sad @backend @mobile
    Cenário: Recusar voto após o encerramento da votação
      Dado que o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o período de votação foi encerrado
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada porque a votação foi encerrada
      E nenhum voto é registrado para o estudante "Ana"

  Regra: Uma cédula só é aceita com assinatura digital válida

    @sad @backend @mobile
    Cenário: Recusar cédula sem assinatura digital válida
      Dado que a eleição está no período de votação
      E o estudante "Ana" está autenticado e atende aos critérios de elegibilidade
      Quando submete uma cédula sem assinatura digital válida
      Então a cédula é recusada
      E nenhum voto é registrado para o estudante "Ana"

    @sad @backend @mobile
    Cenário: Recusar cédula atribuída a outra conta
      Dado que a eleição está no período de votação
      E "Ana" está autenticada e atende aos critérios de elegibilidade
      Quando "Ana" envia uma cédula com o identificador da conta CApp de "Bruno"
      Então a cédula é recusada porque o identificador da cédula diverge da conta autenticada
      E nenhum voto é registrado para "Ana"

  Regra: Na votação identificada, o registro associa o eleitor à sua escolha

    @happy @backend @mobile
    Cenário: Registrar voto identificado de estudante elegível
      Dado que a eleição está configurada para votação identificada
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" ainda não votou nessa eleição
      Quando submete uma cédula assinada com sua chave local, indicando a chapa "Aurora"
      Então a cédula é aceita e o voto é confirmado
      E a assinatura digital é validada com a chave pública enviada na cédula de "Ana"
      E o registro identifica "Ana" como eleitora e "Aurora" como sua escolha
      E a participação de "Ana" é marcada como realizada nessa eleição

    @sad @backend @mobile
    Cenário: Recusar segundo voto na modalidade identificada
      Dado que a eleição está configurada para votação identificada
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" já registrou um voto na chapa "Aurora"
      Quando tenta registrar outro voto na chapa "Horizonte"
      Então a tentativa de voto é recusada
      E o voto original na chapa "Aurora" permanece como o único voto de "Ana" nessa eleição

    @sad @backend @mobile
    Cenário: Impedir dois votos identificados em tentativas simultâneas
      Dado que a eleição está configurada para votação identificada
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" ainda não votou nessa eleição
      Quando são realizadas simultaneamente duas tentativas de voto, uma na chapa "Aurora" e outra na chapa "Horizonte"
      Então apenas uma das tentativas é aceita
      E apenas um voto fica registrado para "Ana" nessa eleição
      E a outra tentativa é recusada por participação já realizada

  Regra: A cédula secreta deve ser verificável sem revelar a escolha

    @happy @backend @mobile
    Cenário: Verificar a inclusão da cédula secreta por meio de recibo assinado
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" está autenticado e atende aos critérios de elegibilidade
      E o estudante "Ana" ainda não votou nessa eleição
      Quando registra um voto secreto
      Então o voto é confirmado
      E o servidor fornece um recibo assinado da cédula recebida
      E "Ana" consegue verificar que sua cédula cifrada consta no quadro público
      E o registro público contém o identificador da conta CApp do eleitor, a cédula cifrada e a assinatura do eleitor
      E a cédula cifrada não revela a escolha em texto claro nos dados públicos

    @happy @backend @mobile
    Cenário: Preservar o sigilo diante da ordem e do momento das votações
      Dado que a eleição está configurada para votação secreta
      E diferentes eleitores já registraram suas cédulas
      Quando uma pessoa observa a ordem e o momento de submissão das cédulas
      Então a publicação não expõe escolha em claro nem aleatoriedade da cifragem

  Regra: Na votação secreta, a escolha do eleitor deve permanecer privada

    @happy @backend @mobile
    Cenário: Registrar voto secreto sem revelar a escolha na confirmação
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" ainda não votou nessa eleição
      Quando registra um voto na chapa "Aurora"
      Então o voto é confirmado
      E a participação de "Ana" é marcada como realizada nessa eleição
      E a confirmação não revela a chapa escolhida

    @happy @backend @mobile
    Cenário: Não revelar a escolha individual nos dados públicos da votação secreta
      Dado que a eleição está configurada para votação secreta
      E "Ana" já registrou sua participação nessa eleição por votação secreta
      E o quadro público contém a cédula de "Ana" em formato cifrado
      Quando uma pessoa consulta os dados públicos da eleição
      Então os dados não publicam a escolha em claro de "Ana"

    @sad @backend @mobile
    Cenário: Recusar segundo voto na modalidade secreta
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" já participou dessa eleição
      Quando tenta registrar outro voto na chapa "Horizonte"
      Então a tentativa de voto é recusada
      E a participação de "Ana" continua limitada a um único voto nessa eleição

    @sad @backend @mobile
    Cenário: Impedir dois votos secretos em tentativas simultâneas
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" ainda não votou nessa eleição
      Quando são realizadas simultaneamente duas tentativas de voto, uma na chapa "Aurora" e outra na chapa "Horizonte"
      Então apenas uma das tentativas é aceita
      E apenas um voto é contabilizado para essa participação
      E a outra tentativa é recusada por participação já realizada

  Regra: A conta continua tendo um único voto ao trocar de dispositivo

    @sad
    Cenário: Usar outra chave não permite votar novamente
      Dado que a eleição está no período de votação
      E "Ana" já participou dessa eleição
      Quando "Ana" tenta votar em outro dispositivo com um novo par de chaves
      Então a tentativa de voto é recusada
      E a participação de "Ana" continua limitada a um único voto nessa eleição

    @happy
    Cenário: Recuperar recibo em outro dispositivo sem a chave privada
      Dado que a eleição está configurada para votação secreta
      E que "Ana" já participou dessa eleição
      E "Ana" está autenticada em outro dispositivo sem a chave privada anterior
      Quando solicita seu recibo de participação
      Então recebe o recibo do voto já aceito
      E nenhum novo voto é registrado

    @happy
    Cenário: Repetir o envio depois de perder a confirmação
      Dado que a eleição está configurada para votação secreta
      E que o voto de "Ana" foi aceito mas a confirmação não chegou ao dispositivo
      E o período de votação foi encerrado
      Quando "Ana" retransmite o mesmo pacote assinado com o mesmo identificador de envio
      Então recebe o recibo do voto já aceito
      E nenhum novo voto é registrado

  Regra: Nulo é uma opção própria e participa da contagem

    @happy
    Cenário: Registrar voto nulo
      Dado que a eleição está no período de votação
      E "Ana" está autenticada e atende aos critérios de elegibilidade
      Quando seleciona a opção "Nulo" e confirma o voto
      Então o voto é confirmado
      E a participação de "Ana" é marcada como realizada nessa eleição
      E a opção "Branco" não é oferecida

  Regra: A apuração exige todas as autoridades e igualdade dos totais

    @happy
    Cenário: Incluir Nulo na igualdade da apuração
      Dado que a votação foi encerrada com dois participantes
      E todas as autoridades forneceram contribuições válidas
      Quando a apuração produz um voto para "Aurora" e um voto nulo
      Então a contagem é consistente com os dois participantes

    @sad
    Cenário: Anular por excesso de votos no agregado
      Dado que a votação foi encerrada com dois participantes
      E todas as autoridades forneceram contribuições válidas
      Quando a soma das opções incluindo Nulo resulta em três votos
      Então a eleição é anulada por inconsistência da contagem
      E nenhum vencedor ou posse é produzido
      E os registros permanecem disponíveis para auditoria

    @sad
    Cenário: Anular por déficit de votos no agregado
      Dado que a votação foi encerrada com dois participantes
      E todas as autoridades forneceram contribuições válidas
      Quando a soma das opções incluindo Nulo resulta em um voto
      Então a eleição é anulada por inconsistência da contagem
      E nenhum vencedor ou posse é produzido
      E os registros permanecem disponíveis para auditoria

    @sad
    Cenário: Manter apuração incompleta quando uma autoridade não colabora
      Dado que o período de votação foi encerrado
      E falta uma contribuição válida de autoridade
      Quando a apuração é solicitada
      Então a apuração permanece incompleta e sem totais definitivos
      E nenhum vencedor ou posse é produzido
