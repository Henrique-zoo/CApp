# language: pt

# Referência: Relatório de Features do Projeto, seção 5.2.
# Recorte I07: fórum de demandas sobre o espaço físico do CA.
@pending_backend @pending_mobile
Funcionalidade: Criação de demandas sobre o espaço físico do CA

  Como usuário do aplicativo
  Quero registrar demandas sobre o espaço físico do CA
  Para comunicar necessidades relacionadas ao local

  Regra: A demanda pertence ao CA ativo e ao fórum do espaço físico

    Cenário: Registrar uma demanda sobre o espaço físico do CA
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      Quando registro uma demanda sobre o espaço físico com o conteúdo "A porta da sala está danificada"
      Então a demanda é criada no fórum do espaço físico do "CA de Computação"
      E o conteúdo registrado é "A porta da sala está danificada"
      E o autor da demanda é "Usuário A"
      E a demanda fica associada somente ao "CA de Computação"
      E o status inicial da demanda é "aberta"
      E a demanda não é encaminhada à coordenação do curso

    Cenário: Permitir a criação por usuário sem vínculo de gestão com o CA
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Estudante de outro curso"
      E não tenho vínculo de gestão com "CA de Computação"
      Quando registro uma demanda sobre o espaço físico com o conteúdo "A lâmpada da sala precisa ser substituída"
      Então a demanda é criada no fórum do espaço físico do "CA de Computação"
      E o autor da demanda é "Estudante de outro curso"
      E a demanda não é encaminhada à coordenação do curso

    Cenário: Recusar a criação sem o conteúdo obrigatório
      Dado que o CA ativo é "CA de Computação"
      E estou autenticado como "Usuário A"
      Quando tento registrar uma demanda sobre o espaço físico sem informar o conteúdo
      Então a demanda não é criada
      E o aplicativo informa que o conteúdo é obrigatório
