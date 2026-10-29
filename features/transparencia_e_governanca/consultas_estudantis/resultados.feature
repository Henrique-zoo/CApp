# language: pt
# Base: relatório de features, seções 2.4, 10.5, 12.2, 12.3, 12.4 e 12.12.
# Decisões complementares do responsável pelo produto:
# A gestão define o público de resultados separadamente do público de votação.
# Resultados ficam disponíveis durante e após a votação ao público autorizado.

@backend @pending_backend
Funcionalidade: Resultados das enquetes de consultas estudantis
  Como usuário autorizado a visualizar os resultados de uma consulta
  Quero acompanhar os votos recebidos pelas opções da enquete
  Para conhecer a opinião dos participantes durante e após a votação

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está cadastrado no CApp
    E que esse Centro Acadêmico é o contexto ativo do usuário

  Regra: O público de resultados é independente do público de votação

    Cenário: Direito de voto não concede acesso aos resultados
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que ele pode acessar a consulta, mas não pertence ao público de visualização dos resultados
      Quando ele consulta o resultado da enquete
      Então o acesso ao resultado é recusado
      E os totais de votos não são apresentados

  Regra: Resultados ficam disponíveis durante e após a votação para o público escolhido pela gestão

    Cenário: Pessoa autorizada acompanha o resultado durante a votação
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      E que a enquete recebeu 2 votos em "Apoio a proposta" e 1 voto em "Não apoio a proposta"
      E que o usuário pertence ao público definido pela gestão para visualizar os resultados
      Quando ele consulta o resultado da enquete
      Então ele visualiza 2 votos em "Apoio a proposta" e 1 voto em "Não apoio a proposta"

    Cenário: Pessoa autorizada consulta o resultado após a votação
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o período de votação terminou há 1 hora
      E que a enquete recebeu 2 votos em "Apoio a proposta" e 1 voto em "Não apoio a proposta"
      E que o usuário pertence ao público definido pela gestão para visualizar os resultados
      Quando ele consulta o resultado da enquete
      Então ele visualiza 2 votos em "Apoio a proposta" e 1 voto em "Não apoio a proposta"

    Cenário: Pessoa fora do público configurado tenta consultar o resultado
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o usuário não pertence ao público definido pela gestão para visualizar os resultados
      Quando ele consulta o resultado da enquete
      Então o acesso ao resultado é recusado
      E os totais de votos não são apresentados
