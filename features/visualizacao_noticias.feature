# language: pt

# Referência: Relatório de Features, seção 3.1.
# @v1 usa data decrescente; @v2 permite escolher a direção da ordenação.
@backend @mobile @pending_backend @pending_mobile
Funcionalidade: Visualização de notícias

  Como estudante
  Quero visualizar notícias do CA ativo
  Para acompanhar informações relevantes da comunidade acadêmica

  @v1
  Cenário: Exibir somente notícias publicadas do CA ativo
    Dado que o CA ativo é "CA de Computação"
    E existem as notícias publicadas "Semana Acadêmica" e "Edital de bolsas" nesse CA
    E existe a notícia "Rascunho de evento" em rascunho nesse CA
    E existe a notícia "Proposta de palestra" pendente de homologação nesse CA
    E existe a notícia "Comunicado rejeitado" rejeitada nesse CA
    E existe a notícia "Aviso arquivado" arquivada nesse CA
    Quando consulto a listagem de notícias
    Então vejo "Semana Acadêmica" e "Edital de bolsas"
    E não vejo notícias em rascunho, pendentes de homologação, rejeitadas ou arquivadas

  @v1
  Cenário: Informar quando não há notícias publicadas
    Dado que o CA ativo é "CA de Computação"
    E não existem notícias publicadas nesse CA
    Quando consulto a listagem de notícias
    Então a listagem informa que não há notícias publicadas

  @v1
  Cenário: Visualizar os detalhes de uma notícia publicada
    Dado que o CA ativo é "CA de Computação"
    E existe a notícia publicada "Mudança no calendário" nesse CA
    E o conteúdo da notícia é "O calendário acadêmico foi atualizado."
    Quando consulto os detalhes da notícia "Mudança no calendário"
    Então vejo o título "Mudança no calendário"
    E vejo o conteúdo "O calendário acadêmico foi atualizado."

  @v1
  Cenário: Ordenar notícias da mais recente para a mais antiga
    Dado que o CA ativo é "CA de Computação"
    E existem as seguintes notícias publicadas nesse CA:
      | título             | data de publicação |
      | Comunicado antigo  | 2026-10-10         |
      | Comunicado recente | 2026-10-15         |
      | Comunicado do meio | 2026-10-12         |
    Quando consulto a listagem de notícias
    Então as notícias aparecem nesta ordem:
      | título             |
      | Comunicado recente |
      | Comunicado do meio |
      | Comunicado antigo  |

  @v1
  Cenário: Exibir notícias somente do CA ativo
    Dado que o CA ativo é "CA de Computação"
    E existe a notícia publicada "Hackathon de Computação" nesse CA
    E existe a notícia publicada "Semana de Matemática" no CA "CA de Matemática"
    Quando consulto a listagem de notícias
    Então vejo "Hackathon de Computação"
    E não vejo "Semana de Matemática"
    Quando altero o CA ativo para "CA de Matemática"
    E consulto novamente a listagem de notícias
    Então vejo "Semana de Matemática"
    E não vejo "Hackathon de Computação"

  @v1
  Cenário: Não disponibilizar os detalhes de uma notícia não publicada
    Dado que o CA ativo é "CA de Computação"
    E existe a notícia "Proposta de palestra" pendente de homologação nesse CA
    Quando tento consultar os detalhes da notícia "Proposta de palestra"
    Então o conteúdo da notícia não é disponibilizado

  @v2
  Regra: O usuário escolhe entre notícias mais recentes ou mais antigas primeiro

    Cenário: Escolher a ordenação da mais antiga para a mais recente
      Dado que o CA ativo possui as notícias publicadas "Antiga" em "2026-10-10" e "Recente" em "2026-10-15"
      Quando escolho a ordenação "Mais antigas primeiro"
      Então a listagem apresenta "Antiga" antes de "Recente"
      E a ordenação não altera quais notícias estão publicadas

    Cenário: Voltar à ordenação da mais recente para a mais antiga
      Dado que o CA ativo possui as notícias publicadas "Antiga" em "2026-10-10" e "Recente" em "2026-10-15"
      E escolhi a ordenação "Mais antigas primeiro"
      Quando escolho a ordenação "Mais recentes primeiro"
      Então a listagem apresenta "Recente" antes de "Antiga"
      E somente notícias disponíveis no contexto ativo são apresentadas
