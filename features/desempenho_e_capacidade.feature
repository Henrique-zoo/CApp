# language: pt

Funcionalidade: Desempenho e capacidade do CApp
  Como equipe responsável pelo CApp
  Quero avaliar cenários reproduzíveis de carga e uso
  Para acompanhar o desempenho conforme crescem os dados e a quantidade de CAs

  Contexto:
    Dado que o ambiente de medição está isolado da produção
    E que os dados utilizados são sintéticos e reproduzíveis

  @backend @pending_backend
  Cenário: Manter latência e taxa de erros dentro das metas na carga normal
    Dado que a massa de dados sintética do perfil "medio" foi preparada com a semente fixa
    E que somente os fluxos de backend implementados e mapeados foram incluídos na carga
    Quando o perfil "normal" de 20 requisições por segundo é executado por 600 segundos
    Então o percentil 95 da latência de leitura não supera 500 milissegundos
    E o percentil 99 da latência de leitura não supera 1000 milissegundos
    E a taxa de erros inesperados não supera 1 por cento
    E a execução registra o consumo de CPU da API e do PostgreSQL

  @backend @pending_backend
  Cenário: Manter a latência das escritas válidas dentro das metas
    Dado que existe um fluxo de escrita válida implementado e mapeado para a medição
    E que a massa de dados sintética do perfil "medio" está preparada
    Quando o perfil de carga que inclui escritas válidas é executado
    Então o percentil 95 da latência das escritas não supera 800 milissegundos
    E o percentil 99 da latência das escritas não supera 1500 milissegundos
    E a taxa de erros inesperados não supera 1 por cento

  @backend @pending_backend
  Cenário: Avaliar a utilização de recursos durante o pico planejado
    Dado que a massa de dados sintética do perfil "medio" está preparada
    E que os fluxos incluídos na carga foram mapeados para operações existentes
    Quando a carga aumenta de 20 para 50 requisições por segundo em 60 segundos
    E permanece no pico por 300 segundos
    Então a CPU média da API não supera 85 por cento
    E a CPU média do PostgreSQL não supera 85 por cento
    E a taxa de erros inesperados não supera 1 por cento

  @backend @pending_backend
  Cenário: Recuperar o comportamento de referência após uma rajada
    Dado que o ambiente está sob uma carga de referência de 20 requisições por segundo
    Quando a carga aumenta para 80 requisições por segundo em 10 segundos
    E permanece nessa carga por 60 segundos antes de retornar ao nível de referência
    Então o sistema retorna ao comportamento de referência em até 60 segundos
    E a medição registra latência, erros e utilização de recursos durante a recuperação

  @backend @pending_backend
  Cenário: Avaliar consultas concorrentes com crescimento e distribuição desigual entre CAs
    Dado que a massa de dados sintética do perfil "grande" contém 200 CAs
    E que os 10 por cento de CAs mais utilizados concentram 60 por cento das requisições e dos registros
    E que os fluxos de leitura avaliados estão implementados e mapeados
    Quando consultas representativas são executadas concorrentemente para diferentes CAs
    Então os resultados de cada consulta correspondem ao CA solicitado
    E a medição registra a carga alcançada e a latência por tipo de operação

  @mobile @pending_mobile
  Cenário: Avaliar inicialização e apresentação do primeiro conteúdo útil
    Dado que o aplicativo está em um dispositivo intermediário com pelo menos 4 GB de RAM
    E que o benchmark usa um build no modo "profile" e o perfil de rede "movel_comum"
    Quando a inicialização fria é medida em 5 repetições
    E a inicialização subsequente é medida em 5 repetições
    Então o percentil 95 da inicialização fria não supera 3000 milissegundos
    E o percentil 95 da inicialização subsequente não supera 1500 milissegundos
    E o percentil 95 da apresentação do primeiro conteúdo útil não supera 2000 milissegundos

  @mobile @pending_mobile
  Cenário: Avaliar a apresentação de conteúdo em uma rede limitada
    Dado que o aplicativo está usando o perfil de rede "limitada"
    E que a jornada de consulta está implementada e pode ser repetida
    Quando o usuário navega por uma listagem e consulta um detalhe
    Então o percentil 95 da apresentação do primeiro conteúdo útil não supera 4000 milissegundos
    E a medição registra a duração da jornada e as falhas observadas

  @mobile @pending_mobile
  Cenário: Avaliar fluidez e uso de memória durante jornadas repetidas
    Dado que a jornada de lista, detalhe, busca e troca de CA está implementada
    E que as medições são realizadas em builds no modo "profile"
    Quando as jornadas são repetidas nos dispositivos intermediário e restrito
    Então o percentil 95 do tempo de quadro não supera 16,7 milissegundos em 60 Hz
    E o percentil 99 do tempo de quadro não supera 33,3 milissegundos em 60 Hz
    E o percentil 95 de memória PSS não supera 250 MB no dispositivo intermediário
    E o percentil 95 de memória PSS não supera 200 MB no dispositivo restrito
    E a execução registra crashes e encerramentos por falta de memória

  @backend @mobile @pending_backend @pending_mobile
  Cenário: Manter a consistência das informações ao trocar o CA ativo
    Dado que o usuário pode selecionar entre diferentes CAs
    E que há conteúdo sintético associado a mais de um CA
    Quando o usuário seleciona outro CA e consulta seu conteúdo
    Então as informações exibidas correspondem ao CA selecionado
    E os resultados da consulta não misturam dados de CAs diferentes
    E a troca de contexto pode ser avaliada tanto na resposta do backend quanto na interface mobile
