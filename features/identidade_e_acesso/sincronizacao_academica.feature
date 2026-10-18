# language: pt
# Referência: Relatório de Features do Projeto, seção 1.4.
# O MVP prevê atualização no acesso do estudante e sincronização periódica básica.
# Estratégias avançadas de retentativa e reconciliação ficam para evolução posterior.
@pending_backend
Funcionalidade: Sincronização acadêmica periódica
  Como estudante
  Quero que meus dados acadêmicos estejam sempre atualizados
  Para que minhas permissões permaneçam corretas

  Contexto:
    Dado que o estudante possui um perfil no CApp com dados acadêmicos previamente associados

  Regra: Dados acadêmicos modificados são atualizados

    @happy @backend
    Cenário: Atualizar os dados acadêmicos no acesso do estudante
      Dado que a fonte institucional contém dados acadêmicos mais recentes que os associados ao estudante
      Quando o estudante acessa o CApp
      Então os dados acadêmicos associados ao estudante refletem as informações atuais confirmadas pela fonte institucional
      E as informações próprias do perfil no CApp permanecem preservadas

    @happy @backend
    Cenário: Atualizar os dados acadêmicos pela rotina periódica
      Dado que a fonte institucional contém dados acadêmicos mais recentes que os associados ao estudante
      Quando ocorre a rotina periódica de sincronização acadêmica
      Então os dados acadêmicos associados ao estudante refletem as informações atuais confirmadas pela fonte institucional
      E as regras de acesso dependentes desses dados consideram as informações atualizadas

  Regra: Alterações acadêmicas refletem nas permissões do estudante

    @sad @backend
    Cenário: Recusar acesso após mudança para um curso que não atende aos critérios
      Dado que o curso atual do estudante atende aos critérios de acesso a uma funcionalidade acadêmica
      E a fonte institucional confirma a mudança para um curso que não atende aos critérios dessa funcionalidade
      Quando os dados acadêmicos do estudante são sincronizados
      E o estudante tenta acessar essa funcionalidade
      Então o acesso à funcionalidade é recusado conforme o curso atualizado

    @happy @backend
    Cenário: Permitir ação após alteração de vínculo que atende aos critérios
      Dado que a situação acadêmica atual do estudante não atende aos critérios de acesso a uma funcionalidade
      E a fonte institucional confirma uma alteração de vínculo que passa a atender aos critérios dessa funcionalidade
      Quando os dados acadêmicos do estudante são sincronizados
      E o estudante tenta acessar essa funcionalidade
      Então o acesso à funcionalidade é permitido conforme a situação acadêmica atualizada

    @sad @backend
    Cenário: Desligamento impede ações que exigem vínculo acadêmico ativo
      Dado que o estudante possui vínculo acadêmico ativo
      E existe uma ação que exige vínculo acadêmico ativo
      E a fonte institucional confirma o desligamento do estudante
      Quando seus dados acadêmicos são sincronizados
      E o estudante tenta realizar essa ação
      Então a situação acadêmica associada ao estudante reflete o desligamento confirmado
      E a ação é recusada por ausência de vínculo acadêmico ativo

  Regra: A indisponibilidade da fonte preserva os dados anteriormente confirmados

    @sad @backend
    Cenário: Falha temporária da fonte institucional durante a sincronização
      Dado que o perfil do estudante possui dados acadêmicos anteriormente confirmados
      E a fonte institucional está temporariamente indisponível
      Quando ocorre uma tentativa de sincronização acadêmica
      Então os últimos dados acadêmicos confirmados permanecem associados ao estudante
      E a falha de consulta não altera, por si só, as permissões derivadas desses dados

  Regra: A sincronização mantém a consistência dos dados acadêmicos

    @happy @backend
    Cenário: Dados acadêmicos sem alteração permanecem consistentes
      Dado que as informações acadêmicas atuais da fonte institucional correspondem às associadas ao estudante
      Quando ocorre uma sincronização acadêmica
      Então as informações acadêmicas associadas ao estudante continuam correspondendo à fonte institucional
      E as informações próprias do perfil no CApp permanecem preservadas
      E as permissões continuam coerentes com a situação acadêmica confirmada
