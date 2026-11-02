# language: pt

# Referência: Relatório de Features, seções 1.2 e 12.1.
# O primeiro acesso cria o perfil; o preenchimento complementar determina sua ativação.
@backend @mobile @pending_backend @pending_mobile
Funcionalidade: Provisionamento e conclusão do perfil do usuário
  Como usuário com identidade institucional verificada
  Quero que meu perfil seja criado automaticamente e possa ser concluído depois
  Para retomar meu cadastro sem perder minha identidade ou meus dados

  Regra: O primeiro acesso cria um único perfil incompleto

    Cenário: Criar perfil incompleto no primeiro acesso institucional
      Dado que a identidade institucional do usuário foi autenticada e verificada
      E ainda não existe um perfil do CApp associado a essa identidade
      Quando o usuário conclui seu primeiro acesso institucional
      Então o CApp cria um perfil associado à identidade autenticada
      E o perfil é criado com o status "Incomplete"
      E a criação do perfil não exige o preenchimento prévio dos dados complementares
      E o aplicativo solicita a conclusão dos dados obrigatórios do perfil

    Cenário: Retomar o preenchimento após um novo acesso
      Dado que o usuário possui um perfil com status "Incomplete"
      E já informou parte dos dados complementares do perfil
      Quando retorna ao CApp com a mesma identidade institucional autenticada
      Então o aplicativo recupera o mesmo perfil incompleto
      E os dados complementares já salvos permanecem disponíveis
      E o aplicativo permite continuar o preenchimento

    Cenário: Acessos simultâneos não duplicam o perfil
      Dado que a identidade institucional do usuário foi autenticada e verificada
      E ainda não existe um perfil do CApp associado a essa identidade
      Quando duas solicitações de primeiro acesso dessa identidade são processadas simultaneamente
      Então existe somente um perfil do CApp para essa identidade
      E as duas solicitações recuperam o mesmo identificador interno

  Regra: A conclusão dos dados obrigatórios ativa o perfil existente

    Cenário: Ativar o perfil após preencher os dados obrigatórios
      Dado que o usuário possui um perfil com status "Incomplete"
      Quando informa todos os dados complementares obrigatórios válidos e confirma a conclusão
      Então o perfil existente passa ao status "Active"
      E os dados informados ficam associados ao mesmo perfil
      E seu identificador interno permanece inalterado

    Cenário: Não ativar um perfil com dados obrigatórios ausentes
      Dado que o usuário possui um perfil com status "Incomplete"
      Quando tenta concluir o perfil sem informar todos os dados obrigatórios
      Então o perfil permanece com o status "Incomplete"
      E o aplicativo informa quais dados obrigatórios ainda precisam ser preenchidos

  Regra: A identidade autenticada determina o perfil recuperado em acessos posteriores

    Cenário: Recuperar o perfil ativo sem criar outro cadastro
      Dado que o usuário possui um perfil com status "Active"
      Quando retorna ao CApp com a mesma identidade institucional autenticada
      Então o aplicativo recupera o mesmo perfil ativo
      E nenhum novo perfil é criado para essa identidade
      E seus dados próprios do CApp permanecem preservados

    Cenário: Alteração do e-mail não cria outro perfil para a mesma identidade
      Dado que o usuário já possui um perfil associado à sua identidade institucional
      E o e-mail dessa identidade foi alterado pela instituição
      Quando retorna ao CApp com a mesma identidade institucional autenticada
      Então o aplicativo recupera o perfil associado à identidade autenticada
      E o identificador interno original permanece inalterado
      E nenhum novo perfil é criado apenas pela alteração do e-mail
