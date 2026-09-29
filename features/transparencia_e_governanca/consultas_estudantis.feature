# language: pt

Funcionalidade: Consultas estudantis e participação acadêmica
  Como estudante ou membro da gestão de um Centro Acadêmico
  Quero criar, gerenciar, participar e auditar consultas públicas estudantis
  Para deliberar sobre temas acadêmicos com transparência e de forma independente de eleições de chapas

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está ativo e regular no CApp
    E que o Centro Acadêmico possui estudantes associados matriculados no curso

  # ---------------------------------------------------------------------------
  # 1. Criação e Configuração de Consultas Estudantis
  # ---------------------------------------------------------------------------
  Regra: Apenas membros autorizados da gestão do CA podem criar e configurar consultas

    Cenário: Membro autorizado da gestão cria uma consulta estudantil com sucesso
      Dado que o usuário é um membro da gestão do CA com permissão de governança
      Quando ele cadastra uma nova consulta estudantil definindo:
        | título                     | Reforma dos Laboratórios de Informática          |
        | descrição                  | Escolha de prioridade na alocação de recursos    |
        | início da votação          | daqui a 2 dias                                   |
        | término da votação         | daqui a 7 dias                                   |
        | opções                     | Manutenção de Hardware, Atualização de Software  |
        | critério de elegibilidade  | Todos os estudantes com matrícula ativa no curso |
      Então a consulta estudantil é registrada com sucesso no sistema
      E fica vinculada ao Centro Acadêmico no estado "Agendada" aguardando o início do período

    Cenário: Usuário sem permissão tenta criar uma consulta estudantil
      Dado que o usuário é um estudante regularmente matriculado, mas não integra a gestão do CA
      Quando ele tenta criar uma consulta estudantil para o Centro Acadêmico
      Então a criação é recusada por falta de permissão administrativa
      E nenhuma nova consulta é registrada para o Centro Acadêmico

  # ---------------------------------------------------------------------------
  # 2. Participação, Elegibilidade e Prevenção de Voto Duplicado
  # ---------------------------------------------------------------------------
  Regra: Apenas estudantes elegíveis podem participar, com garantia de voto único e sigiloso

    Cenário: Estudante elegível registra seu voto com sucesso
      Dado que existe uma consulta estudantil em andamento no Centro Acadêmico
      E que o estudante atende a todos os critérios de elegibilidade da consulta
      E que o estudante ainda não participou dessa consulta
      Quando o estudante escolhe uma das opções válidas e confirma seu voto
      Então a sua participação é confirmada com sucesso
      E o voto é computado de forma anônima no cômputo da consulta

    Cenário: Estudante inelegível tem sua participação recusada
      Dado que existe uma consulta estudantil em andamento com critério exclusivo para "Estudantes Formandos"
      E que o estudante é calouro e não atende aos critérios de elegibilidade
      Quando o estudante tenta submeter um voto nessa consulta
      Então a participação é rejeitada informando que o estudante não é elegível
      E nenhum voto é computado para a consulta

    Cenário: Prevenção estrita de voto duplicado na mesma consulta
      Dado que uma estudante elegível já registrou seu voto em uma consulta em andamento
      Quando ela tenta submeter um novo voto na mesma consulta
      Então a nova tentativa é recusada por duplicidade de voto
      E o cômputo da consulta permanece inalterado com apenas o voto original

  # ---------------------------------------------------------------------------
  # 3. Ciclo de Vida, Período e Fechamento
  # ---------------------------------------------------------------------------
  Regra: A votação só é permitida dentro do período estipulado, podendo ser encerrada no prazo ou por deliberação do CA

    Cenário: Tentativa de participação em consulta que ainda não começou
      Dado que existe uma consulta estudantil agendada com período de votação iniciando no dia seguinte
      Quando um estudante elegível tenta registrar seu voto antecipadamente
      Então a ação é bloqueada com a indicação de que o período de votação ainda não foi iniciado

    Cenário: Tentativa de participação em consulta expirada
      Dado que o período de votação de uma consulta estudantil encerrou há 1 hora
      Quando um estudante elegível tenta registrar um voto
      Então a ação é bloqueada com a indicação de que a consulta já está encerrada
      E o voto não é aceito

    Cenário: Encerramento automático ao término do período estipulado
      Dado que uma consulta estudantil possui período de votação até o horário atual
      Quando o tempo limite de votação é atingido
      Então a consulta transiciona automaticamente para o estado "Encerrada"
      E novas tentativas de participação passam a ser bloqueadas imediatamente

    Cenário: Encerramento manual de consulta por membro autorizado da gestão do CA
      Dado que existe uma consulta estudantil em andamento
      E que o usuário é um membro da gestão do CA com permissão de governança
      Quando ele solicita o encerramento manual da consulta com justificativa registrada
      Então a consulta é finalizada com o estado "Encerrada"
      E a urna de votação é imediatamente fechada para novas participações

  # ---------------------------------------------------------------------------
  # 4. Apuração, Transparência e Independência do Fluxo Eleitoral
  # ---------------------------------------------------------------------------
  Regra: Resultados devem ser transparentes após o encerramento e independentes de eleições de chapas

    Cenário: Consulta dos resultados apurados de uma consulta encerrada
      Dado que uma consulta estudantil foi encerrada e teve sua apuração finalizada
      Quando qualquer estudante vinculado ao Centro Acadêmico acessa os resultados
      Então ele visualiza o total de participantes da consulta
      E visualiza a contagem consolidada e o percentual de cada opção disponível
      E nenhum dado identificável que viole o sigilo do voto individual é exibido

    Cenário: Tentativa de visualização de resultados parciais durante a votação
      Dado que uma consulta estudantil está com a votação em andamento
      Quando um estudante ou gestor tenta acessar a apuração parcial dos votos
      Então os resultados consolidados permanecem ocultos até o término oficial da consulta

    Cenário: Independência entre a consulta estudantil e o processo de eleição de chapas
      Dado que o Centro Acadêmico possui simultaneamente:
        | Processo                    | Estado       |
        | Eleição Geral de Chapas     | Em andamento |
        | Consulta sobre o Calendário | Em andamento |
      Quando um estudante participa da consulta sobre o calendário
      Então essa participação não altera o caderno eleitoral da eleição de chapas
      E as regras de quórum, elegibilidade e sigilo da consulta operam de modo isolado da eleição
