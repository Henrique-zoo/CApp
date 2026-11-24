# language: pt
# A apuração e a auditoria local são entregas distintas. Os passos pendentes
# registram o comportamento esperado até cada plataforma executar o fluxo real.

@v1 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Apuração e divulgação dos resultados eleitorais

  Como participante de uma eleição de Centro Acadêmico
  Quero consultar o resultado e verificar as evidências da apuração
  Para acompanhar o desfecho da votação

  @sad @seguranca_computacional
  Cenário: Recusar apuração antes do prazo
    Dado um pleito secreto cujo prazo de votação ainda está vigente
    Quando a comissão tenta iniciar a apuração desse pleito
    Então a apuração é recusada e as contagens por chapa permanecem indisponíveis

  @happy @seguranca_computacional
  Cenário: Apurar três cédulas após receber todas as contribuições
    Dado um pleito secreto encerrado com dois votos para Aurora e um para Horizonte
    E todas as autoridades enviaram contribuições válidas para o agregado encerrado
    Quando uma estudante consulta o resultado publicado do pleito
    Então o resultado da apuração informa dois votos para Aurora e um para Horizonte

  @sad @seguranca_computacional
  Cenário: Manter a apuração incompleta na ausência de contribuição
    Dado um pleito secreto encerrado em que falta a contribuição de uma autoridade
    Quando uma estudante consulta o resultado publicado do pleito
    Então a apuração consta como incompleta, sem totais definitivos

  @sad @seguranca_computacional
  Cenário: Recusar contribuição inválida
    Dado um pleito secreto encerrado que aguarda contribuições
    Quando uma autoridade envia uma contribuição incompatível com o agregado encerrado
    Então a contribuição é recusada e a apuração continua incompleta

  @sad @seguranca_computacional
  Cenário: Anular a eleição quando a soma diverge da participação
    Dado um pleito secreto encerrado cujos totais apurados divergem do número de eleitores
    Quando uma estudante consulta o resultado publicado do pleito
    Então o resultado é divulgado como anulado por inconsistência da contagem

  @happy
  Cenário: Consultar resultados e informações de auditoria básica
    Dado um pleito com resultado final publicado
    Quando uma estudante consulta a prestação de contas eleitoral
    Então vê o resultado e os dados públicos do processo

  @happy @seguranca_computacional
  Cenário: Conferir a inclusão da própria cédula offline
    Dado que uma eleitora guardou seu recibo e carregou o quadro público
    Quando verifica a inclusão da própria cédula sem conexão
    Então obtém a confirmação de que seu recibo corresponde à entrada do quadro

  @sad @seguranca_computacional
  Cenário: Detectar adulteração de uma cédula na auditoria independente
    Dado uma cópia do quadro em que uma cédula foi adulterada
    Quando uma estudante verifica as evidências eleitorais no CApp
    Então a auditoria sinaliza a adulteração do quadro eleitoral

  @sad @seguranca_computacional
  Cenário: Informar que a completude é inconclusiva sem âncora externa
    Dado uma cópia internamente coerente do quadro sem âncora externa confiável
    Quando uma estudante verifica as evidências eleitorais no CApp
    Então a auditoria informa que as evidências são insuficientes para atestar a completude

  @happy
  Cenário: Efetivar a gestão eleita uma única vez
    Dado um pleito com resultado final apto a declarar uma chapa eleita
    Quando a conclusão da eleição é processada duas vezes
    Então há uma única gestão sucessora correspondente à chapa eleita

  @sad
  Cenário: Impedir a posse com resultado inapto
    Dado um pleito cancelado, anulado ou sem resultado apto a declarar vencedora
    Quando é consultada a sucessão da gestão do Centro Acadêmico
    Então o pleito não efetiva uma nova gestão
