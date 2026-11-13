/// Porta de entrada e obtenção de credenciais para a verificação institucional.
///
/// Uma sessão Firebase persistida não garante que exista um access token
/// Microsoft disponível. A aplicação precisa distinguir a criação dessa sessão
/// da obtenção de credenciais para uma conta já autenticada, sem escolher SDKs,
/// popup ou navegação nativa. Esta biblioteca define essas duas operações.
///
/// ## Uso no fluxo
///
/// Sem conta autenticada, InstitutionalAuthService solicita login.
/// Somente quando a evidência precisa de renovação, solicita
/// getAccessTokenForCurrentUser e confere que
/// a identidade permanece a mesma. O access token retornado segue para o gateway
/// institucional; somente uma resposta válida do backend renova a evidência.
/// A obtenção de credenciais não exige um novo login ou autenticação recente
/// no Firebase.
///
/// A renovação automática usa allowInteraction: false. Uma ação explícita do
/// usuário pode permitir autorização Microsoft quando a tentativa silenciosa
/// indicar essa necessidade. Falhas de rede não devem abrir o provedor.
///
/// O adaptador pode estabelecer a identidade Firebase antes de concluir seu
/// Future. Essa emissão não representa, por si só, sucesso da verificação
/// institucional nem prontidão para a área principal.
///
/// ## Credenciais e falhas
///
/// A obtenção de credenciais deve retornar um access token Microsoft utilizável. O ID token Firebase
/// é uma credencial diferente, obtida separadamente pelo transporte HTTP.
/// Tokens não pertencem ao modelo do perfil nem ao cache institucional.
///
/// Cancelamento, recusa e falha de obtenção propagam; não há retorno vazio ou
/// nulo para representá-los. A implementação concreta traduz falhas reconhecidas
/// do provedor para as categorias consumidas pela aplicação.
library;

import '../errors/microsoft_interaction_required_exception.dart';

/// Operações de autenticação e credenciais consumidas pelo caso de uso.
///
/// O adaptador deve operar sobre a mesma autenticação observada por `AuthSession`.
/// Esta interface não consulta o backend nem decide se a conta é estudantil.
abstract interface class MicrosoftSignIn {
  /// Autentica pela Microsoft e estabelece a sessão Firebase.
  ///
  /// A aplicação consulta a conta no backend antes de solicitar autorização
  /// Graph. Esta operação não obtém tokens MSAL nem limpa seu cache.
  /// Pode emitir mudança de UID antes de concluir. Cancelamento e recusa
  /// propagam com as categorias da aplicação.
  Future<void> signIn();

  /// Obtém o token Microsoft para a conta Firebase atual.
  ///
  /// Primeiro tenta obter a credencial silenciosamente.
  /// Com [allowInteraction] falso, propaga
  /// [MicrosoftInteractionRequiredException] sem abrir interface.
  ///
  /// Não exige autenticação recente no Firebase. A validade
  /// institucional só é renovada após a consulta ao backend.
  ///
  /// Exige uma conta autenticada e preserva sua identidade durante a operação.
  /// Ausência ou mudança da conta sinaliza SessionIdentityChangedException.
  /// O backend deve validar a correspondência entre a identidade Microsoft
  /// da credencial e aquela vinculada ao Firebase; estabilidade do UID e
  /// sugestões de conta na interface não comprovam essa correspondência.
  ///
  /// Retorna um token não vazio, que pode vir do cache de credenciais do provedor.
  /// Falhas de autorização ou comunicação propagam sem produzir token vazio.
  Future<String> getAccessTokenForCurrentUser({bool allowInteraction = true});
}
