# language: pt
# Base: relatório de features, seções 2.4, 10.5, 12.2, 12.3, 12.4 e 12.12.
# Decisões complementares do responsável pelo produto:
# A gestão define o público de votação e a modalidade identificada ou secreta.
# Cada pessoa pode votar uma única vez. O prazo de encerramento é opcional.
# Permissão de chat não concede direito de voto. Consultas não possuem quórum.

@backend @pending_backend
Funcionalidade: Votação em consultas estudantis
  Como estudante autorizado a votar em uma consulta
  Quero conhecer a proposta e registrar meu voto na enquete
  Para participar das decisões da comunidade respeitando as regras da consulta

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está cadastrado no CApp
    E que esse Centro Acadêmico é o contexto ativo do usuário

  Regra: A participação no chat não concede direito de voto

    Cenário: Participação no chat não concede direito de voto
      Dado que existe uma consulta aberta com enquete e chat no Centro Acadêmico ativo
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      E que o estudante pode acessar a consulta e participar do chat, mas não pertence ao público de votação
      Quando o estudante tenta confirmar um voto nessa consulta
      Então o voto é recusado porque o estudante não pertence ao público elegível
      E nenhum voto dessa tentativa é registrado

  Regra: A consulta apresenta sua proposta e os recursos habilitados

    Cenário: Estudante conhece a proposta antes de participar
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      Quando o estudante abre essa consulta
      Então ele visualiza a proposta apresentada pelo Centro Acadêmico
      E visualiza o período de votação e as opções de voto disponíveis

  Regra: A votação na enquete respeita o período quando ele é definido

    Cenário: Estudante registra apoio a uma proposta durante o período de votação
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que ele ainda não votou nessa enquete
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      Quando o estudante seleciona a opção "Apoio a proposta" e confirma seu voto
      Então sua participação na consulta é confirmada
      E a opção "Apoio a proposta" recebe esse voto

    Cenário: Estudante tenta votar antes do início do período
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que o período de votação começa daqui a 1 dia
      Quando o estudante tenta confirmar um voto nessa consulta
      Então o voto é recusado porque o período de votação ainda não começou
      E nenhum voto dessa tentativa é registrado

    Cenário: Estudante tenta votar depois do fim do período
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que o período de votação terminou há 1 hora
      Quando o estudante tenta confirmar um voto nessa consulta
      Então o voto é recusado porque o período de votação terminou
      E nenhum voto dessa tentativa é registrado

  Regra: Cada pessoa pode votar uma única vez na enquete da consulta

    Cenário: Uma segunda tentativa de voto é recusada
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      E que o estudante já registrou seu voto nessa enquete
      Quando o estudante tenta confirmar outro voto nessa enquete
      Então a tentativa é recusada porque ele já votou
      E o voto original é preservado sem acrescentar outro voto

  Regra: A gestão define o público elegível para cada enquete

    Cenário: Pessoa fora do público configurado tenta votar
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      E que o estudante não pertence ao público definido pela gestão para votar nessa enquete
      Quando o estudante tenta confirmar um voto nessa consulta
      Então o voto é recusado porque o estudante não pertence ao público elegível
      E nenhum voto dessa tentativa é registrado

  Regra: A gestão escolhe entre voto identificado e secreto para cada enquete

    Cenário: Voto identificado mantém a associação entre a pessoa e sua escolha
      Dado que existe uma consulta cuja enquete foi configurada pela gestão com voto identificado
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que ele ainda não votou nessa enquete
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      Quando o estudante seleciona a opção "Apoio a proposta" e confirma seu voto
      Então sua participação na consulta é confirmada
      E o voto permanece associado à identidade do estudante

    Cenário: Voto secreto preserva a escolha e impede nova participação
      Dado que existe uma consulta cuja enquete foi configurada pela gestão com voto secreto
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que ele ainda não votou nessa enquete
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      Quando o estudante seleciona a opção "Apoio a proposta" e confirma seu voto
      Então sua participação na consulta é confirmada
      E a escolha registrada não permite identificar o estudante
      E uma nova tentativa de voto desse estudante é recusada

  Regra: A enquete pode permanecer aberta sem prazo de encerramento

    Cenário: Consulta com enquete permite votar sem prazo de encerramento definido
      Dado que existe uma consulta aberta com enquete e sem prazo de encerramento
      E que o estudante pertence ao público definido pela gestão para votar nessa enquete
      E que ele ainda não votou nessa enquete
      Quando o estudante seleciona a opção "Apoio a proposta" e confirma seu voto
      Então sua participação na consulta é confirmada
      E a opção "Apoio a proposta" recebe esse voto
