# language: pt

Funcionalidade: Programa de apadrinhamento estudantil

  Como gestor do centro acadêmico
  Quero definir manualmente padrinhos e afilhados
  Para organizar o programa de apadrinhamento

  @pending_backend @pending_mobile
  Regra: O pareamento manual utiliza participantes válidos do programa

    Cenário: Gestor associa um afilhado a um padrinho
      Dado que o programa de apadrinhamento está configurado
      E existem participantes com inscrição válida no programa
      E o gestor do centro acadêmico está organizando os pareamentos
      Quando ele seleciona um participante como padrinho
      E associa outro participante como afilhado
      Então a associação entre os dois participantes é registrada
      E o pareamento fica disponível na organização do programa

    Cenário: Gestor tenta associar um participante não inscrito
      Dado que o programa de apadrinhamento está configurado
      E o gestor do centro acadêmico está organizando os pareamentos
      E existe uma pessoa que não está inscrita no programa
      Quando ele tenta selecionar essa pessoa como padrinho ou afilhado
      Então a seleção é recusada
      E nenhum pareamento é registrado com essa pessoa

    Cenário: Gestor tenta confirmar um pareamento incompleto
      Dado que o programa de apadrinhamento está configurado
      E o gestor do centro acadêmico está organizando os pareamentos
      Quando ele tenta confirmar o pareamento sem selecionar um padrinho ou um afilhado
      Então o pareamento não é registrado
      E o gestor é informado de que precisa selecionar os dois participantes
