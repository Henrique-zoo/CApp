# language: pt

# Referência: Relatório de Features do Projeto, seções 11.1 e 11.3.
# Registros e permissões permanecem restritos ao CA responsável.
@pending_backend @pending_mobile
Funcionalidade: Registro de ocorrências

  Como membro autorizado da gestão do CA
  Quero registrar ocorrências relacionadas a estudantes ou ao próprio CA
  Para documentar situações relevantes no contexto da comunidade

  Regra: O registro individual exige permissão específica no CA responsável

    Cenário: Registrar ocorrência associada a um estudante
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Gestor A"
      E tenho permissão para registrar ocorrências individuais nesse CA
      E o estudante "Estudante A" pode ser associado a uma ocorrência nesse CA
      Quando registro uma ocorrência individual para "Estudante A" com motivo "Descumprimento de regra interna" e descrição "A situação observada foi relatada à gestão"
      Então a ocorrência é registrada no "CA de Computação"
      E a ocorrência fica associada ao estudante "Estudante A"
      E o registro contém motivo, descrição, data e situação da ocorrência
      E "Gestor A" é identificado como responsável pelo registro

    Cenário: Recusar registro individual sem a permissão específica
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Membro A"
      E meu perfil não tem permissão para registrar ocorrências individuais nesse CA
      Quando tento registrar uma ocorrência individual para "Estudante A"
      Então a ocorrência não é criada
      E o aplicativo informa que não tenho permissão para esse tipo de registro

  Regra: O registro de ocorrência contra o próprio CA usa permissão independente

    Cenário: Registrar ocorrência cujo alvo é o próprio CA
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Gestor B"
      E tenho permissão para registrar ocorrências do próprio CA nesse CA
      Quando registro uma ocorrência contra o próprio CA com motivo "Problema de limpeza" e descrição "Foi identificada uma necessidade de limpeza na área comum"
      Então a ocorrência é registrada no "CA de Computação"
      E o próprio CA é identificado como alvo da ocorrência
      E a ocorrência não exige associação a um estudante
      E o registro contém motivo, descrição, data e situação da ocorrência
      E "Gestor B" é identificado como responsável pelo registro

    Cenário: Permissão para registrar ocorrências individuais não autoriza registro contra o CA
      Dado que o CA ativo é "CA de Computação"
      E meu perfil tem permissão para registrar ocorrências individuais nesse CA
      E meu perfil não tem permissão para registrar ocorrências do próprio CA nesse CA
      Quando tento registrar uma ocorrência contra o próprio CA
      Então a ocorrência não é criada
      E o aplicativo informa que não tenho permissão para esse tipo de registro

    Cenário: Permissão para registrar ocorrências contra o CA não autoriza registros individuais
      Dado que o CA ativo é "CA de Computação"
      E meu perfil tem permissão para registrar ocorrências do próprio CA nesse CA
      E meu perfil não tem permissão para registrar ocorrências individuais nesse CA
      Quando tento registrar uma ocorrência individual para "Estudante A"
      Então a ocorrência não é criada
      E o aplicativo informa que não tenho permissão para esse tipo de registro

  Regra: O registro e a autorização permanecem limitados ao CA responsável

    Cenário: Não utilizar permissão concedida em outro CA
      Dado que o CA ativo é "CA de Matemática"
      E meu perfil tem permissão para registrar ocorrências individuais no "CA de Computação"
      E meu perfil não tem essa permissão no "CA de Matemática"
      Quando tento registrar uma ocorrência individual para "Estudante B"
      Então a ocorrência não é criada
      E o aplicativo informa que não tenho permissão para registrar ocorrências nesse CA

    Cenário: Recusar registro sem motivo obrigatório
      Dado que o CA ativo é "CA de Computação"
      E tenho permissão para registrar ocorrências individuais nesse CA
      E iniciei o registro de uma ocorrência para "Estudante A"
      E preenchi a descrição da ocorrência
      Quando tento salvar a ocorrência sem informar o motivo
      Então a ocorrência não é criada
      E o aplicativo informa que o motivo é obrigatório

    Cenário: Recusar registro sem descrição obrigatória
      Dado que o CA ativo é "CA de Computação"
      E tenho permissão para registrar ocorrências individuais nesse CA
      E iniciei o registro de uma ocorrência para "Estudante A"
      E preenchi o motivo da ocorrência
      Quando tento salvar a ocorrência sem informar a descrição
      Então a ocorrência não é criada
      E o aplicativo informa que a descrição é obrigatória

    Cenário: Não impor banimento global ao registrar ocorrência
      Dado que o CA ativo é "CA de Computação"
      E tenho permissão para registrar ocorrências individuais nesse CA
      E o estudante "Estudante A" participa de outros CAs
      Quando registro uma ocorrência individual para "Estudante A" nesse CA
      Então a ocorrência fica vinculada ao "CA de Computação"
      E o registro não impõe banimento global ao estudante

    Cenário: Preservar o registro ao encerrar uma ocorrência
      Dado que existe uma ocorrência registrada no "CA de Computação"
      Quando a ocorrência é encerrada
      Então o registro da ocorrência é preservado
      E o encerramento não remove o registro histórico
