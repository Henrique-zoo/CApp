# language: pt

@backend @mobile @pending_backend @pending_mobile
Funcionalidade: Programa de apadrinhamento estudantil

  Como gestor do centro acadêmico
  Quero definir manualmente padrinhos e afilhados
  Para organizar o programa de apadrinhamento

  Regra: O pareamento manual utiliza participantes válidos do programa

    Cenário: Gestor associa um afilhado a um padrinho
      Dado que o programa de apadrinhamento está configurado
      E existem participantes com inscrição válida no programa
      E o gestor possui a permissão "Gerenciar o apadrinhamento" no CA do programa
      Quando ele seleciona um participante como padrinho
      E associa outro participante como afilhado
      Então a associação entre os dois participantes é registrada
      E o pareamento fica disponível na organização do programa

    Cenário: Gestor tenta associar um participante não inscrito
      Dado que o programa de apadrinhamento está configurado
      E o gestor possui a permissão "Gerenciar o apadrinhamento" no CA do programa
      E existe uma pessoa que não está inscrita no programa
      Quando ele tenta selecionar essa pessoa como padrinho ou afilhado
      Então a seleção é recusada
      E nenhum pareamento é registrado com essa pessoa

    Cenário: Gestor tenta confirmar um pareamento incompleto
      Dado que o programa de apadrinhamento está configurado
      E o gestor possui a permissão "Gerenciar o apadrinhamento" no CA do programa
      Quando ele tenta confirmar o pareamento sem selecionar um padrinho ou um afilhado
      Então o pareamento não é registrado
      E o gestor é informado de que precisa selecionar os dois participantes

  Regra: O pareamento manual exige a permissão correspondente no CA do programa

    Cenário: Integrante sem a permissão não registra pareamentos
      Dado que o programa de apadrinhamento está configurado
      E existem participantes com inscrição válida no programa
      E o integrante da gestão não possui a permissão "Gerenciar o apadrinhamento" nesse CA
      Quando tenta confirmar um pareamento manual
      Então a operação é recusada por falta de permissão
      E nenhum pareamento é registrado

    Cenário: A permissão em outro CA não autoriza o pareamento neste programa
      Dado que o programa de apadrinhamento está configurado
      E o integrante possui a permissão "Gerenciar o apadrinhamento" somente em outro CA
      Quando tenta confirmar um pareamento manual neste programa
      Então a operação é recusada por falta de permissão
      E os pareamentos do programa permanecem inalterados
