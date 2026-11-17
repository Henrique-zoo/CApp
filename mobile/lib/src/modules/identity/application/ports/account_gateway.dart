/// Operações do perfil interno autenticadas pelo Firebase, sem token Microsoft.
///
/// A restauração consulta usuário e prazo institucional no backend. Somente a
/// sincronização institucional passa pela porta InstitutionalIdentityGateway.
/// A conclusão envia os dados acadêmicos depois de configurar a credencial de
/// senha. Separar essas operações da verificação permite reutilizar a evidência
/// sem renovar o prazo ou depender de autorização Microsoft em cada entrada.
///
/// A infraestrutura converte DTOs e falhas para o vocabulário do módulo. O
/// controlador confere o UID antes/depois de aguardar a porta e é proprietário
/// da publicação e da persistência; uma resposta não comprova, sozinha, que a
/// operação ainda pertence à conta atual. A sugestão Graph não é restaurada aqui.
library;

import '../models/internal_user.dart';

/// Saída da aplicação para ler e concluir a conta da autenticação Firebase atual.
abstract interface class AccountGateway {
  /// Recupera conta e evidência sem consultar o Graph ou estender seu prazo.
  ///
  /// Evidência nula solicita sincronização. IdentityFailureKind.accountNotProvisioned
  /// identifica primeiro acesso; recusas e falhas temporárias propagam com suas
  /// categorias, sem serem tratadas como conta ausente.
  ///
  /// Retorna [AccountSession] com conta incompleta ou ativa. A implementação não
  /// deve provisionar silenciosamente uma conta ausente nem consultar o Graph
  /// para contornar uma recusa. Não recebe senha, decide navegação ou grava cache.
  Future<AccountSession> restoreSession();

  /// Conclui ou atualiza o perfil da mesma conta que autenticou por senha.
  ///
  /// [profile] contém os dados do formulário; a credencial e o e-mail escolhido
  /// já foram tratados no Firebase. O backend valida a evidência e o conteúdo,
  /// recusando conta inativa e nome declarado que substitua um nome do Graph.
  ///
  /// Retorna a fotografia [InternalUser] após a escrita aceita. Falhas esperadas
  /// devem propagar como `IdentityException`, sem simular ativação ou esconder o
  /// diagnóstico. O controlador continua responsável por conferir a identidade,
  /// publicar a conta e persistir o retorno; a porta não emite tokens ou renova
  /// a confirmação institucional.
  Future<InternalUser> completeProfile(ProfileCompletion profile);
}
