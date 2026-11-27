# language: pt

# Issue #107 e Relatório de Features do Projeto, seção 6.2.
# A publicação respeita as permissões por CA da #52 e o vínculo com a disciplina da #118.
@backend @mobile @isolated_database @material_publication
Funcionalidade: Publicação de materiais acadêmicos

  Como integrante autorizado da gestão de um Centro Acadêmico
  Quero disponibilizar materiais acadêmicos
  Para compartilhar conteúdos de apoio aos estudantes

  Contexto:
    Dado que o CA ativo para publicação de materiais é "CA de Computação"
    E a disciplina "Cálculo I" está disponível para publicar materiais no CA ativo
    E em cada um dos CAs "CA de Computação" e "CA de Matemática" há um estudante com acesso à consulta e sem permissão de publicação

  # I06 acompanha a publicação (#111), a validação de links (#112) e a consulta no CA (#114, #115).
  @i06
  Regra: A gestão autorizada pode publicar links externos para os estudantes do CA

    Cenário: Publicar um link externo e disponibilizá-lo aos estudantes
      Dado que o integrante pode gerenciar materiais no CA ativo
      Quando o integrante solicita a publicação de "Referências de Cálculo I" em "Cálculo I" com o link "https://exemplo.edu.br/calculo"
      Então um estudante sem permissão de publicação encontra "Referências de Cálculo I" em "Cálculo I" no "CA de Computação"
      E o estudante consegue acessar o endereço "https://exemplo.edu.br/calculo" a partir do material publicado
      E o material "Referências de Cálculo I" não aparece entre os materiais publicados do "CA de Matemática"

    Esquema do Cenário: Recusar um endereço externo inválido
      Dado que o integrante pode gerenciar materiais no CA ativo
      Quando o integrante solicita a publicação de "Referências de Cálculo I" em "Cálculo I" com o link "<endereco>"
      Então a publicação do material é recusada por endereço externo inválido
      E o material "Referências de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

      Exemplos:
        | endereco          |
        |                   |
        | calculo-sem-url   |

    Cenário: Impedir a publicação de link por integrante sem permissão
      Dado que o integrante não pode gerenciar materiais no CA ativo
      Quando o integrante solicita a publicação de "Referências de Cálculo I" em "Cálculo I" com o link "https://exemplo.edu.br/calculo"
      Então a publicação do material é recusada por falta de permissão
      E o material "Referências de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

    Cenário: Impedir a publicação de link com permissão concedida somente em outro CA
      Dado que a permissão "Gerenciar materiais" do integrante vale somente no "CA de Matemática"
      Quando o integrante solicita a publicação de "Referências de Cálculo I" em "Cálculo I" com o link "https://exemplo.edu.br/calculo"
      Então a publicação do material é recusada por falta de permissão
      E o material "Referências de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"
      E o material "Referências de Cálculo I" não aparece entre os materiais publicados do "CA de Matemática"

    Cenário: Recusar publicação em uma disciplina de outro contexto acadêmico
      Dado que o integrante pode gerenciar materiais no CA ativo
      E a disciplina "Direito Constitucional" está disponível para publicar materiais somente em outro CA
      Quando o integrante solicita a publicação de "Referências de Cálculo I" em "Direito Constitucional" com o link "https://exemplo.edu.br/calculo"
      Então a publicação do material é recusada por disciplina incompatível com o CA ativo
      E o material "Referências de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

  # I11 acompanha o envio (#111), a política de arquivos (#112) e o acesso ao conteúdo (#114, #115).
  @i11
  Regra: A gestão autorizada pode publicar PDFs dentro do tamanho permitido

    Cenário: Publicar um PDF e disponibilizá-lo aos estudantes
      Dado que o integrante pode gerenciar materiais no CA ativo
      E o arquivo "resumo.pdf" é um PDF válido abaixo do limite de tamanho
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então um estudante sem permissão de publicação encontra "Resumo de Cálculo I" em "Cálculo I" no "CA de Computação"
      E o estudante consegue acessar o PDF "resumo.pdf" a partir do material publicado
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Matemática"

    Cenário: Recusar arquivo em formato não aceito
      Dado que o integrante pode gerenciar materiais no CA ativo
      E o arquivo "programa.exe" tem tamanho abaixo do limite e formato não aceito para materiais
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "programa.exe"
      Então a publicação do material é recusada por formato de arquivo não aceito
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

    Cenário: Recusar PDF que excede o limite de tamanho em um byte
      Dado que o integrante pode gerenciar materiais no CA ativo
      E o arquivo "resumo.pdf" é um PDF válido 1 byte acima do limite de tamanho
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então a publicação do material é recusada por tamanho de arquivo acima do limite
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

    Cenário: Publicar PDF com tamanho exatamente no limite permitido
      Dado que o integrante pode gerenciar materiais no CA ativo
      E o arquivo "resumo.pdf" é um PDF válido exatamente no limite de tamanho
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então um estudante sem permissão de publicação encontra "Resumo de Cálculo I" em "Cálculo I" no "CA de Computação"
      E o estudante consegue acessar o PDF "resumo.pdf" a partir do material publicado

    Cenário: Manter o material indisponível quando o envio falha
      Dado que o integrante pode gerenciar materiais no CA ativo
      E o arquivo "resumo.pdf" é um PDF válido abaixo do limite de tamanho
      E o envio do arquivo falhará nesta tentativa
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então o integrante é informado de que o envio do arquivo falhou
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

    Cenário: Publicar uma única vez ao repetir o envio que falhou
      Dado que o integrante pode gerenciar materiais no CA ativo
      E o arquivo "resumo.pdf" é um PDF válido abaixo do limite de tamanho
      E o envio anterior de "resumo.pdf" para publicar "Resumo de Cálculo I" no CA ativo falhou
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então um estudante sem permissão de publicação encontra uma única publicação de "Resumo de Cálculo I" em "Cálculo I" no "CA de Computação"
      E o estudante consegue acessar o PDF "resumo.pdf" a partir do material publicado

    Cenário: Impedir a publicação de PDF por integrante sem permissão
      Dado que o integrante não pode gerenciar materiais no CA ativo
      E o arquivo "resumo.pdf" é um PDF válido abaixo do limite de tamanho
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então a publicação do material é recusada por falta de permissão
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"

    Cenário: Impedir a publicação de PDF com permissão concedida somente em outro CA
      Dado que a permissão "Gerenciar materiais" do integrante vale somente no "CA de Matemática"
      E o arquivo "resumo.pdf" é um PDF válido abaixo do limite de tamanho
      Quando o integrante solicita a publicação de "Resumo de Cálculo I" em "Cálculo I" com o arquivo "resumo.pdf"
      Então a publicação do material é recusada por falta de permissão
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Computação"
      E o material "Resumo de Cálculo I" não aparece entre os materiais publicados do "CA de Matemática"
