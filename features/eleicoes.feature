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

  Regra: Chapa única é permitida e cada participante guarda seu segredo

    Cenário: Preparar eleição com uma única chapa
      Dado que existe uma única chapa aprovada pela comissão
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
