# language: pt
# Os cenários descrevem regras de negócio e resultados observáveis.
# Detalhes técnicos de persistência, concorrência e privacidade devem ser
# verificados nos componentes correspondentes.

@pending_backend @pending_mobile
Funcionalidade: Votação em eleições de Centros Acadêmicos

  Como estudante elegível de um Centro Acadêmico
  Quero consultar as opções e registrar meu voto
  Para participar do processo eleitoral conforme as regras da eleição

  Contexto:
    Dado que existe uma eleição para o Centro Acadêmico "CA Engenharia"
    E a eleição possui as chapas ativas "Aurora" e "Horizonte"
    E existe uma chapa inativa chamada "Legado"

  Regra: Somente eleitores cadastrados antes da abertura podem votar

    @backend @mobile
    Cenário: Recusar voto de estudante não cadastrado previamente
      Dado que a eleição está no período de votação
      E o estudante "Bruno" não consta no cadastro de eleitores definido antes do início da eleição
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada por falta de cadastro eleitoral
      E nenhum voto é registrado para o estudante "Bruno"

  Regra: Somente estudantes com vínculo acadêmico válido podem votar

    @backend @mobile
    Cenário: Recusar voto de estudante sem vínculo acadêmico válido
      Dado que a eleição está no período de votação
      E o estudante "Bruno" não possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada por falta de elegibilidade
      E nenhum voto é registrado para o estudante "Bruno"

    @backend @mobile
    Cenário: Recusar voto de estudante que não atende aos critérios da eleição
      Dado que a eleição está no período de votação
      E o estudante "Bruno" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Bruno" não atende a um critério de elegibilidade definido para a eleição
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada por falta de elegibilidade
      E nenhum voto é registrado para o estudante "Bruno"

  Regra: As opções de voto devem respeitar a situação das chapas

    @backend @mobile
    Cenário: Apresentar somente as chapas ativas como opções de voto
      Dado que a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      Quando consulta as opções de voto
      Então são apresentadas as chapas ativas "Aurora" e "Horizonte"
      E a chapa inativa "Legado" não é apresentada como opção

    @backend @mobile
    Cenário: Recusar voto em chapa inativa
      Dado que a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      Quando tenta registrar um voto na chapa "Legado"
      Então a tentativa de voto é recusada
      E nenhum voto é registrado para o estudante "Ana"

  Regra: O voto só pode ser registrado durante o período de votação

    @backend @mobile
    Cenário: Recusar voto antes do início da votação
      Dado que o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o período de votação ainda não começou
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada porque a votação ainda não começou
      E nenhum voto é registrado para o estudante "Ana"

    @backend @mobile
    Cenário: Recusar voto após o encerramento da votação
      Dado que o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o período de votação foi encerrado
      Quando tenta registrar um voto na chapa "Aurora"
      Então a tentativa de voto é recusada porque a votação foi encerrada
      E nenhum voto é registrado para o estudante "Ana"

  Regra: Uma cédula só é aceita com assinatura digital válida

    @backend @mobile
    Cenário: Recusar cédula sem assinatura digital válida
      Dado que a eleição está no período de votação
      E o estudante "Ana" está cadastrado e atende aos critérios de elegibilidade
      Quando submete uma cédula sem assinatura digital válida
      Então a cédula é recusada
      E nenhum voto é registrado para o estudante "Ana"

    @backend @mobile
    Cenário: Recusar cédula assinada com a chave de outro eleitor
      Dado que a eleição está no período de votação
      E "Ana" está cadastrada e atende aos critérios de elegibilidade
      E "Bruno" possui uma chave pública cadastrada para a eleição
      Quando "Ana" submete uma cédula assinada com a chave privada correspondente à chave pública de "Bruno"
      Então a cédula é recusada porque a assinatura não corresponde à identidade eleitoral de "Ana"
      E nenhum voto é registrado para "Ana"

  Regra: Na votação identificada, o registro associa o eleitor à sua escolha

    @happy @backend @mobile
    Cenário: Registrar voto identificado de estudante elegível
      Dado que a eleição está configurada para votação identificada
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E a chave pública de "Ana" foi cadastrada antes do início da votação
      E o estudante "Ana" ainda não votou nessa eleição
      Quando submete uma cédula assinada com a chave privada correspondente à sua chave pública cadastrada, indicando a chapa "Aurora"
      Então a cédula é aceita e o voto é confirmado
      E a assinatura digital é validada com a chave pública cadastrada de "Ana"
      E o registro identifica "Ana" como eleitora e "Aurora" como sua escolha
      E a participação de "Ana" é marcada como realizada nessa eleição

    @backend @mobile
    Cenário: Recusar segundo voto na modalidade identificada
      Dado que a eleição está configurada para votação identificada
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" já registrou um voto na chapa "Aurora"
      Quando tenta registrar outro voto na chapa "Horizonte"
      Então a tentativa de voto é recusada
      E o voto original na chapa "Aurora" permanece como o único voto de "Ana" nessa eleição

    @backend @mobile
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

    @backend @mobile
    Cenário: Verificar a inclusão da cédula secreta por meio de recibo assinado
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" está cadastrado e atende aos critérios de elegibilidade
      E o estudante "Ana" ainda não votou nessa eleição
      Quando registra um voto secreto
      Então o voto é confirmado
      E o servidor fornece um recibo assinado da cédula recebida
      E "Ana" consegue verificar que sua cédula cifrada consta no quadro público
      E o registro público contém a identificação do eleitor, a cédula cifrada e a assinatura do eleitor
      E a cédula cifrada não revela a escolha em texto claro nos dados públicos

    @backend @mobile
    Cenário: Preservar o sigilo diante da ordem e do momento das votações
      Dado que a eleição está configurada para votação secreta
      E diferentes eleitores já registraram suas cédulas
      Quando uma pessoa observa a ordem e o momento de submissão das cédulas
      Então não consegue determinar a escolha individual de cada eleitor com base nessas informações

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

    @backend @mobile
    Cenário: Não revelar a escolha individual nos dados públicos da votação secreta
      Dado que a eleição está configurada para votação secreta
      E "Ana" já registrou sua participação nessa eleição por votação secreta
      E o quadro público contém a cédula de "Ana" em formato cifrado
      Quando uma pessoa consulta os dados públicos da eleição
      Então não consegue determinar a escolha em texto claro de "Ana" a partir desses dados

    @backend @mobile
    Cenário: Recusar segundo voto na modalidade secreta
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" já participou dessa eleição
      Quando tenta registrar outro voto na chapa "Horizonte"
      Então a tentativa de voto é recusada
      E a participação de "Ana" continua limitada a um único voto nessa eleição

    @backend @mobile
    Cenário: Impedir dois votos secretos em tentativas simultâneas
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação
      E o estudante "Ana" possui vínculo acadêmico válido com o Centro Acadêmico "CA Engenharia"
      E o estudante "Ana" ainda não votou nessa eleição
      Quando são realizadas simultaneamente duas tentativas de voto, uma na chapa "Aurora" e outra na chapa "Horizonte"
      Então apenas uma das tentativas é aceita
      E apenas um voto é contabilizado para essa participação
      E a outra tentativa é recusada por participação já realizada
