# language: pt
@pending_mobile
Funcionalidade: Criação e publicação de notícias
  Como usuário de um Centro Acadêmico
  Quero criar notícias como solicitação e publicar as que um administrador validar
  Para que elas só apareçam aos usuários depois dessa validação

  Regra: Criação de notícia como solicitação
    Como usuário de um Centro Acadêmico
    Quero criar uma notícia
    Para registrá-la como solicitação, comigo como autor, até um administrador validá-la

    @happy @backend @mobile
    Cenário: Qualquer usuário cria uma notícia no seu Centro Acadêmico
      Dado que o usuário faz parte do Centro Acadêmico "CA de Computação"
      Quando ele cria uma notícia com o título "Assembleia geral" e o conteúdo "Pauta da próxima assembleia"
      Então a notícia fica registrada no Centro Acadêmico "CA de Computação" como solicitação
      E o usuário fica vinculado como autor da notícia
      E a data de criação da notícia fica registrada
      E a notícia fica pendente de validação por um administrador
      E a notícia não aparece para os usuários

    @happy @backend @mobile
    Cenário: Notícia recém-criada não aparece antes da validação
      Dado que o usuário faz parte do Centro Acadêmico "CA de Computação"
      Quando ele cria uma notícia com o título "Assembleia geral" e o conteúdo "Pauta da próxima assembleia"
      Então a notícia não aparece para os usuários do Centro Acadêmico "CA de Computação"
      E a notícia permanece pendente de validação por um administrador
      E a notícia não possui data de publicação

  Regra: Preenchimento das informações necessárias
    Como usuário de um Centro Acadêmico
    Quero informar título e conteúdo
    Para que a solicitação só seja aceita com essas informações

    @happy @backend @mobile
    Cenário: Notícia com título e conteúdo é aceita
      Dado que o usuário faz parte do seu Centro Acadêmico
      Quando ele cria uma notícia com o título "Assembleia geral" e o conteúdo "Pauta da próxima assembleia"
      Então o registro da notícia é aceito
      E a notícia fica registrada com o título "Assembleia geral" e o conteúdo "Pauta da próxima assembleia"
      E a notícia fica pendente de validação por um administrador

    @sad @backend @mobile
    Esquema do Cenário: Notícia sem título ou sem conteúdo não é registrada
      Dado que o usuário faz parte do seu Centro Acadêmico
      Quando ele tenta criar uma notícia com o título "<titulo>" e o conteúdo "<conteudo>"
      Então o registro é recusado
      E nenhuma notícia é registrada

  Regra: Edição de uma notícia
    Como usuário autorizado de um Centro Acadêmico
    Quero editar uma notícia desse Centro Acadêmico, já publicada ou ainda não
    Para corrigir título e conteúdo sem mudar a situação de publicação nessa ação

    @happy @backend @mobile
    Cenário: Usuário autorizado corrige notícia ainda não publicada
      Dado que o usuário está autorizado a editar notícias no Centro Acadêmico "CA de Computação"
      E nesse Centro Acadêmico existe uma notícia não publicada com o título "Assembleia" e o conteúdo "Pauta antiga"
      Quando ele altera o título para "Assembleia geral" e o conteúdo para "Pauta atualizada"
      Então a notícia fica registrada com o título "Assembleia geral" e o conteúdo "Pauta atualizada"
      E a notícia continua fora da listagem publicada
      E a notícia não possui data de publicação

    @happy @backend @mobile
    Cenário: Usuário autorizado corrige notícia já publicada
      Dado que o usuário está autorizado a editar notícias no Centro Acadêmico "CA de Computação"
      E nesse Centro Acadêmico existe uma notícia já publicada com o título "Assembleia" e o conteúdo "Pauta antiga"
      Quando ele altera o título para "Assembleia geral" e o conteúdo para "Pauta atualizada"
      Então a notícia fica registrada com o título "Assembleia geral" e o conteúdo "Pauta atualizada"
      E a notícia continua na listagem publicada
      E a data de publicação da notícia permanece registrada

    @sad @backend @mobile
    Esquema do Cenário: Edição que apaga título ou conteúdo é recusada
      Dado que o usuário está autorizado a editar notícias no Centro Acadêmico "CA de Computação"
      E nesse Centro Acadêmico existe uma notícia não publicada com o título "<titulo_anterior>" e o conteúdo "<conteudo_anterior>"
      Quando ele tenta alterar a notícia para o título "<titulo>" e o conteúdo "<conteudo>"
      Então a alteração é recusada
      E a notícia permanece com o título "<titulo_anterior>" e o conteúdo "<conteudo_anterior>"
      E a notícia continua fora da listagem publicada

    @sad @backend @mobile
    Cenário: Usuário autorizado não edita notícia de outro Centro Acadêmico
      Dado que o usuário está autorizado a editar notícias no Centro Acadêmico "CA de Computação"
      E o Centro Acadêmico "CA de História" possui uma notícia não publicada com o título "Visita técnica" e o conteúdo "Roteiro da visita"
      Quando ele tenta alterar essa notícia para o título "Visita cancelada" e o conteúdo "A visita foi cancelada"
      Então a edição é recusada
      E a notícia permanece com o título "Visita técnica" e o conteúdo "Roteiro da visita"

  Regra: Publicação de uma notícia
    Como usuário com publicação direta em um Centro Acadêmico
    Quero publicar uma notícia desse Centro Acadêmico
    Para incluí-la na listagem publicada sem proposta e sem homologação

    @happy @backend @mobile
    Cenário: Publicação direta autorizada inclui a notícia na listagem
      Dado que o usuário possui permissão de publicação direta no Centro Acadêmico "CA de Computação"
      E nesse Centro Acadêmico existe uma notícia não publicada com o título "Calendário acadêmico" e o conteúdo "Datas do semestre"
      Quando ele publica essa notícia diretamente
      Então a notícia entra na listagem publicada do Centro Acadêmico "CA de Computação"
      E a notícia não fica pendente de homologação
      E a data de publicação da notícia fica registrada

    @sad @backend @mobile
    Cenário: Notícia pendente de homologação fica fora da listagem publicada
      Dado que o Centro Acadêmico "CA de Computação" possui uma notícia pendente de homologação
      Quando a listagem de notícias publicadas desse Centro Acadêmico é formada
      Então essa notícia não integra a listagem publicada
      E a notícia permanece pendente de homologação

    @sad @backend @mobile
    Cenário: Notícia rejeitada fica fora da listagem publicada
      Dado que o Centro Acadêmico "CA de Computação" possui uma notícia rejeitada
      Quando a listagem de notícias publicadas desse Centro Acadêmico é formada
      Então essa notícia não integra a listagem publicada
      E a notícia permanece rejeitada

    @sad @backend @mobile
    Esquema do Cenário: Publicação sem título ou sem conteúdo é recusada
      Dado que o usuário possui permissão de publicação direta no Centro Acadêmico "CA de Computação"
      E nesse Centro Acadêmico existe uma notícia não publicada com o título "<titulo>" e o conteúdo "<conteudo>"
      Quando ele tenta publicar essa notícia diretamente
      Então a publicação é recusada
      E a notícia não entra na listagem publicada


  Regra: Restrições de usuários sem permissão
    Como gestor de um Centro Acadêmico
    Quero restringir editar, publicar diretamente e homologar a quem tem a permissão correspondente
    Para impedir essas ações quando a permissão não existe

    @sad @backend @mobile
    Cenário: Usuário sem permissão não altera notícia
      Dado que o usuário não possui permissão para editar notícias no Centro Acadêmico "CA de Computação"
      E nesse Centro Acadêmico existe uma notícia não publicada com o título "Assembleia" e o conteúdo "Pauta original"
      Quando ele tenta alterar o título para "Outro título" e o conteúdo para "Outro texto"
      Então a edição é recusada
      E a notícia permanece com o título "Assembleia" e o conteúdo "Pauta original"

    @sad @backend @mobile
    Cenário: Gestor sem publicação direta não publica a notícia diretamente
      Dado que o gestor do Centro Acadêmico "CA de Computação" não possui permissão de publicação direta
      E nesse Centro Acadêmico existe uma notícia não publicada com o título "Comunicado" e o conteúdo "Texto do comunicado"
      Quando ele tenta publicar essa notícia diretamente
      Então a publicação direta é recusada
      E a notícia não entra na listagem publicada
