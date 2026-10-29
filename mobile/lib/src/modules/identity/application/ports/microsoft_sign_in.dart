/// Porta para a interação Microsoft necessária à verificação institucional.
///
/// A aplicação precisa obter uma credencial delegada e distinguir login de
/// reautenticação, sem escolher popup, navegação nativa ou classes de um SDK.
/// Esta biblioteca estabelece esse contrato externo do caso de uso.
///
/// ## Uso no fluxo
///
/// Sem conta autenticada, InstitutionalAuthService solicita login.
/// Com uma conta conhecida, solicita reautenticação e confere que a identidade
/// permanece a mesma. O access token retornado é encaminhado ao gateway para
/// a verificação institucional.
///
/// O adaptador pode estabelecer a identidade Firebase antes de concluir seu
/// Future. Essa emissão não representa, por si só, sucesso da verificação
/// institucional nem prontidão para a área principal.
///
/// ## Credenciais e falhas
///
/// O resultado deve ser um access token Microsoft utilizável. O ID token Firebase
/// é uma credencial diferente, obtida separadamente pelo transporte HTTP.
/// Tokens não pertencem ao modelo do perfil nem ao cache institucional.
///
/// Cancelamento, recusa e falha de obtenção propagam; não há retorno vazio ou
/// nulo para representá-los. A implementação concreta traduz falhas reconhecidas
/// do provedor para as categorias consumidas pela aplicação.
library;

/// Porta de autenticação Microsoft consumida pelo caso de uso de identidade.
///
/// O adaptador deve operar sobre a mesma autenticação observada por `AuthSession`.
/// Esta interface não consulta o backend nem decide se a conta é estudantil.
abstract interface class MicrosoftSignIn {
  /// Autentica o usuário e devolve o access token Microsoft delegado.
  ///
  /// A implementação deve estabelecer a sessão Firebase usada pelo backend.
  /// O resultado deve ser um token Microsoft não vazio, distinto do Firebase
  /// ID Token. Falhas de autenticação são propagadas ao chamador.
  ///
  /// Pode provocar eventos de mudança de UID antes de o Future concluir.
  /// O retorno contém uma credencial sensível: não deve ser exibida ou registrada.
  /// Cancelamento, recusa e configuração inválida não devem retornar string vazia.
  Future<String> signInAndGetAccessToken();

  /// Reautentica o usuário atual e retorna um novo access token Microsoft.
  ///
  /// Exige uma sessão autenticada. A credencial deve corresponder
  /// ao usuário atual; a operação não deve trocar sua identidade.
  ///
  /// Falhas e cancelamentos são propagados ao chamador.
  /// O vínculo institucional anterior não é prorrogado apenas por obter essa
  /// credencial: é necessário concluir novamente a operação no backend.
  Future<String> reauthenticateAndGetAccessToken();
}
