# language: pt

# Referência: Relatório de Features do Projeto, seção 1.3.
# A associação acadêmica contempla curso e situação acadêmica.
# As integrações e a persistência são verificadas nas implementações de cada componente.

@pending_backend @pending_mobile
Funcionalidade: Associação dos dados acadêmicos
  Como estudante
  Quero que meu curso e minha situação acadêmica sejam identificados automaticamente
  Para que o aplicativo disponibilize funcionalidades adequadas à minha situação

  Regra: Os dados acadêmicos necessários ao contexto do estudante são recuperados com sucesso

    @happy @backend @mobile
    Cenário: Recuperação e associação dos dados acadêmicos
      Dado que o estudante possui uma identidade institucional autenticada
      E a instituição disponibiliza o curso e a situação acadêmica do estudante
      Quando o sistema obtém os dados acadêmicos do estudante
      Então o curso do estudante fica associado ao seu perfil
      E a situação acadêmica do estudante fica associada ao seu perfil
      E essas informações ficam disponíveis para as regras de acesso do aplicativo

  Regra: Dados acadêmicos indisponíveis não podem ser associados como se tivessem sido recuperados

    @sad @backend @mobile
    Cenário: Instituição não disponibiliza os dados acadêmicos do estudante
      Dado que o estudante possui uma identidade institucional autenticada
      E a instituição não disponibiliza os dados acadêmicos necessários para identificar seu contexto
      Quando o sistema tenta obter os dados acadêmicos do estudante
      Então o sistema não associa dados acadêmicos que não foram recuperados
      E o perfil não apresenta esses dados como se estivessem confirmados
      E as regras de acesso não tratam os dados ausentes como informações acadêmicas confirmadas

  Regra: Dados acadêmicos incompletos não devem ser considerados plenamente associados

    @sad @backend @mobile
    Esquema do Cenário: Ausência de uma informação acadêmica necessária
      Dado que o estudante possui uma identidade institucional autenticada
      E a instituição não disponibiliza a informação "<informacao>" do estudante
      Quando o sistema tenta associar os dados acadêmicos ao perfil
      Então a informação "<informacao>" não é apresentada como confirmada
      E o sistema não considera completo o contexto acadêmico do estudante

      Exemplos:
        | informacao          |
        | curso               |
        | situação acadêmica  |

  Regra: Uma falha temporária na obtenção dos dados não confirma informações acadêmicas

    @sad @backend @mobile
    Cenário: Falha temporária ao consultar os dados acadêmicos
      Dado que o estudante possui uma identidade institucional autenticada
      E não foi possível obter os dados acadêmicos da instituição
      Quando o sistema tenta concluir a associação dos dados acadêmicos
      Então o sistema não confirma uma associação que não foi concluída
      E as informações acadêmicas não obtidas permanecem indisponíveis para as regras de acesso