/// Acesso à credencial Microsoft usada nas consultas ao Graph.
///
/// A sessão Firebase pode continuar autenticada quando um access token Microsoft
/// já venceu. Esta porta permite obter outra credencial sem reiniciar o login
/// Firebase e sem expor o SDK que administra o cache de tokens.
///
/// ## Papel no fluxo
///
/// FirebaseMicrosoftAuth consome este contrato ao implementar MicrosoftSignIn.
/// Primeiro solicita obtenção silenciosa; se houver necessidade de participação
/// do usuário e a operação permitir, solicita obtenção interativa. A aplicação
/// decide quando verificar a instituição e se aquela operação admite interação.
///
/// O adaptador MSAL implementa a porta e administra as credenciais pelo SDK.
/// A limpeza atua somente sobre a conta associada a esta instância da porta.
/// A autenticação Firebase, a consulta ao backend e a persistência da evidência
/// institucional são responsabilidades externas a esta porta.
///
/// ## Contratos e falhas
///
/// Tokens retornados são não vazios, sensíveis e destinados ao backend. Não
/// pertencem ao perfil nem ao cache institucional. Necessidade de interação tem
/// exceção própria; falhas de comunicação ou autorização conservam sua categoria.
/// A ausência de conta é sucesso na limpeza e exige interação na obtenção.
library;

import '../errors/microsoft_interaction_required_exception.dart';

/// Obtém tokens Microsoft e permite remover sua sessão local.
abstract interface class MicrosoftTokenProvider {
  /// Obtém uma credencial sem abrir interface de autenticação.
  ///
  /// O SDK pode reutilizar um token vigente ou renová-lo pela rede.
  /// Lança [MicrosoftInteractionRequiredException] se precisar do usuário.
  /// Falhas de comunicação continuam sendo falhas de comunicação.
  Future<String> getAccessTokenSilently();

  /// Abre a autorização Microsoft após uma ação explícita do usuário.
  ///
  /// [loginHint] apenas sugere a conta; não comprova sua identidade.
  /// O provedor pode aproveitar uma autorização existente sem pedir senha.
  /// Cancelamento, recusa e falhas propagam; o retorno é sempre um token não vazio.
  Future<String> getAccessTokenInteractively({String? loginHint});

  /// Remove somente a conta associada e seus tokens no cache SDK.
  ///
  /// Ausência de conta é sucesso. Não encerra a autenticação Firebase
  /// nem remove a evidência institucional persistida pelo CApp.
  Future<void> clearSession();
}
