# language: pt

Funcionalidade: Publicação e transparência de informações institucionais do Centro Acadêmico
  Como estudante ou membro da gestão de um Centro Acadêmico
  Quero publicar, atualizar, versionar e consultar informações institucionais
  Para garantir a transparência da entidade e o livre acesso aos documentos e contatos oficiais da gestão

  Contexto:
    Dado que o "Centro Acadêmico de Ciência da Computação" está ativo no CApp

  # ---------------------------------------------------------------------------
  # 1. Estruturação e Publicação Inicial de Informações Institucionais
  # ---------------------------------------------------------------------------
  Regra: Apenas membros autorizados da gestão ativa podem publicar informações institucionais

    Cenário: Membro autorizado da gestão publica documento institucional com sucesso
      Dado que o usuário é um membro autorizado da gestão ativa do Centro Acadêmico
      Quando ele publica o documento institucional com os seguintes dados:
        | tipo      | Estatuto Social                                     |
        | título    | Estatuto Geral do Centro Acadêmico                  |
        | conteúdo  | Normas fundamentais, direitos e deveres dos membros |
      Então o documento institucional é publicado com a versão "1.0"
      E fica disponível publicamente para consulta da comunidade acadêmica

    Cenário: Membro autorizado publica canais oficiais de contato e composição da gestão
      Dado que o usuário é um membro autorizado da gestão ativa do Centro Acadêmico
      Quando ele publica as informações de contato e a composição da gestão atual com os seguintes dados:
        | e-mail oficial       | contato@cacc.ufba.br         |
        | canal de atendimento | Sala 102 - Pavilhão de Aulas |
        | presidente           | Maria Silva                  |
        | diretora financeira  | João Santos                  |
      Então as informações institucionais são publicadas com sucesso
      E passam a ser exibidas como os dados oficiais vigentes do Centro Acadêmico

    Cenário: Estudante comum tenta publicar documento institucional
      Dado que o usuário é um estudante regularmente matriculado sem cargo na gestão do Centro Acadêmico
      Quando ele tenta publicar um regimento interno para o Centro Acadêmico
      Então a publicação é recusada por falta de permissão administrativa
      E nenhum documento institucional é registrado

    Cenário: Membro de gestão encerrada tenta publicar informações institucionais
      Dado que o usuário fez parte de uma gestão anterior já encerrada do Centro Acadêmico
      Quando ele tenta publicar um novo documento institucional
      Então a publicação é recusada pois o usuário não integra a gestão ativa
      E o acervo documental permanece inalterado

  # ---------------------------------------------------------------------------
  # 2. Atualização e Versionamento de Conteúdos Institucionais
  # ---------------------------------------------------------------------------
  Regra: Atualizações em documentos institucionais devem gerar nova versão e manter histórico

    Cenário: Atualização de documento institucional gera novo registro de versão
      Dado que o Centro Acadêmico possui um "Estatuto Social" publicado na versão "1.0"
      E que o usuário é um membro autorizado da gestão ativa
      Quando ele atualiza o conteúdo do "Estatuto Social" com uma nova redação
      Então uma nova versão "1.1" do documento é publicada
      E a versão "1.1" torna-se a versão vigente para consulta

    Cenário: Preservação do histórico de versões anteriores
      Dado que o "Regimento Interno" possui as versões "1.0" e "1.1" registradas
      Quando qualquer usuário consulta o histórico do "Regimento Interno"
      Então o sistema disponibiliza o acesso à versão anterior "1.0" para auditoria
      E mantém o registro de data e autor da alteração

    Cenário: Tentativa de atualizar documento inexistente
      Dado que o usuário é um membro autorizado da gestão ativa
      Quando ele tenta atualizar o conteúdo de um documento "Carta de Princípios" inexistente
      Então a operação é recusada indicando que o documento base não foi encontrado

  # ---------------------------------------------------------------------------
  # 3. Consulta Pública e Transparência
  # ---------------------------------------------------------------------------
  Regra: Documentos e informações institucionais vigentes são de livre acesso público

    Cenário: Estudante consulta documentos institucionais vigentes
      Dado que o Centro Acadêmico possui documentos institucionais vigentes publicados
      Quando um estudante acessa a seção institucional do Centro Acadêmico
      Então ele visualiza o estatuto social e o regimento interno vigentes
      E visualiza a data da última atualização de cada documento

    Cenário: Estudante consulta canais de contato e membros da gestão
      Dado que o Centro Acadêmico possui contatos e composição da gestão publicados
      Quando um estudante consulta as informações institucionais do Centro Acadêmico
      Então ele visualiza os canais de contato oficiais e a lista de membros da gestão ativa
