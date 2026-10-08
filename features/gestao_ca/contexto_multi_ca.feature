# language: pt

Funcionalidade: Contexto Multi-CA e navegação entre Centros Acadêmicos

  Como estudante da Universidade de Brasília (UnB)
  Quero definir meu Centro Acadêmico favorito e alternar entre diferentes CAs
  Para acompanhar informações e serviços estudantis com isolamento estrito de permissões

  # Regras de Negócio Fundamentais:
  # 1. Preferência vs Vínculo - O CA favorito é estritamente uma preferência de navegação inicial,
  #    sem conferir vínculo de gestão, representatividade institucional ou privilégio administrativo.
  # 2. Mudança de Contexto Ativo - O estudante pode alternar e consultar livremente qualquer CA ativo da UnB.
  # 3. Isolamento de Permissões - Alternar o contexto ativo para outro CA nunca concede permissões
  #    administrativas desse novo CA.
  # 4. Cargos Distintos por CA - Os privilégios de diretoria e gestão pertencem estritamente ao CA
  #    no qual foram atribuídos; ao navegar por outro CA, o estudante atua estritamente como visitante.

  Contexto:
    Dado que existem os seguintes Centros Acadêmicos cadastrados na UnB:
      | nome                                            | sigla |
      | Centro Acadêmico de Engenharia de Redes (CAER)  | CAER  |
      | Centro Acadêmico de Ciência da Computação (CACC) | CACC  |

  Cenário: Definição e abertura do aplicativo no CA favorito do estudante
    Dado que o estudante "242003979@aluno.unb.br" possui "Centro Acadêmico de Engenharia de Redes (CAER)" definido como seu CA favorito
    Quando o estudante abre o aplicativo CApp
    Então o contexto ativo de navegação deve ser "Centro Acadêmico de Engenharia de Redes (CAER)"
    E o estudante visualiza as informações correspondentes a "Centro Acadêmico de Engenharia de Redes (CAER)"
    E o estudante não possui privilégios administrativos no CA ativo apenas por tê-lo como favorito

  Cenário: Alteração do contexto ativo para outro Centro Acadêmico disponível
    Dado que o estudante "242003979@aluno.unb.br" possui "Centro Acadêmico de Engenharia de Redes (CAER)" definido como seu CA favorito
    E está autenticado no aplicativo
    E está com o contexto ativo em "Centro Acadêmico de Engenharia de Redes (CAER)"
    Quando o estudante altera o contexto ativo para "Centro Acadêmico de Ciência da Computação (CACC)"
    Então o contexto ativo de navegação deve ser "Centro Acadêmico de Ciência da Computação (CACC)"
    E o estudante visualiza as informações correspondentes a "Centro Acadêmico de Ciência da Computação (CACC)"
    E a preferência de CA favorito do estudante permanece "Centro Acadêmico de Engenharia de Redes (CAER)"

  Cenário: Isolamento de permissões onde a mudança de contexto não concede privilégios administrativos
    Dado que o estudante "242003979@aluno.unb.br" exerce o cargo de "Diretor de Comunicação" em "Centro Acadêmico de Engenharia de Redes (CAER)"
    E o cargo possui a permissão de "Publicar notícias diretamente"
    E o estudante não possui cargo de gestão em "Centro Acadêmico de Ciência da Computação (CACC)"
    Quando o estudante está com o contexto ativo em "Centro Acadêmico de Engenharia de Redes (CAER)"
    Então o estudante possui permissão para "Publicar notícias diretamente" no CA ativo
    Quando o estudante altera o contexto ativo para "Centro Acadêmico de Ciência da Computação (CACC)"
    Então o estudante não possui permissão para "Publicar notícias diretamente" no CA ativo
    E atua estritamente como visitante no novo contexto

  Cenário: Manutenção de cargos distintos e isolados por CA
    Dado que o estudante "231012345@aluno.unb.br" exerce o cargo de "Presidente" em "Centro Acadêmico de Engenharia de Redes (CAER)"
    E o cargo possui a permissão de "Administrar integrantes da gestão"
    E o estudante exerce o cargo de "Colaborador de Eventos" em "Centro Acadêmico de Ciência da Computação (CACC)"
    E o cargo possui a permissão de "Publicar e gerenciar eventos diretamente"
    Quando o estudante está com o contexto ativo em "Centro Acadêmico de Engenharia de Redes (CAER)"
    Então o estudante possui permissão para "Administrar integrantes da gestão" no CA ativo
    E não possui permissão para "Publicar e gerenciar eventos diretamente" no CA ativo
    Quando o estudante altera o contexto ativo para "Centro Acadêmico de Ciência da Computação (CACC)"
    Então o estudante possui permissão para "Publicar e gerenciar eventos diretamente" no CA ativo
    E não possui permissão para "Administrar integrantes da gestão" no CA ativo
