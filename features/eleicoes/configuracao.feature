# language: pt
# Contrato #377. Configuração e calendário são regras de produto ainda pendentes.

@v1 @backend @mobile @pending_backend @pending_mobile
Funcionalidade: Gerenciamento de eleições de Centros Acadêmicos

  Como integrante da comissão eleitoral indicada pela gestão
  Quero configurar a eleição e administrar seu calendário
  Para realizar o processo sem antecipar uma apuração válida

  Contexto:
    Dado que a comissão eleitoral foi indicada pela gestão do CA
    E estou autenticado como integrante autorizado dessa comissão

  Regra: A comissão escolhe a confidencialidade dos votos ao criar a eleição

    Esquema do Cenário: Escolher a modalidade de votação na criação da eleição
      Dado que estou preenchendo os dados de uma nova eleição do CA
      Quando seleciono votação "<modalidade>" e concluo a criação da eleição
      Então a eleição é criada com votação "<modalidade>"

      Exemplos:
        | modalidade   |
        | secreta      |
        | identificada |

  Regra: A votação secreta permite chapa única com autoridades independentes

    Cenário: Preparar eleição com uma única chapa
      Dado que existe uma única chapa aprovada pela comissão
      E que a eleição está configurada para votação secreta
      Quando a configuração eleitoral é concluída
      Então o manifesto contém a chapa e a opção Nulo
      E as autoridades são o servidor, a comissão e a chapa
      E o servidor não recebe os segredos das outras autoridades

  Regra: Um novo encerramento deve ser posterior ao anterior

    Cenário: Adiar a abertura definindo outra duração
      Dado que a eleição ainda está em preparação
      Quando a comissão adia a abertura e escolhe um fim posterior ao anterior
      Então o novo calendário é registrado com motivo e responsável
      E a comissão não é obrigada a preservar a duração original

    Cenário: Recusar alteração com encerramento igual ao anterior
      Dado que a eleição ainda está em preparação
      Quando a comissão adia a abertura mas mantém o encerramento anterior
      Então a alteração do calendário é recusada

    Cenário: Recusar alteração com encerramento anterior ao anterior
      Dado que a eleição ainda está em preparação
      Quando a comissão adia a abertura e escolhe um fim anterior ao anterior
      Então a alteração do calendário é recusada

    Cenário: Prorrogar uma eleição aberta
      Dado que a eleição está aberta e o prazo vigente ainda não venceu
      Quando a comissão escolhe um encerramento posterior ao vigente
      Então o novo calendário é registrado com motivo e responsável
      E o manifesto e a abertura já ocorrida permanecem inalterados
      E as cédulas já aceitas continuam válidas

    Cenário: Recusar prorrogação após o prazo vigente
      Dado que o prazo vigente da eleição já terminou
      Quando a comissão tenta prorrogar o encerramento
      Então a alteração do calendário é recusada
      E a eleição não é reaberta

    Cenário: Recusar encerramento antecipado válido
      Dado que a eleição está aberta e o prazo vigente ainda não venceu
      Quando a comissão solicita encerrar a votação para apurar o resultado
      Então o encerramento antecipado é recusado
      E nenhuma contagem antecipada por opção é publicada

  Regra: Cancelamento manual interrompe o fluxo e preserva o histórico

    Cenário: Cancelar eleição criada por engano
      Dado que a eleição foi criada por engano e ainda não recebeu votos
      Quando a comissão solicita cancelamento com motivo
      Então a eleição fica cancelada com responsável e instante registrados
      E nenhum vencedor ou posse é produzido

    Cenário: Cancelar eleição aberta sem realizar apuração antecipada
      Dado que a eleição está aberta e já recebeu cédulas
      Quando a comissão solicita cancelamento com motivo
      Então a eleição fica cancelada com responsável e instante registrados
      E nenhuma contagem antecipada por opção é publicada
      E as cédulas e o histórico são preservados
      E nenhum novo voto é aceito

  @seguranca_computacional
  Regra: O período inicial deve delimitar um intervalo válido

    Cenário: Criar eleição com período de votação válido
      Dado que estou preenchendo os dados de uma nova eleição do CA
      Quando informo um período de votação válido e concluo a criação da eleição
      Então o calendário inicial registra o início e o encerramento informados

    Cenário: Recusar período de votação com encerramento anterior ao início
      Dado que estou preenchendo os dados de uma nova eleição do CA
      Quando informo um período cujo encerramento é anterior ao início e tento criar a eleição
      Então a criação da eleição é recusada por período inválido

    Cenário: Recusar período de votação com encerramento igual ao início
      Dado que estou preenchendo os dados de uma nova eleição do CA
      Quando informo um período cujo encerramento é igual ao início e tento criar a eleição
      Então a criação da eleição é recusada por período inválido

  @seguranca_computacional
  Regra: Na votação secreta, a abertura exige configuração e autoridades completas

    Contexto:
      Dado que a eleição está configurada para votação secreta
      E a eleição está no período de votação

    Cenário: Recusar abertura quando falta uma autoridade
      Dado que a eleição está em preparação sem todas as autoridades definidas
      Quando a comissão solicita a abertura da votação
      Então a abertura é recusada por falta de autoridades

    Cenário: Abrir eleição com duas chapas e autoridades independentes
      Dado que a eleição está em preparação com duas chapas aprovadas
      E cada chapa possui uma autoridade independente
      E o servidor e a comissão possuem autoridades independentes
      Quando a comissão solicita a abertura da votação
      Então a abertura é aceita após a validação de todas as autoridades
      E o manifesto publicado contém as duas chapas na ordem definida
      E o manifesto publicado contém a opção Nulo uma única vez na última posição
      E o manifesto contém uma autoridade do servidor, uma da comissão e uma para cada chapa
      E o manifesto é publicado com a assinatura do servidor

  @seguranca_computacional
  Regra: Na votação secreta, o manifesto é imutável desde a publicação

    Contexto:
      Dado que a eleição está configurada para votação secreta

    Cenário: Recusar alteração das opções após publicar o manifesto mesmo sem votos
      Dado que o manifesto oficial da eleição foi publicado e ainda não houve votos
      Quando a comissão tenta alterar as chapas ou a ordem das opções
      Então a alteração da configuração eleitoral é recusada
      E o manifesto publicado permanece inalterado

  Regra: O encerramento normal respeita o prazo vigente

    Esquema do Cenário: Encerrar a votação conforme a modalidade ao atingir o prazo vigente
      Dado que a eleição está configurada para votação <modalidade>
      E que a eleição está aberta e o prazo vigente foi atingido
      Quando o processo eleitoral é encerrado
      Então a eleição passa ao estado encerrado
      E nenhum novo voto é aceito
      E o registro final das cédulas é preservado
      E a apuração <regra_das_contribuicoes>

      Exemplos:
        | modalidade   | regra_das_contribuicoes                                 |
        | secreta      | aguarda as contribuições válidas de todas as autoridades |
        | identificada | não exige contribuições de autoridades criptográficas   |
