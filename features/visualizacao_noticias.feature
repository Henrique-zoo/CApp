# language: pt

# Regra de ordenação proposta para confirmação: data de publicação decrescente.
@pending_backend @pending_mobile
Funcionalidade: Visualização de notícias

  Como estudante
  Quero visualizar notícias do CA ativo
  Para acompanhar informações relevantes da comunidade acadêmica

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

  Cenário: Informar quando não há notícias publicadas
    Dado que o CA ativo é "CA de Computação"
    E não existem notícias publicadas nesse CA
    Quando consulto a listagem de notícias
    Então a listagem informa que não há notícias publicadas

  Cenário: Visualizar os detalhes de uma notícia publicada
    Dado que o CA ativo é "CA de Computação"
    E existe a notícia publicada "Mudança no calendário" nesse CA
    E o conteúdo da notícia é "O calendário acadêmico foi atualizado."
    Quando consulto os detalhes da notícia "Mudança no calendário"
    Então vejo o título "Mudança no calendário"
    E vejo o conteúdo "O calendário acadêmico foi atualizado."

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

  Cenário: Não disponibilizar os detalhes de uma notícia não publicada
    Dado que o CA ativo é "CA de Computação"
    E existe a notícia "Proposta de palestra" pendente de homologação nesse CA
    Quando tento consultar os detalhes da notícia "Proposta de palestra"
    Então o conteúdo da notícia não é disponibilizado
