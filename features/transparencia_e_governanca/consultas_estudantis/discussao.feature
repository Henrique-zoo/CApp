# language: pt
# Base: relatório de features, seções 2.4, 10.5, 12.2, 12.3, 12.4 e 12.12.
# Decisões complementares do responsável pelo produto:
# Chat é opcional e seu público é configurado separadamente do acesso à consulta.
# Encerramento manual ou pelo prazo não bloqueia mensagens no chat habilitado.

@backend @pending_backend
Funcionalidade: Discussão de propostas em consultas estudantis
  Como usuário autorizado a participar do chat de uma consulta
  Quero trocar mensagens sobre a proposta com os demais participantes
  Para contribuir para a discussão da comunidade mesmo quando não houver enquete

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está cadastrado no CApp
    E que esse Centro Acadêmico é o contexto ativo do usuário

  Regra: O público de participação no chat é independente do público de acesso à consulta

    Cenário: Acesso à consulta não concede participação no chat
      Dado que existe uma consulta aberta com chat habilitado e sem enquete no Centro Acadêmico ativo
      E que o usuário pertence ao público de acesso à consulta, mas não ao público de participação no chat
      Quando ele tenta enviar uma mensagem no chat da consulta
      Então o envio é recusado por falta de permissão para participar do chat
      E nenhuma mensagem dessa tentativa é publicada

  Regra: O chat habilitado permite discutir a proposta conforme as permissões de acesso

    Cenário: Usuário autorizado discute uma proposta sem enquete
      Dado que existe uma consulta aberta com chat habilitado e sem enquete no Centro Acadêmico ativo
      E que o usuário está autorizado a acessar essa consulta e participar do chat
      Quando ele envia a mensagem "Sugiro ampliar também o horário aos sábados" no chat da consulta
      Então a mensagem é publicada no chat associado à proposta
      E outros usuários autorizados a acessar o chat podem ler essa mensagem
      E a publicação da mensagem não registra um voto

    Cenário: Usuário sem acesso à consulta tenta enviar mensagem
      Dado que existe uma consulta aberta com chat habilitado e sem enquete no Centro Acadêmico ativo
      E que o usuário não está autorizado a acessar essa consulta
      Quando ele tenta enviar uma mensagem no chat da consulta
      Então o envio é recusado por falta de acesso à consulta
      E nenhuma mensagem dessa tentativa é publicada

  Regra: O encerramento da consulta não bloqueia mensagens no chat habilitado

    Cenário: Chat permanece disponível após encerramento manual
      Dado que uma consulta foi encerrada manualmente e possui chat habilitado
      E que o usuário está autorizado a acessar essa consulta e participar do chat
      Quando ele envia a mensagem "Podemos discutir os próximos passos da proposta" no chat da consulta
      Então a mensagem é publicada no chat associado à proposta
      E outros usuários autorizados a acessar o chat podem ler essa mensagem
      E a consulta permanece encerrada

    Cenário: Chat aceita mensagens após encerramento pelo prazo
      Dado que uma consulta foi encerrada pelo prazo e possui chat habilitado
      E que o usuário está autorizado a acessar essa consulta e participar do chat
      Quando ele envia a mensagem "Quero comentar a proposta mesmo após a votação" no chat da consulta
      Então a mensagem é publicada no chat associado à proposta
      E outros usuários autorizados a acessar o chat podem ler essa mensagem
      E a consulta permanece encerrada
