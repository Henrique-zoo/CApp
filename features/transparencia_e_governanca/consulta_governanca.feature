# language: pt

# Issue: #211 - Especificar cenários BDD de consulta pelos estudantes.
#
# Referências:
# - Relatório de Features: seção 2.1 - CA favorito e contexto ativo.
# - Relatório de Features: seção 2.5 - Vinculação de usuários a perfis administrativos.
# - Relatório de Features: seção 2.6 - Desvinculação de integrantes da gestão.
# - Relatório de Features: seção 2.8 - Exclusão de perfis administrativos.
# - Relatório de Features: seção 10.1 - Configuração de mandato da gestão.
# - Relatório de Features: seção 10.3 - Histórico de gestões.
#
# Escopo:
# Especificar a consulta estudantil da gestão vigente e das gestões anteriores,
# incluindo mandatos, integrantes e cargos registrados desde a adesão do CA ao CApp.
#
# As operações de persistência e consulta no backend e a apresentação no mobile
# serão implementadas e automatizadas nas respectivas issues funcionais.
#
# @backend: cenário destinado à validação do comportamento no backend.
# @mobile: cenário destinado à validação do comportamento no aplicativo.
# @pending_backend: automação do backend ainda pendente.
# @pending_mobile: automação do mobile ainda pendente.

