# language: pt

# Referência: Relatório de Features do Projeto, seção 8.2.
@pending_backend @pending_mobile
Funcionalidade: Consulta do FAQ

  Como estudante
  Quero consultar dúvidas frequentes
  Para resolver questões comuns

  Regra: A consulta considera apenas o FAQ do CA ativo

    Cenário: Consultar perguntas frequentes do CA ativo
      Dado que o CA ativo é "CA de Computação"
      E existe a pergunta frequente "Como solicitar apoio acadêmico?" nesse CA
      E existe a pergunta frequente "Qual o horário da biblioteca?" somente no "CA de Matemática"
      Quando consulto o FAQ
      Então vejo a pergunta "Como solicitar apoio acadêmico?"
      E não vejo a pergunta "Qual o horário da biblioteca?"

    Cenário: Visualizar a resposta de uma pergunta frequente
      Dado que o CA ativo é "CA de Computação"
      E existe a pergunta frequente "Como solicitar apoio acadêmico?" com a resposta "Entre em contato com a gestão do CA"
      Quando seleciono a pergunta "Como solicitar apoio acadêmico?"
      Então vejo a resposta "Entre em contato com a gestão do CA"

    Cenário: Buscar perguntas frequentes por um termo
      Dado que o CA ativo é "CA de Computação"
      E existem perguntas frequentes que correspondem a "matrícula" nesse CA
      E existem perguntas frequentes que não correspondem a "matrícula" nesse CA
      Quando busco no FAQ por "matrícula"
      Então vejo as perguntas correspondentes a "matrícula"
      E não vejo perguntas que não correspondem à busca

    Cenário: Buscar um termo sem correspondências
      Dado que o CA ativo é "CA de Computação"
      E nenhuma pergunta frequente desse CA corresponde a "termo sem correspondência"
      Quando busco no FAQ por "termo sem correspondência"
      Então não vejo perguntas correspondentes à busca

    Cenário: Consultar um FAQ sem perguntas disponíveis
      Dado que o CA ativo é "CA de Computação"
      E não existem perguntas frequentes disponíveis nesse CA
      Quando consulto o FAQ
      Então nenhuma pergunta frequente é apresentada

    Cenário: Não exibir perguntas de outro CA na busca
      Dado que o CA ativo é "CA de Computação"
      E nenhuma pergunta frequente desse CA corresponde a "matrícula"
      E existe uma pergunta frequente que corresponde a "matrícula" somente no "CA de Matemática"
      Quando busco no FAQ por "matrícula"
      Então não vejo a pergunta do "CA de Matemática"
      E não vejo perguntas correspondentes à busca no CA ativo

    Cenário: Atualizar a consulta quando o CA ativo muda
      Dado que o CA ativo é "CA de Computação"
      E existe a pergunta frequente "Como solicitar apoio acadêmico?" nesse CA
      E existe a pergunta frequente "Qual o horário de atendimento?" somente no "CA de Matemática"
      Quando seleciono o "CA de Matemática" como CA ativo
      E consulto o FAQ
      Então vejo a pergunta "Qual o horário de atendimento?"
      E não vejo a pergunta "Como solicitar apoio acadêmico?"
