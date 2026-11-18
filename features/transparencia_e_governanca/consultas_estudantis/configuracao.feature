# language: pt
# Base: relatório de features, seções 2.4, 10.5, 12.2, 12.3, 12.4 e 12.12.
# Decisões complementares do responsável pelo produto:
# Enquete e chat são opcionais; os quatro públicos são configurados separadamente.
# O prazo é opcional; encerramento manual exige permissão no CA.
# O encerramento não bloqueia o chat habilitado. Consultas não possuem quórum.

@backend @pending_backend
Funcionalidade: Configuração de consultas estudantis
  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero apresentar propostas e configurar os recursos, públicos e encerramento das consultas
  Para organizar a participação da comunidade no contexto do meu Centro Acadêmico

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está cadastrado no CApp
    E que esse Centro Acadêmico é o contexto ativo do usuário

  Regra: A criação de consultas exige a permissão específica no Centro Acadêmico da consulta

    Cenário: Integrante autorizado da gestão apresenta uma proposta para consulta
      Dado que o usuário integra a gestão do Centro Acadêmico ativo
      E possui a permissão "Criar consultas estudantis" nesse Centro Acadêmico
      Quando ele cadastra uma consulta estudantil com enquete e os seguintes dados:
        | título            | Ampliação do horário dos laboratórios                     |
        | proposta          | Manter os laboratórios abertos até as 22h nos dias letivos |
        | início da votação | daqui a 2 dias                                            |
        | fim da votação    | daqui a 7 dias                                            |
        | opções de voto    | Apoio a proposta, Não apoio a proposta                     |
        | modalidade        | Identificada                                              |
      Então a consulta é registrada no Centro Acadêmico ativo
      E preserva a proposta, o período e as opções de voto informados

    Cenário: Integrante da gestão sem a permissão específica tenta criar consulta
      Dado que o usuário integra a gestão do Centro Acadêmico ativo
      E não possui a permissão "Criar consultas estudantis" nesse Centro Acadêmico
      Quando ele tenta cadastrar uma consulta estudantil nesse Centro Acadêmico
      Então a criação da consulta é recusada por falta da permissão "Criar consultas estudantis"
      E nenhuma nova consulta é registrada nesse Centro Acadêmico

    Cenário: Estudante sem perfil administrativo tenta criar consulta
      Dado que o usuário é estudante e não integra a gestão do Centro Acadêmico ativo
      Quando ele tenta cadastrar uma consulta estudantil nesse Centro Acadêmico
      Então a criação da consulta é recusada por falta da permissão "Criar consultas estudantis"
      E nenhuma nova consulta é registrada nesse Centro Acadêmico

    Cenário: Permissão concedida em outro Centro Acadêmico não autoriza a criação
      Dado que o usuário possui a permissão "Criar consultas estudantis" somente no "Centro Acadêmico de Matemática"
      Quando ele tenta cadastrar uma consulta estudantil no "Centro Acadêmico de Ciência da Computação"
      Então a criação da consulta é recusada por falta da permissão "Criar consultas estudantis"
      E nenhuma nova consulta é registrada nesse Centro Acadêmico

  Regra: A consulta pode apresentar uma proposta com enquete e chat opcionais

    Cenário: Gestão cria uma consulta para discussão sem enquete
      Dado que o usuário integra a gestão do Centro Acadêmico ativo
      E possui a permissão "Criar consultas estudantis" nesse Centro Acadêmico
      Quando ele cadastra uma consulta com proposta e chat habilitado, sem enquete
      Então a consulta é registrada no Centro Acadêmico ativo
      E nenhuma opção de voto é exigida para essa consulta
      E a consulta oferece discussão pelo chat, sem votação

    Cenário: Gestão cria uma consulta com enquete sem chat
      Dado que o usuário integra a gestão do Centro Acadêmico ativo
      E possui a permissão "Criar consultas estudantis" nesse Centro Acadêmico
      Quando ele cadastra uma consulta com proposta e enquete, sem habilitar o chat
      Então a consulta é registrada no Centro Acadêmico ativo
      E a consulta oferece votação, sem chat

    Cenário: Gestão combina discussão e votação na mesma consulta
      Dado que o usuário integra a gestão do Centro Acadêmico ativo
      E possui a permissão "Criar consultas estudantis" nesse Centro Acadêmico
      Quando ele cadastra uma consulta com proposta, enquete e chat habilitado
      Então a consulta é registrada no Centro Acadêmico ativo
      E a enquete e o chat estão associados à mesma proposta

  Regra: A gestão configura separadamente os públicos de acesso, chat, voto e resultados

    Cenário: Gestão define públicos independentes para os recursos da consulta
      Dado que um integrante autorizado da gestão está configurando uma consulta com enquete e chat
      Quando ele define separadamente quem pode acessar a consulta, participar do chat, votar e visualizar resultados
      Então cada público informado é salvo para a finalidade correspondente
      E configurar um desses públicos não concede automaticamente as demais permissões

  Regra: O encerramento antecipado exige a permissão correspondente na gestão do CA

    Cenário: Integrante autorizado da gestão encerra a consulta antes do prazo
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o período de votação começou há 1 dia e termina daqui a 1 dia
      E que o usuário integra a gestão e possui permissão para encerrar consultas nesse Centro Acadêmico
      Quando ele solicita o encerramento antecipado da consulta
      Então a consulta é encerrada
      E novos votos nessa enquete são recusados

    Cenário: Integrante da gestão sem permissão tenta encerrar a consulta
      Dado que existe uma consulta com enquete no Centro Acadêmico ativo
      E que o usuário integra a gestão, mas não possui permissão para encerrar consultas nesse Centro Acadêmico
      Quando ele solicita o encerramento antecipado da consulta
      Então o encerramento é recusado por falta de permissão
      E a consulta permanece aberta

  Regra: O prazo de encerramento é opcional e encerrar a consulta não bloqueia seu chat

    Cenário: Consulta sem enquete pode permanecer aberta sem prazo de encerramento
      Dado que o usuário integra a gestão do Centro Acadêmico ativo
      E possui a permissão "Criar consultas estudantis" nesse Centro Acadêmico
      Quando ele cadastra uma consulta com proposta e chat, sem enquete e sem prazo de encerramento
      Então a consulta é registrada no Centro Acadêmico ativo
      E ela permanece aberta até que seja encerrada por um integrante autorizado da gestão

    Cenário: Gestão autorizada encerra consulta sem enquete e sem prazo
      Dado que existe uma consulta aberta com chat habilitado, sem enquete e sem prazo de encerramento
      E que o usuário integra a gestão e possui permissão para encerrar consultas nesse Centro Acadêmico
      Quando ele solicita o encerramento da consulta
      Então a consulta é encerrada
      E o chat continua habilitado para o público autorizado

    Cenário: Término do prazo encerra a votação sem impedir a discussão
      Dado que existe uma consulta aberta com enquete e chat no Centro Acadêmico ativo
      E que a consulta tem encerramento previsto para daqui a 1 hora
      Quando o prazo de encerramento da consulta é atingido
      Então a consulta é encerrada
      E novos votos nessa enquete são recusados
      E o chat continua habilitado para o público autorizado