@backend @mobile @pending_backend @pending_mobile
Funcionalidade: Consulta da gestão vigente e do histórico de gestões do Centro Acadêmico

  Como estudante
  Quero consultar as informações da gestão vigente e das gestões anteriores
  Para conhecer a organização, a composição e a trajetória do Centro Acadêmico

  Contexto:
    Dado que o "CA de Computação" está cadastrado no CApp
    E o "CA de Computação" é o Centro Acadêmico ativo
    E estou autenticado no CApp com uma conta de estudante

  # ---------------------------------------------------------------------------
  # 1. Consulta pública da gestão vigente
  # ---------------------------------------------------------------------------

  Regra: Estudantes podem consultar as informações da gestão vigente sem permissão administrativa

    Cenário: Estudante consulta a composição da gestão vigente
      Dado que não possuo cargo nem permissão administrativa no "CA de Computação"
      E existe uma gestão vigente registrada nesse Centro Acadêmico
      E o mandato dessa gestão começa em "2026-01-01" e termina em "2026-12-31"
      E a gestão possui "Ana" no cargo de "Presidente"
      E a gestão possui "Bruno" no cargo de "Vice-Presidente"
      E a gestão possui "Carla" no cargo de "Tesoureira"
      Quando consulto as informações de governança do Centro Acadêmico ativo
      Então consigo visualizar a gestão vigente
      E consigo visualizar as datas de início e término do mandato
      E consigo identificar "Ana" como "Presidente"
      E consigo identificar "Bruno" como "Vice-Presidente"
      E consigo identificar "Carla" como "Tesoureira"

    Cenário: Estudante que não integra a gestão consulta suas informações
      Dado que não faço parte da gestão do "CA de Computação"
      E não possuo permissões administrativas nesse Centro Acadêmico
      E existe uma gestão vigente registrada
      Quando acesso a consulta de governança do Centro Acadêmico ativo
      Então a consulta é disponibilizada normalmente
      E consigo visualizar os integrantes da gestão vigente
      E consigo visualizar os cargos ocupados por cada integrante
      E a ausência de permissão administrativa não impede a consulta

    Cenário: Consultar o Centro Acadêmico que ainda não possui gestão vigente registrada
      Dado que o "CA de Computação" não possui uma gestão vigente registrada no CApp
      Quando consulto as informações de governança do Centro Acadêmico ativo
      Então o aplicativo informa que não existe gestão vigente registrada
      E não apresenta integrantes como membros de uma gestão vigente
      E não apresenta cargos atuais sem registro correspondente

  # ---------------------------------------------------------------------------
  # 2. Consulta do histórico de gestões
  # ---------------------------------------------------------------------------

  Regra: O histórico disponibiliza as gestões registradas no CApp desde a adesão do CA

    Cenário: Estudante consulta as gestões anteriores registradas no aplicativo
      Dado que o "CA de Computação" aderiu ao CApp em "2024-01-01"
      E existe uma gestão registrada com mandato de "2024-01-01" a "2024-12-31"
      E existe uma gestão registrada com mandato de "2025-01-01" a "2025-12-31"
      Quando consulto o histórico de gestões do Centro Acadêmico ativo
      Então encontro a gestão com mandato de "2024-01-01" a "2024-12-31"
      E encontro a gestão com mandato de "2025-01-01" a "2025-12-31"
      E consigo identificar o período de cada gestão registrada

    Cenário: Estudante consulta o histórico sem possuir permissão administrativa
      Dado que não possuo cargo nem permissão administrativa no "CA de Computação"
      E existem gestões anteriores registradas no CApp
      Quando consulto o histórico de gestões do Centro Acadêmico ativo
      Então consigo visualizar as gestões registradas
      E consigo identificar os respectivos períodos de mandato
      E a consulta não exige uma permissão administrativa

    Cenário: Estudante consulta os integrantes e cargos de uma gestão encerrada
      Dado que existe uma gestão encerrada com mandato de "2025-01-01" a "2025-12-31"
      E "Ana" integrou essa gestão no cargo de "Presidente"
      E "Bruno" integrou essa gestão no cargo de "Tesoureiro"
      E "Carla" integrou essa gestão no cargo de "Secretária"
      Quando consulto os detalhes da gestão com mandato de "2025-01-01" a "2025-12-31"
      Então visualizo o período de mandato dessa gestão
      E visualizo "Ana" como antiga "Presidente"
      E visualizo "Bruno" como antigo "Tesoureiro"
      E visualizo "Carla" como antiga "Secretária"
      E consigo identificar a participação histórica de cada integrante

    Cenário: Consultar o histórico quando o Centro Acadêmico ainda não possui gestões registradas
      Dado que o "CA de Computação" aderiu ao CApp
      E nenhuma gestão foi registrada nesse Centro Acadêmico desde sua adesão
      Quando consulto o histórico de gestões do Centro Acadêmico ativo
      Então o aplicativo informa que não existem gestões anteriores registradas
      E não apresenta integrantes sem participação histórica registrada
      E não apresenta cargos sem associação com uma gestão registrada

  # ---------------------------------------------------------------------------
  # 3. Preservação do histórico de integrantes e cargos
  # ---------------------------------------------------------------------------

  Regra: A desvinculação de um integrante não apaga sua participação histórica na gestão

    Cenário: Preservar a participação de um integrante após sua desvinculação
      Dado que existe uma gestão vigente com mandato de "2026-01-01" a "2026-12-31"
      E "Ana" ocupou o cargo de "Tesoureira" nessa gestão
      E o vínculo ativo de "Ana" com esse cargo foi encerrado
      Quando consulto a composição vigente e o histórico de participação dessa gestão
      Então "Ana" não é apresentada entre os integrantes atualmente vinculados à gestão
      E "Ana" continua registrada no histórico de participação dessa gestão
      E o cargo de "Tesoureira" continua associado à participação histórica de "Ana"

  Regra: A exclusão permitida de um cargo não apaga sua identificação nos registros históricos

    Cenário: Preservar um cargo personalizado após sua exclusão da estrutura atual
      Dado que "Bruno" ocupou o cargo personalizado de "Diretoria de Comunicação"
      E esse cargo fez parte da gestão com mandato de "2025-01-01" a "2025-12-31"
      E a exclusão desse cargo da estrutura atual do CA foi permitida
      Quando consulto a composição histórica da gestão de "2025"
      Então "Bruno" continua apresentado como integrante daquela gestão
      E o cargo "Diretoria de Comunicação" continua associado à participação histórica de "Bruno"
      E a exclusão do cargo da estrutura atual não apaga seu registro histórico

  Regra: A transição entre gestões preserva os registros das administrações anteriores

    Cenário: Consultar a nova gestão sem perder a composição da gestão anterior
      Dado que existe uma gestão encerrada com mandato de "2025-01-01" a "2025-12-31"
      E "Ana" integrou a gestão de "2025" como "Presidente"
      E "Bruno" integrou a gestão de "2025" como "Tesoureiro"
      E existe uma nova gestão vigente com mandato de "2026-01-01" a "2026-12-31"
      E "Carla" integra a gestão de "2026" como "Presidente"
      Quando consulto as informações de governança do Centro Acadêmico ativo
      Então a gestão de "2026" é apresentada como a gestão vigente
      E a gestão de "2025" continua disponível no histórico
      E "Ana" continua associada ao cargo de "Presidente" na gestão de "2025"
      E "Bruno" continua associado ao cargo de "Tesoureiro" na gestão de "2025"
      E a composição da gestão anterior não é substituída pela nova composição

  # ---------------------------------------------------------------------------
  # 4. Isolamento das informações conforme o Centro Acadêmico ativo
  # ---------------------------------------------------------------------------

  Regra: A consulta apresenta somente as informações do Centro Acadêmico ativo

    Cenário: Consultar a gestão do CA ativo sem apresentar dados de outros CAs
      Dado que existe uma gestão vigente no "CA de Computação"
      E "Ana" integra a gestão do "CA de Computação"
      E existe uma gestão vigente no "CA de Matemática"
      E "Bruno" integra a gestão do "CA de Matemática"
      E o "CA de Computação" está selecionado como Centro Acadêmico ativo
      Quando consulto as informações de governança
      Então visualizo a gestão do "CA de Computação"
      E visualizo "Ana" como integrante da gestão do CA ativo
      E não visualizo "Bruno" como integrante da gestão do CA ativo
      E não são apresentadas informações exclusivas do "CA de Matemática"

    Cenário: Atualizar as informações exibidas após a troca do CA ativo
      Dado que existe uma gestão vigente no "CA de Computação"
      E "Ana" integra a gestão do "CA de Computação"
      E existe uma gestão vigente no "CA de Matemática"
      E "Bruno" integra a gestão do "CA de Matemática"
      E o "CA de Computação" é o Centro Acadêmico ativo
      Quando altero o Centro Acadêmico ativo para "CA de Matemática"
      E consulto novamente as informações de governança
      Então visualizo a gestão do "CA de Matemática"
      E visualizo "Bruno" como integrante da gestão do CA ativo
      E não visualizo "Ana" como integrante da gestão do CA ativo
      E a consulta utiliza o contexto do Centro Acadêmico selecionado

    Cenário: Utilizar o CA favorito como contexto inicial da consulta
      Dado que o "CA de Computação" é o CA favorito da minha conta
      E o "CA de Computação" é selecionado como contexto inicial
      E existe uma gestão vigente nesse Centro Acadêmico
      Quando acesso as informações de governança sem alterar o contexto ativo
      Então visualizo as informações do "CA de Computação"
      E os dados apresentados pertencem ao CA favorito selecionado como contexto inicial

    Cenário: Não apresentar o histórico de outro CA quando o CA ativo não possui registros
      Dado que o "CA de Computação" não possui gestões registradas no CApp
      E o "CA de Matemática" possui gestões registradas no CApp
      E o "CA de Computação" está selecionado como Centro Acadêmico ativo
      Quando consulto o histórico de gestões
      Então o aplicativo informa que não existem gestões registradas para o "CA de Computação"
      E não apresenta as gestões do "CA de Matemática"
      E não utiliza registros de outro Centro Acadêmico para preencher o histórico do CA ativo
