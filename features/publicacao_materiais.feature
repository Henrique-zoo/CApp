# language: pt

# Referência: Relatório de Features do Projeto, seção 6.2.
# O MVP contempla a publicação de links externos e arquivos PDF por integrantes autorizados.
@backend @mobile
Funcionalidade: Publicação de materiais acadêmicos

  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero disponibilizar materiais acadêmicos
  Para compartilhar conteúdos de apoio aos estudantes

  @pending_backend @pending_mobile
  Regra: A gestão autorizada pode publicar materiais por meio de links externos

    Cenário: Publicar material com link externo válido
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante publica o material "Referências de Cálculo I" com o link externo "https://exemplo.edu.br/calculo"
      Então a publicação do material é concluída
      E o endereço do link externo fica associado ao material publicado

    Cenário: Recusar publicação de material com link externo inválido
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante tenta publicar o material "Referências de Cálculo I" com um link externo inválido
      Então a publicação do material é recusada
      E o material não fica disponível como publicado

    Cenário: Impedir publicação de link externo por integrante sem permissão
      Dado que o CA ativo é "CA de Computação"
      E o integrante não possui a permissão "Gerenciar materiais" nesse CA
      Quando o integrante tenta publicar o material "Referências de Cálculo I" com o link externo "https://exemplo.edu.br/calculo"
      Então a publicação do material é recusada por falta de permissão
      E o material não fica disponível como publicado

    Cenário: Impedir publicação quando a permissão pertence a outro CA
      Dado que o CA ativo é "CA de Computação"
      E o integrante possui a permissão "Gerenciar materiais" somente em outro CA
      Quando o integrante tenta publicar o material "Referências de Cálculo I" com o link externo "https://exemplo.edu.br/calculo"
      Então a publicação do material é recusada por falta de permissão
      E o material não fica disponível como publicado

  @pending_backend @pending_mobile
  Regra: A gestão autorizada pode publicar materiais por meio de arquivos

    Cenário: Publicar material com arquivo PDF válido
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante publica o material "Resumo de Estruturas de Dados" com um arquivo PDF válido
      Então a publicação do material é concluída
      E o arquivo PDF fica associado ao material publicado

    Cenário: Recusar arquivo em formato não aceito
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante tenta publicar o material "Resumo de Estruturas de Dados" com um arquivo em formato não aceito
      Então a publicação do material é recusada
      E o material não fica disponível como publicado

    Cenário: Recusar arquivo acima do limite de tamanho permitido
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante tenta publicar o material "Resumo de Estruturas de Dados" com um arquivo acima do limite de tamanho permitido
      Então a publicação do material é recusada
      E o material não fica disponível como publicado

    Cenário: Aceitar arquivo no limite de tamanho permitido
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante publica o material "Resumo de Estruturas de Dados" com um arquivo no limite de tamanho permitido
      Então a publicação do material é concluída
      E o arquivo fica associado ao material publicado

    Cenário: Não publicar material quando ocorre falha no envio do arquivo
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      Quando o integrante tenta publicar o material "Resumo de Estruturas de Dados" com um arquivo PDF válido, mas ocorre uma falha durante o envio
      Então a publicação do material não é concluída
      E o material não fica disponível como publicado

    Cenário: Permitir nova tentativa após falha no envio do arquivo
      Dado que o CA ativo é "CA de Computação"
      E que o integrante possui a permissão "Gerenciar materiais" no CA ativo
      E que uma tentativa anterior de envio do arquivo do material "Resumo de Estruturas de Dados" falhou
      Quando o integrante realiza uma nova tentativa com um arquivo PDF válido
      Então a publicação do material é concluída
      E o arquivo PDF fica associado ao material publicado

    Cenário: Impedir publicação de arquivo por integrante sem permissão
      Dado que o CA ativo é "CA de Computação"
      E o integrante não possui a permissão "Gerenciar materiais" nesse CA
      Quando o integrante tenta publicar o material "Resumo de Estruturas de Dados" com um arquivo PDF válido
      Então a publicação do material é recusada por falta de permissão
      E o material não fica disponível como publicado

    Cenário: Impedir publicação de arquivo quando a permissão pertence a outro CA
      Dado que o CA ativo é "CA de Computação"
      E o integrante possui a permissão "Gerenciar materiais" somente em outro CA
      Quando o integrante tenta publicar o material "Resumo de Estruturas de Dados" com um arquivo PDF válido
      Então a publicação do material é recusada por falta de permissão
      E o material não fica disponível como publicado
