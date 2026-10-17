import 'authenticated_identity.dart';

/// Motivo funcional de uma falha de autenticação.
enum AuthenticationFailureReason {
  /// Configuração necessária ausente ou inválida.
  configuration,

  /// O usuário cancelou o fluxo de autenticação.
  cancelled,

  /// O backend recusou a identidade institucional.
  institutionalAccountRejected,

  /// O provedor ou a API está indisponível.
  serviceUnavailable,

  /// Não foi possível concluir ou limpar a sessão local.
  sessionOperationFailed,

  /// Falha não classificada durante a autenticação.
  unknown,
}

/// Falha de autenticação com mensagem segura para apresentar na interface.
class AuthenticationFailure implements Exception {
  const AuthenticationFailure(this.reason, this.message);

  /// Classificação funcional da falha.
  final AuthenticationFailureReason reason;

  /// Mensagem que pode ser exibida sem expor detalhes internos.
  final String message;

  @override
  String toString() => message;
}

/// Operações de sessão exigidas pela aplicação.
abstract interface class AuthenticationRepository {
  /// Restaura uma sessão persistida e confirma a identidade com o backend.
  /// Retorna `null` quando não existe usuário autenticado no Firebase.
  Future<AuthenticatedIdentity?> restoreSession();

  /// Autentica com Microsoft via Firebase e valida a identidade no backend.
  Future<AuthenticatedIdentity> signIn();

  /// Encerra a sessão local do Firebase Authentication.
  Future<void> signOut();
}
