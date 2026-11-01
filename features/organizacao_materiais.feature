# language: pt

# Referência principal: Relatório de Features do Projeto, seção 6.3.
# Referências complementares: seções 6.1 e 6.2 e issue #118.
@pending_backend @pending_mobile
Funcionalidade: Organização de materiais por disciplina

  Como estudante
  Quero encontrar materiais organizados por disciplina
  Para localizar conteúdos rapidamente

  Regra: A organização deve respeitar as disciplinas da grade curricular do contexto acadêmico

    Cenário: Visualizar as disciplinas disponíveis para consulta
      Dado que o CA ativo é "CA de Computação"
      E a grade curricular do contexto acadêmico possui as disciplinas "Cálculo I", "Álgebra Linear" e "Física I"
      Quando o estudante acessa a organização de materiais por disciplina
      Então as disciplinas da grade curricular são apresentadas para consulta
      E o estudante pode selecionar uma disciplina para visualizar seus materiais

    Cenário: Não apresentar disciplinas de outro contexto acadêmico
      Dado que o CA ativo é "CA de Computação"
      E a grade curricular desse contexto possui a disciplina "Cálculo I"
      E existe a disciplina "Direito Constitucional" em outro contexto acadêmico
      Quando o estudante acessa a organização de materiais por disciplina
      Então a disciplina "Cálculo I" é apresentada
      E a disciplina "Direito Constitucional" não é apresentada

  Regra: Cada material deve estar associado a uma disciplina compatível com seu contexto acadêmico

    Cenário: Associar um material publicado a uma disciplina compatível
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" nesse CA
      E a disciplina "Cálculo I" pertence à grade curricular do contexto acadêmico
      Quando a gestão associa "Resumo de Cálculo I" à disciplina "Cálculo I"
      Então a associação do material com a disciplina é registrada
      E o material passa a pertencer à organização da disciplina "Cálculo I"
      E o material fica disponível para consulta nessa disciplina

    Cenário: Recusar associação com disciplina de outro contexto acadêmico
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" nesse CA
      E existe a disciplina "Direito Constitucional" em outro contexto acadêmico
      Quando a gestão tenta associar "Resumo de Cálculo I" à disciplina "Direito Constitucional"
      Então a associação é recusada
      E o material não fica associado à disciplina incompatível

    Cenário: Manter a associação do material após uma nova consulta
      Dado que o CA ativo é "CA de Computação"
      E o material publicado "Resumo de Cálculo I" está associado à disciplina "Cálculo I"
      Quando o estudante acessa a organização de materiais por disciplina
      E seleciona a disciplina "Cálculo I"
      Então o material "Resumo de Cálculo I" é apresentado
      Quando o estudante sai da organização de materiais
      E acessa novamente a organização de materiais por disciplina
      E seleciona novamente a disciplina "Cálculo I"
      Então o material "Resumo de Cálculo I" continua associado à disciplina
      E permanece disponível para consulta

  Regra: A consulta deve apresentar somente os materiais publicados associados à disciplina selecionada

    Cenário: Consultar materiais de uma disciplina
      Dado que o CA ativo é "CA de Computação"
      E existem os materiais publicados "Resumo de Cálculo I" e "Exercícios de Cálculo I" associados à disciplina "Cálculo I"
      E existe o material publicado "Resumo de Álgebra Linear" associado à disciplina "Álgebra Linear"
      Quando o estudante seleciona a disciplina "Cálculo I"
      Então o material "Resumo de Cálculo I" é apresentado
      E o material "Exercícios de Cálculo I" é apresentado
      E o material "Resumo de Álgebra Linear" não é apresentado nessa consulta

    Cenário: Organizar materiais de diferentes formatos na mesma disciplina
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" no formato PDF
      E existe o material publicado "Aula de Cálculo I" no formato de link
      E ambos estão associados à disciplina "Cálculo I"
      Quando o estudante seleciona a disciplina "Cálculo I"
      Então o material "Resumo de Cálculo I" é apresentado
      E o material "Aula de Cálculo I" é apresentado
      E ambos ficam disponíveis na organização da disciplina

    Cenário: Não apresentar materiais que ainda não foram publicados
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" associado à disciplina "Cálculo I"
      E existe o material "Rascunho de Cálculo I" ainda não publicado
      Quando o estudante consulta os materiais da disciplina "Cálculo I"
      Então o material "Resumo de Cálculo I" é apresentado
      E o material "Rascunho de Cálculo I" não é apresentado

    Cenário: Atualizar os materiais ao selecionar outra disciplina
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" associado à disciplina "Cálculo I"
      E existe o material publicado "Resumo de Álgebra Linear" associado à disciplina "Álgebra Linear"
      E o estudante está consultando os materiais de "Cálculo I"
      Quando o estudante seleciona a disciplina "Álgebra Linear"
      Então o material "Resumo de Álgebra Linear" é apresentado
      E o material "Resumo de Cálculo I" não é apresentado na consulta atual

    Cenário: Retornar à lista de disciplinas e consultar outra disciplina
      Dado que o CA ativo é "CA de Computação"
      E existem materiais publicados associados às disciplinas "Cálculo I" e "Álgebra Linear"
      E o estudante está consultando os materiais de "Cálculo I"
      Quando o estudante retorna à lista de disciplinas
      E seleciona "Álgebra Linear"
      Então são apresentados os materiais associados à disciplina "Álgebra Linear"
      E os materiais de "Cálculo I" não são apresentados nessa consulta

    Cenário: Acessar um material a partir da organização por disciplina
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" associado à disciplina "Cálculo I"
      Quando o estudante seleciona a disciplina "Cálculo I"
      E seleciona o material "Resumo de Cálculo I"
      Então o material selecionado fica disponível para acesso
      E sua associação com a disciplina "Cálculo I" é mantida

    Cenário: Consultar uma disciplina sem materiais publicados
      Dado que o CA ativo é "CA de Computação"
      E a disciplina "Física I" pertence à grade curricular do contexto acadêmico
      E não existem materiais publicados associados à disciplina "Física I"
      Quando o estudante seleciona a disciplina "Física I"
      Então nenhum material é apresentado
      E o aplicativo informa que não existem materiais disponíveis para essa disciplina

    Cenário: Acessar a organização quando não existem disciplinas disponíveis
      Dado que o CA ativo é "CA de Computação"
      E não existem disciplinas cadastradas na grade curricular do contexto acadêmico
      Quando o estudante acessa a organização de materiais por disciplina
      Então nenhuma disciplina é apresentada para consulta
      E o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais

  Regra: Os materiais e as disciplinas devem respeitar o Centro Acadêmico ativo

    Cenário: Não misturar materiais de diferentes Centros Acadêmicos
      Dado que o CA ativo é "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" associado à disciplina "Cálculo I" nesse CA
      E existe o material publicado "Exercícios de Cálculo I" associado à disciplina "Cálculo I" no "CA de Matemática"
      Quando o estudante consulta os materiais da disciplina "Cálculo I"
      Então o material "Resumo de Cálculo I" é apresentado
      E o material "Exercícios de Cálculo I" não é apresentado

    Cenário: Atualizar a organização ao trocar o Centro Acadêmico ativo
      Dado que o estudante está no "CA de Computação"
      E existe o material publicado "Resumo de Cálculo I" associado à disciplina "Cálculo I" nesse CA
      E existe o material publicado "Exercícios de Cálculo I" associado à disciplina "Cálculo I" no "CA de Matemática"
      Quando o estudante altera o CA ativo para "CA de Matemática"
      E consulta os materiais da disciplina "Cálculo I"
      Então o material "Exercícios de Cálculo I" é apresentado
      E o material "Resumo de Cálculo I" não é apresentado
      E a organização corresponde ao contexto do "CA de Matemática"
