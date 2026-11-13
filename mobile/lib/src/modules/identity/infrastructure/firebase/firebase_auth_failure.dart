/// Tradução comum das falhas Firebase usadas pelos adaptadores de identidade.
///
/// Login Microsoft, credenciais internas e restauração de contas usam o mesmo
/// SDK. Centralizar a classificação evita tratar indisponibilidade como senha
/// rejeitada em apenas um desses caminhos. Os adaptadores continuam responsáveis
/// por capturar a exceção e propagar a pilha original.
///
/// Somente mensagens definidas pelo CApp são públicas. Códigos desconhecidos
/// preservam uma falha externa não classificada; mensagens arbitrárias do SDK
/// não são apresentadas ao usuário nem registradas por esta biblioteca.
library;

import 'package:firebase_auth/firebase_auth.dart';

import '../../application/errors/identity_exception.dart';

/// Classifica uma falha sem alterar a sessão ou iniciar tentativas adicionais.
IdentityException translateFirebaseAuthFailure(
  FirebaseAuthException error,
  StackTrace stack,
) {
  final kind = switch (error.code) {
    'popup-closed-by-user' ||
    'cancelled-popup-request' ||
    'web-context-canceled' ||
    'user-cancelled' => IdentityFailureKind.authenticationCancelled,
    'network-request-failed' => IdentityFailureKind.communicationFailed,
    'too-many-requests' ||
    'quota-exceeded' => IdentityFailureKind.serviceUnavailable,
    'invalid-credential' ||
    'invalid-login-credentials' ||
    'user-disabled' ||
    'user-not-found' ||
    'wrong-password' ||
    'user-mismatch' ||
    'invalid-user-token' ||
    'user-token-expired' ||
    'requires-recent-login' ||
    'account-exists-with-different-credential' ||
    'weak-password' ||
    'invalid-email' ||
    'email-already-in-use' ||
    'credential-already-in-use' => IdentityFailureKind.credentialsRejected,
    _ => IdentityFailureKind.unexpectedExternalFailure,
  };

  final message = switch (error.code) {
    'weak-password' => 'Escolha uma senha que atenda à política do Firebase.',
    'invalid-email' => 'Informe um e-mail válido.',
    'email-already-in-use' || 'credential-already-in-use' =>
      'Este e-mail já está associado a outra conta.',
    _ => null,
  };

  return IdentityException(
    kind: kind,
    cause: error,
    stackTrace: stack,
    publicMessage: message,
  );
}
